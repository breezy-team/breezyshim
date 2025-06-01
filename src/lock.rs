//! Locking of Breezy objects.
use pyo3::prelude::*;

/// Represents a lock on a Breezy object.
///
/// The lock is automatically released when the Lock object is dropped,
/// providing RAII (Resource Acquisition Is Initialization) style locking.
///
/// This ensures that locked resources are properly released even if an error occurs.
pub struct Lock(PyObject);

impl From<PyObject> for Lock {
    fn from(obj: PyObject) -> Self {
        Lock(obj)
    }
}

impl<'py> IntoPyObject<'py> for Lock {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.clone_ref(py).into_bound(py))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            self.0.call_method0(py, "unlock").unwrap();
        });
    }
}
