use crate::branch::GenericBranch;
use crate::debian::TarballKind;
use crate::tree::WorkingTree;
use crate::{
    branch::{Branch, PyBranch},
    tree::PyTree,
    RevisionId,
};
use pyo3::prelude::*;
use std::{collections::HashMap, path::Path, path::PathBuf};

/// A set of distribution branches for Debian package imports.
///
/// This struct represents a collection of distribution branches that can be
/// used when importing Debian source packages.
pub struct DistributionBranchSet(PyObject);

impl<'py> IntoPyObject<'py> for DistributionBranchSet {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.clone_ref(py).into_bound(py))
    }
}

impl DistributionBranchSet {
    /// Create a new DistributionBranchSet instance.
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py.import("breezy.plugins.debian.import_dsc").unwrap();
            let ctr = m.getattr("DistributionBranchSet").unwrap();
            DistributionBranchSet(ctr.call0().unwrap().into())
        })
    }

    /// Add a distribution branch to this set.
    ///
    /// # Arguments
    /// * `branch` - The branch to add to the set
    pub fn add_branch(&self, branch: &DistributionBranch) {
        Python::with_gil(|py| {
            self.0.call_method1(py, "add_branch", (&branch.0,)).unwrap();
        })
    }
}

/// A branch representing a Debian distribution.
///
/// This struct represents a branch used for importing Debian source packages
/// into version control.
pub struct DistributionBranch(PyObject);

impl<'py> IntoPyObject<'py> for DistributionBranch {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.clone_ref(py).into_bound(py))
    }
}

impl DistributionBranch {
    /// Create a new DistributionBranch instance.
    ///
    /// # Arguments
    /// * `branch` - Main branch for the distribution
    /// * `pristine_upstream_branch` - Branch containing pristine upstream sources
    /// * `tree` - Optional tree for the distribution branch
    /// * `pristine_upstream_tree` - Optional tree for the pristine upstream branch
    ///
    /// # Returns
    /// A new DistributionBranch instance
    pub fn new(
        branch: &dyn PyBranch,
        pristine_upstream_branch: &dyn PyBranch,
        tree: Option<&dyn PyTree>,
        pristine_upstream_tree: Option<&dyn PyTree>,
    ) -> Self {
        Python::with_gil(|py| {
            let m = py.import("breezy.plugins.debian.import_dsc").unwrap();
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

    /// Get the revision ID corresponding to a specific version.
    ///
    /// # Arguments
    /// * `version` - The Debian package version
    ///
    /// # Returns
    /// The revision ID corresponding to the version, or an error
    pub fn revid_of_version(
        &self,
        version: &debversion::Version,
    ) -> Result<RevisionId, crate::debian::error::Error> {
        Ok(Python::with_gil(|py| -> PyResult<RevisionId> {
            self.0
                .call_method1(py, "revid_of_version", (version.to_string(),))?
                .extract::<RevisionId>(py)
        })?)
    }

    /// Import a Debian source package (.dsc file) into the distribution branch.
    ///
    /// # Arguments
    /// * `dsc_path` - Path to the .dsc file to import
    /// * `apply_patches` - Whether to apply patches during import
    ///
    /// # Returns
    /// The version string of the imported package, or an error
    pub fn import_package(
        &self,
        dsc_path: &Path,
        apply_patches: bool,
    ) -> Result<String, crate::debian::error::Error> {
        Ok(Python::with_gil(|py| -> PyResult<String> {
            self.0
                .call_method1(py, "import_package", (dsc_path, apply_patches))?
                .extract::<String>(py)
        })?)
    }

    /// Get the working tree associated with this distribution branch.
    ///
    /// # Returns
    /// The working tree, if available
    pub fn tree(&self) -> Option<WorkingTree> {
        Python::with_gil(|py| -> PyResult<Option<WorkingTree>> {
            let tree = self
                .0
                .getattr(py, "tree")?
                .extract::<Option<PyObject>>(py)?;
            if tree.is_none() {
                return Ok(None);
            }
            Ok(Some(WorkingTree::from(tree.unwrap())))
        })
        .unwrap()
    }

    /// Get the branch associated with this distribution branch.
    ///
    /// # Returns
    /// The branch object
    pub fn branch(&self) -> Box<dyn Branch> {
        Python::with_gil(|py| -> PyResult<Box<dyn Branch>> {
            Ok(Box::new(GenericBranch::from(self.0.getattr(py, "branch")?)))
        })
        .unwrap()
    }

    /// Get the pristine-tar source associated with this distribution branch.
    ///
    /// # Returns
    /// The pristine-tar source for accessing upstream tarballs
    pub fn pristine_upstream_source(&self) -> crate::debian::upstream::PristineTarSource {
        Python::with_gil(
            |py| -> PyResult<crate::debian::upstream::PristineTarSource> {
                Ok(crate::debian::upstream::PristineTarSource::from(
                    self.0.getattr(py, "pristine_upstream_source")?,
                ))
            },
        )
        .unwrap()
    }

    /// Create an empty upstream tree in the specified directory.
    ///
    /// # Arguments
    /// * `basedir` - Directory in which to create the empty tree
    ///
    /// # Returns
    /// Ok(()) on success, or an error
    pub fn create_empty_upstream_tree(
        &self,
        basedir: &Path,
    ) -> Result<(), crate::debian::error::Error> {
        Python::with_gil(|py| -> PyResult<()> {
            self.0
                .call_method1(py, "create_empty_upstream_tree", (basedir,))?;
            Ok(())
        })?;
        Ok(())
    }

    /// Extract upstream trees from their revisions into a directory.
    ///
    /// # Arguments
    /// * `upstream_tips` - Mapping from tarball kinds to revision IDs and paths
    /// * `basedir` - Directory in which to extract the upstream tree
    ///
    /// # Returns
    /// Ok(()) on success, or an error
    pub fn extract_upstream_tree(
        &self,
        upstream_tips: &HashMap<TarballKind, (RevisionId, PathBuf)>,
        basedir: &Path,
    ) -> Result<(), crate::debian::error::Error> {
        Ok(Python::with_gil(|py| -> PyResult<()> {
            self.0.call_method1(
                py,
                "extract_upstream_tree",
                (
                    {
                        let dict = pyo3::types::PyDict::new(py);
                        for (k, (r, p)) in upstream_tips {
                            dict.set_item(k.clone(), (r.clone(), p.clone()))?;
                        }
                        dict
                    },
                    basedir,
                ),
            )?;
            Ok(())
        })?)
    }
}
