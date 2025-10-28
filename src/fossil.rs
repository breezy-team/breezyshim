//! Support for detecting Fossil repositories.
//!
//! This module provides a prober for detecting Fossil repositories, but
//! currently does not provide any additional functionality.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

/// A prober that can detect Fossil repositories.
pub struct RemoteFossilProber(PyObject);

impl RemoteFossilProber {
    /// Create a new RemoteFossilProber, returning None if the Fossil plugin is not available.
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.plugins.fossil") {
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
            Some(Self(prober.unbind()))
        })
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for RemoteFossilProber {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_owned().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for RemoteFossilProber {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl std::fmt::Debug for RemoteFossilProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("RemoteFossilProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for RemoteFossilProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_fossil_prober() {
        let _ = RemoteFossilProber::new();
    }
}
