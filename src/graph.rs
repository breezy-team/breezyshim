//! Graph traversal operations on revision graphs.
use crate::revisionid::RevisionId;
use pyo3::exceptions::PyStopIteration;
use pyo3::import_exception;
use pyo3::prelude::*;
use pyo3::types::{PyFrozenSet, PyIterator, PyTuple};

import_exception!(breezy.errors, RevisionNotPresent);

/// Represents a graph of revisions.
///
/// This struct provides methods for traversing and querying relationships
/// between revisions in a version control repository.
pub struct Graph(PyObject);

impl<'py> IntoPyObject<'py> for Graph {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl FromPyObject<'_> for Graph {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Graph(ob.clone().unbind()))
    }
}

impl From<PyObject> for Graph {
    fn from(ob: PyObject) -> Self {
        Graph(ob)
    }
}

struct RevIter(PyObject);

impl Iterator for RevIter {
    type Item = Result<RevisionId, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| match self.0.call_method0(py, "__next__") {
            Ok(item) => Some(Ok(RevisionId::from(item.extract::<Vec<u8>>(py).unwrap()))),
            Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
            Err(e) => Some(Err(e.into())),
        })
    }
}

#[derive(Debug)]
/// Errors that can occur during graph operations.
pub enum Error {
    /// Error indicating that a specified revision is not present in the repository.
    RevisionNotPresent(RevisionId),
}

impl From<PyErr> for Error {
    fn from(e: PyErr) -> Self {
        Python::with_gil(|py| {
            if e.is_instance_of::<RevisionNotPresent>(py) {
                Error::RevisionNotPresent(RevisionId::from(
                    e.into_value(py)
                        .getattr(py, "revision_id")
                        .unwrap()
                        .extract::<Vec<u8>>(py)
                        .unwrap(),
                ))
            } else {
                panic!("unexpected error: {:?}", e)
            }
        })
    }
}

impl Graph {
    /// Get the underlying PyObject.
    pub(crate) fn as_pyobject(&self) -> &PyObject {
        &self.0
    }

    /// Check if one revision is an ancestor of another.
    ///
    /// # Arguments
    ///
    /// * `rev1` - The potential ancestor revision
    /// * `rev2` - The potential descendant revision
    ///
    /// # Returns
    ///
    /// `true` if `rev1` is an ancestor of `rev2`, `false` otherwise
    pub fn is_ancestor(&self, rev1: &RevisionId, rev2: &RevisionId) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "is_ancestor", (rev1.as_bytes(), rev2.as_bytes()))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Iterate through the left-hand ancestry of a revision.
    ///
    /// # Arguments
    ///
    /// * `revid` - The revision ID to start from
    /// * `stop_revisions` - Optional list of revision IDs where iteration should stop
    ///
    /// # Returns
    ///
    /// An iterator that yields revision IDs in the ancestry chain
    pub fn iter_lefthand_ancestry(
        &self,
        revid: &RevisionId,
        stop_revisions: Option<&[RevisionId]>,
    ) -> impl Iterator<Item = Result<RevisionId, Error>> {
        Python::with_gil(|py| {
            let iter = self
                .0
                .call_method1(
                    py,
                    "iter_lefthand_ancestry",
                    (revid.as_bytes(), stop_revisions.map(|x| x.to_vec())),
                )
                .unwrap();
            RevIter(iter)
        })
    }
}

/// A key identifying a specific version of a file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(Vec<String>);

impl From<Vec<String>> for Key {
    fn from(v: Vec<String>) -> Self {
        Key(v)
    }
}

impl From<Key> for Vec<String> {
    fn from(k: Key) -> Self {
        k.0
    }
}

impl<'py> IntoPyObject<'py> for Key {
    type Target = PyTuple;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyTuple::new(py, self.0)?)
    }
}

impl<'py> FromPyObject<'py> for Key {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let tuple = ob.downcast::<PyTuple>()?;
        let mut items = Vec::new();
        for item in tuple {
            items.push(item.extract::<String>()?);
        }
        Ok(Key(items))
    }
}

/// A known graph of file versions
pub struct KnownGraph(PyObject);

impl KnownGraph {
    /// Create a new KnownGraph from a Python object
    pub fn new(py_obj: PyObject) -> Self {
        Self(py_obj)
    }

    /// Get the heads of the given keys
    pub fn heads(&self, keys: Vec<Key>) -> Result<Vec<Key>, crate::error::Error> {
        Python::with_gil(|py| {
            let keys_py: Vec<_> = keys
                .into_iter()
                .map(|k| k.into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            let keys_frozenset = PyFrozenSet::new(py, &keys_py)?;

            let result = self.0.call_method1(py, "heads", (keys_frozenset,))?;

            let mut heads = Vec::new();
            for head_py in result
                .downcast_bound::<PyIterator>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected iterator"))?
            {
                let head = Key::extract_bound(&head_py?)?;
                heads.push(head);
            }

            Ok(heads)
        })
    }
}

impl Clone for KnownGraph {
    fn clone(&self) -> Self {
        Python::with_gil(|py| KnownGraph(self.0.clone_ref(py)))
    }
}
