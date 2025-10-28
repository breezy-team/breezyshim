//! Subversion repository prober.
//!
//! This module provides a prober for Subversion repositories, but no actual
//! implementation is provided.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

/// Prober for Subversion repositories.
///
/// This struct can detect Subversion repositories but requires the Breezy
/// Subversion plugin to be installed.
pub struct SvnRepositoryProber(PyObject);

impl SvnRepositoryProber {
    /// Create a new SvnRepositoryProber instance.
    ///
    /// # Returns
    ///
    /// Some(SvnRepositoryProber) if the Subversion plugin is installed,
    /// None otherwise.
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.plugins.svn") {
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
            Some(Self(prober.unbind()))
        })
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SvnRepositoryProber {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_owned().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for SvnRepositoryProber {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl std::fmt::Debug for SvnRepositoryProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("SvnRepositoryProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for SvnRepositoryProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let _ = SvnRepositoryProber::new();
    }
}
