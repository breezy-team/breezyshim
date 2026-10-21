//! Debian package releasing
use crate::error::Error;
use crate::tree::PyMutableTree;
use pyo3::prelude::*;

/// Errors that can occur when releasing a Debian package.
#[derive(Debug)]
pub enum ReleaseError {
    /// The file was generated and shouldn't be modified directly.
    GeneratedFile,
    /// An error from the underlying Breezy library.
    BrzError(Error),
}

impl From<Error> for ReleaseError {
    fn from(err: Error) -> Self {
        ReleaseError::BrzError(err)
    }
}

impl std::fmt::Display for ReleaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReleaseError::GeneratedFile => write!(f, "Generated file"),
            ReleaseError::BrzError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for ReleaseError {}

/// Release a Debian package by updating the changelog.
///
/// This function updates the changelog to mark the package as released,
/// setting the appropriate fields like the release date.
///
/// # Arguments
/// * `local_tree` - The tree containing the package to release
/// * `subpath` - Path to the debian directory within the tree
///
/// # Returns
/// The name of the tag created for the release, or an error
pub fn release(
    local_tree: &dyn PyMutableTree,
    subpath: &std::path::Path,
) -> Result<String, ReleaseError> {
    pyo3::import_exception!(debmutate.reformatting, GeneratedFile);
    Python::attach(|py| {
        let m = py.import("breezy.plugins.debian.release").unwrap();
        let release = m.getattr("release").unwrap();
        match release.call1((
            local_tree.to_object(py),
            subpath.to_string_lossy().to_string(),
        )) {
            Ok(result) => Ok(result.extract().unwrap()),
            Err(err) if err.is_instance_of::<GeneratedFile>(py) => Err(ReleaseError::GeneratedFile),
            Err(err) => Err(ReleaseError::BrzError(err.into())),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::create_standalone_workingtree;
    use crate::tree::MutableTree;
    use crate::workingtree::WorkingTree;
    use std::path::Path;

    #[test]
    #[serial_test::serial]
    fn test_release_at_root() {
        let env = crate::testing::TestEnv::new();
        if Python::attach(|py| py.import("breezy.plugins.debian").is_err()) {
            // Skip test if brz-debian plugin is not installed
            return;
        }
        let wt = create_standalone_workingtree(&env.working_dir, "2a").unwrap();
        std::fs::create_dir(env.working_dir.join("debian")).unwrap();
        std::fs::write(
            env.working_dir.join("debian/changelog"),
            r#"foo (1.0-1) UNRELEASED; urgency=medium

  * Initial release.

 -- Joe Tester <joe@example.com>  Mon, 01 Jan 2024 00:00:00 +0000
"#,
        )
        .unwrap();
        wt.add(&[Path::new("debian"), Path::new("debian/changelog")])
            .unwrap();
        wt.build_commit()
            .message("Initial commit")
            .commit()
            .unwrap();

        assert_eq!(release(&wt, Path::new("")).unwrap(), "debian/1.0-1");
        std::mem::drop(env);
    }
}
