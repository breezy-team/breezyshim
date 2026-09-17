//! Knit versioned files implementation

#![allow(missing_docs)]

use crate::versionedfiles::PyVersionedFiles;
use pyo3::prelude::*;

pub struct KnitVersionedFiles(Py<PyAny>);

impl KnitVersionedFiles {
    pub fn new(py_obj: Py<PyAny>) -> Self {
        Self(py_obj)
    }

    pub fn from_transport(
        py: Python,
        transport: &crate::transport::Transport,
        file_mode: Option<u32>,
        dir_mode: Option<u32>,
        access_mode: Option<&str>,
    ) -> PyResult<Self> {
        let knit_mod = crate::import_first(py, &["breezy.bzr.knit", "bzrformats.knit"])?;
        let kvf_cls = knit_mod.getattr("KnitVersionedFiles")?;

        let kwargs = pyo3::types::PyDict::new(py);
        if let Some(mode) = file_mode {
            kwargs.set_item("file_mode", mode)?;
        }
        if let Some(mode) = dir_mode {
            kwargs.set_item("dir_mode", mode)?;
        }
        if let Some(mode) = access_mode {
            kwargs.set_item("access_mode", mode)?;
        }

        let obj = kvf_cls.call((transport.as_pyobject().clone_ref(py),), Some(&kwargs))?;
        Ok(KnitVersionedFiles(obj.unbind()))
    }
}

impl Clone for KnitVersionedFiles {
    fn clone(&self) -> Self {
        Python::attach(|py| KnitVersionedFiles(self.0.clone_ref(py)))
    }
}

impl PyVersionedFiles for KnitVersionedFiles {
    fn to_object(&self, py: Python) -> Py<PyAny> {
        self.0.clone_ref(py)
    }
}

impl<'py> IntoPyObject<'py> for KnitVersionedFiles {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for KnitVersionedFiles {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(KnitVersionedFiles(ob.to_owned().unbind()))
    }
}
