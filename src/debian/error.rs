use crate::error::Error as BrzError;
use debversion::Version;
use pyo3::import_exception;
use pyo3::prelude::*;

pyo3::import_exception!(breezy.plugins.debian.builder, BuildFailedError);
import_exception!(breezy.plugins.debian.import_dsc, UpstreamAlreadyImported);
import_exception!(breezy.plugins.debian.upstream.branch, DistCommandfailed);
import_exception!(breezy.plugins.debian.upstream, PackageVersionNotPresent);
import_exception!(breezy.plugins.debian.upstream, MissingUpstreamTarball);
import_exception!(breezy.plugins.debian.changelog, UnreleasedChanges);
import_exception!(breezy.plugins.debian.import_dsc, VersionAlreadyImported);

#[derive(Debug)]
pub enum Error {
    BrzError(BrzError),
    BuildFailed,
    UpstreamAlreadyImported(String),
    VersionAlreadyImported {
        package: String,
        version: Version,
        tag_name: String,
    },
    DistCommandFailed(String),
    PackageVersionNotPresent {
        package: String,
        version: String,
    },
    MissingUpstreamTarball {
        package: String,
        version: String,
    },
    UnreleasedChanges,
    ChangeLogError(debian_changelog::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::BrzError(err) => write!(f, "{}", err),
            Error::BuildFailed => write!(f, "Build failed"),
            Error::UpstreamAlreadyImported(version) => {
                write!(f, "Upstream version {} already imported", version)
            }
            Error::DistCommandFailed(err) => write!(f, "Dist command failed: {}", err),
            Error::PackageVersionNotPresent { package, version } => {
                write!(f, "Package {} version {} not present", package, version)
            }
            Error::MissingUpstreamTarball { package, version } => {
                write!(
                    f,
                    "Missing upstream tarball for {} version {}",
                    package, version
                )
            }
            Error::UnreleasedChanges => write!(f, "Unreleased changes"),
            Error::ChangeLogError(err) => write!(f, "{}", err),
            Error::VersionAlreadyImported {
                package,
                version,
                tag_name,
            } => {
                write!(
                    f,
                    "Version {} of package {} already imported with tag {}",
                    version, package, tag_name
                )
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<BrzError> for Error {
    fn from(err: BrzError) -> Error {
        Error::BrzError(err)
    }
}

impl From<debian_changelog::Error> for Error {
    fn from(err: debian_changelog::Error) -> Error {
        Error::ChangeLogError(err)
    }
}

impl From<PyErr> for Error {
    fn from(err: PyErr) -> Error {
        Python::with_gil(|py| {
            let brz_error: BrzError = err.into();
            if let BrzError::Other(ref err) = brz_error {
                if err.is_instance_of::<UpstreamAlreadyImported>(py) {
                    let v = err.value_bound(py);
                    Error::UpstreamAlreadyImported(v.getattr("version").unwrap().extract().unwrap())
                } else if err.is_instance_of::<VersionAlreadyImported>(py) {
                    let v = err.value_bound(py);
                    Error::VersionAlreadyImported {
                        package: v.getattr("package").unwrap().extract().unwrap(),
                        version: v.getattr("version").unwrap().extract().unwrap(),
                        tag_name: v.getattr("tag_name").unwrap().extract().unwrap(),
                    }
                } else if err.is_instance_of::<DistCommandfailed>(py) {
                    let v = err.value_bound(py);
                    Error::DistCommandFailed(v.getattr("error").unwrap().extract().unwrap())
                } else if err.is_instance_of::<PackageVersionNotPresent>(py) {
                    let v = err.value_bound(py);
                    Error::PackageVersionNotPresent {
                        package: v.getattr("package").unwrap().extract().unwrap(),
                        version: v.getattr("version").unwrap().extract().unwrap(),
                    }
                } else if err.is_instance_of::<MissingUpstreamTarball>(py) {
                    let v = err.value_bound(py);
                    Error::MissingUpstreamTarball {
                        package: v.getattr("package").unwrap().extract().unwrap(),
                        version: v.getattr("version").unwrap().extract().unwrap(),
                    }
                } else if err.is_instance_of::<BuildFailedError>(py) {
                    Error::BuildFailed
                } else if err.is_instance_of::<UnreleasedChanges>(py) {
                    Error::UnreleasedChanges
                } else {
                    Error::BrzError(brz_error)
                }
            } else {
                Error::BrzError(brz_error)
            }
        })
    }
}

impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        match err {
            Error::BrzError(err) => err.into(),
            Error::BuildFailed => BuildFailedError::new_err(("Build failed",)).into(),
            Error::UpstreamAlreadyImported(version) => {
                UpstreamAlreadyImported::new_err((version,)).into()
            }
            Error::DistCommandFailed(err) => DistCommandfailed::new_err((err,)).into(),
            Error::PackageVersionNotPresent { package, version } => {
                PackageVersionNotPresent::new_err((package, version)).into()
            }
            Error::MissingUpstreamTarball { package, version } => {
                MissingUpstreamTarball::new_err((package, version)).into()
            }
            Error::UnreleasedChanges => UnreleasedChanges::new_err(()).into(),
            Error::ChangeLogError(_err) => todo!(),
            Error::VersionAlreadyImported {
                package,
                version,
                tag_name,
            } => VersionAlreadyImported::new_err((package, version, tag_name)).into(),
        }
    }
}
