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
            let m = match py.import_bound("breezy.git") {
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
            Some(Self(prober.to_object(py)))
        })
    }
}

impl FromPyObject<'_> for RemoteGitProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_object(obj.py())))
    }
}

impl ToPyObject for RemoteGitProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl std::fmt::Debug for RemoteGitProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("RemoteGitProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for RemoteGitProber {}

/// Format for bare local Git repositories.
pub struct BareLocalGitControlDirFormat(PyObject);

impl BareLocalGitControlDirFormat {
    /// Create a new BareLocalGitControlDirFormat.
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py
                .import_bound("breezy.git")
                .expect("Failed to import breezy.git");
            let format = m
                .getattr("BareLocalGitControlDirFormat")
                .expect("Failed to get BareLocalGitControlDirFormat");

            Self(
                format
                    .call0()
                    .expect("Failed to create BareLocalGitControlDirFormat")
                    .to_object(py),
            )
        })
    }
}

impl ToPyObject for BareLocalGitControlDirFormat {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl crate::controldir::AsFormat for BareLocalGitControlDirFormat {
    fn as_format(&self) -> Option<crate::controldir::ControlDirFormat> {
        Some(Python::with_gil(|py| {
            crate::controldir::ControlDirFormat::from(self.0.clone_ref(py))
        }))
    }
}
