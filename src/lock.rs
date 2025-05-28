//! Locking of Breezy objects.
use pyo3::prelude::*;

crate::wrapped_py!(Lock);

impl Drop for Lock {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            self.0.call_method0(py, "unlock").unwrap();
        });
    }
}
