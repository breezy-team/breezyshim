//! Operations between repositories.
use crate::error::Error;
use crate::repository::{GenericRepository, PyRepository};
use crate::RevisionId;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::collections::HashMap;

/// Trait for types that can be converted to Python InterRepository objects.
///
/// This trait is implemented by types that represent a Breezy InterRepository,
/// which handles operations between repositories.
pub trait PyInterRepository: ToPyObject + std::any::Any + std::fmt::Debug {}

/// Generic wrapper for a Python InterRepository object.
///
/// This struct provides a Rust interface to a Breezy InterRepository object.
pub struct GenericInterRepository(PyObject);

impl ToPyObject for GenericInterRepository {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl FromPyObject<'_> for GenericInterRepository {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(GenericInterRepository(obj.to_object(obj.py())))
    }
}

impl PyInterRepository for GenericInterRepository {}

impl GenericInterRepository {
    /// Create a new GenericInterRepository from a Python object.
    ///
    /// # Arguments
    ///
    /// * `obj` - The Python object representing a Breezy InterRepository
    ///
    /// # Returns
    ///
    /// A new GenericInterRepository wrapping the provided Python object
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }
}

impl std::fmt::Debug for GenericInterRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("GenericInterRepository({:?})", self.0))
    }
}

/// Get an InterRepository for operations between two repositories.
///
/// # Arguments
///
/// * `source` - The source repository
/// * `target` - The target repository
///
/// # Returns
///
/// A boxed InterRepository trait object that can perform operations between the repositories
///
/// # Errors
///
/// Returns an error if the operation fails, such as if the repositories are incompatible
pub fn get<S: PyRepository, T: PyRepository>(
    source: &S,
    target: &T,
) -> Result<Box<dyn InterRepository>, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.repository")?;
        let interrepo = m.getattr("InterRepository")?;
        let inter_repository =
            interrepo.call_method1("get", (source.to_object(py), target.to_object(py)))?;
        Ok(
            Box::new(GenericInterRepository::new(inter_repository.to_object(py)))
                as Box<dyn InterRepository>,
        )
    })
}

/// Trait for operations between repositories.
///
/// This trait defines the operations that can be performed between two repositories,
/// such as fetching revisions from one repository to another.
pub trait InterRepository: std::fmt::Debug {
    /// Get the source repository.
    ///
    /// # Returns
    ///
    /// The source repository
    fn get_source(&self) -> GenericRepository;

    /// Get the target repository.
    ///
    /// # Returns
    ///
    /// The target repository
    fn get_target(&self) -> GenericRepository;

    /// Fetch references from the source repository to the target repository.
    ///
    /// # Arguments
    ///
    /// * `get_changed_refs` - A mutex-protected function to get the references to fetch
    /// * `lossy` - If true, lossy conversion is allowed
    /// * `overwrite` - If true, existing references can be overwritten
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error if the operation fails
    // TODO: This should really be on InterGitRepository
    fn fetch_refs(
        &self,
        get_changed_refs: std::sync::Mutex<
            Box<
                dyn FnMut(
                        &HashMap<Vec<u8>, (Vec<u8>, Option<RevisionId>)>,
                    ) -> HashMap<Vec<u8>, (Vec<u8>, Option<RevisionId>)>
                    + Send,
            >,
        >,
        lossy: bool,
        overwrite: bool,
    ) -> Result<(), Error>;
}

impl<T: PyInterRepository> InterRepository for T {
    fn get_source(&self) -> GenericRepository {
        Python::with_gil(|py| -> PyResult<GenericRepository> {
            let source = self.to_object(py).getattr(py, "source")?;
            Ok(GenericRepository::new(source.to_object(py)))
        })
        .unwrap()
    }

    fn get_target(&self) -> GenericRepository {
        Python::with_gil(|py| -> PyResult<GenericRepository> {
            let target = self.to_object(py).getattr(py, "target")?;
            Ok(GenericRepository::new(target.to_object(py)))
        })
        .unwrap()
    }

    // TODO: This should really be on InterGitRepository
    fn fetch_refs(
        &self,
        get_changed_refs: std::sync::Mutex<
            Box<
                dyn FnMut(
                        &HashMap<Vec<u8>, (Vec<u8>, Option<RevisionId>)>,
                    ) -> HashMap<Vec<u8>, (Vec<u8>, Option<RevisionId>)>
                    + Send,
            >,
        >,
        lossy: bool,
        overwrite: bool,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
            let get_changed_refs = pyo3::types::PyCFunction::new_closure_bound(
                py,
                None,
                None,
                move |args, _kwargs| {
                    let refs = args
                        .extract::<(HashMap<Vec<u8>, (Vec<u8>, Option<RevisionId>)>,)>()
                        .unwrap()
                        .0;
                    // Call get_changed_refs
                    let result = if let Ok(mut get_changed_refs) = get_changed_refs.lock() {
                        get_changed_refs(&refs)
                    } else {
                        refs
                    };

                    Python::with_gil(|py| -> PyResult<PyObject> {
                        let ret = PyDict::new_bound(py);

                        for (k, (v, r)) in result {
                            ret.set_item(
                                PyBytes::new_bound(py, k.as_slice()),
                                (
                                    PyBytes::new_bound(py, v.as_slice()),
                                    r.map(|r| r.into_py(py)),
                                ),
                            )?;
                        }

                        // We need to change the return type since pyo3::Python can't be sent between
                        // threads
                        Ok(ret.into_py(py))
                    })
                },
            )
            .unwrap();
            self.to_object(py).call_method1(
                py,
                "fetch_refs",
                (get_changed_refs, lossy, overwrite),
            )?;
            Ok(())
        })
    }
}
