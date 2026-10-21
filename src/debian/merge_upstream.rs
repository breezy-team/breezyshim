use crate::branch::PyBranch;
use crate::debian::error::Error;
use crate::debian::upstream::PyUpstreamSource;
use crate::debian::TarballKind;
use crate::tree::PyTree;
use crate::workingtree::PyWorkingTree;
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
    tree: &dyn PyWorkingTree,
    subpath: &Path,
    tarball_filenames: &[&Path],
    package: &str,
    version: &str,
    current_version: Option<&str>,
    upstream_branch: &dyn PyBranch,
    upstream_revisions: HashMap<TarballKind, (RevisionId, PathBuf)>,
    merge_type: Option<&str>,
    force: bool,
    force_pristine_tar: bool,
    committer: Option<&str>,
    files_excluded: Option<&[&Path]>,
) -> Result<Vec<(TarballKind, String, RevisionId, Option<bool>, PathBuf)>, Error> {
    Python::attach(|py| {
        let m = PyModule::import(py, "breezy.plugins.debian.merge_upstream").unwrap();
        let do_import = m.getattr("do_import").unwrap();
        let kwargs = PyDict::new(py);
        kwargs.set_item("tree", tree.to_object(py))?;
        kwargs.set_item("subpath", subpath.to_string_lossy().to_string())?;
        let tarball_filenames: Vec<String> = tarball_filenames
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        kwargs.set_item("tarball_filenames", tarball_filenames)?;
        kwargs.set_item("package", package)?;
        kwargs.set_item("version", version)?;
        kwargs.set_item("current_version", current_version)?;
        kwargs.set_item("upstream_branch", upstream_branch.to_object(py))?;
        let upstream_revisions: HashMap<TarballKind, (RevisionId, String)> = upstream_revisions
            .into_iter()
            .map(|(k, (r, p))| (k, (r, p.to_string_lossy().to_string())))
            .collect();
        kwargs.set_item("upstream_revisions", upstream_revisions)?;
        kwargs.set_item("merge_type", merge_type)?;
        kwargs.set_item("force", force)?;
        kwargs.set_item("force_pristine_tar", force_pristine_tar)?;
        kwargs.set_item("committer", committer)?;
        let files_excluded: Option<Vec<String>> =
            files_excluded.map(|fs| fs.iter().map(|p| p.to_string_lossy().to_string()).collect());
        kwargs.set_item("files_excluded", files_excluded)?;
        Ok(do_import.call((), Some(&kwargs))?.extract()?)
    })
}

/// Find tarballs for a specific package and version.
///
/// # Arguments
/// * `orig_dir` - Directory containing orig tarballs
/// * `tree` - The working tree
/// * `package` - Package name
/// * `version` - Version string
/// * `locations` - List of additional locations to search for tarballs
///
/// # Returns
/// A list of paths to found tarballs, or an error
pub fn get_tarballs(
    orig_dir: &Path,
    tree: &dyn PyTree,
    package: &str,
    version: &str,
    locations: &[&Path],
) -> Result<Vec<PathBuf>, Error> {
    Python::attach(|py| {
        let m = PyModule::import(py, "breezy.plugins.debian.merge_upstream").unwrap();
        let get_tarballs = m.getattr("get_tarballs").unwrap();
        let locations: Vec<String> = locations
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        Ok(get_tarballs
            .call1((orig_dir, tree.to_object(py), package, version, locations))?
            .extract()?)
    })
}

/// Get revision IDs for already imported upstream versions.
///
/// # Arguments
/// * `upstream_source` - The upstream source to check
/// * `package` - Package name
/// * `new_upstream_version` - The new upstream version being imported
///
/// # Returns
/// A list of tuples with component information: (kind, version, revid, pristine_tar_imported, path)
pub fn get_existing_imported_upstream_revids<T: PyUpstreamSource>(
    upstream_source: &T,
    package: &str,
    new_upstream_version: &str,
) -> Result<Vec<(TarballKind, String, RevisionId, Option<bool>, PathBuf)>, Error> {
    Python::attach(|py| {
        let m = PyModule::import(py, "breezy.plugins.debian.merge_upstream").unwrap();
        let get_existing_imported_upstream_revids =
            m.getattr("get_existing_imported_upstream_revids").unwrap();
        Ok(get_existing_imported_upstream_revids
            .call1((upstream_source.as_pyobject(), package, new_upstream_version))?
            .extract()?)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::create_standalone_workingtree;
    use crate::tree::MutableTree;
    use crate::workingtree::WorkingTree;

    #[test]
    #[serial_test::serial]
    fn test_do_import_no_subpath() {
        let env = crate::testing::TestEnv::new();
        if Python::attach(|py| py.import("breezy.plugins.debian").is_err()) {
            // Skip test if brz-debian plugin is not installed
            return;
        }
        let wt = create_standalone_workingtree(&env.working_dir, "2a").unwrap();
        std::fs::write(env.working_dir.join("README"), "content").unwrap();
        wt.add(&[Path::new("README")]).unwrap();
        let revid = wt
            .build_commit()
            .message("Initial commit")
            .commit()
            .unwrap();

        let upstream_dir = env.temp_dir.path().join("foo-1.0");
        std::fs::create_dir(&upstream_dir).unwrap();
        std::fs::write(upstream_dir.join("hello.c"), "int main() {}\n").unwrap();
        let tarball = env.temp_dir.path().join("foo_1.0.orig.tar.gz");
        Python::attach(|py| -> PyResult<()> {
            let tar = py
                .import("tarfile")?
                .call_method1("open", (tarball.to_str().unwrap(), "w:gz"))?;
            let kwargs = PyDict::new(py);
            kwargs.set_item("arcname", "foo-1.0")?;
            tar.call_method("add", (upstream_dir.to_str().unwrap(),), Some(&kwargs))?;
            tar.call_method0("close")?;
            Ok(())
        })
        .unwrap();

        let branch = wt.branch();
        let imported = do_import(
            &wt,
            Path::new(""),
            &[tarball.as_path()],
            "foo",
            "1.0",
            None,
            &branch,
            HashMap::from([(TarballKind::Orig, (revid, PathBuf::from("")))]),
            None,
            false,
            false,
            None,
            Some(&[Path::new("*.o")]),
        )
        .unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].1, "upstream-1.0");
        std::mem::drop(env);
    }

    #[test]
    #[serial_test::serial]
    fn test_get_tarballs_from_location() {
        let env = crate::testing::TestEnv::new();
        if Python::attach(|py| py.import("breezy.plugins.debian").is_err()) {
            // Skip test if brz-debian plugin is not installed
            return;
        }
        let wt = create_standalone_workingtree(&env.working_dir, "2a").unwrap();

        let upstream_dir = env.temp_dir.path().join("foo-1.0");
        std::fs::create_dir(&upstream_dir).unwrap();
        std::fs::write(upstream_dir.join("hello.c"), "int main() {}\n").unwrap();
        let tarball = env.temp_dir.path().join("foo_1.0.orig.tar.gz");
        Python::attach(|py| -> PyResult<()> {
            let tar = py
                .import("tarfile")?
                .call_method1("open", (tarball.to_str().unwrap(), "w:gz"))?;
            let kwargs = PyDict::new(py);
            kwargs.set_item("arcname", "foo-1.0")?;
            tar.call_method("add", (upstream_dir.to_str().unwrap(),), Some(&kwargs))?;
            tar.call_method0("close")?;
            Ok(())
        })
        .unwrap();

        let orig_dir = env.temp_dir.path().join("orig");
        let tarballs = get_tarballs(&orig_dir, &wt, "foo", "1.0", &[tarball.as_path()]).unwrap();
        assert_eq!(tarballs.len(), 1);
        assert!(orig_dir.join("foo_1.0.orig.tar.gz").exists());
        std::mem::drop(env);
    }
}
