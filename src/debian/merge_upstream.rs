use crate::branch::Branch;
use crate::debian::upstream::UpstreamSource;
use crate::error::Error;
use crate::tree::Tree;
use crate::workingtree::WorkingTree;
use crate::RevisionId;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Import new tarballs.
///
/// # Arguments
/// * `tree` - Working tree to operate in
/// * `subpath` - Subpath to operate in
/// * `tarball_filenames` - List of tarball filenames as tuples with (path, component)
/// * `package` - Package name
/// * `version` - New upstream version to merge
/// * `current_version` - Current upstream version in tree
/// * `upstream_branch` - Optional upstream branch to merge from
/// * `upstream_revisions` - Dictionary mapping versions to upstream revisions
/// * `merge_type` - Merge type
/// * `committer` - Committer string to use
/// * `files_excluded` - Files to exclude
///
/// # Returns
/// List with (component, tag, revid, pristine_tar_imported, subpath) tuples
pub fn do_import(
    tree: &WorkingTree,
    subpath: &Path,
    tarball_filenames: &[(&Path, &str)],
    package: &str,
    version: &str,
    current_version: &str,
    upstream_branch: &dyn Branch,
    upstream_revisions: HashMap<String, RevisionId>,
    merge_type: Option<&str>,
    force: bool,
    force_pristine_tar: bool,
    committer: Option<&str>,
    files_excluded: Option<&[&Path]>,
) -> Result<Vec<(String, String, RevisionId, bool, PathBuf)>, Error> {
    Python::with_gil(|py| {
        let m = PyModule::import_bound(py, "breezy.plugins.debian.merge_upstream").unwrap();
        let do_import = m.getattr("do_import").unwrap();
        let kwargs = PyDict::new_bound(py);
        kwargs.set_item("tree", tree.to_object(py))?;
        kwargs.set_item("subpath", subpath)?;
        kwargs.set_item("tarball_filenames", tarball_filenames.to_vec())?;
        kwargs.set_item("package", package)?;
        kwargs.set_item("version", version)?;
        kwargs.set_item("current_version", current_version)?;
        kwargs.set_item("upstream_branch", upstream_branch.to_object(py))?;
        kwargs.set_item("upstream_revisions", upstream_revisions.to_object(py))?;
        kwargs.set_item("merge_type", merge_type)?;
        kwargs.set_item("force", force)?;
        kwargs.set_item("force_pristine_tar", force_pristine_tar)?;
        kwargs.set_item("committer", committer)?;
        kwargs.set_item("files_excluded", files_excluded)?;
        Ok(do_import.call((), Some(&kwargs))?.extract()?)
    })
}

pub fn get_tarballs(
    orig_dir: &Path,
    tree: &dyn Tree,
    package: &str,
    version: &str,
    locations: &[&Path],
) -> Result<Vec<PathBuf>, Error> {
    Python::with_gil(|py| {
        let m = PyModule::import_bound(py, "breezy.plugins.debian.merge_upstream").unwrap();
        let get_tarballs = m.getattr("get_tarballs").unwrap();
        Ok(get_tarballs
            .call1((
                orig_dir,
                tree.to_object(py),
                package,
                version,
                locations.to_vec(),
            ))?
            .extract()?)
    })
}

pub fn get_existing_imported_upstream_revids(
    upstream_source: &UpstreamSource,
    package: &str,
    new_upstream_version: &str,
) -> Result<Vec<(String, String, RevisionId, Option<bool>, PathBuf)>, Error> {
    Python::with_gil(|py| {
        let m = PyModule::import_bound(py, "breezy.plugins.debian.merge_upstream").unwrap();
        let get_existing_imported_upstream_revids =
            m.getattr("get_existing_imported_upstream_revids").unwrap();
        Ok(get_existing_imported_upstream_revids
            .call1((upstream_source.to_object(py), package, new_upstream_version))?
            .extract()?)
    })
}
