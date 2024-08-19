//! Subversion repository prober.
//!
//! This module provides a prober for Subversion repositories, but no actual
//! implementation is provided.
use crate::controldir::Prober;
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

pub struct SvnRepositoryProber(PyObject);

impl SvnRepositoryProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import_bound("breezy.plugins.svn") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.svn");
                    }
                }
            };
            let prober = m
                .getattr("SvnRepositoryProber")
                .expect("Failed to get SvnRepositoryProber");
            Some(Self(prober.to_object(py)))
        })
    }
}

impl FromPyObject<'_> for SvnRepositoryProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_object(obj.py())))
    }
}

impl ToPyObject for SvnRepositoryProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Prober for SvnRepositoryProber {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let _ = SvnRepositoryProber::new();
    }
}
