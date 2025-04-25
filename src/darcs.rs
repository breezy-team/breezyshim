//! Darcs prober.
//!
//! This module provides a prober for Darcs repositories. It can detect
//! darcs repositories but does not provide any additional functionality.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

/// A prober for Darcs repositories.
pub struct DarcsProber(PyObject);

impl DarcsProber {
    /// Create a new Darcs prober instance.
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import_bound("breezy.plugins.darcs") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.darcs");
                    }
                }
            };
            let prober = m.getattr("DarcsProber").expect("Failed to get DarcsProber");
            Some(Self(prober.to_object(py)))
        })
    }
}

impl FromPyObject<'_> for DarcsProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_object(obj.py())))
    }
}

impl ToPyObject for DarcsProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl std::fmt::Debug for DarcsProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("DarcsProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for DarcsProber {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let _ = DarcsProber::new();
    }
}
