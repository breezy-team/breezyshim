//! Tree merging.
use crate::branch::PyBranch;
use crate::graph::Graph;
use crate::hooks::HookDict;
use crate::transform::TreeTransform;
use crate::tree::PyTree;
use crate::RevisionId;
use pyo3::import_exception;
use pyo3::prelude::*;
use pyo3::types::PyDict;

import_exception!(breezy.errors, UnrelatedBranches);

pub enum Error {
    UnrelatedBranches,
}

impl From<PyErr> for Error {
    fn from(e: PyErr) -> Self {
        Python::with_gil(|py| {
            if e.is_instance_of::<UnrelatedBranches>(py) {
                Error::UnrelatedBranches
            } else {
                panic!("unexpected error: {:?}", e)
            }
        })
    }
}

pub struct Merger(PyObject);

pub enum MergeType {
    Merge3,
}

impl From<PyObject> for Merger {
    fn from(obj: PyObject) -> Self {
        Merger(obj)
    }
}

impl Merger {
    pub fn new<T: PyTree, B: PyBranch>(branch: &B, this_tree: &T, revision_graph: &Graph) -> Self {
        Python::with_gil(|py| {
            let m = py.import("breezy.merge").unwrap();
            let cls = m.getattr("Merger").unwrap();
            let kwargs = PyDict::new(py);
            kwargs
                .set_item("this_tree", this_tree)
                .unwrap();
            kwargs
                .set_item("revision_graph", revision_graph)
                .unwrap();
            let merger = cls.call((branch,), Some(&kwargs)).unwrap();
            Merger(merger.into())
        })
    }

    pub fn find_base(&self) -> Result<Option<RevisionId>, crate::error::Error> {
        Python::with_gil(|py| match self.0.call_method0(py, "find_base") {
            Ok(_py_obj) => Ok(self
                .0
                .getattr(py, "base_rev_id")
                .unwrap()
                .extract(py)
                .unwrap()),
            Err(err) => {
                if err.is_instance_of::<UnrelatedBranches>(py) {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        })
        .map_err(Into::into)
    }

    pub fn set_other_revision<B: PyBranch>(
        &mut self,
        other_revision: &RevisionId,
        other_branch: &B,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(
                py,
                "set_other_revision",
                (&other_revision, other_branch),
            )?;
            Ok(())
        })
    }

    pub fn set_base_revision<B: PyBranch>(
        &mut self,
        base_revision: &RevisionId,
        base_branch: &B,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(
                py,
                "set_base_revision",
                (&base_revision, base_branch),
            )?;
            Ok(())
        })
    }

    pub fn set_merge_type(&mut self, merge_type: MergeType) {
        Python::with_gil(|py| {
            let m = py.import("breezy.merge").unwrap();
            let merge_type = match merge_type {
                MergeType::Merge3 => m.getattr("Merge3Merger").unwrap(),
            };
            self.0.setattr(py, "merge_type", merge_type).unwrap();
        })
    }

    pub fn make_merger(&self) -> Result<Submerger, crate::error::Error> {
        Python::with_gil(|py| {
            let merger = self.0.call_method0(py, "make_merger")?;
            Ok(Submerger(merger))
        })
    }

    pub fn from_revision_ids<T: PyTree, B1: PyBranch, B2: PyBranch>(
        other_tree: &T,
        other_branch: &B1,
        other: &RevisionId,
        tree_branch: &B2,
    ) -> Result<Self, Error> {
        Python::with_gil(|py| {
            let m = py.import("breezy.merge").unwrap();
            let cls = m.getattr("Merger").unwrap();
            let kwargs = PyDict::new(py);
            kwargs
                .set_item("other_branch", other_branch)
                .unwrap();
            kwargs.set_item("other", other).unwrap();
            kwargs
                .set_item("tree_branch", tree_branch)
                .unwrap();
            let merger = cls.call_method(
                "from_revision_ids",
                (other_tree,),
                Some(&kwargs),
            )?;
            Ok(Merger(merger.into()))
        })
    }
}

pub struct Submerger(PyObject);

impl Submerger {
    pub fn make_preview_transform(&self) -> Result<TreeTransform, crate::error::Error> {
        Python::with_gil(|py| {
            let transform = self
                .0
                .call_method0(py, "make_preview_transform")?;
            Ok(TreeTransform::from(transform))
        })
    }
}

lazy_static::lazy_static! {
    pub static ref MERGE_HOOKS: HookDict = HookDict::new("breezy.merge", "Merger", "hooks");
}
