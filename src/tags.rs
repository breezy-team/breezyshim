use crate::revisionid::RevisionId;
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct Tags(PyObject);

impl From<PyObject> for Tags {
    fn from(obj: PyObject) -> Self {
        Tags(obj)
    }
}

#[derive(Debug)]
pub enum Error {
    NoSuchTag(String),
    TagAlreadyExists(String),
    Other(PyErr),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoSuchTag(tag) => write!(f, "No such tag: {}", tag),
            Error::TagAlreadyExists(tag) => write!(f, "Tag already exists: {}", tag),
            Error::Other(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

pyo3::import_exception!(breezy.errors, NoSuchTag);

impl From<PyErr> for Error {
    fn from(err: PyErr) -> Self {
        Python::with_gil(|py| {
            if err.is_instance_of::<NoSuchTag>(py) {
                Error::NoSuchTag(err.into_value(py).getattr(py, "tag_name").unwrap().extract(py).unwrap())
            } else {
                Error::Other(err)
            }
        })
    }
}

impl Tags {
    pub fn get_reverse_tag_dict(&self) -> PyResult<HashMap<RevisionId, HashSet<String>>> {
        Python::with_gil(|py| {
            self
                .0
                .call_method0(py, "get_reverse_tag_dict")?
                .extract(py)
        })
    }

    pub fn get_tag_dict(&self) -> PyResult<HashMap<String, HashSet<RevisionId>>> {
        Python::with_gil(|py| {
            self
                .0
                .call_method0(py, "get_tag_dict")?
                .extract(py)
        })
    }

    pub fn lookup_tag(&self, tag: &str) -> Result<RevisionId, Error> {
        Ok(Python::with_gil(|py| {
            self.0
                .call_method1(py, "lookup_tag", (tag,))?
                .extract(py)
        })?)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "has_tag", (tag,)).unwrap()
                .extract(py).unwrap()
        })
    }

    pub fn set_tag(&self, tag: &str, revision_id: &RevisionId) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "set_tag", (tag, revision_id.to_object(py)))
        })?;
        Ok(())
    }

    pub fn delete_tag(&self, tag: &str) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "delete_tag", (tag,))
        })?;
        Ok(())
    }
}
