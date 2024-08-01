use crate::error::Error;
use crate::tree::MutableTree;
use pyo3::prelude::*;

#[derive(Debug)]
pub enum ReleaseError {
    GeneratedFile,
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

pub fn release(
    local_tree: &dyn MutableTree,
    subpath: &std::path::Path,
) -> Result<String, ReleaseError> {
    pyo3::import_exception!(debmutate.reformatting, GeneratedFile);
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.plugins.debian.release").unwrap();
        let release = m.getattr("release").unwrap();
        match release.call1((local_tree.to_object(py), subpath)) {
            Ok(result) => Ok(result.extract().unwrap()),
            Err(err) if err.is_instance_of::<GeneratedFile>(py) => Err(ReleaseError::GeneratedFile),
            Err(err) => Err(ReleaseError::BrzError(err.into())),
        }
    })
}
