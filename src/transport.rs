//! Transport module
use crate::error::Error;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::path::{Path, PathBuf};

/// A transport represents a way to access content in a branch.
pub struct Transport(Py<PyAny>);

impl Transport {
    /// Create a new transport from a Python object.
    pub fn new(obj: Py<PyAny>) -> Self {
        Transport(obj)
    }

    /// Get the underlying Py<PyAny>.
    pub(crate) fn as_pyobject(&self) -> &Py<PyAny> {
        &self.0
    }

    /// Get the base URL of this transport.
    pub fn base(&self) -> url::Url {
        pyo3::Python::attach(|py| {
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
        pyo3::Python::attach(|py| {
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
        pyo3::Python::attach(|py| {
            Ok(self
                .0
                .call_method1(py, "local_abspath", (path.to_string_lossy().as_ref(),))?
                .extract::<PathBuf>(py)?)
        })
    }

    /// Check if a path exists in this transport.
    pub fn has(&self, path: &str) -> Result<bool, Error> {
        pyo3::Python::attach(|py| {
            Ok(self
                .0
                .call_method1(py, "has", (path,))?
                .extract::<bool>(py)
                .unwrap())
        })
    }

    /// Ensure the base directory exists.
    pub fn ensure_base(&self) -> Result<(), Error> {
        pyo3::Python::attach(|py| {
            self.0.call_method0(py, "ensure_base")?;
            Ok(())
        })
    }

    /// Create all the directories leading up to the final path component.
    pub fn create_prefix(&self) -> Result<(), Error> {
        pyo3::Python::attach(|py| {
            self.0.call_method0(py, "create_prefix")?;
            Ok(())
        })
    }

    /// Create a new transport with a different path.
    pub fn clone(&self, path: &str) -> Result<Transport, Error> {
        pyo3::Python::attach(|py| {
            let o = self.0.call_method1(py, "clone", (path,))?;
            Ok(Transport(o))
        })
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Transport {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Transport(obj.to_owned().unbind()))
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

/// Build the Python list that is handed to breezy as `possible_transports`.
pub(crate) fn possible_transports_to_py<'py>(
    py: Python<'py>,
    possible_transports: &[Transport],
) -> PyResult<Bound<'py, PyList>> {
    PyList::new(py, possible_transports.iter().map(|t| t.0.clone_ref(py)))
}

/// Copy the list breezy was given back, so the transports it added are kept.
pub(crate) fn possible_transports_from_py(
    list: &Bound<PyList>,
    possible_transports: &mut Vec<Transport>,
) {
    possible_transports.clear();
    possible_transports.extend(list.iter().map(|t| Transport(t.unbind())));
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
    pyo3::Python::attach(|py| {
        let urlutils = py.import("breezy.transport").unwrap();
        let kwargs = PyDict::new(py);
        let list = possible_transports
            .as_deref()
            .map(|t| possible_transports_to_py(py, t))
            .transpose()?;
        kwargs.set_item("possible_transports", list.as_ref())?;
        let o = urlutils.call_method("get_transport", (url.to_string(),), Some(&kwargs));
        if let (Some(list), Some(possible_transports)) = (list.as_ref(), possible_transports) {
            possible_transports_from_py(list, possible_transports);
        }
        Ok(Transport(o?.unbind()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_transport() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();
        let transport = get_transport(&url, None).unwrap();

        // Test base URL
        let base = transport.base();
        assert!(base.to_string().starts_with("file://"));
    }

    #[test]
    fn test_transport_is_local() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();
        let transport = get_transport(&url, None).unwrap();

        assert!(transport.is_local());
    }

    #[test]
    fn test_transport_local_abspath() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();
        let transport = get_transport(&url, None).unwrap();

        let path = Path::new("test.txt");
        let abspath = transport.local_abspath(path).unwrap();
        assert!(abspath.is_absolute());
    }

    #[test]
    fn test_transport_has() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();
        let transport = get_transport(&url, None).unwrap();

        // Test for non-existent file
        let exists = transport.has("nonexistent.txt").unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_transport_ensure_base() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();
        let transport = get_transport(&url, None).unwrap();

        let result = transport.ensure_base();
        assert!(result.is_ok());
    }

    #[test]
    fn test_transport_create_prefix() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();
        let transport = get_transport(&url, None).unwrap();

        let result = transport.create_prefix();
        assert!(result.is_ok());
    }

    #[test]
    fn test_transport_clone() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();
        let transport = get_transport(&url, None).unwrap();

        let cloned = transport.clone("subdir").unwrap();
        let cloned_base = cloned.base();
        assert!(cloned_base.to_string().contains("subdir"));
    }

    #[test]
    fn test_transport_into_pyobject() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();
        let transport = get_transport(&url, None).unwrap();

        Python::attach(|py| {
            let _pyobj = transport.into_pyobject(py).unwrap();
        });
    }

    #[test]
    fn test_get_transport_with_possible_transports() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();

        let mut possible_transports = vec![];
        let transport = get_transport(&url, Some(&mut possible_transports)).unwrap();

        let base = transport.base();
        assert!(base.to_string().starts_with("file://"));
    }

    #[test]
    fn test_get_transport_fills_possible_transports() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(td.path()).unwrap();

        let mut possible_transports = vec![];
        let transport = get_transport(&url, Some(&mut possible_transports)).unwrap();

        assert_eq!(possible_transports.len(), 1);
        assert_eq!(possible_transports[0].base(), transport.base());
    }
}
