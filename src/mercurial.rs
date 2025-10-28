//! Mercurial prober.
//!
//! This allows detecting Mercurial repositories, but does not provide any
//! functionality to interact with them.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

/// Prober for Mercurial repositories.
///
/// This struct can detect Mercurial repositories but does not provide
/// functionality to interact with them directly. It requires the Breezy
/// Mercurial plugin to be installed.
pub struct SmartHgProber(PyObject);

impl SmartHgProber {
    /// Create a new SmartHgProber instance.
    ///
    /// # Returns
    ///
    /// Some(SmartHgProber) if the Mercurial plugin is installed,
    /// None otherwise.
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.plugins.hg") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.hg");
                    }
                }
            };
            let prober = m
                .getattr("SmartHgProber")
                .expect("Failed to get SmartHgProber");
            Some(Self(prober.unbind()))
        })
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SmartHgProber {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_owned().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for SmartHgProber {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl std::fmt::Debug for SmartHgProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("SmartHgProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for SmartHgProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_hg_prober() {
        let _ = SmartHgProber::new();
    }
}
