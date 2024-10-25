use crate::branch::GenericBranch;
use crate::tree::GenericTree;
use crate::debian::TarballKind;
use crate::{branch::Branch, tree::Tree, RevisionId};
use pyo3::prelude::*;
use std::{collections::HashMap, path::Path, path::PathBuf};

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

    pub fn import_package(
        &self,
        dsc_path: &Path,
        apply_patches: bool,
    ) -> Result<String, crate::debian::Error> {
        Ok(Python::with_gil(|py| -> PyResult<String> {
            self.0
                .call_method1(
                    py,
                    "import_package",
                    (dsc_path.to_object(py), apply_patches),
                )?
                .extract::<String>(py)
        })?)
    }

    pub fn tree(&self) -> Option<Box<dyn Tree>> {
        Python::with_gil(|py| -> PyResult<Option<Box<dyn Tree>>> {
            let tree = self
                .0
                .getattr(py, "tree")?
                .extract::<Option<PyObject>>(py)?;
            if tree.is_none() {
                return Ok(None);
            }
            Ok(Some(Box::new(GenericTree::from(tree.unwrap()))))
        })
        .unwrap()
    }

    pub fn branch(&self) -> Box<dyn Branch> {
        Python::with_gil(|py| -> PyResult<Box<dyn Branch>> {
            Ok(Box::new(GenericBranch::new(self.0.getattr(py, "branch")?)))
        })
        .unwrap()
    }

    pub fn create_empty_upstream_tree(&self, basedir: &Path) -> Result<(), crate::debian::Error> {
        Python::with_gil(|py| -> PyResult<()> {
            self.0
                .call_method1(py, "create_empty_upstream_tree", (basedir.to_object(py),))?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn extract_upstream_tree(
        &self,
        upstream_tips: &HashMap<TarballKind, (RevisionId, PathBuf)>,
        basedir: &Path,
    ) -> Result<(), crate::debian::Error> {
        Ok(Python::with_gil(|py| -> PyResult<()> {
            self.0.call_method1(
                py,
                "extract_upstream_tree",
                (upstream_tips.to_object(py), basedir.to_object(py)),
            )?;
            Ok(())
        })?)
    }
}
