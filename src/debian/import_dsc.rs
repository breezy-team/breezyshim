use crate::{branch::Branch, tree::Tree, RevisionId};
use pyo3::prelude::*;

pub struct DistributionBranchSet(PyObject);

impl ToPyObject for DistributionBranchSet {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl DistributionBranchSet {
    /// Create a new DistributionBranchSet instance.
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.plugins.debian.import_dsc").unwrap();
            let ctr = m.getattr("DistributionBranchSet").unwrap();
            DistributionBranchSet(ctr.call0().unwrap().into())
        })
    }

    pub fn add_branch(&self, branch: &DistributionBranch) {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "add_branch", (branch.to_object(py),))
                .unwrap();
        })
    }
}

pub struct DistributionBranch(PyObject);

impl ToPyObject for DistributionBranch {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl DistributionBranch {
    pub fn new(
        branch: &dyn Branch,
        pristine_upstream_branch: &dyn Branch,
        tree: Option<&dyn Tree>,
        pristine_upstream_tree: Option<&dyn Tree>,
    ) -> Self {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.plugins.debian.import_dsc").unwrap();
            let ctr = m.getattr("DistributionBranch").unwrap();
            DistributionBranch(
                ctr.call1((
                    branch.to_object(py),
                    pristine_upstream_branch.to_object(py),
                    tree.map(|t| t.to_object(py)),
                    pristine_upstream_tree.map(|t| t.to_object(py)),
                ))
                .unwrap()
                .into(),
            )
        })
    }

    pub fn revid_of_version(
        &self,
        version: &debversion::Version,
    ) -> Result<RevisionId, crate::debian::Error> {
        Ok(Python::with_gil(|py| -> PyResult<RevisionId> {
            self.0
                .call_method1(py, "revid_of_version", (version.to_object(py),))?
                .extract::<RevisionId>(py)
        })?)
    }
}
