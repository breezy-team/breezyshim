//! Revision tags
use crate::error::Error;
use crate::revisionid::RevisionId;
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct Tags(PyObject);

impl From<PyObject> for Tags {
    fn from(obj: PyObject) -> Self {
        Tags(obj)
    }
}

impl Tags {
    pub fn get_reverse_tag_dict(
        &self,
    ) -> Result<HashMap<RevisionId, HashSet<String>>, crate::error::Error> {
        Python::with_gil(|py| self.0.call_method0(py, "get_reverse_tag_dict")?.extract(py))
            .map_err(Into::into)
    }

    pub fn get_tag_dict(&self) -> Result<HashMap<String, RevisionId>, crate::error::Error> {
        Python::with_gil(|py| self.0.call_method0(py, "get_tag_dict")?.extract(py))
            .map_err(Into::into)
    }

    pub fn lookup_tag(&self, tag: &str) -> Result<RevisionId, Error> {
        Ok(Python::with_gil(|py| {
            self.0.call_method1(py, "lookup_tag", (tag,))?.extract(py)
        })?)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "has_tag", (tag,))
                .unwrap()
                .extract(py)
                .unwrap()
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
        Python::with_gil(|py| self.0.call_method1(py, "delete_tag", (tag,)))?;
        Ok(())
    }
}
