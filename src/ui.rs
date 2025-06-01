//! UI Factory

use pyo3::prelude::*;

/// Python UI factory trait.
pub trait PyUIFactory: std::any::Any + std::fmt::Debug {
    /// Get the underlying Python object for this UI factory.
    fn to_object(&self, py: Python) -> PyObject;
}

/// UI factory trait.
pub trait UIFactory: std::fmt::Debug {}

impl<T: PyUIFactory> UIFactory for T {}

/// UI factory that does not output anything.
pub struct SilentUIFactory(PyObject);

impl SilentUIFactory {
    /// Create a new silent UI factory.
    pub fn new() -> Self {
        Python::with_gil(|py| {
            SilentUIFactory(
                py.import("breezy.ui")
                    .unwrap()
                    .getattr("SilentUIFactory")
                    .unwrap()
                    .call0()
                    .unwrap()
                    .unbind(),
            )
        })
    }
}

impl Default for SilentUIFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic wrapper for a Python UI factory.
pub struct GenericUIFactory(PyObject);

impl<'py> IntoPyObject<'py> for GenericUIFactory {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl FromPyObject<'_> for GenericUIFactory {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(GenericUIFactory(obj.clone().unbind()))
    }
}

impl GenericUIFactory {
    /// Create a new generic UI factory from a Python object.
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }
}

impl PyUIFactory for GenericUIFactory {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl std::fmt::Debug for GenericUIFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("GenericUIFactory({:?})", self.0))
    }
}

impl<'py> IntoPyObject<'py> for SilentUIFactory {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl PyUIFactory for SilentUIFactory {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl std::fmt::Debug for SilentUIFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("SilentUIFactory({:?})", self.0))
    }
}

/// Install a UI factory globally.
pub fn install_ui_factory(factory: &dyn PyUIFactory) {
    Python::with_gil(|py| {
        let m = py.import("breezy.ui").unwrap();
        m.setattr("ui_factory", factory.to_object(py)).unwrap();
    });
}

/// Get the current global UI factory.
pub fn get_ui_factory() -> Box<dyn PyUIFactory> {
    Box::new(GenericUIFactory::new(Python::with_gil(|py| {
        let m = py.import("breezy.ui").unwrap();
        m.getattr("ui_factory").unwrap().unbind()
    }))) as Box<dyn PyUIFactory>
}

/// Run a function with a silent UI factory temporarily installed.
pub fn with_silent_ui_factory<R>(f: impl FnOnce() -> R) -> R {
    let old_factory = get_ui_factory();
    let new_factory = SilentUIFactory::new();
    install_ui_factory(&new_factory);
    let r = f();
    install_ui_factory(old_factory.as_ref());
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_silent_factory() {
        let _ = SilentUIFactory::new();
    }

    #[test]
    fn test_run_with_silent_factory() {
        with_silent_ui_factory(|| {
            crate::version::version();
        });
    }
}
