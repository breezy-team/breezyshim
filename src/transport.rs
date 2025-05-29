//! Transport module
use crate::error::Error;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::{Path, PathBuf};

/// A transport represents a way to access content in a branch.
pub struct Transport(PyObject);

impl Transport {
    /// Create a new transport from a Python object.
    pub fn new(obj: PyObject) -> Self {
        Transport(obj)
    }

    /// Get the underlying PyObject.
    pub(crate) fn as_pyobject(&self) -> &PyObject {
        &self.0
    }

    /// Get the base URL of this transport.
    pub fn base(&self) -> url::Url {
        pyo3::Python::with_gil(|py| {
            self.as_pyobject()
                .getattr(py, "base")
                .unwrap()
                .extract::<String>(py)
                .unwrap()
                .parse()
                .unwrap()
        })
    }

    /// Check if this is a local transport.
    pub fn is_local(&self) -> bool {
        pyo3::import_exception!(breezy.errors, NotLocalUrl);
        pyo3::Python::with_gil(|py| {
            self.0
                .call_method1(py, "local_abspath", (".",))
                .map(|_| true)
                .or_else(|e| {
                    if e.is_instance_of::<NotLocalUrl>(py) {
                        Ok::<_, PyErr>(false)
                    } else {
                        panic!("Unexpected error: {:?}", e)
                    }
                })
                .unwrap()
        })
    }

    /// Get the local absolute path for a path within this transport.
    pub fn local_abspath(&self, path: &Path) -> Result<PathBuf, Error> {
        pyo3::Python::with_gil(|py| {
            Ok(self
                .0
                .call_method1(py, "local_abspath", (path,))
                .unwrap()
                .extract::<PathBuf>(py)
                .unwrap())
        })
    }

    /// Check if a path exists in this transport.
    pub fn has(&self, path: &str) -> Result<bool, Error> {
        pyo3::Python::with_gil(|py| {
            Ok(self
                .0
                .call_method1(py, "has", (path,))?
                .extract::<bool>(py)
                .unwrap())
        })
    }

    /// Ensure the base directory exists.
    pub fn ensure_base(&self) -> Result<(), Error> {
        pyo3::Python::with_gil(|py| {
            self.0.call_method0(py, "ensure_base")?;
            Ok(())
        })
    }

    /// Create all the directories leading up to the final path component.
    pub fn create_prefix(&self) -> Result<(), Error> {
        pyo3::Python::with_gil(|py| {
            self.0.call_method0(py, "create_prefix")?;
            Ok(())
        })
    }

    /// Create a new transport with a different path.
    pub fn clone(&self, path: &str) -> Result<Transport, Error> {
        pyo3::Python::with_gil(|py| {
            let o = self.0.call_method1(py, "clone", (path,))?;
            Ok(Transport(o))
        })
    }
}

impl FromPyObject<'_> for Transport {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Transport(obj.clone().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for Transport {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

/// Get a transport for a given URL.
///
/// # Arguments
/// * `url` - The URL to get a transport for
/// * `possible_transports` - Optional list of transports to try reusing
pub fn get_transport(
    url: &url::Url,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<Transport, Error> {
    pyo3::Python::with_gil(|py| {
        let urlutils = py.import("breezy.transport").unwrap();
        let kwargs = PyDict::new(py);
        kwargs.set_item(
            "possible_transports",
            possible_transports.map(|t| {
                t.iter()
                    .map(|t| t.0.clone_ref(py))
                    .collect::<Vec<PyObject>>()
            }),
        )?;
        let o = urlutils.call_method("get_transport", (url.to_string(),), Some(&kwargs))?;
        Ok(Transport(o.unbind()))
    })
}
