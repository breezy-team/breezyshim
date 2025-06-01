use crate::debian::apt::Apt;
use crate::tree::PyTree;
use debversion::Version;
use pyo3::prelude::*;

/// Status of a Debian package in version control compared to the archive.
#[derive(PartialEq, Eq)]
pub enum UpToDateStatus {
    /// The package in version control is up to date with the archive.
    UpToDate,
    /// The package is missing a changelog file.
    MissingChangelog,
    /// The package does not exist in the archive.
    PackageMissingInArchive {
        /// The name of the package that is missing.
        package: String,
    },
    /// The version in the tree does not exist in the archive.
    TreeVersionNotInArchive {
        /// The version found in the tree.
        tree_version: Version,
        /// The versions available in the archive.
        archive_versions: Vec<Version>,
    },
    /// There's a newer version in the archive than in the tree.
    NewArchiveVersion {
        /// The newest version in the archive.
        archive_version: Version,
        /// The version in the tree.
        tree_version: Version,
    },
}

/// Check if a Debian package in version control is up to date with the archive.
///
/// # Arguments
/// * `tree` - The tree containing the Debian package
/// * `subpath` - The path to the Debian directory in the tree
/// * `apt` - The APT interface to use for checking archive versions
///
/// # Returns
/// The status of the package compared to the archive
pub fn check_up_to_date(
    tree: &dyn PyTree,
    subpath: &std::path::Path,
    apt: &impl Apt,
) -> PyResult<UpToDateStatus> {
    use pyo3::import_exception;
    import_exception!(breezy.plugins.debian.vcs_up_to_date, MissingChangelogError);
    import_exception!(
        breezy.plugins.debian.vcs_up_to_date,
        PackageMissingInArchive
    );
    import_exception!(
        breezy.plugins.debian.vcs_up_to_date,
        TreeVersionNotInArchive
    );
    import_exception!(breezy.plugins.debian.vcs_up_to_date, NewArchiveVersion);
    Python::with_gil(|py| {
        let m = py.import("breezy.plugins.debian.vcs_up_to_date")?;
        let check_up_to_date = m.getattr("check_up_to_date")?;
        match check_up_to_date.call1((tree.to_object(py), subpath.to_path_buf(), apt.as_pyobject()))
        {
            Err(e) if e.is_instance_of::<MissingChangelogError>(py) => {
                Ok(UpToDateStatus::MissingChangelog)
            }
            Err(e) if e.is_instance_of::<PackageMissingInArchive>(py) => {
                Ok(UpToDateStatus::PackageMissingInArchive {
                    package: e.into_value(py).getattr(py, "package")?.extract(py)?,
                })
            }
            Err(e) if e.is_instance_of::<TreeVersionNotInArchive>(py) => {
                let value = e.into_value(py);
                Ok(UpToDateStatus::TreeVersionNotInArchive {
                    tree_version: value.getattr(py, "tree_version")?.extract(py)?,
                    archive_versions: value
                        .getattr(py, "archive_versions")?
                        .extract::<Vec<Version>>(py)?,
                })
            }
            Err(e) if e.is_instance_of::<NewArchiveVersion>(py) => {
                let value = e.into_value(py);
                Ok(UpToDateStatus::NewArchiveVersion {
                    archive_version: value.getattr(py, "archive_version")?.extract(py)?,
                    tree_version: value.getattr(py, "tree_version")?.extract(py)?,
                })
            }
            Ok(_o) => Ok(UpToDateStatus::UpToDate),
            Err(e) => Err(e),
        }
    })
}
