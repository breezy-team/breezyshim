//! Tree transformation API.
use crate::tree::{PathBuf, PyTree, TreeChange};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyString};
use pyo3::types::PyTupleMethods;

crate::wrapped_py!(TreeTransform);

crate::wrapped_py!(Conflict);

impl Clone for Conflict {
    fn clone(&self) -> Self {
        Python::with_gil(|py| Conflict(self.0.clone_ref(py)))
    }
}

impl Conflict {
    pub fn associated_filenames(&self) -> Result<Vec<PathBuf>, crate::error::Error> {
        let mut v: Vec<PathBuf> = vec![];

        Python::with_gil(|py| {
            let ret = self.0.getattr(py, "associated_filenames")?;

            for item in ret.bind(py).try_iter()? {
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

    pub fn cleanup<T: PyTree>(&self, tree: &T) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(py, "cleanup", (tree,))?;
            Ok(())
        })
    }
}

crate::wrapped_py!(PreviewTree);

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

            for item in ret.bind(py).try_iter()? {
                v.push(item?.extract()?);
            }

            Ok(Box::new(v.into_iter()) as Box<dyn Iterator<Item = TreeChange>>)
        })
    }

    pub fn cooked_conflicts(&self) -> Result<Vec<Conflict>, crate::error::Error> {
        let mut v: Vec<Conflict> = vec![];

        Python::with_gil(|py| {
            let ret = self.to_object(py).getattr(py, "cooked_conflicts")?;

            for item in ret.bind(py).try_iter()? {
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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TransId(String);

impl FromPyObject<'_> for TransId {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(TransId(ob.extract::<String>()?))
    }
}

impl<'py> IntoPyObject<'py> for &TransId {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let obj = PyString::new(py, &self.0);
        Ok(obj)
    }
}


#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum RawConflict {
    UnversionedExecutability(TransId),
    NonFileExecutability(TransId),
    Overwrite(TransId, String),
    ParentLoop(TransId),
    UnversionedParent(TransId),
    VersioningNoContents(TransId),
    VersioningBadKind(TransId),
    Duplicate(TransId, TransId, String),
    MissingParent(TransId),
    NonDirectoryParent(TransId),
}

impl<'py> IntoPyObject<'py> for &RawConflict {
    type Target = PyTuple;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            RawConflict::UnversionedExecutability(id) => {
                PyTuple::new(py, &[("unversioned executability", id)])
            }
            RawConflict::NonFileExecutability(id) => {
                PyTuple::new(py, &[("non-file executability", id)])
            }
            RawConflict::Overwrite(id, path) => {
                PyTuple::new(py, &[("overwrite", id, path)])
            }
            RawConflict::ParentLoop(id) => {
                PyTuple::new(py, &[("parent loop", id)])
            }
            RawConflict::UnversionedParent(id) => {
                PyTuple::new(py, &[("unversioned parent", id)])
            }
            RawConflict::VersioningNoContents(id) => {
                PyTuple::new(py, &[("versioning no contents", id)])
            }
            RawConflict::VersioningBadKind(id) => {
                PyTuple::new(py, &[("versioning bad kind", id)])
            }
            RawConflict::Duplicate(id1, id2, path) => PyTuple::new(
                py,
                &[(
                    "duplicate",
                    id1,
                    id2,
                    path
                )],
            ),
            RawConflict::MissingParent(id) => {
                PyTuple::new(py, &[("missing parent", id)])
            }
            RawConflict::NonDirectoryParent(id) => {
                PyTuple::new(py, &[("non-directory parent", id)])
            }
        }
    }

}

impl FromPyObject<'_> for RawConflict {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let tuple = ob.extract::<Bound<PyTuple>>()?;

        match tuple.get_item(0)?.extract::<String>()?.as_str() {
            "unversioned executability" => Ok(Self::UnversionedExecutability(TransId(
                tuple.get_item(1)?.extract::<String>()?,
            ))),
            "non-file executability" => Ok(Self::NonFileExecutability(TransId(
                tuple.get_item(1)?.extract::<String>()?,
            ))),
            "overwrite" => Ok(Self::Overwrite(
                TransId(tuple.get_item(1)?.extract::<String>()?),
                tuple.get_item(2)?.extract::<String>()?,
            )),
            "parent loop" => Ok(Self::ParentLoop(TransId(
                tuple.get_item(1)?.extract::<String>()?,
            ))),
            "unversioned parent" => Ok(Self::UnversionedParent(TransId(
                tuple.get_item(1)?.extract::<String>()?,
            ))),
            "versioning no contents" => Ok(Self::VersioningNoContents(TransId(
                tuple.get_item(1)?.extract::<String>()?,
            ))),
            "versioning bad kind" => Ok(Self::VersioningBadKind(TransId(
                tuple.get_item(1)?.extract::<String>()?,
            ))),
            "duplicate" => Ok(Self::Duplicate(
                TransId(tuple.get_item(1)?.extract::<String>()?),
                TransId(tuple.get_item(2)?.extract::<String>()?),
                tuple.get_item(3)?.extract::<String>()?,
            )),
            "missing parent" => Ok(Self::MissingParent(TransId(
                tuple.get_item(1)?.extract::<String>()?,
            ))),
            "non-directory parent" => Ok(Self::NonDirectoryParent(TransId(
                tuple.get_item(1)?.extract::<String>()?,
            ))),
            _ => Err(PyErr::new::<PyValueError, _>(format!(
                "Unknown conflict type: {}",
                tuple.get_item(0)?.extract::<String>()?
            ))),
        }
    }
}
