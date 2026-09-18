//! Locking of Breezy objects.
use pyo3::prelude::*;

/// Represents a lock on a Breezy object.
///
/// The lock is automatically released when the Lock object is dropped,
/// providing RAII (Resource Acquisition Is Initialization) style locking.
///
/// This ensures that locked resources are properly released even if an error occurs.
pub struct Lock(Py<PyAny>);

impl From<Py<PyAny>> for Lock {
    fn from(obj: Py<PyAny>) -> Self {
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
        Python::attach(|py| {
            if let Err(e) = self.0.call_method0(py, "unlock") {
                // Drop can't propagate errors, so log instead of panicking.
                log::warn!("Lock::unlock failed during drop: {}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::create_standalone_workingtree;
    use crate::branch::PyBranch;
    use crate::workingtree::WorkingTree;

    #[test]
    fn test_lock_drop_survives_already_unlocked() {
        // A branch already unlocked elsewhere must not panic on drop.
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();
        Python::attach(|py| {
            let branch = wt.branch().to_object(py);
            branch.call_method0(py, "lock_write").unwrap();
            branch.call_method0(py, "unlock").unwrap();
            let lock = Lock::from(branch);
            std::mem::drop(lock);
        });
    }
}
