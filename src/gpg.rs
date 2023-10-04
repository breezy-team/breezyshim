use crate::repository::Repository;
use crate::RevisionId;
use pyo3::import_exception;
use pyo3::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    GPGNotInstalled,
}

#[derive(Debug)]
pub enum Mode {
    Normal,
    Detach,
    Clear,
}

#[derive(Debug)]
pub enum Status {
    Valid,
    KeyMissing(String),
    NotValid(String),
    NotSigned,
    Expired(String),
}

import_exception!(breezy.gpg, GPGNotInstalled);

impl From<PyErr> for Error {
    fn from(e: PyErr) -> Self {
        Python::with_gil(|py| {
            if e.is_instance_of::<GPGNotInstalled>(py) {
                Error::GPGNotInstalled
            } else {
                panic!("unexpected exception: {:?}", e)
            }
        })
    }
}

pub struct GPGStrategy(PyObject);

impl ToPyObject for GPGStrategy {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl FromPyObject<'_> for GPGStrategy {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        Ok(GPGStrategy(ob.to_object(ob.py())))
    }
}

pub struct VerificationResult {}

impl FromPyObject<'_> for VerificationResult {
    fn extract(_ob: &PyAny) -> PyResult<Self> {
        Ok(VerificationResult {})
    }
}

impl ToPyObject for VerificationResult {
    fn to_object(&self, py: Python) -> PyObject {
        py.None()
    }
}

pub fn bulk_verify_signatures(
    repository: &Repository,
    revids: &[&RevisionId],
    strategy: &GPGStrategy,
) -> Result<
    (
        HashMap<String, usize>,
        Vec<(RevisionId, VerificationResult, String)>,
        bool,
    ),
    Error,
> {
    Python::with_gil(|py| {
        let gpg = PyModule::import(py, "breezy.gpg").unwrap();
        let bulk_verify_signatures = gpg.getattr("bulk_verify_signatures").unwrap();
        let r = bulk_verify_signatures
            .call1((
                repository.to_object(py),
                revids.iter().map(|r| r.to_object(py)).collect::<Vec<_>>(),
                strategy.to_object(py),
            ))
            .map_err(|e| -> Error { e.into() })?;

        Ok(r.extract::<(
            HashMap<String, usize>,
            Vec<(RevisionId, VerificationResult, String)>,
            bool,
        )>()
        .unwrap())
    })
}
