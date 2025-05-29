//! UI-layer location handling
use pyo3::prelude::*;
use url::Url;

/// Convert a CVS root string to a URL.
///
/// # Arguments
///
/// * `cvsroot` - The CVS root string to convert
///
/// # Returns
///
/// A URL representing the CVS repository location
pub fn cvs_to_url(cvsroot: &str) -> Url {
    Python::with_gil(|py| {
        let breezy_location = py.import("breezy.location").unwrap();

        breezy_location
            .call_method1("cvs_to_url", (cvsroot,))
            .unwrap()
            .extract::<String>()
            .unwrap()
            .parse()
            .unwrap()
    })
}

#[test]
fn test_cvs_to_url() {
    assert_eq!(
        cvs_to_url(":pserver:anonymous@localhost:/var/lib/cvs"),
        Url::parse("cvs+pserver://anonymous@localhost/var/lib/cvs").unwrap()
    );
}

/// Convert an RCP (remote copy) location string to a URL.
///
/// RCP locations are in the format "user@host:/path/to/repo" and are converted
/// to URLs like "ssh://user@host/path/to/repo".
///
/// # Arguments
///
/// * `rcp_location` - The RCP location string to convert
///
/// # Returns
///
/// A Result containing the converted URL or an error string
pub fn rcp_location_to_url(rcp_location: &str) -> Result<Url, String> {
    Python::with_gil(|py| {
        let breezy_location = py.import("breezy.location").unwrap();

        Ok(breezy_location
            .call_method1("rcp_location_to_url", (rcp_location,))
            .map_err(|e| e.to_string())?
            .extract::<String>()
            .unwrap()
            .parse()
            .unwrap())
    })
}

#[test]
fn test_rcp_location_to_url() {
    assert_eq!(
        rcp_location_to_url("user@host:/path/to/repo").unwrap(),
        Url::parse("ssh://user@host/path/to/repo").unwrap()
    );
}

/// Trait for types that can be converted to a location that Breezy understands.
///
/// This trait is implemented by types that can be converted to a string
/// representation that can be used as a location in Breezy API calls.
pub trait AsLocation {
    /// Convert the object to a Python object representing a location.
    ///
    /// # Returns
    ///
    /// A Python object (string) representing the location
    fn as_location(&self) -> PyObject;
}

impl AsLocation for &url::Url {
    fn as_location(&self) -> PyObject {
        Python::with_gil(|py| {
            pyo3::types::PyString::new(py, &self.to_string())
                .unbind()
                .into()
        })
    }
}

#[test]
fn test_as_location_url() {
    Python::with_gil(|py| {
        assert_eq!(
            Url::parse("ssh://user@host/path/to/repo")
                .unwrap()
                .as_ref()
                .as_location()
                .extract::<String>(py)
                .unwrap(),
            "ssh://user@host/path/to/repo"
        );
    });
}

impl AsLocation for &str {
    fn as_location(&self) -> PyObject {
        Python::with_gil(|py| pyo3::types::PyString::new(py, self).unbind().into())
    }
}

#[test]
fn test_as_location_str() {
    Python::with_gil(|py| {
        assert_eq!(
            "ssh://user@host/path/to/repo"
                .as_location()
                .extract::<String>(py)
                .unwrap(),
            "ssh://user@host/path/to/repo"
        );
    });
}

impl AsLocation for &std::path::Path {
    fn as_location(&self) -> PyObject {
        Python::with_gil(|py| {
            pyo3::types::PyString::new(py, self.to_str().unwrap())
                .unbind()
                .into()
        })
    }
}

#[test]
fn test_as_location_path() {
    Python::with_gil(|py| {
        assert_eq!(
            std::path::Path::new("/path/to/repo")
                .as_location()
                .extract::<String>(py)
                .unwrap(),
            "/path/to/repo"
        );
    });
}
