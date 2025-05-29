//! Graph traversal operations on revision graphs.
use crate::revisionid::RevisionId;
use pyo3::exceptions::PyStopIteration;
use pyo3::import_exception;
use pyo3::prelude::*;

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
