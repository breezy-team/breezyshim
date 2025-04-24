use crate::debian::apt::Apt;
use crate::tree::PyTree;
use debversion::Version;
use pyo3::prelude::*;

#[derive(PartialEq, Eq)]
pub enum UpToDateStatus {
    UpToDate,
    MissingChangelog,
    PackageMissingInArchive {
        package: String,
    },
    TreeVersionNotInArchive {
        tree_version: Version,
        archive_versions: Vec<Version>,
    },
    NewArchiveVersion {
        archive_version: Version,
        tree_version: Version,
    },
}

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
        let m = py.import_bound("breezy.plugins.debian.vcs_up_to_date")?;
        let check_up_to_date = m.getattr("check_up_to_date")?;
        match check_up_to_date.call1((
            tree.to_object(py),
            subpath.to_path_buf(),
            &apt.to_object(py),
        )) {
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
