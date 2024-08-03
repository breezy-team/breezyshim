use crate::error::Error;
use crate::repository::Repository;
use crate::RevisionId;
use pyo3::prelude::*;
use std::collections::HashMap;

pub struct PyInterRepository(PyObject);

impl ToPyObject for PyInterRepository {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl InterRepository for PyInterRepository {}

pub fn get(source: &Repository, target: &Repository) -> Result<Box<dyn InterRepository>, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.repository")?;
        let interrepo = m.getattr("InterRepository")?;
        let inter_repository =
            interrepo.call_method1("get", (source.to_object(py), target.to_object(py)))?;
        Ok(Box::new(PyInterRepository(inter_repository.to_object(py))) as Box<dyn InterRepository>)
    })
}

pub trait InterRepository: ToPyObject {
    fn get_source(&self) -> Repository {
        Python::with_gil(|py| -> PyResult<Repository> {
            let source = self.to_object(py).getattr(py, "source")?;
            Ok(Repository::new(source.to_object(py)))
        })
        .unwrap()
    }

    fn get_target(&self) -> Repository {
        Python::with_gil(|py| -> PyResult<Repository> {
            let target = self.to_object(py).getattr(py, "target")?;
            Ok(Repository::new(target.to_object(py)))
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
                    if let Ok(mut get_changed_refs) = get_changed_refs.lock() {
                        get_changed_refs(&refs)
                    } else {
                        refs
                    }
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
