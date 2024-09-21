//! GPG related functions and types.
use crate::repository::Repository;
use crate::RevisionId;
use pyo3::import_exception;
use pyo3::prelude::*;

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

impl GPGStrategy {
    pub fn new(branch_config: &crate::config::BranchConfig) -> Self {
        Python::with_gil(|py| {
            let gpg = PyModule::import_bound(py, "breezy.gpg").unwrap();
            let gpg_strategy = gpg.getattr("GPGStrategy").unwrap();
            let branch_config = branch_config.to_object(py);
            let strategy = gpg_strategy.call1((branch_config,)).unwrap();
            GPGStrategy(strategy.to_object(py))
        })
    }

    pub fn set_acceptable_keys(&self, keys: &[String]) {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "set_acceptable_keys", (keys.join(","),))
                .unwrap();
        })
    }
}

impl ToPyObject for GPGStrategy {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl FromPyObject<'_> for GPGStrategy {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(GPGStrategy(ob.to_object(ob.py())))
    }
}

#[derive(Debug)]
pub enum VerificationResult {
    Valid(String),
    KeyMissing(String),
    NotValid(String),
    NotSigned,
    Expired(String),
}

impl VerificationResult {
    pub fn key(&self) -> Option<&str> {
        match self {
            VerificationResult::Valid(key) => Some(key),
            VerificationResult::KeyMissing(key) => Some(key),
            VerificationResult::NotValid(key) => Some(key),
            VerificationResult::Expired(key) => Some(key),
            _ => None,
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self, VerificationResult::Valid(_))
    }

    pub fn is_key_missing(&self) -> bool {
        matches!(self, VerificationResult::KeyMissing(_))
    }

    pub fn is_not_valid(&self) -> bool {
        matches!(self, VerificationResult::NotValid(_))
    }

    pub fn is_not_signed(&self) -> bool {
        matches!(self, VerificationResult::NotSigned)
    }

    pub fn is_expired(&self) -> bool {
        matches!(self, VerificationResult::Expired(_))
    }
}

pub fn bulk_verify_signatures(
    repository: &Repository,
    revids: &[&RevisionId],
    strategy: &GPGStrategy,
) -> Result<Vec<(RevisionId, VerificationResult)>, Error> {
    Python::with_gil(|py| {
        let gpg = PyModule::import_bound(py, "breezy.gpg").unwrap();
        let bulk_verify_signatures = gpg.getattr("bulk_verify_signatures").unwrap();
        let r = bulk_verify_signatures
            .call1((
                repository.to_object(py),
                revids.iter().map(|r| r.to_object(py)).collect::<Vec<_>>(),
                strategy.to_object(py),
            ))
            .map_err(|e| -> Error { e.into() })
            .unwrap();

        let (_count, result, _all_verifiable) = r
            .extract::<(PyObject, Vec<(RevisionId, isize, String)>, bool)>()
            .unwrap();

        let result: Vec<(RevisionId, VerificationResult)> = result
            .into_iter()
            .map(|(revid, status, key)| {
                let status = match status {
                    0 => VerificationResult::Valid(key),
                    1 => VerificationResult::KeyMissing(key),
                    2 => VerificationResult::NotValid(key),
                    3 => VerificationResult::NotSigned,
                    4 => VerificationResult::Expired(key),
                    _ => panic!("unexpected status: {}", status),
                };
                (revid, status)
            })
            .collect::<Vec<_>>();

        Ok(result)
    })
}

pub struct GPGContext(PyObject);

pub struct GPGKey {
    pub fpr: String,
}

impl FromPyObject<'_> for GPGKey {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(GPGKey {
            fpr: ob.getattr("fpr").unwrap().extract().unwrap(),
        })
    }
}

impl GPGContext {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let gpg = PyModule::import_bound(py, "gpg").unwrap();
            let gpg_context = gpg.getattr("Context").unwrap();
            let context = gpg_context.call0().unwrap();
            GPGContext(context.to_object(py))
        })
    }

    pub fn keylist(&self, secret: bool) -> Vec<GPGKey> {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "keylist", (secret,))
                .unwrap()
                .extract::<Vec<GPGKey>>(py)
                .unwrap()
        })
    }

    pub fn key_export_minimal(&self, key: &str) -> Vec<u8> {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "key_export_minimal", (key,))
                .unwrap()
                .extract::<Vec<u8>>(py)
                .unwrap()
        })
    }
}
