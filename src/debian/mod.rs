//! Debian specific functionality.
//!
//! This module provides functionality for working with Debian packages.
//!
//! It mostly wraps the `breezy.plugins.debian` module from the Breezy VCS.
pub mod apt;
pub mod release;
pub mod vcs_up_to_date;

use crate::error::Error;
use crate::tree::{Tree, WorkingTree};
use crate::Branch;
use debian_control::changes::Changes;

use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Debug)]
pub enum BuildError {
    BuildFailed,
    PackageVersionNotPresent(String, debversion::Version),
    MissingUpstreamTarball(String, debversion::Version),
    Other(pyo3::PyErr),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BuildError::Other(e) => write!(f, "Python error: {}", e),
            BuildError::BuildFailed => write!(f, "Build failed"),
            BuildError::PackageVersionNotPresent(package, version) => {
                write!(f, "Version {} of package {} not present", version, package)
            }
            BuildError::MissingUpstreamTarball(package, version) => {
                write!(
                    f,
                    "Upstream tarball for version {} of package {} not present",
                    version, package
                )
            }
        }
    }
}

impl std::error::Error for BuildError {}

impl From<pyo3::PyErr> for BuildError {
    fn from(e: pyo3::PyErr) -> Self {
        pyo3::import_exception!(breezy.plugins.debian.builder, BuildFailedError);
        pyo3::import_exception!(breezy.plugins.debian.upstream, PackageVersionNotPresent);
        pyo3::import_exception!(breezy.plugins.debian.upstream, MissingUpstreamTarball);

        pyo3::Python::with_gil(|py| {
            if e.is_instance_of::<BuildFailedError>(py) {
                BuildError::BuildFailed
            } else if e.is_instance_of::<PackageVersionNotPresent>(py) {
                let args = e.value_bound(py);
                let package_name = args
                    .getattr("package")
                    .unwrap()
                    .extract::<String>()
                    .unwrap();
                let version = args
                    .getattr("version")
                    .unwrap()
                    .extract::<debversion::Version>()
                    .unwrap();
                BuildError::PackageVersionNotPresent(package_name, version)
            } else if e.is_instance_of::<MissingUpstreamTarball>(py) {
                let args = e.value_bound(py);
                let package_name = args
                    .getattr("package")
                    .unwrap()
                    .extract::<String>()
                    .unwrap();
                let version = args
                    .getattr("version")
                    .unwrap()
                    .extract::<debversion::Version>()
                    .unwrap();
                BuildError::MissingUpstreamTarball(package_name, version)
            } else {
                BuildError::Other(e)
            }
        })
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
) -> Result<Changes, BuildError> {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| -> PyResult<Changes> {
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
    .map_err(BuildError::from)
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
