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
            Ok(entrypoint
                .extract::<Vec<Py<PyAny>>>()?
                .into_iter()
                .map(Hook)
                .collect())
        })
    }
}
