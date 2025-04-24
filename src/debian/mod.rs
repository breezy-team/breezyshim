//! Debian specific functionality.
//!
//! This module provides functionality for working with Debian packages.
//!
//! It mostly wraps the `breezy.plugins.debian` module from the Breezy VCS.
pub mod apt;
pub mod debcommit;
pub mod directory;
pub mod error;
pub mod import_dsc;
pub mod merge_upstream;
pub mod release;
pub mod upstream;
pub mod vcs_up_to_date;

pub const DEFAULT_BUILD_DIR: &str = "../build-area";
pub const DEFAULT_ORIG_DIR: &str = "..";
pub const DEFAULT_RESULT_DIR: &str = "..";

use crate::debian::error::Error as DebianError;
use crate::error::Error;
use crate::tree::{PyTree, Tree, WorkingTree};
use crate::branch::{Branch, PyBranch};
use std::collections::HashMap;
use std::path::PathBuf;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    Debian,
    Ubuntu,
    Kali,
}

impl std::fmt::Display for Vendor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Vendor::Debian => write!(f, "debian"),
            Vendor::Ubuntu => write!(f, "ubuntu"),
            Vendor::Kali => write!(f, "kali"),
        }
    }
}

impl std::str::FromStr for Vendor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debian" => Ok(Vendor::Debian),
            "ubuntu" => Ok(Vendor::Ubuntu),
            "kali" => Ok(Vendor::Kali),
            _ => Err(format!("Invalid vendor: {}", s)),
        }
    }
}

impl FromPyObject<'_> for Vendor {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let vendor = ob.extract::<String>()?;
        match vendor.as_str() {
            "debian" => Ok(Vendor::Debian),
            "ubuntu" => Ok(Vendor::Ubuntu),
            "kali" => Ok(Vendor::Kali),
            _ => Err(PyValueError::new_err((format!(
                "Invalid vendor: {}",
                vendor
            ),))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash, Default)]
pub enum VersionKind {
    #[default]
    Auto,
    Snapshot,
    Release,
}

impl std::str::FromStr for VersionKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(VersionKind::Auto),
            "snapshot" => Ok(VersionKind::Snapshot),
            "release" => Ok(VersionKind::Release),
            _ => Err(format!("Invalid version kind: {}", s)),
        }
    }
}

impl ToPyObject for VersionKind {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            VersionKind::Auto => "auto".to_object(py),
            VersionKind::Snapshot => "snapshot".to_object(py),
            VersionKind::Release => "release".to_object(py),
        }
    }
}

impl std::fmt::Display for VersionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VersionKind::Auto => write!(f, "auto"),
            VersionKind::Snapshot => write!(f, "snapshot"),
            VersionKind::Release => write!(f, "release"),
        }
    }
}

impl FromPyObject<'_> for VersionKind {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let kind = ob.extract::<String>()?;
        match kind.as_str() {
            "auto" => Ok(VersionKind::Auto),
            "snapshot" => Ok(VersionKind::Snapshot),
            "release" => Ok(VersionKind::Release),
            _ => Err(PyValueError::new_err((format!(
                "Invalid version kind: {}",
                kind
            ),))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub enum TarballKind {
    Orig,
    Additional(String),
}

impl serde::ser::Serialize for TarballKind {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TarballKind::Orig => serializer.serialize_none(),
            TarballKind::Additional(kind) => serializer.serialize_some(kind),
        }
    }
}

impl<'a> serde::de::Deserialize<'a> for TarballKind {
    fn deserialize<D: serde::de::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let kind = Option::<String>::deserialize(deserializer)?;
        Ok(kind.into())
    }
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
    branch: &dyn PyBranch,
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
    tree: &dyn PyTree,
    branch: &dyn PyBranch,
    subpath: Option<&std::path::Path>,
    vendor: Option<Vendor>,
) -> Result<String, Error> {
    Python::with_gil(|py| {
        let result = py.import_bound("breezy.plugins.debian")?.call_method1(
            "tree_debian_tag_name",
            (
                tree.to_object(py),
                branch.to_object(py),
                subpath,
                vendor.map(|v| v.to_string()),
            ),
        )?;

        Ok(result.extract()?)
    })
}

// TODO(jelmer): deduplicate this with the suite_to_distribution function
// in debian-analyzer
/// Infer the distribution from a suite.
///
/// When passed the name of a suite (anything in the distributions field of
/// a changelog) it will infer the distribution from that (i.e. Debian or
/// Ubuntu).
///
/// # Arguments
/// * `suite`: the string containing the suite
///
/// # Returns
/// Vendor or None if the distribution cannot be inferred.
pub fn suite_to_distribution(suite: &str) -> Option<Vendor> {
    Python::with_gil(|py| -> PyResult<Option<Vendor>> {
        let result = py
            .import_bound("breezy.plugins.debian.util")?
            .call_method1("suite_to_distribution", (suite,))?;

        Ok(result.extract()?)
    })
    .unwrap()
}
