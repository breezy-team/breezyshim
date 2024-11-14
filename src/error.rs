//! Error handling for the Breezy Python bindings
use crate::transform::RawConflict;
use pyo3::import_exception;
use pyo3::prelude::*;
use pyo3::PyErr;
use url::Url;

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
import_exception!(breezy.git.remote, ProtectedBranchHookDeclined);
import_exception!(http.client, IncompleteRead);
import_exception!(breezy.bzr, LineEndingError);
import_exception!(breezy.errors, InvalidHttpResponse);
import_exception!(breezy.errors, AlreadyControlDirError);
import_exception!(breezy.errors, AlreadyBranchError);
import_exception!(breezy.errors, DivergedBranches);
import_exception!(breezy.workspace, WorkspaceDirty);
import_exception!(breezy.transport, NoSuchFile);
import_exception!(breezy.commit, PointlessCommit);
import_exception!(breezy.errors, NoWhoami);
import_exception!(breezy.errors, NoSuchTag);
import_exception!(breezy.errors, TagAlreadyExists);
import_exception!(breezy.forge, ForgeLoginRequired);
import_exception!(breezy.forge, UnsupportedForge);
import_exception!(breezy.forge, MergeProposalExists);
import_exception!(breezy.errors, UnsupportedOperation);
import_exception!(breezy.errors, NoRepositoryPresent);
import_exception!(breezy.errors, LockFailed);
import_exception!(breezy.errors, LockContention);
import_exception!(breezy.transport, FileExists);
import_exception!(breezy.errors, NoSuchRevisionInTree);
import_exception!(breezy.tree, MissingNestedTree);
import_exception!(breezy.transform, ImmortalLimbo);
import_exception!(breezy.transform, MalformedTransform);
import_exception!(breezy.transform, TransformRenameFailed);
import_exception!(breezy.errors, UnexpectedHttpStatus);
import_exception!(breezy.errors, BadHttpRequest);
import_exception!(breezy.errors, TransportNotPossible);
import_exception!(breezy.errors, IncompatibleFormat);
import_exception!(breezy.errors, NoSuchRevision);
import_exception!(breezy.forge, NoSuchProject);
import_exception!(breezy.plugins.gitlab.forge, ForkingDisabled);
import_exception!(breezy.plugins.gitlab.forge, GitLabConflict);
import_exception!(breezy.plugins.gitlab.forge, ProjectCreationTimeout);
import_exception!(breezy.forge, SourceNotDerivedFromTarget);
import_exception!(breezy.controldir, BranchReferenceLoop);
import_exception!(breezy.errors, RedirectRequested);
import_exception!(breezy.errors, ConflictsInTree);
import_exception!(breezy.errors, NoRoundtrippingSupport);
import_exception!(breezy.errors, NoCompatibleInter);

lazy_static::lazy_static! {
    // Only present in breezy << 4.0
    pub static ref BreezyConnectionError: Option<PyObject> = { Python::with_gil(|py| {
        let m = py.import_bound("breezy.errors").unwrap();
        m.getattr("ConnectionError").ok().map(|x| x.to_object(py))
    })
};
}

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
    IncompleteRead(Vec<u8>, Option<usize>),
    LineEndingError(String),
    InvalidHttpResponse(
        String,
        String,
        Option<String>,
        std::collections::HashMap<String, String>,
    ),
    AlreadyControlDir(std::path::PathBuf),
    AlreadyBranch(std::path::PathBuf),
    DivergedBranches,
    WorkspaceDirty(std::path::PathBuf),
    NoSuchFile(std::path::PathBuf),
    PointlessCommit,
    NoWhoami,
    NoSuchTag(String),
    TagAlreadyExists(String),
    Socket(std::io::Error),
    ForgeLoginRequired,
    UnsupportedForge(url::Url),
    ForgeProjectExists(String),
    MergeProposalExists(url::Url, Option<url::Url>),
    UnsupportedOperation(String, String),
    ProtectedBranchHookDeclined(String),
    NoRepositoryPresent,
    LockFailed(String),
    FileExists(std::path::PathBuf, Option<String>),
    LockContention(String, String),
    NotImplemented,
    NoSuchRevisionInTree(crate::RevisionId),
    MissingNestedTree(std::path::PathBuf),
    /// Failed to delete transform temporary directory
    ImmortalLimbo(std::path::PathBuf),
    MalformedTransform(Vec<RawConflict>),
    TransformRenameFailed(
        std::path::PathBuf,
        std::path::PathBuf,
        String,
        std::io::Error,
    ),
    UnexpectedHttpStatus {
        url: url::Url,
        code: u16,
        extra: Option<String>,
        headers: std::collections::HashMap<String, String>,
    },
    Timeout,
    BadHttpRequest(Url, String),
    TransportNotPossible(String),
    IncompatibleFormat(String, String),
    NoSuchRevision(crate::RevisionId),
    NoSuchProject(String),
    ForkingDisabled(String),
    ProjectCreationTimeout(String, chrono::Duration),
    GitLabConflict(String),
    SourceNotDerivedFromTarget,
    BranchReferenceLoop,
    RedirectRequested {
        source: url::Url,
        target: url::Url,
        is_permanent: bool,
    },
    ConflictsInTree,
    NoRoundtrippingSupport,
    NoCompatibleInter,
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::InvalidURL(e.to_string(), None)
    }
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
            Self::IncompleteRead(partial, expected) => {
                write!(f, "Incomplete read: {:?} {:?}", partial, expected)
            }
            Self::LineEndingError(e) => write!(f, "Line ending error: {}", e),
            Self::InvalidHttpResponse(s, c, b, _hs) => {
                if let Some(b) = b {
                    write!(f, "Invalid HTTP response: {} {}: {}", s, c, b)
                } else {
                    write!(f, "Invalid HTTP response: {} {}", s, c)
                }
            }
            Self::AlreadyControlDir(p) => write!(f, "Already exists: {}", p.display()),
            Self::AlreadyBranch(p) => write!(f, "Already a branch: {}", p.display()),
            Self::DivergedBranches => write!(f, "Diverged branches"),
            Self::WorkspaceDirty(p) => write!(f, "Workspace dirty at {}", p.display()),
            Self::NoSuchFile(p) => write!(f, "No such file: {}", p.to_string_lossy()),
            Self::PointlessCommit => write!(f, "Pointless commit"),
            Self::NoWhoami => write!(f, "No whoami"),

            Self::NoSuchTag(tag) => write!(f, "No such tag: {}", tag),
            Self::TagAlreadyExists(tag) => write!(f, "Tag already exists: {}", tag),
            Self::Socket(e) => write!(f, "socket error: {}", e),
            Self::ForgeLoginRequired => write!(f, "Forge login required"),
            Self::UnsupportedForge(url) => write!(f, "Unsupported forge: {}", url),
            Self::ForgeProjectExists(p) => write!(f, "Forge project exists: {}", p),
            Self::MergeProposalExists(p, r) => {
                if let Some(r) = r {
                    write!(f, "Merge proposal exists: {} -> {}", p, r)
                } else {
                    write!(f, "Merge proposal exists: {}", p)
                }
            }
            Self::UnsupportedOperation(a, b) => write!(f, "Unsupported operation: {} on {}", a, b),
            Self::ProtectedBranchHookDeclined(e) => {
                write!(f, "Protected branch hook declined: {}", e)
            }
            Self::NoRepositoryPresent => write!(f, "No repository present"),
            Self::LockFailed(w) => write!(f, "Lock failed: {}", w),
            Self::FileExists(p, r) => {
                if let Some(r) = r {
                    write!(f, "File exists: {}: {}", p.display(), r)
                } else {
                    write!(f, "File exists: {}", p.display())
                }
            }
            Self::LockContention(a, b) => write!(f, "Lock contention: {} {}", a, b),
            Self::NotImplemented => write!(f, "Not implemented"),
            Self::NoSuchRevisionInTree(rev) => write!(f, "No such revision in tree: {}", rev),
            Self::MissingNestedTree(p) => write!(f, "Missing nested tree: {}", p.display()),
            Self::ImmortalLimbo(p) => write!(
                f,
                "Failed to delete transform temporary directory: {}",
                p.display()
            ),
            Self::MalformedTransform(e) => write!(f, "Malformed transform: {:?}", e),
            Self::TransformRenameFailed(a, b, c, d) => write!(
                f,
                "Transform rename failed: {} -> {}: {}: {}",
                a.display(),
                b.display(),
                c,
                d
            ),
            Self::UnexpectedHttpStatus {
                url,
                code,
                extra,
                headers: _,
            } => {
                if let Some(extra) = extra {
                    write!(f, "Unexpected HTTP status: {} {}: {}", url, code, extra)
                } else {
                    write!(f, "Unexpected HTTP status: {} {}", url, code)
                }
            }
            Self::Timeout => write!(f, "Timeout"),
            Self::BadHttpRequest(url, msg) => write!(f, "Bad HTTP request: {} {}", url, msg),
            Self::TransportNotPossible(e) => write!(f, "Transport not possible: {}", e),
            Self::IncompatibleFormat(a, b) => {
                write!(f, "Incompatible format: {} is not compatible with {}", a, b)
            }
            Self::NoSuchRevision(rev) => write!(f, "No such revision: {}", rev),
            Self::NoSuchProject(p) => write!(f, "No such project: {}", p),
            Self::ForkingDisabled(p) => write!(f, "Forking disabled: {}", p),
            Self::ProjectCreationTimeout(p, t) => {
                write!(f, "Project creation timeout: {} after {} seconds", p, t)
            }
            Self::GitLabConflict(p) => write!(f, "GitLab conflict: {}", p),
            Self::ConflictsInTree => write!(f, "Conflicts in tree"),
            Self::SourceNotDerivedFromTarget => write!(f, "Source not derived from target"),
            Self::BranchReferenceLoop => write!(f, "Branch reference loop"),
            Self::NoRoundtrippingSupport => write!(f, "No roundtripping support"),
            Self::NoCompatibleInter => write!(f, "No compatible inter"),
            Self::RedirectRequested {
                source,
                target,
                is_permanent,
            } => {
                write!(
                    f,
                    "Redirect requested: {} -> {} (permanent: {})",
                    source, target, is_permanent
                )
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<PyErr> for Error {
    fn from(err: PyErr) -> Self {
        pyo3::import_exception!(socket, error);
        pyo3::Python::with_gil(|py| {
            let value = err.value_bound(py);
            if err.is_instance_of::<UnknownFormatError>(py) {
                Error::UnknownFormat(value.getattr("format").unwrap().extract().unwrap())
            } else if err.is_instance_of::<NotBranchError>(py) {
                Error::NotBranchError(
                    value.getattr("path").unwrap().extract().unwrap(),
                    value.getattr("detail").unwrap().extract().unwrap(),
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
                Error::ConnectionError(err.to_string())
            } else if err.is_instance_of::<UnsupportedFormatError>(py) {
                Error::UnsupportedFormat(value.getattr("format").unwrap().extract().unwrap())
            } else if err.is_instance_of::<UnsupportedVcs>(py) {
                Error::UnsupportedVcs(value.getattr("vcs").unwrap().extract().unwrap())
            } else if err.is_instance_of::<RemoteGitError>(py) {
                if let Ok(e) = value.getattr("msg").unwrap().extract() {
                    Error::RemoteGitError(e)
                } else {
                    // Just get it from the args tuple
                    Error::RemoteGitError(value.getattr("args").unwrap().extract().unwrap())
                }
            } else if err.is_instance_of::<IncompleteRead>(py) {
                Error::IncompleteRead(
                    value.getattr("partial").unwrap().extract().unwrap(),
                    value.getattr("expected").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<LineEndingError>(py) {
                Error::LineEndingError(value.getattr("file").unwrap().extract().unwrap())
            } else if err.is_instance_of::<AlreadyControlDirError>(py) {
                Error::AlreadyControlDir(value.getattr("path").unwrap().extract().unwrap())
            } else if err.is_instance_of::<AlreadyBranchError>(py) {
                Error::AlreadyBranch(value.getattr("path").unwrap().extract().unwrap())
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
            } else if err.is_instance_of::<error>(py) {
                Error::Socket(std::io::Error::from_raw_os_error(
                    value.getattr("errno").unwrap().extract().unwrap(),
                ))
            } else if err.is_instance_of::<ForgeLoginRequired>(py) {
                Error::ForgeLoginRequired
            } else if err.is_instance_of::<UnsupportedForge>(py) {
                let branch = value.getattr("branch").unwrap();

                if let Ok(url) = branch.getattr("user_url") {
                    Error::UnsupportedForge(url.extract::<String>().unwrap().parse().unwrap())
                } else {
                    Error::UnsupportedForge(branch.extract::<String>().unwrap().parse().unwrap())
                }
            } else if err.is_instance_of::<MergeProposalExists>(py) {
                let source_url: String = value.getattr("url").unwrap().extract().unwrap();
                let existing_proposal = value.getattr("existing_proposal").unwrap();
                let target_url: Option<String> = if existing_proposal.is_none() {
                    None
                } else {
                    Some(existing_proposal.getattr("url").unwrap().extract().unwrap())
                };
                Error::MergeProposalExists(
                    source_url.parse().unwrap(),
                    target_url.map(|u| u.parse().unwrap()),
                )
            } else if err.is_instance_of::<UnsupportedOperation>(py) {
                Error::UnsupportedOperation(
                    value.getattr("mname").unwrap().extract().unwrap(),
                    value.getattr("tname").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<ProtectedBranchHookDeclined>(py) {
                Error::ProtectedBranchHookDeclined(value.getattr("msg").unwrap().extract().unwrap())
            } else if err.is_instance_of::<NoRepositoryPresent>(py) {
                Error::NoRepositoryPresent
            } else if err.is_instance_of::<LockFailed>(py) {
                let why = value.getattr("why").unwrap();
                if why.is_none() {
                    Error::LockFailed("".to_string())
                } else {
                    let why = why.call_method0("__str__").unwrap();
                    Error::LockFailed(why.extract().unwrap())
                }
            } else if err.is_instance_of::<FileExists>(py) {
                Error::FileExists(
                    std::path::PathBuf::from(
                        value.getattr("path").unwrap().extract::<String>().unwrap(),
                    ),
                    value.getattr("extra").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<LockContention>(py) {
                Error::LockContention(
                    value
                        .getattr("lock")
                        .unwrap()
                        .call_method0("__str__")
                        .unwrap()
                        .extract()
                        .unwrap(),
                    value.getattr("msg").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<pyo3::exceptions::PyNotImplementedError>(py) {
                Error::NotImplemented
            } else if err.is_instance_of::<NoSuchRevisionInTree>(py) {
                Error::NoSuchRevisionInTree(
                    value.getattr("revision_id").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<MissingNestedTree>(py) {
                Error::MissingNestedTree(std::path::PathBuf::from(
                    value.getattr("path").unwrap().extract::<String>().unwrap(),
                ))
            } else if err.is_instance_of::<ImmortalLimbo>(py) {
                Error::ImmortalLimbo(std::path::PathBuf::from(
                    value
                        .getattr("limbo_dir")
                        .unwrap()
                        .extract::<String>()
                        .unwrap(),
                ))
            } else if err.is_instance_of::<MalformedTransform>(py) {
                Error::MalformedTransform(value.getattr("conflicts").unwrap().extract().unwrap())
            } else if err.is_instance_of::<TransformRenameFailed>(py) {
                Error::TransformRenameFailed(
                    std::path::PathBuf::from(
                        value
                            .getattr("from_path")
                            .unwrap()
                            .extract::<String>()
                            .unwrap(),
                    ),
                    std::path::PathBuf::from(
                        value
                            .getattr("to_path")
                            .unwrap()
                            .extract::<String>()
                            .unwrap(),
                    ),
                    value.getattr("why").unwrap().extract().unwrap(),
                    std::io::Error::from_raw_os_error(
                        value.getattr("errno").unwrap().extract::<i32>().unwrap(),
                    ),
                )
            } else if err.is_instance_of::<UnexpectedHttpStatus>(py) {
                Error::UnexpectedHttpStatus {
                    url: value
                        .getattr("path")
                        .unwrap()
                        .extract::<String>()
                        .unwrap()
                        .parse()
                        .unwrap(),
                    code: value.getattr("code").unwrap().extract().unwrap(),
                    extra: value.getattr("extra").unwrap().extract().unwrap(),
                    headers: value.getattr("headers").unwrap().extract().unwrap(),
                }
            } else if err.is_instance_of::<pyo3::exceptions::PyTimeoutError>(py) {
                Error::Timeout
            } else if err.is_instance_of::<BadHttpRequest>(py) {
                Error::BadHttpRequest(
                    value
                        .getattr("path")
                        .unwrap()
                        .extract::<String>()
                        .unwrap()
                        .parse()
                        .unwrap(),
                    value.getattr("reason").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<TransportNotPossible>(py) {
                Error::TransportNotPossible(value.getattr("msg").unwrap().extract().unwrap())
            } else if err.is_instance_of::<IncompatibleFormat>(py) {
                let format = value.getattr("format").unwrap();
                let controldir = value.getattr("controldir").unwrap();
                Error::IncompatibleFormat(
                    if let Ok(format) = format.extract::<String>() {
                        format
                    } else {
                        format
                            .call_method0("get_format_string")
                            .unwrap()
                            .extract()
                            .unwrap()
                    },
                    if let Ok(controldir) = controldir.extract::<String>() {
                        controldir
                    } else {
                        controldir
                            .call_method0("get_format_string")
                            .unwrap()
                            .extract()
                            .unwrap()
                    },
                )
            } else if err.is_instance_of::<NoSuchRevision>(py) {
                Error::NoSuchRevision(value.getattr("revision").unwrap().extract().unwrap())
            } else if err.is_instance_of::<NoSuchProject>(py) {
                Error::NoSuchProject(value.getattr("project").unwrap().extract().unwrap())
            } else if err.is_instance_of::<ForkingDisabled>(py) {
                Error::ForkingDisabled(value.getattr("project").unwrap().extract().unwrap())
            } else if err.is_instance_of::<ProjectCreationTimeout>(py) {
                Error::ProjectCreationTimeout(
                    value.getattr("project").unwrap().extract().unwrap(),
                    value.getattr("timeout").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<GitLabConflict>(py) {
                Error::GitLabConflict(value.getattr("reason").unwrap().extract().unwrap())
            } else if err.is_instance_of::<ConflictsInTree>(py) {
                Error::ConflictsInTree
            } else if err.is_instance_of::<SourceNotDerivedFromTarget>(py) {
                Error::SourceNotDerivedFromTarget
            } else if BreezyConnectionError
                .as_ref()
                .and_then(|cls| {
                    Python::with_gil(|py| Some(err.is_instance_bound(py, cls.bind(py))))
                })
                .unwrap_or(false)
            {
                Error::ConnectionError(err.to_string())
            } else if err.is_instance_of::<RedirectRequested>(py) {
                Error::RedirectRequested {
                    source: value
                        .getattr("source")
                        .unwrap()
                        .extract::<String>()
                        .unwrap()
                        .parse()
                        .unwrap(),
                    target: value
                        .getattr("target")
                        .unwrap()
                        .extract::<String>()
                        .unwrap()
                        .parse()
                        .unwrap(),
                    is_permanent: value.getattr("is_permanent").unwrap().extract().unwrap(),
                }
            } else if err.is_instance_of::<NoRoundtrippingSupport>(py) {
                Error::NoRoundtrippingSupport
            } else if err.is_instance_of::<NoCompatibleInter>(py) {
                Error::NoCompatibleInter
            // Intentionally sorted below the more specific errors
            } else if err.is_instance_of::<InvalidHttpResponse>(py) {
                Error::InvalidHttpResponse(
                    value.getattr("path").unwrap().extract().unwrap(),
                    value.getattr("msg").unwrap().extract().unwrap(),
                    value.getattr("orig_error").unwrap().extract().unwrap(),
                    value.getattr("headers").unwrap().extract().unwrap(),
                )
            } else if err.is_instance_of::<TransportError>(py) {
                Error::TransportError(value.getattr("msg").unwrap().extract().unwrap())
            } else if err.is_instance_of::<BranchReferenceLoop>(py) {
                Error::BranchReferenceLoop
            } else {
                if std::env::var("BRZ_ERROR").is_ok() {
                    // Print backtrace
                    err.print(py);
                }
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
            Error::NoColocatedBranchSupport => {
                Python::with_gil(|py| NoColocatedBranchSupport::new_err((py.None(),)))
            }
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
            Error::IncompleteRead(partial, expected) => Python::with_gil(|py| {
                let bytes = pyo3::types::PyBytes::new_bound(py, partial.as_slice());
                IncompleteRead::new_err((bytes.unbind(), expected))
            }),
            Error::LineEndingError(e) => LineEndingError::new_err((e,)),
            Error::InvalidHttpResponse(status, msg, orig_error, headers) => {
                InvalidHttpResponse::new_err((status, msg, orig_error, headers))
            }
            Error::AlreadyControlDir(path) => {
                AlreadyControlDirError::new_err((path.to_string_lossy().to_string(),))
            }
            Error::AlreadyBranch(path) => {
                AlreadyBranchError::new_err((path.to_string_lossy().to_string(),))
            }
            Error::DivergedBranches => {
                Python::with_gil(|py| DivergedBranches::new_err((py.None(), py.None())))
            }
            Error::WorkspaceDirty(p) => WorkspaceDirty::new_err((p.to_string_lossy().to_string(),)),
            Error::NoSuchFile(p) => NoSuchFile::new_err(p.to_string_lossy().to_string()),
            Error::PointlessCommit => PointlessCommit::new_err(()),
            Error::NoWhoami => NoWhoami::new_err(()),
            Error::NoSuchTag(tag) => NoSuchTag::new_err((tag,)),
            Error::TagAlreadyExists(tag) => TagAlreadyExists::new_err((tag,)),
            Error::Socket(e) => {
                pyo3::import_exception!(socket, error);
                error::new_err((e.raw_os_error().unwrap(),))
            }
            Error::ForgeLoginRequired => {
                Python::with_gil(|py| ForgeLoginRequired::new_err((py.None(),)))
            }
            Error::UnsupportedForge(url) => UnsupportedForge::new_err((url.to_string(),)),
            Error::ForgeProjectExists(name) => AlreadyControlDirError::new_err((name.to_string(),)),
            Error::MergeProposalExists(source, _target) => {
                Python::with_gil(|py| MergeProposalExists::new_err((source.to_string(), py.None())))
            }
            Error::UnsupportedOperation(mname, tname) => {
                UnsupportedOperation::new_err((mname, tname))
            }
            Error::ProtectedBranchHookDeclined(msg) => ProtectedBranchHookDeclined::new_err((msg,)),
            Error::NoRepositoryPresent => {
                Python::with_gil(|py| NoRepositoryPresent::new_err((py.None(),)))
            }
            Error::LockFailed(why) => Python::with_gil(|py| LockFailed::new_err((py.None(), why))),
            Error::FileExists(p, extra) => {
                FileExists::new_err((p.to_string_lossy().to_string(), extra))
            }
            Error::LockContention(_lock, msg) => {
                Python::with_gil(|py| LockContention::new_err((py.None(), msg)))
            }
            Error::NotImplemented => pyo3::exceptions::PyNotImplementedError::new_err(()),
            Error::NoSuchRevisionInTree(rev) => {
                Python::with_gil(|py| NoSuchRevisionInTree::new_err((py.None(), rev.to_string())))
            }
            Error::MissingNestedTree(p) => {
                MissingNestedTree::new_err((p.to_string_lossy().to_string(),))
            }
            Error::ImmortalLimbo(p) => ImmortalLimbo::new_err((p.to_string_lossy().to_string(),)),
            Error::MalformedTransform(conflicts) => {
                MalformedTransform::new_err((Python::with_gil(|py| conflicts.to_object(py)),))
            }
            Error::TransformRenameFailed(from_path, to_path, why, error) => {
                TransformRenameFailed::new_err((
                    from_path.to_string_lossy().to_string(),
                    to_path.to_string_lossy().to_string(),
                    why,
                    PyErr::from(error),
                ))
            }
            Error::UnexpectedHttpStatus {
                url,
                code,
                extra,
                headers,
            } => UnexpectedHttpStatus::new_err((url.to_string(), code, extra, headers)),
            Error::Timeout => pyo3::exceptions::PyTimeoutError::new_err(()),
            Error::BadHttpRequest(url, reason) => {
                BadHttpRequest::new_err((url.to_string(), reason))
            }
            Error::TransportNotPossible(e) => TransportNotPossible::new_err((e,)),
            Error::IncompatibleFormat(a, b) => IncompatibleFormat::new_err((a, b)),
            Error::NoSuchRevision(rev) => {
                Python::with_gil(|py| NoSuchRevision::new_err((py.None(), rev.to_string())))
            }
            Error::NoSuchProject(p) => NoSuchProject::new_err((p,)),
            Error::ForkingDisabled(p) => ForkingDisabled::new_err((p,)),
            Error::ProjectCreationTimeout(p, t) => ProjectCreationTimeout::new_err((p, t)),
            Error::GitLabConflict(p) => GitLabConflict::new_err((p,)),
            Error::ConflictsInTree => ConflictsInTree::new_err(()),
            Error::SourceNotDerivedFromTarget => SourceNotDerivedFromTarget::new_err(()),
            Error::BranchReferenceLoop => BranchReferenceLoop::new_err(()),
            Error::RedirectRequested {
                source,
                target,
                is_permanent,
            } => RedirectRequested::new_err((source.to_string(), target.to_string(), is_permanent)),
            Error::NoRoundtrippingSupport => {
                Python::with_gil(|py| NoRoundtrippingSupport::new_err((py.None(), py.None())))
            }
            Error::NoCompatibleInter => {
                Python::with_gil(|py| NoCompatibleInter::new_err((py.None(), py.None())))
            }
        }
    }
}

#[test]
fn test_error_unknownformat() {
    let e = Error::UnknownFormat("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of UnknownFormatError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<UnknownFormatError>(py));
    });
}

#[test]
fn test_error_notbrancherror() {
    let e = Error::NotBranchError("foo".to_string(), Some("bar".to_string()));
    let p: PyErr = e.into();
    // Verify that p is an instance of NotBranchError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<NotBranchError>(py));
    });
}

#[test]
fn test_error_nocolocatedbranchsupport() {
    let e = Error::NoColocatedBranchSupport;
    let p: PyErr = e.into();
    // Verify that p is an instance of NoColocatedBranchSupport
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<NoColocatedBranchSupport>(py), "{}", p);
    });
}

#[test]
fn test_error_dependencynotpresent() {
    let e = Error::DependencyNotPresent("foo".to_string(), "bar".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of DependencyNotPresent
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<DependencyNotPresent>(py));
    });
}

#[test]
fn test_error_permissiondenied() {
    let e = Error::PermissionDenied(std::path::PathBuf::from("foo"), Some("bar".to_string()));
    let p: PyErr = e.into();
    // Verify that p is an instance of PermissionDenied
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<PermissionDenied>(py));
    });
}

#[test]
fn test_error_unsupportedprotocol() {
    let e = Error::UnsupportedProtocol("foo".to_string(), Some("bar".to_string()));
    let p: PyErr = e.into();
    // Verify that p is an instance of UnsupportedProtocol
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<UnsupportedProtocol>(py));
    });
}

#[test]
fn test_error_unusableredirect() {
    let e = Error::UnusableRedirect("foo".to_string(), "bar".to_string(), "baz".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of UnusableRedirect
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<UnusableRedirect>(py));
    });
}

#[test]
fn test_error_connectionerror() {
    let e = Error::ConnectionError("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of PyConnectionError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<pyo3::exceptions::PyConnectionError>(py));
    });
}

#[test]
fn test_error_invalidurl() {
    let e = Error::InvalidURL("foo".to_string(), Some("bar".to_string()));
    let p: PyErr = e.into();
    // Verify that p is an instance of InvalidURL
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<InvalidURL>(py));
    });
}

#[test]
fn test_error_transporterror() {
    let e = Error::TransportError("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of TransportError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<TransportError>(py));
    });
}

#[test]
fn test_error_unsupportedformat() {
    let e = Error::UnsupportedFormat("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of UnsupportedFormatError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<UnsupportedFormatError>(py));
    });
}

#[test]
fn test_error_unsupportedvcs() {
    let e = Error::UnsupportedVcs("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of UnsupportedVcs
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<UnsupportedVcs>(py));
    });
}

#[test]
fn test_error_remotegiterror() {
    let e = Error::RemoteGitError("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of RemoteGitError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<RemoteGitError>(py));
    });
}

#[test]
fn test_error_incompleteread() {
    let e = Error::IncompleteRead(vec![1, 2, 3], Some(4));
    let p: PyErr = e.into();
    // Verify that p is an instance of IncompleteRead
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<IncompleteRead>(py), "{}", p);
    });
}

#[test]
fn test_error_lineendingerror() {
    let e = Error::LineEndingError("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of LineEndingError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<LineEndingError>(py));
    });
}

#[test]
fn test_error_invalidhttpresponse() {
    let e = Error::InvalidHttpResponse(
        "foo".to_string(),
        "bar".to_string(),
        Some("baz".to_string()),
        std::collections::HashMap::new(),
    );
    let p: PyErr = e.into();
    // Verify that p is an instance of InvalidHttpResponse
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<InvalidHttpResponse>(py));
    });
}

#[test]
fn test_error_alreadyexists() {
    let e = Error::AlreadyControlDir(std::path::PathBuf::from("foo"));
    let p: PyErr = e.into();
    // Verify that p is an instance of AlreadyControlDirError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<AlreadyControlDirError>(py), "{}", p);
    });
}

#[test]
fn test_error_divergedbranches() {
    let e = Error::DivergedBranches;
    let p: PyErr = e.into();
    // Verify that p is an instance of DivergedBranches
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<DivergedBranches>(py), "{}", p);
    });
}

#[test]
#[ignore] // WorkspaceDirty takes a tree argument, which is not implemented
fn test_error_workspacedirty() {
    let e = Error::WorkspaceDirty(std::path::PathBuf::from("foo"));
    let p: PyErr = e.into();
    // Verify that p is an instance of WorkspaceDirty
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<WorkspaceDirty>(py), "{}", p);
    });
}

#[test]
fn test_error_nosuchfile() {
    let e = Error::NoSuchFile(std::path::PathBuf::from("foo"));
    let p: PyErr = e.into();
    // Verify that p is an instance of NoSuchFile
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<NoSuchFile>(py));
    });
}

#[test]
fn test_error_pointlesscommit() {
    let e = Error::PointlessCommit;
    let p: PyErr = e.into();
    // Verify that p is an instance of PointlessCommit
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<PointlessCommit>(py));
    });
}

#[test]
fn test_error_nowhoami() {
    let e = Error::NoWhoami;
    let p: PyErr = e.into();
    // Verify that p is an instance of NoWhoami
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<NoWhoami>(py), "{}", p);
    });
}

#[test]
fn test_error_nosuchtag() {
    let e = Error::NoSuchTag("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of NoSuchTag
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<NoSuchTag>(py));
    });
}

#[test]
fn test_error_tagalreadyexists() {
    let e = Error::TagAlreadyExists("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of TagAlreadyExists
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<TagAlreadyExists>(py));
    });
}

#[test]
fn test_error_socket() {
    let e = Error::Socket(std::io::Error::from_raw_os_error(0));
    let p: PyErr = e.into();
    // Verify that p is an instance of error
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<pyo3::exceptions::PyOSError>(py));
    });
}

#[test]
fn test_error_other() {
    let e = Error::Other(PyErr::new::<UnknownFormatError, _>((("foo",),)));
    let p: PyErr = e.into();
    // Verify that p is an instance of error
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<pyo3::exceptions::PyException>(py));
    });
}

#[test]
fn test_error_forge_login_required() {
    let e = Error::ForgeLoginRequired;
    let p: PyErr = e.into();
    // Verify that p is an instance of ForgeLoginRequired
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<ForgeLoginRequired>(py));
    });
}

#[test]
fn test_error_unsupported_forge() {
    let e = Error::UnsupportedForge("http://example.com".parse().unwrap());
    let p: PyErr = e.into();
    // Verify that p is an instance of UnsupportedForge
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<UnsupportedForge>(py));
    });
}

#[test]
fn test_error_forge_project_exists() {
    let e = Error::ForgeProjectExists("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of AlreadyControlDirError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<AlreadyControlDirError>(py), "{}", p);
    });
}

#[test]
fn test_error_merge_proposal_exists() {
    let e = Error::MergeProposalExists(
        "http://source.com".parse().unwrap(),
        Some("http://target.com".parse().unwrap()),
    );
    let p: PyErr = e.into();
    // Verify that p is an instance of MergeProposalExists
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<MergeProposalExists>(py), "{}", p);
    });
}

#[test]
#[ignore] // UnsupportedOperation takes two arguments, which is not implemented
fn test_error_unsupported_operation() {
    let e = Error::UnsupportedOperation("foo".to_string(), "bar".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of UnsupportedOperation
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<UnsupportedOperation>(py), "{}", p);
    });
}

#[test]
fn test_error_protected_branch_hook_declined() {
    let e = Error::ProtectedBranchHookDeclined("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of ProtectedBranchHookDeclined
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<ProtectedBranchHookDeclined>(py), "{}", p);
    });
}

#[test]
#[ignore] // NoRepositoryPresent takes an argument, which is not implemented
fn test_error_no_repository_present() {
    let e = Error::NoRepositoryPresent;
    let p: PyErr = e.into();
    // Verify that p is an instance of NoRepositoryPresent
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<NoRepositoryPresent>(py), "{}", p);
    });
}

#[test]
#[ignore] // LockFailed takes a lockfile argument, which is not implemented
fn test_error_lock_failed() {
    let e = Error::LockFailed("bar".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of LockFailed
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<LockFailed>(py), "{}", p);
    });
}

#[test]
fn test_error_file_exists() {
    let e = Error::FileExists(std::path::PathBuf::from("foo"), Some("bar".to_string()));
    let p: PyErr = e.into();
    // Verify that p is an instance of FileExists
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<FileExists>(py), "{}", p);
    });
}

#[test]
fn test_error_lock_contention() {
    let e = Error::LockContention("foo".to_string(), "bar".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of LockContention
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<LockContention>(py), "{}", p);
    });
}

#[test]
fn test_error_notimplementederror() {
    let e = Error::NotImplemented;
    let p: PyErr = e.into();
    // Verify that p is an instance of PyNotImplementedError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<pyo3::exceptions::PyNotImplementedError>(py));
    });
}

#[test]
fn test_missing_nested_tree() {
    let e = Error::MissingNestedTree(std::path::PathBuf::from("foo"));
    let p: PyErr = e.into();
    // Verify that p is an instance of MissingNestedTree
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<MissingNestedTree>(py), "{}", p);
    });
}

#[test]
fn test_immortal_limbo() {
    let e = Error::ImmortalLimbo(std::path::PathBuf::from("foo"));
    let p: PyErr = e.into();
    // Verify that p is an instance of ImmortalLimbo
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<ImmortalLimbo>(py), "{}", p);
    });
}

#[test]
fn test_malformed_transform() {
    let e = Error::MalformedTransform(vec![]);

    let p: PyErr = e.into();
    // Verify that p is an instance of MalformedTransform
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<MalformedTransform>(py), "{}", p);
    });
}

#[test]
fn test_transform_rename_failed() {
    let e = Error::TransformRenameFailed(
        std::path::PathBuf::from("foo"),
        std::path::PathBuf::from("bar"),
        "baz".to_string(),
        std::io::Error::new(std::io::ErrorKind::NotFound, "foo"),
    );
    let p: PyErr = e.into();
    // Verify that p is an instance of TransformRenameFailed
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<TransformRenameFailed>(py), "{}", p,);
    });
}

#[test]
fn test_unexpected_http_status() {
    let e = Error::UnexpectedHttpStatus {
        url: url::Url::parse("http://example.com").unwrap(),
        code: 404,
        extra: Some("bar".to_string()),
        headers: std::collections::HashMap::new(),
    };
    let p: PyErr = e.into();
    // Verify that p is an instance of UnexpectedHttpStatus
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<UnexpectedHttpStatus>(py), "{}", p);
    });
}

#[test]
fn test_timeout() {
    let e = Error::Timeout;
    let p: PyErr = e.into();
    // Verify that p is an instance of PyTimeoutError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<pyo3::exceptions::PyTimeoutError>(py));
    });
}

#[test]
fn test_bad_http_request() {
    let e = Error::BadHttpRequest("http://example.com".parse().unwrap(), "foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of BadHttpRequest
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<BadHttpRequest>(py), "{}", p);
    });
}

#[test]
fn test_transport_not_possible() {
    let e = Error::TransportNotPossible("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of TransportNotPossible
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<TransportNotPossible>(py), "{}", p);
    });
}

#[test]
fn test_incompatible_format() {
    let e = Error::IncompatibleFormat("foo".to_string(), "bar".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of IncompatibleFormat
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<IncompatibleFormat>(py), "{}", p);
    });
}

#[test]
fn test_no_such_project() {
    let e = Error::NoSuchProject("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of NoSuchProject
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<NoSuchProject>(py), "{}", p);
    });
}

#[test]
fn test_forking_disabled() {
    let e = Error::ForkingDisabled("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of ForkingDisabled
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<ForkingDisabled>(py), "{}", p);
    });
}

#[test]
fn test_gitlab_conflict() {
    let e = Error::GitLabConflict("foo".to_string());
    let p: PyErr = e.into();
    // Verify that p is an instance of GitLabConflict
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<GitLabConflict>(py), "{}", p);
    });
}

#[test]
fn test_conflicts_in_tree() {
    let e = Error::ConflictsInTree;
    let p: PyErr = e.into();
    // Verify that p is an instance of ConflictsInTree
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<ConflictsInTree>(py), "{}", p);
    });
}

#[test]
fn test_project_creation_timeout() {
    let e = Error::ProjectCreationTimeout("foo".to_string(), chrono::Duration::seconds(0));
    let p: PyErr = e.into();
    // Verify that p is an instance of ProjectCreationTimeout
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<ProjectCreationTimeout>(py), "{}", p);
    });
}

#[test]
fn test_already_branch() {
    let e = Error::AlreadyBranch(std::path::PathBuf::from("foo"));
    let p: PyErr = e.into();
    // Verify that p is an instance of AlreadyBranchError
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<AlreadyBranchError>(py), "{}", p);
    });
}

#[test]
fn test_redirect_requested() {
    let e = Error::RedirectRequested {
        source: "http://example.com".parse().unwrap(),
        target: "http://example.com".parse().unwrap(),
        is_permanent: true,
    };
    let p: PyErr = e.into();
    // Verify that p is an instance of RedirectRequested
    Python::with_gil(|py| {
        assert!(p.is_instance_of::<RedirectRequested>(py), "{}", p);
    });
}
