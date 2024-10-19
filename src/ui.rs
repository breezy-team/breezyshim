//! UI Factory

use pyo3::prelude::*;

pub trait UIFactory: ToPyObject {}

pub struct SilentUIFactory(PyObject);

impl SilentUIFactory {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            SilentUIFactory(
                py.import_bound("breezy.ui")
                    .unwrap()
                    .getattr("SilentUIFactory")
                    .unwrap()
                    .call0()
                    .unwrap()
                    .to_object(py),
            )
        })
    }
}

impl Default for SilentUIFactory {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GenericUIFactory(PyObject);

impl ToPyObject for GenericUIFactory {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl UIFactory for GenericUIFactory {}

impl ToPyObject for SilentUIFactory {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl UIFactory for SilentUIFactory {}

pub fn install_ui_factory(factory: &dyn UIFactory) {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.ui").unwrap();
        m.setattr("ui_factory", factory.to_object(py)).unwrap();
    });
}

pub fn get_ui_factory() -> Box<dyn UIFactory> {
    Box::new(GenericUIFactory(Python::with_gil(|py| {
        let m = py.import_bound("breezy.ui").unwrap();
        m.getattr("ui_factory").unwrap().to_object(py)
    }))) as Box<dyn UIFactory>
}

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
