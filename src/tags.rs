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
    pub fn get_reverse_tag_dict(&self) -> PyResult<HashMap<RevisionId, HashSet<String>>> {
        Python::with_gil(|py| {
            Ok(self
                .0
                .call_method0(py, "get_reverse_tag_dict")?
                .extract(py)?)
        })
    }
}
