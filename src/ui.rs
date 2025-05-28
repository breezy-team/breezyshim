//! UI Factory

use pyo3::prelude::*;

pub trait PyUIFactory: ToPyObject + std::any::Any + std::fmt::Debug {}

pub trait UIFactory: std::fmt::Debug {}

impl<T: PyUIFactory> UIFactory for T {}

crate::wrapped_py!(SilentUIFactory);

impl SilentUIFactory {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            SilentUIFactory(
                py.import("breezy.ui")
                    .unwrap()
                    .getattr("SilentUIFactory")
                    .unwrap()
                    .call0()
                    .unwrap()
                    .unbind()
            )
        })
    }
}

impl Default for SilentUIFactory {
    fn default() -> Self {
        Self::new()
    }
}

crate::wrapped_py!(GenericUIFactory);


impl GenericUIFactory {
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }
}

impl PyUIFactory for GenericUIFactory {}

impl std::fmt::Debug for GenericUIFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("GenericUIFactory({:?})", self.0))
    }
}


impl PyUIFactory for SilentUIFactory {}

impl std::fmt::Debug for SilentUIFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("SilentUIFactory({:?})", self.0))
    }
}

pub fn install_ui_factory(factory: &dyn PyUIFactory) {
    Python::with_gil(|py| {
        let m = py.import("breezy.ui").unwrap();
        m.setattr("ui_factory", factory).unwrap();
    });
}

pub fn get_ui_factory() -> Box<dyn PyUIFactory> {
    Box::new(GenericUIFactory::from(Python::with_gil(|py| {
        let m = py.import("breezy.ui").unwrap();
        m.getattr("ui_factory").unwrap().unbind()
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
