//! Tree transformation API.
use crate::tree::{PathBuf, Tree};
use pyo3::prelude::*;

pub struct TreeTransform(PyObject);

#[derive(Clone)]
pub struct TreeChange {}

impl From<PyObject> for TreeChange {
    fn from(_ob: PyObject) -> Self {
        TreeChange {}
    }
}

impl FromPyObject<'_> for TreeChange {
    fn extract_bound(_ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(TreeChange {})
    }
}

#[derive(Clone)]
pub struct Conflict(PyObject);

impl Conflict {
    pub fn associated_filenames(&self) -> Result<Vec<PathBuf>, crate::error::Error> {
        let mut v: Vec<PathBuf> = vec![];

        Python::with_gil(|py| {
            let ret = self.0.getattr(py, "associated_filenames")?;

            for item in ret.bind(py).iter()? {
                v.push(item?.extract()?);
            }

            Ok(v)
        })
    }

    pub fn describe(&self) -> Result<String, crate::error::Error> {
        Python::with_gil(|py| {
            let ret = self.0.call_method0(py, "describe")?;
            Ok(ret.extract(py)?)
        })
    }

    pub fn cleanup(&self, tree: &dyn Tree) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(py, "cleanup", (tree.to_object(py),))?;
            Ok(())
        })
    }
}

pub struct PreviewTree(PyObject);

impl ToPyObject for PreviewTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl From<PyObject> for PreviewTree {
    fn from(ob: PyObject) -> Self {
        PreviewTree(ob)
    }
}

impl Tree for PreviewTree {}

impl TreeTransform {
    pub fn finalize(&self) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py).call_method0(py, "finalize")?;
            Ok(())
        })
    }

    pub fn iter_changes(
        &self,
    ) -> Result<Box<dyn Iterator<Item = TreeChange>>, crate::error::Error> {
        let mut v: Vec<TreeChange> = vec![];

        Python::with_gil(|py| {
            let ret = self.to_object(py).call_method0(py, "iter_changes")?;

            for item in ret.bind(py).iter()? {
                v.push(item?.extract()?);
            }

            Ok(Box::new(v.into_iter()) as Box<dyn Iterator<Item = TreeChange>>)
        })
    }

    pub fn cooked_conflicts(&self) -> Result<Vec<Conflict>, crate::error::Error> {
        let mut v: Vec<Conflict> = vec![];

        Python::with_gil(|py| {
            let ret = self.to_object(py).getattr(py, "cooked_conflicts")?;

            for item in ret.bind(py).iter()? {
                v.push(Conflict(item?.into()));
            }

            Ok(v)
        })
    }

    pub fn get_preview_tree(&self) -> Result<PreviewTree, crate::error::Error> {
        Python::with_gil(|py| {
            let ret = self.to_object(py).getattr(py, "preview_tree")?;
            Ok(PreviewTree(ret))
        })
    }
}

impl From<PyObject> for TreeTransform {
    fn from(ob: PyObject) -> Self {
        TreeTransform(ob)
    }
}

impl ToPyObject for TreeTransform {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl FromPyObject<'_> for TreeTransform {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(TreeTransform(ob.clone().unbind()))
    }
}
