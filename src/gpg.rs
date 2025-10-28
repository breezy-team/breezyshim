//! GPG related functions and types.
use crate::repository::PyRepository;
use crate::RevisionId;
use pyo3::import_exception;
use pyo3::prelude::*;

#[derive(Debug)]
/// Errors that can occur when working with GPG.
pub enum Error {
    /// GPG is not installed on the system.
    GPGNotInstalled,
}

#[derive(Debug)]
/// GPG signing modes.
pub enum Mode {
    /// Normal signing mode.
    Normal,
    /// Detached signature mode.
    Detach,
    /// Clear signature mode.
    Clear,
}

#[derive(Debug)]
/// Status of a GPG signature verification.
pub enum Status {
    /// Signature is valid.
    Valid,
    /// Signature key is missing from the keyring.
    KeyMissing(String),
    /// Signature with the specified key is not valid.
    NotValid(String),
    /// Content is not signed.
    NotSigned,
    /// Signature key has expired.
    Expired(String),
}

import_exception!(breezy.gpg, GPGNotInstalled);

impl From<PyErr> for Error {
    fn from(e: PyErr) -> Self {
        Python::attach(|py| {
            if e.is_instance_of::<GPGNotInstalled>(py) {
                Error::GPGNotInstalled
            } else {
                panic!("unexpected exception: {:?}", e)
            }
        })
    }
}

/// Strategy for handling GPG signatures.
pub struct GPGStrategy(Py<PyAny>);

impl GPGStrategy {
    fn to_object(&self) -> &Py<PyAny> {
        &self.0
    }
    /// Create a new GPG strategy with the given branch configuration.
    pub fn new(branch_config: &crate::config::BranchConfig) -> Self {
        Python::attach(|py| {
            let gpg = PyModule::import(py, "breezy.gpg").unwrap();
            let gpg_strategy = gpg.getattr("GPGStrategy").unwrap();
            let branch_config = branch_config.clone().into_pyobject(py).unwrap().unbind();
            let strategy = gpg_strategy.call1((branch_config,)).unwrap();
            GPGStrategy(strategy.unbind())
        })
    }

    /// Set the GPG keys that are acceptable for validating signatures.
    pub fn set_acceptable_keys(&self, keys: &[String]) {
        Python::attach(|py| {
            self.0
                .call_method1(py, "set_acceptable_keys", (keys.join(","),))
                .unwrap();
        })
    }
}

impl<'py> IntoPyObject<'py> for GPGStrategy {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for GPGStrategy {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GPGStrategy(ob.to_owned().unbind()))
    }
}

#[derive(Debug)]
/// Result of verifying a GPG signature.
pub enum VerificationResult {
    /// Signature is valid with the specified key.
    Valid(String),
    /// Signature uses a key that is missing from the keyring.
    KeyMissing(String),
    /// Signature with the given key is not valid.
    NotValid(String),
    /// Content is not signed with a GPG signature.
    NotSigned,
    /// Signature is from an expired key.
    Expired(String),
}

impl VerificationResult {
    /// Returns the key string for the signature if available.
    pub fn key(&self) -> Option<&str> {
        match self {
            VerificationResult::Valid(key) => Some(key),
            VerificationResult::KeyMissing(key) => Some(key),
            VerificationResult::NotValid(key) => Some(key),
            VerificationResult::Expired(key) => Some(key),
            _ => None,
        }
    }

    /// Check if the verification result indicates a valid signature.
    pub fn is_valid(&self) -> bool {
        matches!(self, VerificationResult::Valid(_))
    }

    /// Check if the verification result indicates a missing key.
    pub fn is_key_missing(&self) -> bool {
        matches!(self, VerificationResult::KeyMissing(_))
    }

    /// Check if the verification result indicates an invalid signature.
    pub fn is_not_valid(&self) -> bool {
        matches!(self, VerificationResult::NotValid(_))
    }

    /// Check if the verification result indicates the content is not signed.
    pub fn is_not_signed(&self) -> bool {
        matches!(self, VerificationResult::NotSigned)
    }

    /// Check if the verification result indicates an expired key.
    pub fn is_expired(&self) -> bool {
        matches!(self, VerificationResult::Expired(_))
    }
}

/// Bulk verify GPG signatures for a set of revisions.
///
/// # Arguments
///
/// * `repository` - The repository containing the revisions
/// * `revids` - List of revision IDs to verify signatures for
/// * `strategy` - GPG strategy to use for verification
///
/// # Returns
///
/// A vector of tuples containing revision IDs and their verification results
pub fn bulk_verify_signatures<R: PyRepository>(
    repository: &R,
    revids: &[&RevisionId],
    strategy: &GPGStrategy,
) -> Result<Vec<(RevisionId, VerificationResult)>, Error> {
    Python::attach(|py| {
        let gpg = PyModule::import(py, "breezy.gpg").unwrap();
        let bulk_verify_signatures = gpg.getattr("bulk_verify_signatures").unwrap();
        let r = bulk_verify_signatures
            .call1((
                repository.to_object(py),
                revids
                    .iter()
                    .map(|r| (*r).clone().into_pyobject(py).unwrap())
                    .collect::<Vec<_>>(),
                strategy.to_object(),
            ))
            .map_err(|e| -> Error { e.into() })
            .unwrap();

        let (_count, result, _all_verifiable) = r
            .extract::<(Py<PyAny>, Vec<(RevisionId, isize, String)>, bool)>()
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

/// Context for interacting with GPG.
pub struct GPGContext(Py<PyAny>);

/// Represents a GPG key.
pub struct GPGKey {
    /// Fingerprint of the GPG key.
    pub fpr: String,
}

impl<'a, 'py> FromPyObject<'a, 'py> for GPGKey {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GPGKey {
            fpr: ob.getattr("fpr").unwrap().extract().unwrap(),
        })
    }
}

impl GPGContext {
    /// Create a new GPG context.
    pub fn new() -> Self {
        Python::attach(|py| {
            let gpg = PyModule::import(py, "gpg").unwrap();
            let gpg_context = gpg.getattr("Context").unwrap();
            let context = gpg_context.call0().unwrap();
            GPGContext(context.unbind())
        })
    }

    /// List GPG keys.
    ///
    /// # Arguments
    ///
    /// * `secret` - If true, list only secret keys. Otherwise, list all keys.
    ///
    /// # Returns
    ///
    /// A vector of GPG keys.
    pub fn keylist(&self, secret: bool) -> Vec<GPGKey> {
        Python::attach(|py| {
            self.0
                .call_method1(py, "keylist", (secret,))
                .unwrap()
                .extract::<Vec<GPGKey>>(py)
                .unwrap()
        })
    }

    /// Export the minimal form of a GPG key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key ID or fingerprint to export
    ///
    /// # Returns
    ///
    /// The exported key data as a byte vector.
    pub fn key_export_minimal(&self, key: &str) -> Vec<u8> {
        Python::attach(|py| {
            self.0
                .call_method1(py, "key_export_minimal", (key,))
                .unwrap()
                .extract::<Vec<u8>>(py)
                .unwrap()
        })
    }
}
