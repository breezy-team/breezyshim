use pyo3::import_exception;
use pyo3::prelude::*;
use pyo3::PyErr;

import_exception!(breezy.errors, UnknownFormatError);
import_exception!(breezy.errors, NotBranchError);
import_exception!(breezy.controldir, NoColocatedBranchSupport);
import_exception!(breezy.errors, DependencyNotPresent);
import_exception!(breezy.errors, PermissionDenied);
import_exception!(breezy.transport, UnsupportedProtocol);
import_exception!(breezy.transport, UnusableRedirect);
import_exception!(breezy.urlutils, InvalidURL);
import_exception!(breezy.errors, TransportError);
import_exception!(breezy.errors, UnsupportedFormatError);
import_exception!(breezy.errors, UnsupportedVcs);
import_exception!(breezy.git.remote, RemoteGitError);
import_exception!(http.client, IncompleteRead);
import_exception!(breezy.bzr, LineEndingError);
import_exception!(breezy.errors, InvalidHttpResponse);
import_exception!(breezy.errors, AlreadyControlDirError);
import_exception!(breezy.errors, DivergedBranches);
import_exception!(breezy.workspace, WorkspaceDirty);
import_exception!(breezy.transport, NoSuchFile);
import_exception!(breezy.commit, PointlessCommit);
import_exception!(breezy.commit, NoWhoami);
import_exception!(breezy.errors, NoSuchTag);
import_exception!(breezy.errors, TagAlreadyExists);

#[derive(Debug)]
pub enum Error {
    Other(PyErr),
    UnknownFormat(String),
    NotBranchError(String, Option<String>),
    NoColocatedBranchSupport,
    DependencyNotPresent(String, String),
    PermissionDenied(std::path::PathBuf, Option<String>),
    UnsupportedProtocol(String, Option<String>),
    UnusableRedirect(String, String, String),
    ConnectionError(String),
    InvalidURL(String, Option<String>),
    TransportError(String),
    UnsupportedFormat(String),
    UnsupportedVcs(String),
    RemoteGitError(String),
    IncompleteRead,
    LineEndingError(String),
    InvalidHttpResponse(
        String,
        String,
        Option<String>,
        std::collections::HashMap<String, String>,
    ),
    AlreadyExists,
    DivergedBranches,
    WorkspaceDirty(std::path::PathBuf),
    NoSuchFile(std::path::PathBuf),
    PointlessCommit,
    NoWhoami,
    NoSuchTag(String),
    TagAlreadyExists(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Other(e) => write!(f, "Error::Other({})", e),
            Self::UnknownFormat(s) => write!(f, "Unknown format: {}", s),
            Self::NotBranchError(path, detail) => {
                if let Some(detail) = detail {
                    write!(f, "Not a branch: {}: {}", path, detail)
                } else {
                    write!(f, "Not a branch: {}", path)
                }
            }
            Self::NoColocatedBranchSupport => write!(f, "No colocated branch support"),
            Self::DependencyNotPresent(d, r) => write!(f, "Dependency {} not present: {}", d, r),
            Self::PermissionDenied(p, r) => {
                if let Some(r) = r {
                    write!(f, "Permission denied: {}: {}", p.display(), r)
                } else {
                    write!(f, "Permission denied: {}", p.display())
                }
            }
            Self::UnsupportedProtocol(p, r) => {
                if let Some(r) = r {
                    write!(f, "Unsupported protocol: {}: {}", p, r)
                } else {
                    write!(f, "Unsupported protocol: {}", p)
                }
            }
            Self::UnusableRedirect(p, r, u) => {
                write!(f, "Unusable redirect: {}: {} -> {}", p, r, u)
            }
            Self::ConnectionError(e) => write!(f, "Connection error: {}", e),
            Self::InvalidURL(p, r) => {
                if let Some(r) = r {
                    write!(f, "Invalid URL: {}: {}", p, r)
                } else {
                    write!(f, "Invalid URL: {}", p)
                }
            }
            Self::TransportError(e) => write!(f, "Transport error: {}", e),
            Self::UnsupportedFormat(s) => write!(f, "Unsupported format: {}", s),
            Self::UnsupportedVcs(s) => write!(f, "Unsupported VCS: {}", s),
            Self::RemoteGitError(e) => write!(f, "Remote Git error: {}", e),
            Self::IncompleteRead => write!(f, "Incomplete read"),
            Self::LineEndingError(e) => write!(f, "Line ending error: {}", e),
            Self::InvalidHttpResponse(s, c, b, _hs) => {
                if let Some(b) = b {
                    write!(f, "Invalid HTTP response: {} {}: {}", s, c, b)
                } else {
                    write!(f, "Invalid HTTP response: {} {}", s, c)
                }
            }
            Self::AlreadyExists => write!(f, "Already exists"),
            Self::DivergedBranches => write!(f, "Diverged branches"),
            Self::WorkspaceDirty(p) => write!(f, "Workspace dirty at {}", p.display()),
            Self::NoSuchFile(p) => write!(f, "No such file: {}", p.to_string_lossy()),
            Self::PointlessCommit => write!(f, "Pointless commit"),
            Self::NoWhoami => write!(f, "No whoami"),

            Self::NoSuchTag(tag) => write!(f, "No such tag: {}", tag),
            Self::TagAlreadyExists(tag) => write!(f, "Tag already exists: {}", tag),
        }
    }
}

impl std::error::Error for Error {}

impl From<PyErr> for Error {
    fn from(err: PyErr) -> Self {
        pyo3::Python::with_gil(|py| {
            let value = err.value_bound(py);
            if err.is_instance_of::<UnknownFormatError>(py) {
                Error::UnknownFormat(value.getattr("format").unwrap().extract().unwrap())
            } else if err.is_instance_of::<NotBranchError>(py) {
                Error::NotBranchError(
                    value.getattr("path").unwrap().extract().unwrap(),
                    value.getattr("details").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<NoColocatedBranchSupport>(py) {
                Error::NoColocatedBranchSupport
            } else if err.is_instance_of::<DependencyNotPresent>(py) {
                Error::DependencyNotPresent(
                    value.getattr("library").unwrap().extract().unwrap(),
                    value.getattr("error").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<PermissionDenied>(py) {
                Error::PermissionDenied(
                    value.getattr("path").unwrap().extract().unwrap(),
                    value.getattr("extra").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<UnsupportedProtocol>(py) {
                Error::UnsupportedProtocol(
                    value.getattr("url").unwrap().extract().unwrap(),
                    value.getattr("extra").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<UnusableRedirect>(py) {
                Error::UnusableRedirect(
                    value.getattr("source").unwrap().extract().unwrap(),
                    value.getattr("target").unwrap().extract().unwrap(),
                    value.getattr("reason").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<InvalidURL>(py) {
                Error::InvalidURL(
                    value.getattr("path").unwrap().extract().unwrap(),
                    value.getattr("extra").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<pyo3::exceptions::PyConnectionError>(py) {
                Error::ConnectionError(value.getattr("message").unwrap().extract().unwrap())
            } else if err.is_instance_of::<TransportError>(py) {
                Error::TransportError(value.getattr("message").unwrap().extract().unwrap())
            } else if err.is_instance_of::<UnsupportedFormatError>(py) {
                Error::UnsupportedFormat(value.getattr("format").unwrap().extract().unwrap())
            } else if err.is_instance_of::<UnsupportedVcs>(py) {
                Error::UnsupportedVcs(value.getattr("vcs").unwrap().extract().unwrap())
            } else if err.is_instance_of::<RemoteGitError>(py) {
                Error::RemoteGitError(value.getattr("msg").unwrap().extract().unwrap())
            } else if err.is_instance_of::<IncompleteRead>(py) {
                Error::IncompleteRead
            } else if err.is_instance_of::<LineEndingError>(py) {
                Error::LineEndingError(value.getattr("file").unwrap().extract().unwrap())
            } else if err.is_instance_of::<InvalidHttpResponse>(py) {
                Error::InvalidHttpResponse(
                    value.getattr("path").unwrap().extract().unwrap(),
                    value.getattr("msg").unwrap().extract().unwrap(),
                    value.getattr("orig_error").unwrap().extract().unwrap(),
                    value.getattr("headers").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<AlreadyControlDirError>(py) {
                Error::AlreadyExists
            } else if err.is_instance_of::<DivergedBranches>(py) {
                Error::DivergedBranches
            } else if err.is_instance_of::<WorkspaceDirty>(py) {
                let value = err.into_value(py);
                let tree = value.getattr(py, "tree").unwrap();
                let path = value.getattr(py, "path").unwrap();
                let path = tree
                    .call_method1(py, "abspath", (path,))
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap();
                Error::WorkspaceDirty(std::path::PathBuf::from(path))
            } else if err.is_instance_of::<NoSuchFile>(py) {
                Error::NoSuchFile(std::path::PathBuf::from(
                    value.getattr("path").unwrap().extract::<String>().unwrap(),
                ))
            } else if err.is_instance_of::<PointlessCommit>(py) {
                Error::PointlessCommit
            } else if err.is_instance_of::<NoWhoami>(py) {
                Error::NoWhoami
            } else if err.is_instance_of::<NoSuchTag>(py) {
                Error::NoSuchTag(value.getattr("tag_name").unwrap().extract().unwrap())
            } else if err.is_instance_of::<TagAlreadyExists>(py) {
                Error::TagAlreadyExists(value.getattr("tag_name").unwrap().extract().unwrap())
            } else {
                Self::Other(err)
            }
        })
    }
}

impl From<Error> for PyErr {
    fn from(e: Error) -> Self {
        match e {
            Error::Other(e) => e,
            Error::UnknownFormat(s) => UnknownFormatError::new_err((s,)),
            Error::NotBranchError(path, details) => NotBranchError::new_err((path, details)),
            Error::NoColocatedBranchSupport => NoColocatedBranchSupport::new_err(()),
            Error::DependencyNotPresent(library, error) => {
                DependencyNotPresent::new_err((library, error))
            }
            Error::PermissionDenied(path, reason) => PermissionDenied::new_err((path, reason)),
            Error::UnsupportedProtocol(url, error) => UnsupportedProtocol::new_err((url, error)),
            Error::UnusableRedirect(source, target, reason) => {
                UnusableRedirect::new_err((source, target, reason))
            }
            Error::ConnectionError(e) => pyo3::exceptions::PyConnectionError::new_err((e,)),
            Error::InvalidURL(path, reason) => InvalidURL::new_err((path, reason)),
            Error::TransportError(e) => TransportError::new_err((e,)),
            Error::UnsupportedFormat(s) => UnsupportedFormatError::new_err((s,)),
            Error::UnsupportedVcs(s) => UnsupportedVcs::new_err((s,)),
            Error::RemoteGitError(e) => RemoteGitError::new_err((e,)),
            Error::IncompleteRead => IncompleteRead::new_err(()),
            Error::LineEndingError(e) => LineEndingError::new_err((e,)),
            Error::InvalidHttpResponse(status, msg, orig_error, headers) => {
                InvalidHttpResponse::new_err((status, msg, orig_error, headers))
            }
            Error::AlreadyExists => AlreadyControlDirError::new_err(()),
            Error::DivergedBranches => DivergedBranches::new_err(()),
            Error::WorkspaceDirty(p) => WorkspaceDirty::new_err(p.to_string_lossy().to_string()),
            Error::NoSuchFile(p) => NoSuchFile::new_err(p.to_string_lossy().to_string()),
            Error::PointlessCommit => PointlessCommit::new_err(()),
            Error::NoWhoami => NoWhoami::new_err(()),
            Error::NoSuchTag(tag) => NoSuchTag::new_err((tag,)),
            Error::TagAlreadyExists(tag) => TagAlreadyExists::new_err((tag,)),
        }
    }
}
