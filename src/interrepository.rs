use crate::error::Error;
use crate::repository::Repository;
use pyo3::prelude::*;

pub struct InterRepository(PyObject);

pub fn get(source: &Repository, target: &Repository) -> Result<InterRepository, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.repository")?;
        let interrepo = m.getattr("InterRepository")?;
        let inter_repository = interrepo.call_method1(
            "get_inter_repository",
            (source.to_object(py), target.to_object(py)),
        )?;
        Ok(InterRepository(inter_repository.to_object(py)))
    })
}
