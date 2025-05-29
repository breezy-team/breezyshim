//! Bazaar-specific functionality.
//!
//! This module provides types and functions for working with Bazaar repositories.
//! Bazaar was the original version control system that Breezy evolved from.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

pub mod tree;

/// A Bazaar file identifier.
///
/// Bazaar uses unique identifiers for files, which allow it to track files across
/// renames and other operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileId(Vec<u8>);

impl Default for FileId {
    fn default() -> Self {
        Self::new()
    }
}

impl FileId {
    /// Create a new empty file identifier.
    ///
    /// # Returns
    ///
    /// A new FileId instance with an empty identifier.
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl From<&str> for FileId {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<String> for FileId {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<&[u8]> for FileId {
    fn from(s: &[u8]) -> Self {
        Self(s.to_vec())
    }
}

impl From<Vec<u8>> for FileId {
    fn from(s: Vec<u8>) -> Self {
        Self(s)
    }
}

impl From<FileId> for Vec<u8> {
    fn from(s: FileId) -> Self {
        s.0
    }
}

impl From<FileId> for String {
    fn from(s: FileId) -> Self {
        String::from_utf8(s.0).unwrap()
    }
}

impl<'py> pyo3::IntoPyObject<'py> for FileId {
    type Target = pyo3::PyAny;
    type Output = pyo3::Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: pyo3::Python<'py>) -> Result<Self::Output, Self::Error> {
        self.0.into_pyobject(py)
    }
}

impl pyo3::FromPyObject<'_> for FileId {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let bytes = ob.extract::<Vec<u8>>()?;
        Ok(Self(bytes))
    }
}

/// Generate a Bazaar revision identifier.
///
/// # Parameters
///
/// * `username` - The username to associate with the revision.
/// * `timestamp` - Optional timestamp for the revision, in seconds since the epoch.
///
/// # Returns
///
/// A byte vector containing the generated revision identifier.
pub fn gen_revision_id(username: &str, timestamp: Option<usize>) -> Vec<u8> {
    Python::with_gil(|py| {
        let m = py.import("breezy.bzr.generate_ids").unwrap();
        let gen_revision_id = m.getattr("gen_revision_id").unwrap();
        gen_revision_id
            .call1((username, timestamp))
            .unwrap()
            .extract()
            .unwrap()
    })
}

#[test]
fn test_gen_revision_id() {
    gen_revision_id("user", None);
}

/// Generate a Bazaar file identifier from a name.
///
/// # Parameters
///
/// * `name` - The name to use for generating the file identifier.
///
/// # Returns
///
/// A byte vector containing the generated file identifier.
pub fn gen_file_id(name: &str) -> Vec<u8> {
    Python::with_gil(|py| {
        let m = py.import("breezy.bzr.generate_ids").unwrap();
        let gen_file_id = m.getattr("gen_file_id").unwrap();
        gen_file_id.call1((name,)).unwrap().extract().unwrap()
    })
}

#[test]
fn test_file_id() {
    gen_file_id("somename");
}

/// A prober for remote Bazaar repositories.
///
/// This prober can detect whether a remote location contains a Bazaar repository.
pub struct RemoteBzrProber(PyObject);

impl RemoteBzrProber {
    /// Create a new RemoteBzrProber.
    ///
    /// # Returns
    ///
    /// Some(RemoteBzrProber) if Bazaar is available, None otherwise.
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.bzr") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.bzr");
                    }
                }
            };
            let prober = m
                .getattr("RemoteBzrProber")
                .expect("Failed to get RemoteBzrProber");
            Some(Self(prober.unbind()))
        })
    }
}

impl FromPyObject<'_> for RemoteBzrProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.clone().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for RemoteBzrProber {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}


impl std::fmt::Debug for RemoteBzrProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("RemoteBzrProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for RemoteBzrProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}
