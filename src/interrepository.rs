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
pub trait PyInterRepository: for<'py> IntoPyObject<'py> + std::any::Any + std::fmt::Debug {
    /// Get the underlying Python object for this inter-repository.
    fn to_object(&self, py: Python<'_>) -> Py<PyAny>;
}

/// Generic wrapper for a Python InterRepository object.
///
/// This struct provides a Rust interface to a Breezy InterRepository object.
pub struct GenericInterRepository(Py<PyAny>);

impl<'py> IntoPyObject<'py> for GenericInterRepository {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for GenericInterRepository {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GenericInterRepository(obj.to_owned().unbind()))
    }
}

impl PyInterRepository for GenericInterRepository {
    fn to_object(&self, py: Python<'_>) -> Py<PyAny> {
        self.0.clone_ref(py)
    }
}

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
    pub fn new(obj: Py<PyAny>) -> Self {
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
    Python::attach(|py| {
        let m = py.import("breezy.repository")?;
        let interrepo = m.getattr("InterRepository")?;
        let inter_repository =
            interrepo.call_method1("get", (source.to_object(py), target.to_object(py)))?;
        Ok(
            Box::new(GenericInterRepository::new(inter_repository.unbind()))
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
        Python::attach(|py| -> PyResult<GenericRepository> {
            let source = self.to_object(py).getattr(py, "source")?;
            Ok(GenericRepository::new(source))
        })
        .unwrap()
    }

    fn get_target(&self) -> GenericRepository {
        Python::attach(|py| -> PyResult<GenericRepository> {
            let target = self.to_object(py).getattr(py, "target")?;
            Ok(GenericRepository::new(target))
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
        Python::attach(|py| {
            let get_changed_refs =
                pyo3::types::PyCFunction::new_closure(py, None, None, move |args, _kwargs| {
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

                    Python::attach(|py| -> PyResult<Py<PyAny>> {
                        let ret = PyDict::new(py);

                        for (k, (v, r)) in result {
                            ret.set_item(
                                PyBytes::new(py, k.as_slice()),
                                (
                                    PyBytes::new(py, v.as_slice()),
                                    r.map(|r| r.into_pyobject(py).unwrap().unbind()),
                                ),
                            )?;
                        }

                        // We need to change the return type since pyo3::Python can't be sent between
                        // threads
                        Ok(ret.unbind().into())
                    })
                })
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
