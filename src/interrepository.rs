//! Operations between repositories.
use crate::error::Error;
use crate::repository::{GenericRepository, PyRepository};
use crate::RevisionId;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::collections::HashMap;

pub trait PyInterRepository: ToPyObject + std::any::Any + std::fmt::Debug {}

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
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }
}

impl std::fmt::Debug for GenericInterRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("GenericInterRepository({:?})", self.0))
    }
}

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

pub trait InterRepository: std::fmt::Debug {
    fn get_source(&self) -> GenericRepository;
    fn get_target(&self) -> GenericRepository;

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
