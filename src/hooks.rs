//! Hooks
use pyo3::prelude::*;

/// Dictionary-like container for Breezy hooks.
pub struct HookDict(Py<PyAny>);

/// Represents an individual hook function.
pub struct Hook(Py<PyAny>);

impl HookDict {
    /// Create a new hook dictionary.
    ///
    /// # Arguments
    ///
    /// * `module` - The Python module containing the hook point
    /// * `cls` - The class name within the module
    /// * `name` - The name of the hook point
    pub fn new(module: &str, cls: &str, name: &str) -> Self {
        Python::attach(|py| -> PyResult<HookDict> {
            let module = PyModule::import(py, module)?;
            let cls = module.getattr(cls)?;
            let entrypoint = cls.getattr(name)?;
            Ok(Self(entrypoint.unbind()))
        })
        .unwrap()
    }

    /// Clear all hooks registered for a given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the hook point
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails
    pub fn clear(&self, name: &str) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            let entrypoint = self.0.bind(py).get_item(name)?;
            entrypoint.call_method0("clear")?;
            Ok(())
        })
    }

    /// Add a hook function for a given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the hook point
    /// * `func` - The hook function to add
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails
    pub fn add(&self, name: &str, func: Hook) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            let entrypoint = self.0.bind(py).get_item(name)?;
            entrypoint.call_method1("add", (func.0,))?;
            Ok(())
        })
    }

    /// Get all hook functions registered for a given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the hook point
    ///
    /// # Returns
    ///
    /// A vector of hook functions, or an error if the operation fails
    pub fn get(&self, name: &str) -> Result<Vec<Hook>, crate::error::Error> {
        Python::attach(|py| {
            let entrypoint = self.0.bind(py).get_item(name)?;
            // HookPoint is iterable but not a Sequence, so iterate rather than extract.
            Ok(entrypoint
                .try_iter()?
                .map(|r| r.map(|obj| Hook(obj.unbind())))
                .collect::<PyResult<Vec<_>>>()?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn merge_hooks() -> HookDict {
        crate::init();
        HookDict::new("breezy.merge", "Merger", "hooks")
    }

    /// `get` succeeds on a real breezy hook point and returns a `Vec`. The
    /// hook point is iterable but not a `Sequence`, which is why `get`
    /// iterates rather than extracting a `Vec` directly.
    #[test]
    fn test_get_returns_vec() {
        let hooks = merge_hooks();
        let registered = hooks.get("post_merge").unwrap();
        // We don't assert a length: other tests or breezy itself may register
        // hooks on this process-wide hook point. We only care that iterating
        // the hook point succeeds.
        let _ = registered;
    }

    /// A hook registered on the underlying breezy hook point shows up in the
    /// result of `get`, exercising the iteration code path.
    #[test]
    fn test_get_returns_registered_hook() {
        let hooks = merge_hooks();

        let before = hooks.get("post_merge").unwrap().len();

        Python::attach(|py| {
            let entrypoint = hooks.0.bind(py).get_item("post_merge").unwrap();
            let func = py
                .eval(
                    std::ffi::CString::new("lambda merger: None")
                        .unwrap()
                        .as_c_str(),
                    None,
                    None,
                )
                .unwrap();
            entrypoint
                .call_method1("hook", (func, "breezyshim test hook"))
                .unwrap();
        });

        let after = hooks.get("post_merge").unwrap();
        assert_eq!(before + 1, after.len());

        // Clean up so this process-wide hook point is left as we found it.
        Python::attach(|py| {
            let entrypoint = hooks.0.bind(py).get_item("post_merge").unwrap();
            entrypoint
                .call_method1("uninstall", ("breezyshim test hook",))
                .unwrap();
        });
        assert_eq!(before, hooks.get("post_merge").unwrap().len());
    }

    /// `get` on an unknown hook name returns an error rather than panicking.
    #[test]
    fn test_get_unknown_name() {
        let hooks = merge_hooks();
        assert!(hooks.get("nonexistent_hook").is_err());
    }
}
