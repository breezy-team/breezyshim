//! Debian specific functionality.
//!
//! This module provides functionality for working with Debian packages.
//!
//! It mostly wraps the `breezy.plugins.debian` module from the Breezy VCS.
pub mod apt;
pub mod error;
pub mod merge_upstream;
pub mod release;
pub mod upstream;
pub mod vcs_up_to_date;

pub const DEFAULT_BUILD_DIR: &str = "../build-area";
pub const DEFAULT_ORIG_DIR: &str = "..";
pub const DEFAULT_RESULT_DIR: &str = "..";

use crate::debian::error::Error as DebianError;
use crate::error::Error;
use crate::tree::{Tree, WorkingTree};
use crate::Branch;
use std::collections::HashMap;
use std::path::PathBuf;

use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub enum TarballKind {
    Orig,
    Additional(String),
}

impl From<Option<String>> for TarballKind {
    fn from(kind: Option<String>) -> Self {
        match kind {
            Some(kind) => TarballKind::Additional(kind),
            None => TarballKind::Orig,
        }
    }
}

impl From<TarballKind> for Option<String> {
    fn from(kind: TarballKind) -> Self {
        match kind {
            TarballKind::Orig => None,
            TarballKind::Additional(kind) => Some(kind),
        }
    }
}

impl FromPyObject<'_> for TarballKind {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let kind = ob.extract::<Option<String>>()?;
        Ok(kind.into())
    }
}

impl ToPyObject for TarballKind {
    fn to_object(&self, py: Python) -> PyObject {
        let o: Option<String> = self.clone().into();
        o.to_object(py)
    }
}

impl IntoPy<PyObject> for TarballKind {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

pub fn build_helper(
    local_tree: &WorkingTree,
    subpath: &std::path::Path,
    branch: &dyn Branch,
    target_dir: &std::path::Path,
    builder: &str,
    guess_upstream_branch_url: bool,
    apt_repo: Option<&dyn apt::Apt>,
) -> Result<HashMap<String, PathBuf>, DebianError> {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| -> PyResult<HashMap<String, PathBuf>> {
        let locals = PyDict::new_bound(py);
        locals.set_item("local_tree", local_tree)?;
        locals.set_item("subpath", subpath)?;
        locals.set_item("branch", branch)?;
        locals.set_item("target_dir", target_dir)?;
        locals.set_item("builder", builder)?;
        locals.set_item("guess_upstream_branch_url", guess_upstream_branch_url)?;

        if let Some(apt_repo) = apt_repo {
            locals.set_item("apt", apt_repo.to_object(py))?;
        }

        py.import_bound("breezy.plugins.debian.cmds")?
            .call_method1("_build_helper", (locals,))?
            .extract()
    })
    .map_err(DebianError::from)
}

/// Return the name of the debian tag for the given tree and branch.
///
/// # Arguments
/// * `tree` - The tree to get the debian tag name for.
/// * `branch` - The branch to get the debian tag name for.
/// * `subpath` - The subpath to get the debian tag name for.
/// * `vendor` - The vendor to get the debian tag name for.
///
/// # Returns
/// The name of the debian tag.
pub fn tree_debian_tag_name(
    tree: &dyn Tree,
    branch: &dyn Branch,
    subpath: Option<&std::path::Path>,
    vendor: Option<String>,
) -> Result<String, Error> {
    Python::with_gil(|py| {
        let result = py.import_bound("breezy.plugins.debian")?.call_method1(
            "tree_debian_tag_name",
            (tree.to_object(py), branch.to_object(py), subpath, vendor),
        )?;

        Ok(result.extract()?)
    })
}
