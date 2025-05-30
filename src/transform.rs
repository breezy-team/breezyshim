//! Tree transformation API.
use crate::tree::{PathBuf, PyTree};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::types::PyTupleMethods;

/// A tree transform is used to apply a set of changes to a tree.
pub struct TreeTransform(PyObject);

#[derive(Clone)]
/// Represents a change to a file or directory in a tree transformation.
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

/// Represents a conflict that occurs during a tree transformation.
pub struct Conflict(PyObject);

impl Clone for Conflict {
    fn clone(&self) -> Self {
        Python::with_gil(|py| Conflict(self.0.clone_ref(py)))
    }
}

impl Conflict {
    /// Get the file paths associated with this conflict.
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

    /// Get a human-readable description of this conflict.
    pub fn describe(&self) -> Result<String, crate::error::Error> {
        Python::with_gil(|py| {
            let ret = self.0.call_method0(py, "describe")?;
            Ok(ret.extract(py)?)
        })
    }

    /// Clean up any temporary files created by this conflict.
    pub fn cleanup<T: PyTree>(&self, tree: &T) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(py, "cleanup", (tree.to_object(py),))?;
            Ok(())
        })
    }
}

/// A tree that shows what a tree would look like after applying a transform.
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

impl TreeTransform {
    /// Apply the transform to the tree.
    pub fn finalize(&self) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py).call_method0(py, "finalize")?;
            Ok(())
        })
    }

    /// Iterate through the changes in this transform.
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

    /// Get a list of conflicts that would occur when applying this transform.
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

    /// Get a preview tree showing what would happen if this transform was applied.
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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
/// An identifier for a transformation operation.
pub struct TransId(String);

impl FromPyObject<'_> for TransId {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(TransId(ob.extract::<String>()?))
    }
}

impl ToPyObject for TransId {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
/// Enum representing different types of conflicts that can occur during transformation.
pub enum RawConflict {
    /// Conflict caused by trying to change executability of an unversioned file.
    UnversionedExecutability(TransId),
    /// Conflict caused by trying to set executability on a non-file.
    NonFileExecutability(TransId),
    /// Conflict caused by trying to overwrite an existing file with different content.
    Overwrite(TransId, String),
    /// Conflict caused by a directory loop in the parent structure.
    ParentLoop(TransId),
    /// Conflict caused by trying to version a file with an unversioned parent.
    UnversionedParent(TransId),
    /// Conflict caused by trying to version a file without contents.
    VersioningNoContents(TransId),
    /// Conflict caused by trying to version a file with an unsupported kind.
    VersioningBadKind(TransId),
    /// Conflict caused by trying to add the same file path twice.
    Duplicate(TransId, TransId, String),
    /// Conflict caused by a missing parent directory.
    MissingParent(TransId),
    /// Conflict caused by a parent that is not a directory.
    NonDirectoryParent(TransId),
}

impl ToPyObject for RawConflict {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            RawConflict::UnversionedExecutability(id) => {
                PyTuple::new_bound(py, &[("unversioned executability", id.to_object(py))])
                    .to_object(py)
            }
            RawConflict::NonFileExecutability(id) => {
                PyTuple::new_bound(py, &[("non-file executability", id.to_object(py))])
                    .to_object(py)
            }
            RawConflict::Overwrite(id, path) => {
                PyTuple::new_bound(py, &[("overwrite", id.to_object(py), path.to_object(py))])
                    .to_object(py)
            }
            RawConflict::ParentLoop(id) => {
                PyTuple::new_bound(py, &[("parent loop", id.to_object(py))]).to_object(py)
            }
            RawConflict::UnversionedParent(id) => {
                PyTuple::new_bound(py, &[("unversioned parent", id.to_object(py))]).to_object(py)
            }
            RawConflict::VersioningNoContents(id) => {
                PyTuple::new_bound(py, &[("versioning no contents", id.to_object(py))])
                    .to_object(py)
            }
            RawConflict::VersioningBadKind(id) => {
                PyTuple::new_bound(py, &[("versioning bad kind", id.to_object(py))]).to_object(py)
            }
            RawConflict::Duplicate(id1, id2, path) => PyTuple::new_bound(
                py,
                &[(
                    "duplicate",
                    id1.to_object(py),
                    id2.to_object(py),
                    path.to_object(py),
                )],
            )
            .to_object(py),
            RawConflict::MissingParent(id) => {
                PyTuple::new_bound(py, &[("missing parent", id.to_object(py))]).to_object(py)
            }
            RawConflict::NonDirectoryParent(id) => {
                PyTuple::new_bound(py, &[("non-directory parent", id.to_object(py))]).to_object(py)
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
