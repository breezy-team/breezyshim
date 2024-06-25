use pyo3::import_exception;
use pyo3::PyErr;

import_exception!(breezy.errors, UnknownFormatError);

#[derive(Debug)]
pub enum Error {
    Other(PyErr),
    UnknownFormat(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Other(e) => write!(f, "Error::Other({})", e),
            Self::UnknownFormat(s) => write!(f, "Unknown format: {}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<PyErr> for Error {
    fn from(err: PyErr) -> Self {
        pyo3::Python::with_gil(|py| {
            if err.is_instance_of::<UnknownFormatError>(py) {
                let value = err.into_value(py);
                Error::UnknownFormat(value.getattr(py, "format").unwrap().extract(py).unwrap())
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
        }
    }
}
