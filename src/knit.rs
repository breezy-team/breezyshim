//! Knit versioned files implementation

#![allow(missing_docs)]

use crate::graph::Key;
use crate::versionedfiles::PyVersionedFiles;
use pyo3::prelude::*;

pub struct KnitVersionedFiles(PyObject);

impl KnitVersionedFiles {
    pub fn new(py_obj: PyObject) -> Self {
        Self(py_obj)
    }

    pub fn from_transport(
        py: Python,
        transport: &crate::transport::Transport,
        file_mode: Option<u32>,
        dir_mode: Option<u32>,
        access_mode: Option<&str>,
    ) -> PyResult<Self> {
        let knit_mod = py.import("breezy.bzr.knit")?;
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
        Python::with_gil(|py| KnitVersionedFiles(self.0.clone_ref(py)))
    }
}

impl PyVersionedFiles for KnitVersionedFiles {
    fn to_object(&self, py: Python) -> PyObject {
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

pub struct KnitPlainFactory {
    pub key: Key,
    pub parents: Vec<Key>,
    pub sha1: Option<String>,
    pub delta: Option<Vec<u8>>,
}

impl KnitPlainFactory {
    pub fn new(key: Key, parents: Vec<Key>, sha1: Option<String>, delta: Option<Vec<u8>>) -> Self {
        Self {
            key,
            parents,
            sha1,
            delta,
        }
    }
}

impl<'py> IntoPyObject<'py> for KnitPlainFactory {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let knit_mod = py.import("breezy.bzr.knit")?;
        let factory_cls = knit_mod.getattr("KnitPlainFactory")?;

        // Create empty factory
        let factory = factory_cls.call0()?;

        // Set attributes
        factory.setattr("key", self.key.into_pyobject(py)?)?;

        let parent_tuples: Vec<_> = self
            .parents
            .into_iter()
            .map(|p| p.into_pyobject(py))
            .collect::<Result<Vec<_>, _>>()?;
        let parents_py = pyo3::types::PyTuple::new(py, parent_tuples)?;
        factory.setattr("parents", parents_py)?;

        if let Some(sha1) = self.sha1 {
            factory.setattr("sha1", pyo3::types::PyBytes::new(py, sha1.as_bytes()))?;
        }
        if let Some(delta) = self.delta {
            factory.setattr("delta", pyo3::types::PyBytes::new(py, &delta))?;
        }

        Ok(factory)
    }
}

pub struct KnitAnnotateFactory {
    pub key: Key,
    pub parents: Vec<Key>,
    pub annotated_lines: Vec<(Key, String)>,
}

impl KnitAnnotateFactory {
    pub fn new(key: Key, parents: Vec<Key>, annotated_lines: Vec<(Key, String)>) -> Self {
        Self {
            key,
            parents,
            annotated_lines,
        }
    }
}

impl<'py> IntoPyObject<'py> for KnitAnnotateFactory {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let knit_mod = py.import("breezy.bzr.knit")?;
        let factory_cls = knit_mod.getattr("KnitAnnotateFactory")?;

        // Create empty factory
        let factory = factory_cls.call0()?;

        // Set attributes
        factory.setattr("key", self.key.into_pyobject(py)?)?;

        let parent_tuples: Vec<_> = self
            .parents
            .into_iter()
            .map(|p| p.into_pyobject(py))
            .collect::<Result<Vec<_>, _>>()?;
        let parents_py = pyo3::types::PyTuple::new(py, parent_tuples)?;
        factory.setattr("parents", parents_py)?;

        let lines_list = pyo3::types::PyList::empty(py);
        for (origin_key, line) in self.annotated_lines {
            let tuple = pyo3::types::PyTuple::new(
                py,
                &[
                    origin_key.into_pyobject(py)?.into_any(),
                    line.into_pyobject(py)?.into_any(),
                ],
            )?;
            lines_list.append(tuple)?;
        }
        factory.setattr("annotated", lines_list)?;

        Ok(factory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knit_plain_factory() {
        crate::init();
        crate::init_bzr();

        pyo3::Python::with_gil(|py| {
            let key = Key::from(vec!["file1".to_string()]);
            let parents = vec![Key::from(vec!["parent1".to_string()])];
            let factory = KnitPlainFactory::new(
                key.clone(),
                parents,
                Some("abc123".to_string()),
                Some(b"delta content".to_vec()),
            );

            // Test conversion to PyObject
            let _py_obj = factory.into_pyobject(py).unwrap();
        });
    }

    #[test]
    fn test_knit_annotate_factory() {
        crate::init();
        crate::init_bzr();

        pyo3::Python::with_gil(|py| {
            let key = Key::from(vec!["file1".to_string()]);
            let parents = vec![];
            let annotated_lines = vec![
                (
                    Key::from(vec!["origin1".to_string()]),
                    "line1\n".to_string(),
                ),
                (
                    Key::from(vec!["origin1".to_string()]),
                    "line2\n".to_string(),
                ),
            ];
            let factory = KnitAnnotateFactory::new(key, parents, annotated_lines);

            // Test conversion to PyObject
            let _py_obj = factory.into_pyobject(py).unwrap();
        });
    }

    // Note: Full KnitVersionedFiles tests require complex setup with indices and data access
    // which are difficult to mock in unit tests. Integration tests would be more appropriate.
}
