use crate::error::Error;
use crate::repository::Repository;
use pyo3::prelude::*;

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
        let inter_repository = interrepo.call_method1(
            "get_inter_repository",
            (source.to_object(py), target.to_object(py)),
        )?;
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
}
