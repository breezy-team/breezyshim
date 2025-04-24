//! Support for detecting Fossil repositories.
//!
//! This module provides a prober for detecting Fossil repositories, but
//! currently does not provide any additional functionality.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

pub struct RemoteFossilProber(PyObject);

impl RemoteFossilProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import_bound("breezy.plugins.fossil") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.fossil");
                    }
                }
            };
            let prober = m
                .getattr("RemoteFossilProber")
                .expect("Failed to get RemoteFossilProber");
            Some(Self(prober.to_object(py)))
        })
    }
}

impl FromPyObject<'_> for RemoteFossilProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_object(obj.py())))
    }
}

impl ToPyObject for RemoteFossilProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl std::fmt::Debug for RemoteFossilProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("RemoteFossilProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for RemoteFossilProber {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_fossil_prober() {
        let _ = RemoteFossilProber::new();
    }
}
