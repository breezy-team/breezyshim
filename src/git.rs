//! Git version control system support.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

/// A prober that can detect remote Git repositories.
pub struct RemoteGitProber(PyObject);

/// The SHA1 hash consisting of all zeros, representing the absence of a commit in Git.
pub const ZERO_SHA: &[u8] = b"0000000000000000000000000000000000000000";

impl RemoteGitProber {
    /// Create a new RemoteGitProber, returning None if the Git plugin is not available.
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.git") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.git");
                    }
                }
            };
            let prober = m
                .getattr("RemoteGitProber")
                .expect("Failed to get GitProber");
            Some(Self(prober.unbind()))
        })
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for RemoteGitProber {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_owned().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for RemoteGitProber {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl std::fmt::Debug for RemoteGitProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("RemoteGitProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for RemoteGitProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

/// Format for bare local Git repositories.
pub struct BareLocalGitControlDirFormat(PyObject);

impl BareLocalGitControlDirFormat {
    /// Create a new BareLocalGitControlDirFormat.
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py
                .import("breezy.git")
                .expect("Failed to import breezy.git");
            let format = m
                .getattr("BareLocalGitControlDirFormat")
                .expect("Failed to get BareLocalGitControlDirFormat");

            Self(
                format
                    .call0()
                    .expect("Failed to create BareLocalGitControlDirFormat")
                    .unbind(),
            )
        })
    }
}

impl<'py> IntoPyObject<'py> for BareLocalGitControlDirFormat {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl crate::controldir::AsFormat for BareLocalGitControlDirFormat {
    fn as_format(&self) -> Option<crate::controldir::ControlDirFormat> {
        Some(Python::with_gil(|py| {
            crate::controldir::ControlDirFormat::from(self.0.clone_ref(py))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::AsFormat;

    #[test]
    fn test_zero_sha() {
        assert_eq!(ZERO_SHA.len(), 40);
        assert_eq!(ZERO_SHA, b"0000000000000000000000000000000000000000");
    }

    #[test]
    fn test_remote_git_prober_new() {
        // This may return None if git plugin is not available
        let _prober = RemoteGitProber::new();
    }

    #[test]
    fn test_remote_git_prober_debug() {
        if let Some(prober) = RemoteGitProber::new() {
            let debug_str = format!("{:?}", prober);
            assert!(debug_str.contains("RemoteGitProber"));
        }
    }

    #[test]
    fn test_bare_local_git_control_dir_format() {
        // This test will only pass if git plugin is available
        let result = std::panic::catch_unwind(|| BareLocalGitControlDirFormat::new());

        if let Ok(format) = result {
            let _opt_format = format.as_format();
        }
    }

    #[test]
    fn test_remote_git_prober_into_pyobject() {
        if let Some(prober) = RemoteGitProber::new() {
            Python::with_gil(|py| {
                let _pyobj = prober.into_pyobject(py).unwrap();
            });
        }
    }

    #[test]
    fn test_bare_local_git_into_pyobject() {
        let result = std::panic::catch_unwind(|| BareLocalGitControlDirFormat::new());

        if let Ok(format) = result {
            Python::with_gil(|py| {
                let _pyobj = format.into_pyobject(py).unwrap();
            });
        }
    }
}
