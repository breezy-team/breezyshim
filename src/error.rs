use pyo3::import_exception;
use pyo3::PyErr;

import_exception!(breezy.errors, UnknownFormatError);
import_exception!(breezy.errors, NotBranchError);
import_exception!(breezy.controldir, NoColocatedBranchSupport);
import_exception!(breezy.errors, DependencyNotPresent);

#[derive(Debug)]
pub enum Error {
    Other(PyErr),
    UnknownFormat(String),
    NotBranchError(String, Option<String>),
    NoColocatedBranchSupport,
    DependencyNotPresent(String, String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Other(e) => write!(f, "Error::Other({})", e),
            Self::UnknownFormat(s) => write!(f, "Unknown format: {}", s),
            Self::NotBranchError(path, detail) => {
                if let Some(detail) = detail {
                    write!(f, "Not a branch: {}: {}", path, detail)
                } else {
                    write!(f, "Not a branch: {}", path)
                }
            }
            Self::NoColocatedBranchSupport => write!(f, "No colocated branch support"),
            Self::DependencyNotPresent(d, r) => write!(f, "Dependency {} not present: {}", d, r),
        }
    }
}

impl std::error::Error for Error {}

impl From<PyErr> for Error {
    fn from(err: PyErr) -> Self {
        pyo3::Python::with_gil(|py| {
            let value = err.value(py);
            if err.is_instance_of::<UnknownFormatError>(py) {
                Error::UnknownFormat(value.getattr("format").unwrap().extract().unwrap())
            } else if err.is_instance_of::<NotBranchError>(py) {
                Error::NotBranchError(
                    value.getattr("path").unwrap().extract().unwrap(),
                    value.getattr("details").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<NoColocatedBranchSupport>(py) {
                Error::NoColocatedBranchSupport
            } else if err.is_instance_of::<DependencyNotPresent>(py) {
                Error::DependencyNotPresent(
                    value.getattr("library").unwrap().extract().unwrap(),
                    value.getattr("error").unwrap().extract().unwrap(),
                )
            } else {
                Self::Other(err)
            }
        })
    }
}

impl From<Error> for PyErr {
    fn from(e: Error) -> Self {
        match e {
            Error::Other(e) => e,
            Error::UnknownFormat(s) => UnknownFormatError::new_err((s,)),
            Error::NotBranchError(path, details) => NotBranchError::new_err((path, details)),
            Error::NoColocatedBranchSupport => NoColocatedBranchSupport::new_err(()),
            Error::DependencyNotPresent(library, error) => {
                DependencyNotPresent::new_err((library, error))
            }
        }
    }
}
