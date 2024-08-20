use crate::error::Error as BrzError;
use pyo3::import_exception;
use pyo3::prelude::*;

pyo3::import_exception!(breezy.plugins.debian.builder, BuildFailedError);
import_exception!(breezy.plugins.debian.import_dsc, UpstreamAlreadyImported);
import_exception!(breezy.plugins.debian.upstream.branch, DistCommandfailed);
import_exception!(breezy.plugins.debian.upstream, PackageVersionNotPresent);
import_exception!(breezy.plugins.debian.upstream, MissingUpstreamTarball);

#[derive(Debug)]
pub enum Error {
    BrzError(BrzError),
    BuildFailed,
    UpstreamAlreadyImported(String),
    DistCommandFailed(String),
    PackageVersionNotPresent { package: String, version: String },
    MissingUpstreamTarball { package: String, version: String },
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
        }
    }
}

impl std::error::Error for Error {}

impl From<BrzError> for Error {
    fn from(err: BrzError) -> Error {
        Error::BrzError(err)
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
                } else {
                    Error::BrzError(brz_error)
                }
            } else {
                Error::BrzError(brz_error)
            }
        })
    }
}
