//! Debian package releasing
use crate::error::Error;
use crate::tree::PyMutableTree;
use pyo3::prelude::*;

/// Errors that can occur when releasing a Debian package.
#[derive(Debug)]
pub enum ReleaseError {
    /// The file was generated and shouldn't be modified directly.
    GeneratedFile,
    /// An error from the underlying Breezy library.
    BrzError(Error),
}

impl From<Error> for ReleaseError {
    fn from(err: Error) -> Self {
        ReleaseError::BrzError(err)
    }
}

impl std::fmt::Display for ReleaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReleaseError::GeneratedFile => write!(f, "Generated file"),
            ReleaseError::BrzError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for ReleaseError {}

/// Release a Debian package by updating the changelog.
///
/// This function updates the changelog to mark the package as released,
/// setting the appropriate fields like the release date.
///
/// # Arguments
/// * `local_tree` - The tree containing the package to release
/// * `subpath` - Path to the debian directory within the tree
///
/// # Returns
/// The version string of the released package, or an error
pub fn release(
    local_tree: &dyn PyMutableTree,
    subpath: &std::path::Path,
) -> Result<String, ReleaseError> {
    pyo3::import_exception!(debmutate.reformatting, GeneratedFile);
    Python::with_gil(|py| {
        let m = py.import("breezy.plugins.debian.release").unwrap();
        let release = m.getattr("release").unwrap();
        match release.call1((local_tree.to_object(py), subpath)) {
            Ok(result) => Ok(result.extract().unwrap()),
            Err(err) if err.is_instance_of::<GeneratedFile>(py) => Err(ReleaseError::GeneratedFile),
            Err(err) => Err(ReleaseError::BrzError(err.into())),
        }
    })
}
