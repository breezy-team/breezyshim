//! Code hosting services and merge proposals.
use crate::branch::{py_tag_selector, GenericBranch, PyBranch};
use crate::error::Error;
use crate::revisionid::RevisionId;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::hash::Hash;

/// Represents a code forge (hosting service) like GitHub, GitLab, etc.
pub struct Forge(Py<PyAny>);

impl Clone for Forge {
    fn clone(&self) -> Self {
        Forge(Python::attach(|py| self.0.clone_ref(py)))
    }
}

impl From<Py<PyAny>> for Forge {
    fn from(obj: Py<PyAny>) -> Self {
        Forge(obj)
    }
}

impl std::fmt::Debug for MergeProposal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Just print the URL for now
        let mut s = f.debug_struct("MergeProposal");
        if let Ok(url) = self.url() {
            s.field("url", &url);
        }
        s.finish()
    }
}

impl std::fmt::Display for MergeProposal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let url = self.url().unwrap();
        write!(f, "{}", url)
    }
}

/// Status of a merge proposal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MergeProposalStatus {
    /// All merge proposals regardless of status.
    All,
    /// Open merge proposals that haven't been merged or closed.
    Open,
    /// Closed merge proposals that weren't merged.
    Closed,
    /// Merged merge proposals that have been accepted and integrated.
    Merged,
}

/// A comment on a merge proposal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProposalComment {
    /// Login name of the author, if known.
    pub author: Option<String>,
    /// Text of the comment.
    pub body: String,
    /// When the comment was made.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Web URL of the comment, if any.
    pub url: Option<String>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ProposalComment {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(ProposalComment {
            author: ob.getattr("author")?.extract()?,
            body: ob.getattr("body")?.extract()?,
            created_at: extract_datetime(&ob.getattr("created_at")?)?,
            url: ob.getattr("url")?.extract()?,
        })
    }
}

/// The verdict of a review of a merge proposal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReviewVerdict {
    /// The reviewer approved the proposal.
    Approved,
    /// The reviewer requested changes.
    ChangesRequested,
    /// The reviewer only commented.
    Commented,
    /// The review was dismissed.
    Dismissed,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ReviewVerdict {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let value: String = ob.getattr("value")?.extract()?;
        match value.as_str() {
            "approved" => Ok(ReviewVerdict::Approved),
            "changes-requested" => Ok(ReviewVerdict::ChangesRequested),
            "commented" => Ok(ReviewVerdict::Commented),
            "dismissed" => Ok(ReviewVerdict::Dismissed),
            _ => Err(PyValueError::new_err(format!(
                "unknown review verdict: {value}"
            ))),
        }
    }
}

/// A review of a merge proposal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProposalReview {
    /// Login name of the reviewer, if known.
    pub author: Option<String>,
    /// The verdict of the review.
    pub verdict: ReviewVerdict,
    /// Summary text of the review, if any.
    pub body: Option<String>,
    /// When the review was submitted.
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    /// The head revision the review applied to, when known.
    pub revision: Option<RevisionId>,
    /// Web URL of the review, if any.
    pub url: Option<String>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ProposalReview {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(ProposalReview {
            author: ob.getattr("author")?.extract()?,
            verdict: ob.getattr("verdict")?.extract()?,
            body: ob.getattr("body")?.extract()?,
            submitted_at: extract_datetime(&ob.getattr("submitted_at")?)?,
            revision: ob.getattr("revision")?.extract()?,
            url: ob.getattr("url")?.extract()?,
        })
    }
}

/// The side of a diff a line number refers to.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DiffSide {
    /// The old version of the file.
    Old,
    /// The new version of the file.
    New,
}

impl DiffSide {
    fn as_str(self) -> &'static str {
        match self {
            DiffSide::Old => "old",
            DiffSide::New => "new",
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for DiffSide {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let value: String = ob.getattr("value")?.extract()?;
        match value.as_str() {
            "old" => Ok(DiffSide::Old),
            "new" => Ok(DiffSide::New),
            _ => Err(PyValueError::new_err(format!("unknown diff side: {value}"))),
        }
    }
}

/// A discussion on a line of the diff of a merge proposal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReviewThread {
    /// Opaque identifier, for [`MergeProposal::set_review_thread_resolved`].
    pub id: String,
    /// Path of the file the thread is on.
    pub path: String,
    /// The side of the diff `line` refers to.
    pub side: DiffSide,
    /// Line number in the diff against `revision`, if known.
    pub line: Option<u32>,
    /// The head revision `line` refers to.
    pub revision: Option<RevisionId>,
    /// Whether the thread refers to an older version of the diff.
    pub outdated: bool,
    /// Whether the thread has been resolved.
    pub resolved: bool,
    /// Whether the current user may (un)resolve the thread.
    pub resolvable: bool,
    /// Web URL of the thread, if any.
    pub url: Option<String>,
    /// The comments in the thread, oldest first.
    pub comments: Vec<ProposalComment>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ReviewThread {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(ReviewThread {
            id: ob.getattr("id")?.extract()?,
            path: ob.getattr("path")?.extract()?,
            side: ob.getattr("side")?.extract()?,
            line: ob.getattr("line")?.extract()?,
            revision: ob.getattr("revision")?.extract()?,
            outdated: ob.getattr("outdated")?.extract()?,
            resolved: ob.getattr("resolved")?.extract()?,
            resolvable: ob.getattr("resolvable")?.extract()?,
            url: ob.getattr("url")?.extract()?,
            comments: ob
                .getattr("comments")?
                .try_iter()?
                .map(|c| c?.extract())
                .collect::<PyResult<_>>()?,
        })
    }
}

/// Extract a Python datetime, treating naive ones as UTC.
fn extract_datetime(ob: &Bound<'_, PyAny>) -> PyResult<chrono::DateTime<chrono::Utc>> {
    if let Ok(dt) = ob.extract::<chrono::DateTime<chrono::FixedOffset>>() {
        return Ok(dt.to_utc());
    }
    Ok(ob.extract::<chrono::NaiveDateTime>()?.and_utc())
}

impl MergeProposalStatus {
    /// Get all possible merge proposal statuses.
    pub fn all() -> Vec<Self> {
        vec![MergeProposalStatus::All]
    }
}

impl std::str::FromStr for MergeProposalStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(MergeProposalStatus::All),
            "open" => Ok(MergeProposalStatus::Open),
            "merged" => Ok(MergeProposalStatus::Merged),
            "closed" => Ok(MergeProposalStatus::Closed),
            _ => Err(format!("Invalid merge proposal status: {}", s)),
        }
    }
}

impl std::fmt::Display for MergeProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeProposalStatus::All => write!(f, "all"),
            MergeProposalStatus::Open => write!(f, "open"),
            MergeProposalStatus::Merged => write!(f, "merged"),
            MergeProposalStatus::Closed => write!(f, "closed"),
        }
    }
}

impl<'py> IntoPyObject<'py> for MergeProposalStatus {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.to_string().into_pyobject(py).unwrap().into_any())
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for MergeProposalStatus {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let status = ob.extract::<String>()?;
        match status.as_str() {
            "all" => Ok(MergeProposalStatus::All),
            "open" => Ok(MergeProposalStatus::Open),
            "merged" => Ok(MergeProposalStatus::Merged),
            "closed" => Ok(MergeProposalStatus::Closed),
            _ => Err(PyValueError::new_err((format!(
                "Invalid merge proposal status: {}",
                status
            ),))),
        }
    }
}

/// Whether a merge proposal will be merged automatically.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AutoMergeStatus {
    /// Auto-merge can not be enabled.
    Unavailable,
    /// Auto-merge can be enabled.
    Available,
    /// Auto-merge is enabled.
    Enabled,
}

impl<'a, 'py> FromPyObject<'a, 'py> for AutoMergeStatus {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let value = ob.getattr("value")?.extract::<String>()?;
        match value.as_str() {
            "unavailable" => Ok(AutoMergeStatus::Unavailable),
            "available" => Ok(AutoMergeStatus::Available),
            "enabled" => Ok(AutoMergeStatus::Enabled),
            _ => Err(PyValueError::new_err(format!(
                "Invalid auto-merge status: {}",
                value
            ))),
        }
    }
}

/// Whether a merge proposal can be merged by the current user.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MergeStatus {
    /// The head revision this status applies to.
    pub head_revision: RevisionId,
    /// Whether the current user can merge the proposal now.
    pub mergeable: bool,
    /// Whether merging requires overriding branch protection; `mergeable`
    /// is true in that case.
    pub as_admin_only: bool,
    /// Why the proposal can not be merged, when it can not.
    pub reason: Option<String>,
    /// Whether auto-merge is enabled, or can be enabled.
    pub auto_merge: AutoMergeStatus,
}

impl<'a, 'py> FromPyObject<'a, 'py> for MergeStatus {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(MergeStatus {
            head_revision: ob.getattr("head_revision")?.extract()?,
            mergeable: ob.getattr("mergeable")?.extract()?,
            as_admin_only: ob.getattr("as_admin_only")?.extract()?,
            reason: ob.getattr("reason")?.extract()?,
            auto_merge: ob.getattr("auto_merge")?.extract()?,
        })
    }
}

/// Kind of event in the history of a merge proposal.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProposalEventKind {
    /// New revisions were pushed.
    Push,
    /// A comment was posted.
    Comment,
    /// A review without a verdict was submitted.
    Review,
    /// The proposal was approved.
    Approval,
    /// An approval was withdrawn or changes were requested.
    Unapproval,
    /// A kind that breezy knows about but this wrapper does not yet.
    Unknown(String),
}

impl<'a, 'py> FromPyObject<'a, 'py> for ProposalEventKind {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let value = ob.getattr("value")?.extract::<String>()?;
        Ok(match value.as_str() {
            "push" => ProposalEventKind::Push,
            "comment" => ProposalEventKind::Comment,
            "review" => ProposalEventKind::Review,
            "approval" => ProposalEventKind::Approval,
            "unapproval" => ProposalEventKind::Unapproval,
            _ => ProposalEventKind::Unknown(value),
        })
    }
}

/// An event in the history of a merge proposal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProposalEvent {
    /// What happened.
    pub kind: ProposalEventKind,
    /// Login name of who did it, if known.
    pub actor: Option<String>,
    /// When it happened.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Head revision at the time, if known.
    pub revision: Option<RevisionId>,
    /// Whether the actor is a bot account.
    pub actor_is_bot: bool,
}

impl<'a, 'py> FromPyObject<'a, 'py> for ProposalEvent {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(ProposalEvent {
            kind: ob.getattr("kind")?.extract()?,
            actor: ob.getattr("actor")?.extract()?,
            timestamp: extract_datetime(&ob.getattr("timestamp")?)?,
            revision: ob.getattr("revision")?.extract()?,
            actor_is_bot: ob.getattr("actor_is_bot")?.extract()?,
        })
    }
}

/// Combined state of the CI for a revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CIState {
    /// CI is still running or waiting to run.
    Pending,
    /// CI passed.
    Success,
    /// CI failed.
    Failure,
}

impl<'a, 'py> FromPyObject<'a, 'py> for CIState {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let value = ob.getattr("value")?.extract::<String>()?;
        match value.as_str() {
            "pending" => Ok(CIState::Pending),
            "success" => Ok(CIState::Success),
            "failure" => Ok(CIState::Failure),
            _ => Err(PyValueError::new_err(format!("Invalid CI state: {value}"))),
        }
    }
}

/// A CI job or check that failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CIFailure {
    /// Name of the job or check.
    pub name: String,
    /// Web URL with details, if any.
    pub url: Option<String>,
    /// Log of the job as the forge provides it, which may include terminal
    /// escape sequences; for checks without a log, their description.
    pub log: String,
}

impl<'a, 'py> FromPyObject<'a, 'py> for CIFailure {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(CIFailure {
            name: ob.getattr("name")?.extract()?,
            url: ob.getattr("url")?.extract()?,
            log: ob.getattr("log")?.extract()?,
        })
    }
}

/// A merge proposal (pull request) on a code hosting service.
pub struct MergeProposal(Py<PyAny>);

impl Clone for MergeProposal {
    fn clone(&self) -> Self {
        MergeProposal(Python::attach(|py| self.0.clone_ref(py)))
    }
}

impl From<Py<PyAny>> for MergeProposal {
    fn from(obj: Py<PyAny>) -> Self {
        MergeProposal(obj)
    }
}

impl MergeProposal {
    /// Create a merge proposal reference from a URL.
    pub fn from_url(url: &url::Url) -> Result<Self, Error> {
        get_proposal_by_url(url)
    }

    /// Reopens a previously closed merge proposal.
    pub fn reopen(&self) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.0.call_method0(py, "reopen")?;
            Ok(())
        })
    }

    /// Closes an open merge proposal without merging it.
    pub fn close(&self) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.0.call_method0(py, "close")?;
            Ok(())
        })
    }

    /// Returns the URL of the merge proposal.
    pub fn url(&self) -> Result<url::Url, crate::error::Error> {
        Python::attach(|py| {
            let url = self.0.getattr(py, "url")?;
            Ok(url.extract::<String>(py)?.parse()?)
        })
    }

    /// Checks if the merge proposal has been merged.
    pub fn is_merged(&self) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            let is_merged = self.0.call_method0(py, "is_merged")?;
            is_merged.extract(py).map_err(Into::into)
        })
    }

    /// Checks if the merge proposal has been closed without being merged.
    pub fn is_closed(&self) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            let is_closed = self.0.call_method0(py, "is_closed")?;
            is_closed.extract(py).map_err(Into::into)
        })
    }

    /// Retrieves the title of the merge proposal.
    pub fn get_title(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| {
            let title = self.0.call_method0(py, "get_title")?;
            title.extract(py).map_err(Into::into)
        })
    }

    /// Sets the title of the merge proposal.
    pub fn set_title(&self, title: Option<&str>) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.0.call_method1(py, "set_title", (title,))?;
            Ok(())
        })
    }

    /// Retrieves the commit message associated with the merge proposal.
    pub fn get_commit_message(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| {
            let commit_message = self.0.call_method0(py, "get_commit_message")?;
            commit_message.extract(py).map_err(Into::into)
        })
    }

    /// Sets the commit message for the merge proposal.
    pub fn set_commit_message(
        &self,
        commit_message: Option<&str>,
    ) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.0
                .call_method1(py, "set_commit_message", (commit_message,))?;
            Ok(())
        })
    }

    /// Returns the URL of the target branch for this merge proposal,
    /// or `None` if the forge does not expose one (e.g. the target repo
    /// was deleted or made private).
    pub fn get_target_branch_url(&self) -> Result<Option<url::Url>, crate::error::Error> {
        Python::attach(|py| {
            let target_branch_url = self.0.call_method0(py, "get_target_branch_url")?;
            if target_branch_url.is_none(py) {
                return Ok(None);
            }
            target_branch_url
                .extract::<String>(py)?
                .parse::<url::Url>()
                .map(Some)
                .map_err(Into::into)
        })
    }

    /// Returns the URL of the source branch for this merge proposal,
    /// or `None` if the forge does not expose one (e.g. the source fork
    /// was deleted).
    pub fn get_source_branch_url(&self) -> Result<Option<url::Url>, crate::error::Error> {
        Python::attach(|py| {
            let source_branch_url = self.0.call_method0(py, "get_source_branch_url")?;
            if source_branch_url.is_none(py) {
                return Ok(None);
            }
            source_branch_url
                .extract::<String>(py)?
                .parse::<url::Url>()
                .map(Some)
                .map_err(Into::into)
        })
    }

    /// Returns the revision id of the tip of the source branch for this
    /// merge proposal, or `None` if the forge cannot resolve it.
    pub fn get_source_revision(&self) -> Result<Option<RevisionId>, crate::error::Error> {
        Python::attach(|py| {
            let revision = self.0.call_method0(py, "get_source_revision")?;
            if revision.is_none(py) {
                return Ok(None);
            }
            Ok(Some(revision.extract::<RevisionId>(py)?))
        })
    }

    /// Post a comment on the merge proposal. Forwards to the forge's
    /// `post_comment(body)` method (e.g. GitHub adds an issue
    /// comment, GitLab adds a discussion note). Forges that don't
    /// support comment posting raise `NotImplementedError` upstream;
    /// this wrapper propagates that as the usual `Error::Other`.
    pub fn post_comment(&self, body: &str) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.0.call_method1(py, "post_comment", (body,))?;
            Ok(())
        })
    }

    /// Set the merge proposal's target branch name. Used by the
    /// publisher when the role-to-branch mapping for a campaign
    /// changes and the existing proposal needs to be redirected
    /// rather than closed and re-opened.
    pub fn set_target_branch_name(&self, name: &str) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.0.call_method1(py, "set_target_branch_name", (name,))?;
            Ok(())
        })
    }

    /// Retrieves the description of the merge proposal.
    pub fn get_description(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| {
            let description = self.0.call_method0(py, "get_description")?;
            description.extract(py).map_err(Into::into)
        })
    }

    /// Sets the description of the merge proposal.
    pub fn set_description(&self, description: Option<&str>) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.0.call_method1(py, "set_description", (description,))?;
            Ok(())
        })
    }

    /// Checks if the merge proposal can currently be merged.
    pub fn can_be_merged(&self) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            let can_be_merged = self.0.call_method0(py, "can_be_merged")?;
            can_be_merged.extract(py).map_err(Into::into)
        })
    }

    /// Checks if the merge proposal supports automatic merging.
    pub fn supports_auto_merge(&self) -> bool {
        Python::attach(|py| {
            self.0
                .getattr(py, "supports_auto_merge")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Merges the merge proposal, optionally using automatic merge.
    ///
    /// The `auto` parameter determines whether to use automatic merging.
    pub fn merge(&self, auto: bool) -> Result<(), Error> {
        self.merge_with(None, auto, None)
    }

    /// Merges the merge proposal, or enables auto-merge if `auto` is set.
    ///
    /// With `expected_revision` set, the forge refuses if the head of the
    /// proposal is a different revision, which is reported as
    /// `Error::HeadChanged`. Versions of Breezy that predate
    /// `expected_revision` reject it with a `TypeError`.
    pub fn merge_with(
        &self,
        commit_message: Option<&str>,
        auto: bool,
        expected_revision: Option<&RevisionId>,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("commit_message", commit_message)?;
            kwargs.set_item("auto", auto)?;
            if let Some(revision) = expected_revision {
                kwargs.set_item("expected_revision", revision.clone())?;
            }
            self.0.call_method(py, "merge", (), Some(&kwargs))?;
            Ok(())
        })
    }

    /// Returns whether the current user can merge this proposal.
    ///
    /// Returns `Error::NotImplemented` for forges that do not support this
    /// and with versions of Breezy that predate
    /// `MergeProposal.get_merge_status`.
    pub fn get_merge_status(&self) -> Result<MergeStatus, Error> {
        Python::attach(|py| {
            let mp = self.0.bind(py);
            if !mp.hasattr("get_merge_status")? {
                return Err(Error::NotImplemented);
            }
            mp.call_method0("get_merge_status")?
                .extract()
                .map_err(Into::into)
        })
    }

    /// Has the forge rebase the source branch onto the target branch.
    ///
    /// Returns `false` if the source branch was already up to date, and
    /// `true` if a rebase was done or started; GitLab rebases in the
    /// background. With `expected_revision` set, a different head is
    /// reported as `Error::HeadChanged`.
    ///
    /// Returns `Error::NotImplemented` for forges that do not support this
    /// and with versions of Breezy that predate `MergeProposal.rebase`.
    pub fn rebase(&self, expected_revision: Option<&RevisionId>) -> Result<bool, Error> {
        Python::attach(|py| {
            let mp = self.0.bind(py);
            if !mp.hasattr("rebase")? {
                return Err(Error::NotImplemented);
            }
            mp.call_method1("rebase", (expected_revision.cloned(),))?
                .extract()
                .map_err(Into::into)
        })
    }

    /// Returns the web URL for viewing the merge proposal in a browser.
    pub fn get_web_url(&self) -> Result<url::Url, crate::error::Error> {
        Python::attach(|py| {
            let web_url = self.0.call_method0(py, "get_web_url")?;
            web_url
                .extract::<String>(py)?
                .parse::<url::Url>()
                .map_err(Into::into)
        })
    }

    /// Returns the username of the person who merged this proposal, if it has been merged.
    pub fn get_merged_by(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| {
            let merged_by = self.0.call_method0(py, "get_merged_by")?;
            merged_by.extract(py).map_err(Into::into)
        })
    }

    /// Returns the username of the person who created this proposal, if known.
    ///
    /// Returns `Error::NotImplemented` with versions of Breezy that predate
    /// `MergeProposal.get_author`.
    pub fn get_author(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| Ok(self.optional_method(py, "get_author")?.call0()?.extract()?))
    }

    /// Returns the events in the history of this proposal, oldest first.
    ///
    /// Returns `Error::NotImplemented` with versions of Breezy that predate
    /// `MergeProposal.iter_events`.
    pub fn iter_events(&self) -> Result<impl Iterator<Item = ProposalEvent>, Error> {
        let events: Vec<ProposalEvent> = Python::attach(|py| -> Result<_, Error> {
            self.optional_method(py, "iter_events")?
                .call0()?
                .try_iter()?
                .map(|event| Ok(event?.extract()?))
                .collect()
        })?;
        Ok(events.into_iter())
    }

    /// Returns the name of the project the proposal's source branch lives in, if known.
    pub fn get_source_project(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| {
            let project = self.0.call_method0(py, "get_source_project")?;
            project.extract(py).map_err(Into::into)
        })
    }

    /// Returns the name of the project the proposal targets, if known.
    pub fn get_target_project(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| {
            let project = self.0.call_method0(py, "get_target_project")?;
            project.extract(py).map_err(Into::into)
        })
    }

    /// Returns the date and time when this proposal was merged, if it has been merged.
    pub fn get_merged_at(
        &self,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, crate::error::Error> {
        Python::attach(|py| {
            let merged_at = self.0.bind(py).call_method0("get_merged_at")?;
            if merged_at.is_none() {
                return Ok(None);
            }
            Ok(Some(extract_datetime(&merged_at)?))
        })
    }

    /// Returns the comments on the proposal as a whole, oldest first.
    ///
    /// Comments on lines of the diff are not included. Eagerly drains the
    /// underlying Python iterator; the first per-item error short-circuits
    /// and is returned.
    pub fn iter_comments(&self) -> Result<impl Iterator<Item = ProposalComment>, Error> {
        let ret: Vec<ProposalComment> = Python::attach(|py| -> Result<_, Error> {
            self.optional_method(py, "iter_comments")?
                .call0()?
                .try_iter()?
                .map(|c| Ok(c?.extract()?))
                .collect()
        })?;
        Ok(ret.into_iter())
    }

    /// Returns the discussions on lines of the diff.
    ///
    /// Eagerly drains the underlying Python iterator; the first per-item
    /// error short-circuits and is returned.
    pub fn iter_review_threads(&self) -> Result<impl Iterator<Item = ReviewThread>, Error> {
        let ret: Vec<ReviewThread> = Python::attach(|py| -> Result<_, Error> {
            self.optional_method(py, "iter_review_threads")?
                .call0()?
                .try_iter()?
                .map(|t| Ok(t?.extract()?))
                .collect()
        })?;
        Ok(ret.into_iter())
    }

    /// Marks a review thread as resolved or unresolved.
    pub fn set_review_thread_resolved(&self, thread_id: &str, resolved: bool) -> Result<(), Error> {
        Python::attach(|py| {
            self.optional_method(py, "set_review_thread_resolved")?
                .call1((thread_id, resolved))?;
            Ok(())
        })
    }

    /// Returns the submitted reviews of the proposal.
    ///
    /// Eagerly drains the underlying Python iterator; the first per-item
    /// error short-circuits and is returned.
    pub fn iter_reviews(&self) -> Result<impl Iterator<Item = ProposalReview>, Error> {
        let ret: Vec<ProposalReview> = Python::attach(|py| -> Result<_, Error> {
            self.optional_method(py, "iter_reviews")?
                .call0()?
                .try_iter()?
                .map(|r| Ok(r?.extract()?))
                .collect()
        })?;
        Ok(ret.into_iter())
    }

    /// Returns the diff of this proposal as the forge shows it, as a unified
    /// diff in git format.
    ///
    /// Returns `Error::NotImplemented` with versions of Breezy that predate
    /// `MergeProposal.get_diff`.
    pub fn get_diff(&self) -> Result<Vec<u8>, Error> {
        Python::attach(|py| {
            self.optional_method(py, "get_diff")?
                .call0()?
                .extract::<Vec<u8>>()
                .map_err(Into::into)
        })
    }

    /// Returns the revisions the diff of this proposal is made from.
    ///
    /// Returns `Error::NotImplemented` with versions of Breezy that predate
    /// `MergeProposal.get_diff_refs`.
    pub fn get_diff_refs(&self) -> Result<DiffRefs, Error> {
        Python::attach(|py| {
            self.optional_method(py, "get_diff_refs")?
                .call0()?
                .extract()
                .map_err(Into::into)
        })
    }

    /// Returns the contents of `path` at `revision`, read through the forge
    /// API rather than from a clone. `revision` may be the head of a proposal
    /// from a fork.
    ///
    /// Returns `Error::NoSuchFile` if the file does not exist at that
    /// revision, and `Error::NotImplemented` if the forge or the version of
    /// Breezy does not support this.
    pub fn get_file_text(&self, path: &str, revision: &RevisionId) -> Result<Vec<u8>, Error> {
        Python::attach(|py| {
            self.optional_method(py, "get_file_text")?
                .call1((path, revision.clone()))?
                .extract()
                .map_err(Into::into)
        })
    }

    /// Returns the combined state of the CI for the head of the proposal, or
    /// `None` if there is no CI for it.
    pub fn get_ci_state(&self) -> Result<Option<CIState>, Error> {
        Python::attach(|py| {
            Ok(self
                .optional_method(py, "get_ci_state")?
                .call0()?
                .extract()?)
        })
    }

    /// Returns a lazy iterator over the CI jobs and checks that failed for
    /// `revision`, or for the head of the proposal if `None`.
    ///
    /// Logs are fetched as the iterator advances.
    pub fn iter_ci_failures(
        &self,
        revision: Option<&RevisionId>,
    ) -> Result<impl Iterator<Item = Result<CIFailure, Error>>, Error> {
        let iter = Python::attach(|py| -> Result<Py<PyAny>, Error> {
            Ok(self
                .optional_method(py, "iter_ci_failures")?
                .call1((revision.cloned(),))?
                .try_iter()?
                .into_any()
                .unbind())
        })?;
        Ok(std::iter::from_fn(move || {
            Python::attach(|py| match iter.call_method0(py, "__next__") {
                Ok(failure) => Some(failure.extract(py).map_err(Into::into)),
                Err(e) if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) => None,
                Err(e) => Some(Err(e.into())),
            })
        }))
    }

    /// Starts CI that waits for approval by a maintainer, as forges require
    /// for some contributions from forks, for `revision` or the head of the
    /// proposal if `None`. Returns the names of what was started.
    pub fn approve_ci(&self, revision: Option<&RevisionId>) -> Result<Vec<String>, Error> {
        Python::attach(|py| {
            Ok(self
                .optional_method(py, "approve_ci")?
                .call1((revision.cloned(),))?
                .extract()?)
        })
    }

    /// Posts a review with inline comments, replies to existing threads and
    /// an outcome, and returns its URL if the forge has one.
    ///
    /// `revision` is the head the comments were written against; if the
    /// proposal's head moved, this fails with `Error::HeadChanged`. Breezy
    /// refuses reviews that say nothing, unless they approve.
    ///
    /// Returns `Error::NotImplemented` with versions of Breezy that predate
    /// `MergeProposal.post_review`.
    pub fn post_review(
        &self,
        revision: &RevisionId,
        body: Option<&str>,
        comments: &[InlineComment],
        replies: &[ThreadReply],
        outcome: ReviewOutcome,
    ) -> Result<Option<url::Url>, Error> {
        Python::attach(|py| {
            let mp = self.0.bind(py);
            if !mp.hasattr("post_review")? {
                return Err(Error::NotImplemented);
            }
            let m = py.import("breezy.forge")?;
            let diff_side = m.getattr("DiffSide")?;
            let inline_comment = m.getattr("InlineComment")?;
            let thread_reply = m.getattr("ThreadReply")?;
            let comments = comments
                .iter()
                .map(|c| {
                    let kwargs = PyDict::new(py);
                    kwargs.set_item("old_path", &c.old_path)?;
                    kwargs.set_item("old_line", c.old_line)?;
                    kwargs.set_item("new_line", c.new_line)?;
                    let side = diff_side.call1((c.side.as_str(),))?;
                    inline_comment.call((&c.path, c.line, side, &c.body), Some(&kwargs))
                })
                .collect::<PyResult<Vec<_>>>()?;
            let replies = replies
                .iter()
                .map(|r| thread_reply.call1((&r.thread_id, &r.body)))
                .collect::<PyResult<Vec<_>>>()?;
            let kwargs = PyDict::new(py);
            kwargs.set_item("body", body)?;
            kwargs.set_item("comments", comments)?;
            kwargs.set_item("replies", replies)?;
            kwargs.set_item(
                "outcome",
                m.getattr("ReviewOutcome")?.call1((outcome.as_str(),))?,
            )?;
            let url = mp.call_method("post_review", (revision.clone(),), Some(&kwargs))?;
            if url.is_none() {
                return Ok(None);
            }
            Ok(Some(url.extract::<String>()?.parse()?))
        })
    }

    /// Look up a method that older versions of Breezy may not have.
    fn optional_method<'py>(
        &self,
        py: Python<'py>,
        name: &str,
    ) -> Result<Bound<'py, PyAny>, crate::error::Error> {
        let mp = self.0.bind(py);
        if !mp.hasattr(name)? {
            return Err(crate::error::Error::NotImplemented);
        }
        Ok(mp.getattr(name)?)
    }

    /// Returns the names of the labels attached to this proposal.
    pub fn get_labels(&self) -> Result<Vec<String>, crate::error::Error> {
        Python::attach(|py| Ok(self.optional_method(py, "get_labels")?.call0()?.extract()?))
    }

    /// Attaches a label to this proposal, creating the label if it does not exist yet.
    pub fn add_label(&self, name: &str) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.optional_method(py, "add_label")?.call1((name,))?;
            Ok(())
        })
    }

    /// Removes a label from this proposal.
    ///
    /// Whether removing a label that is not attached is an error depends on the forge.
    pub fn remove_label(&self, name: &str) -> Result<(), crate::error::Error> {
        Python::attach(|py| {
            self.optional_method(py, "remove_label")?.call1((name,))?;
            Ok(())
        })
    }

    /// Returns whether this proposal is marked as a draft or work in progress.
    ///
    /// Returns `Error::NotImplemented` both for forges whose proposal objects
    /// do not define `is_work_in_progress` and for those that define it but
    /// raise `NotImplementedError` at call time.
    pub fn is_work_in_progress(&self) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            Ok(self
                .optional_method(py, "is_work_in_progress")?
                .call0()?
                .extract()?)
        })
    }

    /// Returns when this proposal was created, or `None` if the forge does
    /// not record it.
    ///
    /// Returns `Error::NotImplemented` both for forges whose proposal objects
    /// do not define `get_created_at` and for those that define it but raise
    /// `NotImplementedError` at call time.
    pub fn get_created_at(
        &self,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, crate::error::Error> {
        Python::attach(|py| {
            let created_at = self.optional_method(py, "get_created_at")?.call0()?;
            if created_at.is_none() {
                return Ok(None);
            }
            Ok(Some(extract_datetime(&created_at)?))
        })
    }

    /// Returns when this proposal was last updated, or `None` if the forge
    /// does not record it.
    ///
    /// Returns `Error::NotImplemented` both for forges whose proposal objects
    /// do not define `get_updated_at` and for those that define it but raise
    /// `NotImplementedError` at call time.
    pub fn get_updated_at(
        &self,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, crate::error::Error> {
        Python::attach(|py| {
            let updated_at = self.optional_method(py, "get_updated_at")?.call0()?;
            if updated_at.is_none() {
                return Ok(None);
            }
            Ok(Some(extract_datetime(&updated_at)?))
        })
    }
}

/// The revisions a merge proposal's diff was made from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffRefs {
    /// Head of the source branch.
    pub head_revision: RevisionId,
    /// Merge base of the source and target branches, if known.
    pub base_revision: Option<RevisionId>,
    /// Head of the target branch when the diff was made, if known.
    pub start_revision: Option<RevisionId>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for DiffRefs {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(DiffRefs {
            head_revision: ob.getattr("head_revision")?.extract()?,
            base_revision: ob.getattr("base_revision")?.extract()?,
            start_revision: ob.getattr("start_revision")?.extract()?,
        })
    }
}


/// A comment on a line of a merge proposal's diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineComment {
    /// Path of the file, on the side the comment is on.
    pub path: String,
    /// Line number on that side.
    pub line: u32,
    /// Side of the diff the line is on.
    pub side: DiffSide,
    /// Text of the comment.
    pub body: String,
    /// Path of the file before the change, if it was renamed.
    pub old_path: Option<String>,
    /// Line number in the old file, for forges that need both line numbers
    /// for unchanged lines (GitLab).
    pub old_line: Option<u32>,
    /// Line number in the new file, likewise.
    pub new_line: Option<u32>,
}

/// A reply to an existing review thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadReply {
    /// Opaque id of the thread, as the forge reports it.
    pub thread_id: String,
    /// Text of the reply.
    pub body: String,
}

/// What a review asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewOutcome {
    /// Comment without approving or requesting changes.
    Comment,
    /// Approve the proposal.
    Approve,
    /// Request changes to the proposal.
    RequestChanges,
}

impl ReviewOutcome {
    fn as_str(self) -> &'static str {
        match self {
            ReviewOutcome::Comment => "comment",
            ReviewOutcome::Approve => "approve",
            ReviewOutcome::RequestChanges => "request-changes",
        }
    }
}

#[pyclass]
/// Builder for creating merge proposals.
pub struct ProposalBuilder(Py<PyAny>, Py<PyDict>);

impl ProposalBuilder {
    /// Sets the description for the merge proposal being built.
    pub fn description(self, description: &str) -> Self {
        Python::attach(|py| {
            self.1
                .bind(py)
                .set_item("description", description)
                .unwrap();
        });
        self
    }

    /// Sets the labels for the merge proposal being built.
    pub fn labels(self, labels: &[&str]) -> Self {
        Python::attach(|py| {
            self.1.bind(py).set_item("labels", labels).unwrap();
        });
        self
    }

    /// Sets the reviewers for the merge proposal being built.
    pub fn reviewers(self, reviewers: &[&str]) -> Self {
        Python::attach(|py| {
            self.1.bind(py).set_item("reviewers", reviewers).unwrap();
        });
        self
    }

    /// Sets whether to allow collaboration for the merge proposal being built.
    pub fn allow_collaboration(self, allow_collaboration: bool) -> Self {
        Python::attach(|py| {
            self.1
                .bind(py)
                .set_item("allow_collaboration", allow_collaboration)
                .unwrap();
        });
        self
    }

    /// Sets the title for the merge proposal being built.
    pub fn title(self, title: &str) -> Self {
        Python::attach(|py| {
            self.1.bind(py).set_item("title", title).unwrap();
        });
        self
    }

    /// Sets the commit message for the merge proposal being built.
    pub fn commit_message(self, commit_message: &str) -> Self {
        Python::attach(|py| {
            self.1
                .bind(py)
                .set_item("commit_message", commit_message)
                .unwrap();
        });
        self
    }

    /// Sets whether the merge proposal is a work in progress.
    pub fn work_in_progress(self, work_in_progress: bool) -> Self {
        Python::attach(|py| {
            self.1
                .bind(py)
                .set_item("work_in_progress", work_in_progress)
                .unwrap();
        });
        self
    }

    /// Creates the merge proposal with all configured properties.
    pub fn build(self) -> Result<MergeProposal, crate::error::Error> {
        Python::attach(|py| {
            let kwargs = self.1;
            let proposal = self
                .0
                .call_method(py, "create_proposal", (), Some(kwargs.bind(py)))?;
            Ok(MergeProposal::from(proposal))
        })
    }
}

impl Forge {
    fn to_object(&self) -> &Py<PyAny> {
        &self.0
    }
    /// Retrieves a merge proposal by its URL.
    pub fn get_proposal_by_url(
        &self,
        url: &url::Url,
    ) -> Result<MergeProposal, crate::error::Error> {
        Python::attach(|py| {
            let proposal = self
                .0
                .call_method1(py, "get_proposal_by_url", (url.as_str(),))?;
            Ok(MergeProposal::from(proposal))
        })
    }

    /// Returns the web URL for a given branch on this forge.
    pub fn get_web_url(&self, branch: &dyn PyBranch) -> Result<url::Url, crate::error::Error> {
        Python::attach(|py| {
            let forge_obj = self.to_object();
            let branch_obj = branch.to_object(py);
            let url = forge_obj
                .call_method1(py, "get_web_url", (&branch_obj,))?
                .extract::<String>(py)?;
            Ok(url.parse::<url::Url>()?)
        })
    }

    /// Returns the base URL of this forge.
    pub fn base_url(&self) -> url::Url {
        Python::attach(|py| {
            let base_url = self.0.getattr(py, "base_url").unwrap();
            base_url.extract::<String>(py).unwrap().parse().unwrap()
        })
    }

    /// Returns the kind of forge (e.g., GitHub, GitLab).
    pub fn forge_kind(&self) -> String {
        Python::attach(|py| self.0.bind(py).get_type().name().unwrap().to_string())
    }

    /// Returns the name of the forge.
    pub fn forge_name(&self) -> String {
        Python::attach(|py| self.0.bind(py).get_type().name().unwrap().to_string())
    }

    /// Returns the format used for merge proposal descriptions on this forge.
    pub fn merge_proposal_description_format(&self) -> String {
        Python::attach(|py| {
            let merge_proposal_description_format = self
                .to_object()
                .getattr(py, "merge_proposal_description_format")
                .unwrap();
            merge_proposal_description_format.extract(py).unwrap()
        })
    }

    /// Checks if this forge supports setting commit messages for merge proposals.
    pub fn supports_merge_proposal_commit_message(&self) -> bool {
        Python::attach(|py| {
            let supports_merge_proposal_commit_message = self
                .to_object()
                .getattr(py, "supports_merge_proposal_commit_message")
                .unwrap();
            supports_merge_proposal_commit_message.extract(py).unwrap()
        })
    }

    /// Checks if this forge supports setting titles for merge proposals.
    pub fn supports_merge_proposal_title(&self) -> bool {
        Python::attach(|py| {
            let supports_merge_proposal_title = self
                .to_object()
                .getattr(py, "supports_merge_proposal_title")
                .unwrap();
            supports_merge_proposal_title.extract(py).unwrap()
        })
    }

    /// Checks if this forge supports adding labels to merge proposals.
    pub fn supports_merge_proposal_labels(&self) -> bool {
        Python::attach(|py| {
            let supports_merge_proposal_labels = self
                .to_object()
                .getattr(py, "supports_merge_proposal_labels")
                .unwrap();
            supports_merge_proposal_labels.extract(py).unwrap()
        })
    }

    /// Creates a proposal builder for a merge proposal from one branch to another.
    pub fn get_proposer(
        &self,
        from_branch: &dyn PyBranch,
        to_branch: &dyn PyBranch,
    ) -> Result<ProposalBuilder, crate::error::Error> {
        Python::attach(|py| {
            let from_branch_obj = from_branch.to_object(py);
            let to_branch_obj = to_branch.to_object(py);
            Ok(ProposalBuilder(
                self.0
                    .call_method1(py, "get_proposer", (from_branch_obj, to_branch_obj))?,
                PyDict::new(py).into(),
            ))
        })
    }

    /// Returns an iterator over merge proposals owned by the current user.
    ///
    /// Eagerly drains the underlying Python iterator; the first per-item
    /// error short-circuits and is returned. Use
    /// [`Forge::iter_my_proposals_lazy`] for streaming with per-item
    /// error handling.
    pub fn iter_my_proposals(
        &self,
        status: Option<MergeProposalStatus>,
        author: Option<String>,
    ) -> Result<impl Iterator<Item = MergeProposal>, Error> {
        let ret: Vec<MergeProposal> = self
            .iter_my_proposals_lazy(status, author)?
            .collect::<Result<_, _>>()?;
        Ok(ret.into_iter())
    }

    /// Returns a lazy iterator over merge proposals owned by the current user.
    ///
    /// Each item is a `Result` so that per-item Python errors are surfaced
    /// rather than panicking — the underlying breezy iterators can raise
    /// lazily (e.g. the GitHub plugin only checks auth on first `__next__`).
    pub fn iter_my_proposals_lazy(
        &self,
        status: Option<MergeProposalStatus>,
        author: Option<String>,
    ) -> Result<impl Iterator<Item = Result<MergeProposal, Error>>, Error> {
        let iter = Python::attach(|py| -> Result<Py<PyAny>, Error> {
            Ok(self
                .0
                .call_method(py, "iter_my_proposals", (status, author), None)?
                .bind(py)
                .try_iter()?
                .into_any()
                .unbind())
        })?;
        Ok(std::iter::from_fn(move || {
            Python::attach(|py| match iter.call_method0(py, "__next__") {
                Ok(proposal) => Some(Ok(MergeProposal::from(proposal))),
                Err(e) if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) => None,
                Err(e) => Some(Err(e.into())),
            })
        }))
    }

    /// Returns an iterator over the names of the forks owned by the given owner.
    ///
    /// Defaults to the current user. Eagerly drains the underlying Python
    /// iterator; the first per-item error short-circuits and is returned.
    pub fn iter_my_forks(
        &self,
        owner: Option<&str>,
    ) -> Result<impl Iterator<Item = String>, Error> {
        let ret: Vec<String> = Python::attach(|py| -> Result<_, Error> {
            self.0
                .call_method1(py, "iter_my_forks", (owner,))?
                .bind(py)
                .try_iter()?
                .map(|project| Ok(project?.extract::<String>()?))
                .collect()
        })?;
        Ok(ret.into_iter())
    }

    /// Calls a Python method that yields merge proposals and collects them.
    ///
    /// Returns `Error::NotImplemented` if the forge lacks the method, as
    /// older versions of Breezy do.
    fn collect_proposals(
        &self,
        method: &str,
        status: Option<MergeProposalStatus>,
        kwargs: impl FnOnce(&Bound<'_, PyDict>) -> PyResult<()>,
    ) -> Result<Vec<MergeProposal>, Error> {
        Python::attach(|py| {
            let forge = self.0.bind(py);
            if !forge.hasattr(method)? {
                return Err(Error::NotImplemented);
            }
            let args = PyDict::new(py);
            if let Some(status) = status {
                args.set_item("status", status)?;
            }
            kwargs(&args)?;
            forge
                .call_method(method, (), Some(&args))?
                .try_iter()?
                .map(|mp| Ok(MergeProposal::from(mp?.unbind())))
                .collect()
        })
    }

    /// Returns the proposals the current user, or one of their teams, is
    /// asked to review.
    ///
    /// The status defaults to open.
    pub fn iter_my_review_requests(
        &self,
        status: Option<MergeProposalStatus>,
    ) -> Result<impl Iterator<Item = MergeProposal>, Error> {
        Ok(self
            .collect_proposals("iter_my_review_requests", status, |_| Ok(()))?
            .into_iter())
    }

    /// Returns the proposals the current user has reviewed.
    ///
    /// Some forges drop the review request once the review is submitted, so
    /// these are not necessarily returned by [`Forge::iter_my_review_requests`].
    /// The status defaults to open.
    pub fn iter_my_reviewed_proposals(
        &self,
        status: Option<MergeProposalStatus>,
    ) -> Result<impl Iterator<Item = MergeProposal>, Error> {
        Ok(self
            .collect_proposals("iter_my_reviewed_proposals", status, |_| Ok(()))?
            .into_iter())
    }

    /// Returns the proposals in projects the current user maintains.
    ///
    /// `exclude` takes path prefixes of projects to leave out, such as
    /// `"org"` or `"org/project"`. The status defaults to open.
    pub fn iter_proposals_in_my_projects(
        &self,
        status: Option<MergeProposalStatus>,
        exclude: &[&str],
    ) -> Result<impl Iterator<Item = MergeProposal>, Error> {
        Ok(self
            .collect_proposals("iter_proposals_in_my_projects", status, |kwargs| {
                kwargs.set_item("exclude", exclude.to_vec())
            })?
            .into_iter())
    }

    /// Deletes the project with the given name, as yielded by [`Forge::iter_my_forks`].
    pub fn delete_project(&self, project: &str) -> Result<(), Error> {
        Python::attach(|py| {
            self.0.call_method1(py, "delete_project", (project,))?;
            Ok(())
        })
    }

    /// Gets a branch derived from a main branch with the given name and optional owner, returning a GenericBranch.
    pub fn get_derived_branch_as_generic(
        &self,
        main_branch: &dyn PyBranch,
        name: &str,
        owner: Option<&str>,
        preferred_schemes: Option<&[&str]>,
    ) -> Result<GenericBranch, crate::error::Error> {
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            if let Some(owner) = owner {
                kwargs.set_item("owner", owner)?;
            }
            if let Some(preferred_schemes) = preferred_schemes {
                kwargs.set_item("preferred_schemes", preferred_schemes)?;
            }
            let branch = self.0.call_method(
                py,
                "get_derived_branch",
                (main_branch.to_object(py), name),
                Some(&kwargs),
            )?;
            Ok(GenericBranch::from(branch))
        })
    }

    /// Returns an iterator over merge proposals from one branch to another.
    pub fn iter_proposals(
        &self,
        source_branch: &dyn PyBranch,
        target_branch: &dyn PyBranch,
        status: MergeProposalStatus,
    ) -> Result<impl Iterator<Item = MergeProposal>, crate::error::Error> {
        Python::attach(move |py| {
            let kwargs = PyDict::new(py);
            let source_branch_obj = source_branch.to_object(py);
            let target_branch_obj = target_branch.to_object(py);
            kwargs.set_item("status", status.to_string())?;
            let proposal_iter: Py<PyAny> = self.0.call_method(
                py,
                "iter_proposals",
                (&source_branch_obj, &target_branch_obj),
                Some(&kwargs),
            )?;

            let mut ret = Vec::new();
            loop {
                match proposal_iter.call_method0(py, "__next__") {
                    Ok(proposal) => {
                        ret.push(MergeProposal::from(proposal));
                    }
                    Err(e) => {
                        if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) {
                            break;
                        } else {
                            return Err(e.into());
                        }
                    }
                }
            }
            Ok(ret.into_iter())
        })
    }

    /// Publishes a derived branch and returns the branch and its URL.
    pub fn publish_derived_as_generic_branch(
        &self,
        local_branch: &dyn PyBranch,
        base_branch: &dyn PyBranch,
        name: &str,
        overwrite: Option<bool>,
        owner: Option<&str>,
        revision_id: Option<&RevisionId>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> Result<(GenericBranch, url::Url), crate::error::Error> {
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            let local_branch_obj = local_branch.to_object(py);
            let base_branch_obj = base_branch.to_object(py);
            let forge_obj = self.to_object();

            kwargs.set_item("local_branch", local_branch_obj)?;
            kwargs.set_item("base_branch", base_branch_obj)?;
            kwargs.set_item("name", name)?;
            if let Some(overwrite) = overwrite {
                kwargs.set_item("overwrite", overwrite)?;
            }
            if let Some(owner) = owner {
                kwargs.set_item("owner", owner)?;
            }
            if let Some(revision_id) = revision_id {
                kwargs.set_item("revision_id", revision_id.clone())?;
            }
            if let Some(tag_selector) = tag_selector {
                kwargs.set_item("tag_selector", py_tag_selector(py, tag_selector)?)?;
            }
            let (b, u): (Py<PyAny>, String) = forge_obj
                .call_method(py, "publish_derived", (), Some(&kwargs))?
                .extract(py)?;
            Ok((GenericBranch::from(b), u.parse::<url::Url>()?))
        })
    }

    /// Returns the URL for pushing to a branch on this forge.
    pub fn get_push_url(&self, branch: &dyn PyBranch) -> url::Url {
        Python::attach(|py| {
            let forge_obj = self.to_object();
            let branch_obj = branch.to_object(py);
            let url = forge_obj
                .call_method1(py, "get_push_url", (&branch_obj,))
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            url.parse::<url::Url>().unwrap()
        })
    }

    /// Returns the URL for a user's profile on this forge.
    pub fn get_user_url(&self, user: &str) -> Result<url::Url, crate::error::Error> {
        Python::attach(|py| {
            let url = self
                .to_object()
                .call_method1(py, "get_user_url", (user,))?
                .extract::<String>(py)?;
            Ok(url.parse::<url::Url>()?)
        })
    }

    /// Returns the username of the currently authenticated user.
    pub fn get_current_user(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| {
            let user = self
                .to_object()
                .call_method0(py, "get_current_user")?
                .extract::<Option<String>>(py)?;
            Ok(user)
        })
    }
}

impl std::fmt::Debug for Forge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Forge");
        if let Ok(base_url) = self.base_url().to_string().parse::<url::Url>() {
            s.field("base_url", &base_url);
        }
        s.finish()
    }
}

impl std::fmt::Display for Forge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base_url = self.base_url();
        write!(f, "{}", base_url)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Forge {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Forge(ob.to_owned().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for Forge {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

/// Returns the forge associated with the given branch.
pub fn get_forge(branch: &dyn PyBranch) -> Result<Forge, Error> {
    Python::attach(|py| {
        let m = py.import("breezy.forge").unwrap();
        let forge = m.call_method1("get_forge", (branch.to_object(py),))?;
        Ok(Forge(forge.unbind()))
    })
}

/// Returns a forge instance for the given hostname.
pub fn get_forge_by_hostname(hostname: &str) -> Result<Forge, Error> {
    Python::attach(|py| {
        let m = py.import("breezy.forge").unwrap();
        let forge = m.call_method1("get_forge_by_hostname", (hostname,))?;
        Ok(Forge(forge.unbind()))
    })
}

/// Extracts a title from a description text.
pub fn determine_title(description: &str) -> Result<String, String> {
    Python::attach(|py| {
        let m = py.import("breezy.forge").unwrap();
        let title = match m.call_method1("determine_title", (description,)) {
            Ok(title) => title,
            Err(e) => return Err(e.to_string()),
        };
        match title.extract::<String>() {
            Ok(title) => Ok(title),
            Err(e) => Err(e.to_string()),
        }
    })
}

/// Returns an iterator over all available forge instances.
pub fn iter_forge_instances() -> impl Iterator<Item = Forge> {
    let ret = Python::attach(|py| {
        let m = py.import("breezy.forge").unwrap();
        let f = m.getattr("iter_forge_instances").unwrap();
        let instances = f.call0().unwrap();
        instances
            .try_iter()
            .unwrap()
            .map(|i| Forge(i.unwrap().unbind()))
            .collect::<Vec<_>>()
    });
    ret.into_iter()
}

/// Creates a new project on a forge with the given name and optional summary.
pub fn create_project(name: &str, summary: Option<&str>) -> Result<(), Error> {
    Python::attach(|py| {
        let m = py.import("breezy.forge").unwrap();
        m.call_method1("create_project", (name, summary))?;
        Ok(())
    })
}

/// Retrieves a merge proposal by its URL.
pub fn get_proposal_by_url(url: &url::Url) -> Result<MergeProposal, Error> {
    Python::attach(|py| {
        let m = py.import("breezy.forge").unwrap();
        let proposal = m.call_method1("get_proposal_by_url", (url.to_string(),))?;
        Ok(MergeProposal::from(proposal.unbind()))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        AutoMergeStatus, DiffSide, Forge, MergeProposal, MergeStatus, ProposalComment,
        ProposalReview, ReviewThread, ReviewVerdict,
    };
    use crate::error::Error;
    use crate::revisionid::RevisionId;
    use pyo3::prelude::*;

    const FAKES: &std::ffi::CStr = cr#"
import dataclasses
import datetime
import enum
from types import SimpleNamespace


class Side(enum.Enum):
    OLD = "old"
    NEW = "new"


class Verdict(enum.Enum):
    APPROVED = "approved"
    CHANGES_REQUESTED = "changes-requested"
    COMMENTED = "commented"
    DISMISSED = "dismissed"


class FakeForge:
    def __init__(self):
        self.calls = []

    def iter_my_forks(self, owner=None):
        self.calls.append(("iter_my_forks", owner))
        for name in ["one", "two"]:
            yield "%s/%s" % (owner or "me", name)

    def delete_project(self, name):
        self.calls.append(("delete_project", name))

    def iter_my_review_requests(self, status="open"):
        self.calls.append(("iter_my_review_requests", status))
        yield FakeProposal()

    def iter_my_reviewed_proposals(self, status="open"):
        self.calls.append(("iter_my_reviewed_proposals", status))
        yield FakeProposal()
        yield FakeProposal()

    def iter_proposals_in_my_projects(self, status="open", exclude=()):
        self.calls.append(("iter_proposals_in_my_projects", status))
        self.calls.append(("exclude", ",".join(exclude)))
        return iter([])


class BrokenForge:
    def iter_my_forks(self, owner=None):
        yield "me/one"
        raise RuntimeError("no forks")

    def delete_project(self, name):
        raise RuntimeError("no such project")

    def get_current_user(self):
        raise RuntimeError("bad credentials")

    def iter_my_review_requests(self, status="open"):
        yield FakeProposal()
        raise RuntimeError("bad credentials")

    def get_user_url(self, user):
        raise RuntimeError("bad credentials")

    def get_web_url(self, branch):
        return "not a url"

    def publish_derived(self, **kwargs):
        return (None, "not a url")


class FakeProposal:
    def __init__(self):
        self.calls = []
        self.labels = ["bug"]

    def get_labels(self):
        return list(self.labels)

    def add_label(self, name):
        self.labels.append(name)

    def remove_label(self, name):
        self.labels.remove(name)

    def get_source_project(self):
        return "me/one"

    def get_target_project(self):
        return None

    def get_author(self):
        return "me"

    def iter_comments(self):
        yield SimpleNamespace(
            author="alice",
            body="Looks good",
            created_at=datetime.datetime(2024, 1, 2, 3, 4, 5),
            url="https://example.com/c1",
        )
        yield SimpleNamespace(
            author=None,
            body="Me too",
            created_at=datetime.datetime(
                2024, 1, 3, tzinfo=datetime.timezone(datetime.timedelta(hours=1))
            ),
            url=None,
        )
        yield SimpleNamespace(
            author="carol",
            body="UTC-aware",
            created_at=datetime.datetime(
                2024, 1, 4, 5, 6, 7, tzinfo=datetime.timezone.utc
            ),
            url=None,
        )

    def iter_reviews(self):
        yield SimpleNamespace(
            author="bob",
            verdict=Verdict.APPROVED,
            body=None,
            submitted_at=datetime.datetime(2024, 1, 4),
            revision=b"git-v1:abc",
            url="https://example.com/r1",
        )
        yield SimpleNamespace(
            author=None,
            verdict=Verdict.CHANGES_REQUESTED,
            body="Please fix",
            submitted_at=datetime.datetime(2024, 1, 5),
            revision=None,
            url=None,
        )
        yield SimpleNamespace(
            author="dan",
            verdict=Verdict.COMMENTED,
            body="just a note",
            submitted_at=datetime.datetime(2024, 1, 6),
            revision=None,
            url=None,
        )
        yield SimpleNamespace(
            author="erin",
            verdict=Verdict.DISMISSED,
            body=None,
            submitted_at=datetime.datetime(2024, 1, 7),
            revision=None,
            url=None,
        )

    def get_diff(self):
        return b"diff --git a/foo b/foo\n"

    def get_diff_refs(self):
        class DiffRefs:
            head_revision = b"git-v1:aaaa"
            base_revision = b"git-v1:bbbb"
            start_revision = None
        return DiffRefs()

    def iter_review_threads(self):
        yield SimpleNamespace(
            id="RT_1",
            path="README",
            side=Side.NEW,
            line=3,
            revision=b"git-v1:abc",
            outdated=False,
            resolved=False,
            resolvable=True,
            url="https://example.com/t1",
            comments=(
                SimpleNamespace(
                    author="alice",
                    body="Typo",
                    created_at=datetime.datetime(2024, 1, 2),
                    url=None,
                ),
            ),
        )
        yield SimpleNamespace(
            id="RT_2",
            path="setup.py",
            side=Side.OLD,
            line=None,
            revision=None,
            outdated=True,
            resolved=True,
            resolvable=False,
            url=None,
            comments=(),
        )

    def set_review_thread_resolved(self, thread_id, resolved=True):
        self.calls.append(("set_review_thread_resolved", thread_id, resolved))

    def get_file_text(self, path, revision):
        if path != "README":
            from breezy.transport import NoSuchFile
            raise NoSuchFile(path)
        return b"contents at " + revision

    def is_work_in_progress(self):
        return True

    def get_created_at(self):
        return datetime.datetime(2026, 1, 1, 10, 0, 0)

    def get_updated_at(self):
        return datetime.datetime(2026, 1, 2, 11, 30, 0, tzinfo=datetime.timezone.utc)

    def get_merged_at(self):
        return datetime.datetime(2026, 1, 3, 12, 0, 0, tzinfo=datetime.timezone.utc)


class NaiveMergedProposal:
    def get_merged_at(self):
        return datetime.datetime(2026, 1, 3, 12, 0, 0)


class UnmergedProposal:
    def get_merged_at(self):
        return None


class CIState(enum.Enum):
    PENDING = "pending"
    SUCCESS = "success"
    FAILURE = "failure"


@dataclasses.dataclass(frozen=True)
class CIFailure:
    name: str
    url: object
    log: str


class CIProposal:
    def __init__(self, state=CIState.FAILURE):
        self.state = state
        self.calls = []

    def get_ci_state(self):
        return self.state

    def iter_ci_failures(self, revision=None):
        self.calls.append(("iter_ci_failures", revision))
        yield CIFailure("test", "https://ci.example.com/1", "boom")
        yield CIFailure("lint", None, "")
        raise RuntimeError("log is gone")

    def approve_ci(self, revision=None):
        self.calls.append(("approve_ci", revision))
        return ["CI"]


class BrokenCIProposal:
    def get_ci_state(self):
        return "red"

    def iter_ci_failures(self, revision=None):
        raise NotImplementedError(self.iter_ci_failures)

    def approve_ci(self, revision=None):
        raise RuntimeError("forbidden")


class ReviewableProposal:
    def __init__(self):
        self.calls = []

    def post_review(self, revision, *, body=None, comments=(), replies=(), outcome=None):
        from breezy.forge import HeadChanged

        if revision != b"git-v1:aaaa":
            raise HeadChanged(revision, b"git-v1:aaaa")
        self.calls.append((
            revision,
            body,
            [(c.path, c.line, c.side.value, c.body, c.old_path, c.old_line, c.new_line)
             for c in comments],
            [(r.thread_id, r.body) for r in replies],
            outcome.value,
        ))
        return "https://example.com/review/1" if outcome.value != "comment" else None


class OldProposal:
    pass


class MergeStatusProposal:
    def merge(self, commit_message=None, auto=False, expected_revision=None):
        if expected_revision not in (None, b"git-v1:aaaa"):
            from breezy.forge import HeadChanged

            raise HeadChanged(expected_revision, b"git-v1:aaaa")
        self.calls = [(commit_message, auto, expected_revision)]

    def get_merge_status(self):
        from types import SimpleNamespace

        return SimpleNamespace(
            head_revision=b"git-v1:aaaa",
            mergeable=False,
            as_admin_only=False,
            reason="ci must pass",
            auto_merge=SimpleNamespace(value="available"),
        )


class OldForge:
    pass


class MergeableProposal:
    def __init__(self):
        self.calls = []

    def merge(self, commit_message=None, auto=False):
        self.calls.append((commit_message, auto))


class RebaseProposal:
    def rebase(self, expected_revision=None):
        if expected_revision is None:
            return False
        if expected_revision != b"git-v1:aaaa":
            from breezy.forge import HeadChanged

            raise HeadChanged(expected_revision, b"git-v1:aaaa")
        return True


class UnsupportedProposal:
    def iter_comments(self):
        raise NotImplementedError(self.iter_comments)

    def iter_review_threads(self):
        raise NotImplementedError(self.iter_review_threads)

    def set_review_thread_resolved(self, thread_id, resolved=True):
        raise NotImplementedError(self.set_review_thread_resolved)

    def iter_reviews(self):
        raise NotImplementedError(self.iter_reviews)

    def get_merge_status(self):
        raise NotImplementedError(self.get_merge_status)

    def iter_events(self):
        raise NotImplementedError(self.iter_events)

    def rebase(self, expected_revision=None):
        raise NotImplementedError(self.rebase)


class Kind(enum.Enum):
    PUSH = "push"
    APPROVAL = "approval"
    FUTURE = "future"


class TimelineProposal:
    def iter_events(self):
        yield SimpleNamespace(
            kind=Kind.PUSH,
            actor=None,
            timestamp=datetime.datetime(2026, 1, 1, 10, 0),
            revision=b"git-v1:aaaa",
            actor_is_bot=False,
        )
        yield SimpleNamespace(
            kind=Kind.APPROVAL,
            actor="ci[bot]",
            timestamp=datetime.datetime(2026, 1, 2, 10, 0, tzinfo=datetime.timezone.utc),
            revision=None,
            actor_is_bot=True,
        )
        yield SimpleNamespace(
            kind=Kind.FUTURE,
            actor=None,
            timestamp=datetime.datetime(2026, 1, 3, 10, 0, tzinfo=datetime.timezone.utc),
            revision=None,
            actor_is_bot=False,
        )


class BrokenProposal:
    url = "not a url"

    def get_diff(self):
        raise RuntimeError("no diff")

    def get_diff_refs(self):
        raise RuntimeError("no refs")

    def get_author(self):
        raise RuntimeError("no author")

    def is_work_in_progress(self):
        raise NotImplementedError(self.is_work_in_progress)

    def get_created_at(self):
        return "yesterday"

    def get_labels(self):
        raise NotImplementedError(self.get_labels)

    def remove_label(self, name):
        raise RuntimeError("no such label")

    def iter_events(self):
        raise RuntimeError("no events")

    def get_source_project(self):
        raise RuntimeError("no source project")

    def get_target_project(self):
        raise RuntimeError("no target project")

    def iter_comments(self):
        yield SimpleNamespace(author="x", body="y", created_at="not a date", url=None)

    def iter_review_threads(self):
        yield SimpleNamespace(
            id="RT_1",
            path="README",
            side=SimpleNamespace(value="middle"),
            line=3,
            revision=None,
            outdated=False,
            resolved=False,
            resolvable=True,
            url=None,
            comments=(),
        )

    def set_review_thread_resolved(self, thread_id, resolved=True):
        raise RuntimeError("no such thread")

    def iter_reviews(self):
        yield SimpleNamespace(
            author="x",
            verdict=SimpleNamespace(value="bogus"),
            body=None,
            submitted_at=datetime.datetime(2024, 1, 4),
            revision=None,
            url=None,
        )

    def get_file_text(self, path, revision):
        raise NotImplementedError(self.get_file_text)
"#;

    fn fake(py: Python<'_>, name: &str) -> Py<PyAny> {
        let m = PyModule::from_code(
            py,
            FAKES,
            c"breezyshim_forge_fakes.py",
            c"breezyshim_forge_fakes",
        )
        .unwrap();
        m.getattr(name).unwrap().call0().unwrap().unbind()
    }

    fn calls(py: Python<'_>, obj: &Py<PyAny>) -> Vec<(String, Option<String>)> {
        obj.getattr(py, "calls").unwrap().extract(py).unwrap()
    }

    #[test]
    fn test_iter_my_forks() {
        let obj = Python::attach(|py| fake(py, "FakeForge"));
        let forge = Forge::from(Python::attach(|py| obj.clone_ref(py)));
        assert_eq!(
            forge.iter_my_forks(None).unwrap().collect::<Vec<_>>(),
            vec!["me/one", "me/two"]
        );
        assert_eq!(
            forge
                .iter_my_forks(Some("other"))
                .unwrap()
                .collect::<Vec<_>>(),
            vec!["other/one", "other/two"]
        );
        assert_eq!(
            Python::attach(|py| calls(py, &obj)),
            vec![
                ("iter_my_forks".to_string(), None),
                ("iter_my_forks".to_string(), Some("other".to_string())),
            ]
        );
    }

    #[test]
    fn test_iter_my_forks_error() {
        let forge = Forge::from(Python::attach(|py| fake(py, "BrokenForge")));
        assert!(forge.iter_my_forks(None).is_err());
    }

    #[test]
    fn test_proposal_listing() {
        use super::MergeProposalStatus;
        let obj = Python::attach(|py| fake(py, "FakeForge"));
        let forge = Forge::from(Python::attach(|py| obj.clone_ref(py)));
        let mps: Vec<_> = forge.iter_my_review_requests(None).unwrap().collect();
        assert_eq!(mps[0].get_author().unwrap(), Some("me".to_string()));
        assert_eq!(mps.len(), 1);
        assert_eq!(
            forge
                .iter_my_reviewed_proposals(Some(MergeProposalStatus::Merged))
                .unwrap()
                .count(),
            2
        );
        assert_eq!(
            forge
                .iter_proposals_in_my_projects(Some(MergeProposalStatus::All), &["org", "x/y"])
                .unwrap()
                .count(),
            0
        );
        assert_eq!(
            Python::attach(|py| calls(py, &obj)),
            vec![
                (
                    "iter_my_review_requests".to_string(),
                    Some("open".to_string())
                ),
                (
                    "iter_my_reviewed_proposals".to_string(),
                    Some("merged".to_string())
                ),
                (
                    "iter_proposals_in_my_projects".to_string(),
                    Some("all".to_string())
                ),
                ("exclude".to_string(), Some("org,x/y".to_string())),
            ]
        );
    }

    #[test]
    fn test_proposal_listing_errors() {
        use crate::error::Error;
        let forge = Forge::from(Python::attach(|py| fake(py, "BrokenForge")));
        assert!(matches!(
            forge.iter_my_review_requests(None),
            Err(Error::Other(_))
        ));
        let forge = Forge::from(Python::attach(|py| fake(py, "OldForge")));
        assert!(matches!(
            forge.iter_my_review_requests(None),
            Err(Error::NotImplemented)
        ));
        assert!(matches!(
            forge.iter_my_reviewed_proposals(None),
            Err(Error::NotImplemented)
        ));
        assert!(matches!(
            forge.iter_proposals_in_my_projects(None, &[]),
            Err(Error::NotImplemented)
        ));
    }

    #[test]
    fn test_delete_project() {
        let obj = Python::attach(|py| fake(py, "FakeForge"));
        let forge = Forge::from(Python::attach(|py| obj.clone_ref(py)));
        forge.delete_project("me/one").unwrap();
        assert_eq!(
            Python::attach(|py| calls(py, &obj)),
            vec![("delete_project".to_string(), Some("me/one".to_string()))]
        );
    }

    #[test]
    fn test_delete_project_error() {
        let forge = Forge::from(Python::attach(|py| fake(py, "BrokenForge")));
        assert!(forge.delete_project("me/one").is_err());
    }

    #[test]
    fn test_get_source_and_target_project() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(mp.get_source_project().unwrap(), Some("me/one".to_string()));
        assert_eq!(mp.get_target_project().unwrap(), None);
    }

    #[test]
    fn test_forge_errors_do_not_panic() {
        let forge = Forge::from(Python::attach(|py| fake(py, "BrokenForge")));
        assert!(matches!(forge.get_current_user(), Err(Error::Other(_))));
        assert!(matches!(
            forge.get_user_url("someone"),
            Err(Error::Other(_))
        ));
        let branch = crate::branch::GenericBranch::from(Python::attach(|py| py.None()));
        assert!(matches!(
            forge.get_web_url(&branch),
            Err(Error::InvalidURL(..))
        ));
        assert!(matches!(
            forge.publish_derived_as_generic_branch(&branch, &branch, "x", None, None, None, None),
            Err(Error::InvalidURL(..))
        ));
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(mp.url(), Err(Error::InvalidURL(..))));
    }

    #[test]
    fn test_get_author() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(mp.get_author().unwrap(), Some("me".to_string()));
    }

    #[test]
    fn test_get_author_unsupported() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        assert!(matches!(
            mp.get_author(),
            Err(crate::error::Error::NotImplemented)
        ));
    }

    #[test]
    fn test_iter_events() {
        use super::{ProposalEvent, ProposalEventKind};
        use chrono::TimeZone;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "TimelineProposal")));
        assert_eq!(
            mp.iter_events().unwrap().collect::<Vec<_>>(),
            vec![
                ProposalEvent {
                    kind: ProposalEventKind::Push,
                    actor: None,
                    timestamp: chrono::Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap(),
                    revision: Some(crate::RevisionId::from(b"git-v1:aaaa".to_vec())),
                    actor_is_bot: false,
                },
                ProposalEvent {
                    kind: ProposalEventKind::Approval,
                    actor: Some("ci[bot]".to_string()),
                    timestamp: chrono::Utc.with_ymd_and_hms(2026, 1, 2, 10, 0, 0).unwrap(),
                    revision: None,
                    actor_is_bot: true,
                },
                ProposalEvent {
                    kind: ProposalEventKind::Unknown("future".to_string()),
                    actor: None,
                    timestamp: chrono::Utc.with_ymd_and_hms(2026, 1, 3, 10, 0, 0).unwrap(),
                    revision: None,
                    actor_is_bot: false,
                },
            ]
        );
    }

    #[test]
    fn test_iter_events_errors() {
        use crate::error::Error;
        for name in ["OldProposal", "UnsupportedProposal"] {
            let mp = MergeProposal::from(Python::attach(|py| fake(py, name)));
            assert!(matches!(mp.iter_events(), Err(Error::NotImplemented)));
        }
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(mp.iter_events(), Err(Error::Other(_))));
    }

    #[test]
    fn test_get_author_error() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(mp.get_author().is_err());
    }

    #[test]
    fn test_labels() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(mp.get_labels().unwrap(), vec!["bug".to_string()]);
        mp.add_label("new label").unwrap();
        assert_eq!(
            mp.get_labels().unwrap(),
            vec!["bug".to_string(), "new label".to_string()]
        );
        mp.remove_label("bug").unwrap();
        assert_eq!(mp.get_labels().unwrap(), vec!["new label".to_string()]);
    }

    #[test]
    fn test_labels_unsupported() {
        use crate::error::Error;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        assert!(matches!(mp.get_labels(), Err(Error::NotImplemented)));
        assert!(matches!(mp.add_label("x"), Err(Error::NotImplemented)));
        assert!(matches!(mp.remove_label("x"), Err(Error::NotImplemented)));
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(mp.get_labels(), Err(Error::NotImplemented)));
        assert!(matches!(mp.remove_label("x"), Err(Error::Other(_))));
    }

    #[test]
    fn test_get_merged_at() {
        use chrono::TimeZone;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(
            mp.get_merged_at().unwrap(),
            Some(chrono::Utc.with_ymd_and_hms(2026, 1, 3, 12, 0, 0).unwrap())
        );
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "NaiveMergedProposal")));
        assert_eq!(
            mp.get_merged_at().unwrap(),
            Some(chrono::Utc.with_ymd_and_hms(2026, 1, 3, 12, 0, 0).unwrap())
        );
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "UnmergedProposal")));
        assert_eq!(mp.get_merged_at().unwrap(), None);
    }

    #[test]
    fn test_metadata() {
        use chrono::TimeZone;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert!(mp.is_work_in_progress().unwrap());
        assert_eq!(
            mp.get_created_at().unwrap(),
            Some(chrono::Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap())
        );
        assert_eq!(
            mp.get_updated_at().unwrap(),
            Some(chrono::Utc.with_ymd_and_hms(2026, 1, 2, 11, 30, 0).unwrap())
        );
    }

    #[test]
    fn test_metadata_unsupported() {
        use crate::error::Error;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        assert!(matches!(
            mp.is_work_in_progress(),
            Err(Error::NotImplemented)
        ));
        assert!(matches!(mp.get_created_at(), Err(Error::NotImplemented)));
        assert!(matches!(mp.get_updated_at(), Err(Error::NotImplemented)));
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(
            mp.is_work_in_progress(),
            Err(Error::NotImplemented)
        ));
        assert!(matches!(mp.get_created_at(), Err(Error::Other(_))));
    }

    #[test]
    fn test_get_source_and_target_project_error() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(mp.get_source_project().is_err());
        assert!(mp.get_target_project().is_err());
    }

    #[test]
    fn test_iter_comments() {
        use chrono::TimeZone;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(
            mp.iter_comments().unwrap().collect::<Vec<_>>(),
            vec![
                ProposalComment {
                    author: Some("alice".to_string()),
                    body: "Looks good".to_string(),
                    created_at: chrono::Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
                    url: Some("https://example.com/c1".to_string()),
                },
                ProposalComment {
                    author: None,
                    body: "Me too".to_string(),
                    created_at: chrono::Utc.with_ymd_and_hms(2024, 1, 2, 23, 0, 0).unwrap(),
                    url: None,
                },
                ProposalComment {
                    author: Some("carol".to_string()),
                    body: "UTC-aware".to_string(),
                    created_at: chrono::Utc.with_ymd_and_hms(2024, 1, 4, 5, 6, 7).unwrap(),
                    url: None,
                },
            ]
        );
    }

    #[test]
    fn test_get_ci_state() {
        use super::CIState;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "CIProposal")));
        assert_eq!(mp.get_ci_state().unwrap(), Some(CIState::Failure));
        let mp = MergeProposal::from(Python::attach(|py| {
            let obj = fake(py, "CIProposal");
            obj.setattr(py, "state", py.None()).unwrap();
            obj
        }));
        assert_eq!(mp.get_ci_state().unwrap(), None);
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenCIProposal")));
        assert!(mp.get_ci_state().is_err());
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        assert!(matches!(
            mp.get_ci_state(),
            Err(crate::error::Error::NotImplemented)
        ));
    }

    #[test]
    fn test_iter_ci_failures() {
        use super::CIFailure;
        use crate::revisionid::RevisionId;
        let obj = Python::attach(|py| fake(py, "CIProposal"));
        let mp = MergeProposal::from(Python::attach(|py| obj.clone_ref(py)));
        let revision = RevisionId::from(b"rev-1".to_vec());
        let mut failures = mp.iter_ci_failures(Some(&revision)).unwrap();
        assert_eq!(
            failures.next().unwrap().unwrap(),
            CIFailure {
                name: "test".to_string(),
                url: Some("https://ci.example.com/1".to_string()),
                log: "boom".to_string(),
            }
        );
        assert_eq!(
            failures.next().unwrap().unwrap(),
            CIFailure {
                name: "lint".to_string(),
                url: None,
                log: String::new(),
            }
        );
        assert!(failures.next().unwrap().is_err());
        assert!(failures.next().is_none());
        let calls: Vec<(String, Option<Vec<u8>>)> =
            Python::attach(|py| obj.getattr(py, "calls").unwrap().extract(py).unwrap());
        assert_eq!(
            calls,
            vec![("iter_ci_failures".to_string(), Some(b"rev-1".to_vec()))]
        );
    }

    #[test]
    fn test_iter_reviews() {
        use chrono::TimeZone;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(
            mp.iter_reviews().unwrap().collect::<Vec<_>>(),
            vec![
                ProposalReview {
                    author: Some("bob".to_string()),
                    verdict: ReviewVerdict::Approved,
                    body: None,
                    submitted_at: chrono::Utc.with_ymd_and_hms(2024, 1, 4, 0, 0, 0).unwrap(),
                    revision: Some(RevisionId::from(b"git-v1:abc".to_vec())),
                    url: Some("https://example.com/r1".to_string()),
                },
                ProposalReview {
                    author: None,
                    verdict: ReviewVerdict::ChangesRequested,
                    body: Some("Please fix".to_string()),
                    submitted_at: chrono::Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap(),
                    revision: None,
                    url: None,
                },
                ProposalReview {
                    author: Some("dan".to_string()),
                    verdict: ReviewVerdict::Commented,
                    body: Some("just a note".to_string()),
                    submitted_at: chrono::Utc.with_ymd_and_hms(2024, 1, 6, 0, 0, 0).unwrap(),
                    revision: None,
                    url: None,
                },
                ProposalReview {
                    author: Some("erin".to_string()),
                    verdict: ReviewVerdict::Dismissed,
                    body: None,
                    submitted_at: chrono::Utc.with_ymd_and_hms(2024, 1, 7, 0, 0, 0).unwrap(),
                    revision: None,
                    url: None,
                },
            ]
        );
    }

    #[test]
    fn test_comments_and_reviews_unsupported() {
        for name in ["OldProposal", "UnsupportedProposal"] {
            let mp = MergeProposal::from(Python::attach(|py| fake(py, name)));
            assert!(matches!(mp.iter_comments(), Err(Error::NotImplemented)));
            assert!(matches!(mp.iter_reviews(), Err(Error::NotImplemented)));
        }
    }

    #[test]
    fn test_comments_and_reviews_invalid() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(mp.iter_comments(), Err(Error::Other(_))));
        assert!(matches!(mp.iter_reviews(), Err(Error::Other(_))));
    }

    #[test]
    fn test_get_diff() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(mp.get_diff().unwrap(), b"diff --git a/foo b/foo\n".to_vec());
    }

    #[test]
    fn test_get_diff_unsupported() {
        use crate::error::Error;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        assert!(matches!(mp.get_diff(), Err(Error::NotImplemented)));
    }

    #[test]
    fn test_get_diff_error() {
        use crate::error::Error;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(mp.get_diff(), Err(Error::Other(_))));
    }

    #[test]
    fn test_get_diff_refs() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(
            mp.get_diff_refs().unwrap(),
            super::DiffRefs {
                head_revision: b"git-v1:aaaa".to_vec().into(),
                base_revision: Some(b"git-v1:bbbb".to_vec().into()),
                start_revision: None,
            }
        );
    }

    #[test]
    fn test_get_diff_refs_errors() {
        use crate::error::Error;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        assert!(matches!(mp.get_diff_refs(), Err(Error::NotImplemented)));
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(mp.get_diff_refs(), Err(Error::Other(_))));
    }

    #[test]
    fn test_get_file_text() {
        use crate::error::Error;
        use crate::revisionid::RevisionId;
        let revision = RevisionId::from(b"rev-1".to_vec());
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(
            mp.get_file_text("README", &revision).unwrap(),
            b"contents at rev-1".to_vec()
        );
        assert!(matches!(
            mp.get_file_text("missing", &revision),
            Err(Error::NoSuchFile(p)) if p == std::path::Path::new("missing")
        ));
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(
            mp.get_file_text("README", &revision),
            Err(Error::NotImplemented)
        ));
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        assert!(matches!(
            mp.get_file_text("README", &revision),
            Err(Error::NotImplemented)
        ));
    }

    #[test]
    fn test_iter_ci_failures_unsupported() {
        use crate::error::Error;
        for name in ["OldProposal", "BrokenCIProposal"] {
            let mp = MergeProposal::from(Python::attach(|py| fake(py, name)));
            assert!(matches!(
                mp.iter_ci_failures(None).map(|_| ()),
                Err(Error::NotImplemented)
            ));
        }
    }

    #[test]
    fn test_approve_ci() {
        let obj = Python::attach(|py| fake(py, "CIProposal"));
        let mp = MergeProposal::from(Python::attach(|py| obj.clone_ref(py)));
        assert_eq!(mp.approve_ci(None).unwrap(), vec!["CI".to_string()]);
        let calls: Vec<(String, Option<Vec<u8>>)> =
            Python::attach(|py| obj.getattr(py, "calls").unwrap().extract(py).unwrap());
        assert_eq!(calls, vec![("approve_ci".to_string(), None)]);
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenCIProposal")));
        assert!(matches!(
            mp.approve_ci(None),
            Err(crate::error::Error::Other(_))
        ));
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        assert!(matches!(
            mp.approve_ci(None),
            Err(crate::error::Error::NotImplemented)
        ));
    }

    #[test]
    fn test_iter_review_threads() {
        use chrono::TimeZone;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "FakeProposal")));
        assert_eq!(
            mp.iter_review_threads().unwrap().collect::<Vec<_>>(),
            vec![
                ReviewThread {
                    id: "RT_1".to_string(),
                    path: "README".to_string(),
                    side: DiffSide::New,
                    line: Some(3),
                    revision: Some(RevisionId::from(b"git-v1:abc".to_vec())),
                    outdated: false,
                    resolved: false,
                    resolvable: true,
                    url: Some("https://example.com/t1".to_string()),
                    comments: vec![ProposalComment {
                        author: Some("alice".to_string()),
                        body: "Typo".to_string(),
                        created_at: chrono::Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
                        url: None,
                    }],
                },
                ReviewThread {
                    id: "RT_2".to_string(),
                    path: "setup.py".to_string(),
                    side: DiffSide::Old,
                    line: None,
                    revision: None,
                    outdated: true,
                    resolved: true,
                    resolvable: false,
                    url: None,
                    comments: vec![],
                },
            ]
        );
    }

    #[test]
    fn test_set_review_thread_resolved() {
        let obj = Python::attach(|py| fake(py, "FakeProposal"));
        let mp = MergeProposal::from(Python::attach(|py| obj.clone_ref(py)));
        mp.set_review_thread_resolved("RT_1", true).unwrap();
        mp.set_review_thread_resolved("RT_2", false).unwrap();
        let calls: Vec<(String, String, bool)> =
            Python::attach(|py| obj.getattr(py, "calls").unwrap().extract(py).unwrap());
        assert_eq!(
            calls,
            vec![
                (
                    "set_review_thread_resolved".to_string(),
                    "RT_1".to_string(),
                    true
                ),
                (
                    "set_review_thread_resolved".to_string(),
                    "RT_2".to_string(),
                    false
                ),
            ]
        );
    }

    #[test]
    fn test_review_threads_unsupported() {
        for name in ["OldProposal", "UnsupportedProposal"] {
            let mp = MergeProposal::from(Python::attach(|py| fake(py, name)));
            assert!(matches!(
                mp.iter_review_threads(),
                Err(Error::NotImplemented)
            ));
            assert!(matches!(
                mp.set_review_thread_resolved("RT_1", true),
                Err(Error::NotImplemented)
            ));
        }
    }

    #[test]
    fn test_review_threads_invalid() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(matches!(mp.iter_review_threads(), Err(Error::Other(_))));
        assert!(matches!(
            mp.set_review_thread_resolved("RT_1", true),
            Err(Error::Other(_))
        ));
    }

    fn has_review_types() -> bool {
        // Older versions of Breezy lack the review types.
        Python::attach(|py| {
            py.import("breezy.forge")?
                .getattr("InlineComment")
                .map(|_| ())
        })
        .is_ok()
    }

    #[test]
    fn test_post_review() {
        use super::{DiffSide, InlineComment, ReviewOutcome, ThreadReply};
        use crate::revisionid::RevisionId;
        if !has_review_types() {
            return;
        }
        let obj = Python::attach(|py| fake(py, "ReviewableProposal"));
        let mp = MergeProposal::from(Python::attach(|py| obj.clone_ref(py)));
        let head = RevisionId::from(b"git-v1:aaaa".to_vec());
        let url = mp
            .post_review(
                &head,
                Some("Looks good"),
                &[InlineComment {
                    path: "foo".to_string(),
                    line: 3,
                    side: DiffSide::Old,
                    body: "why?".to_string(),
                    old_path: Some("bar".to_string()),
                    old_line: Some(3),
                    new_line: None,
                }],
                &[ThreadReply {
                    thread_id: "t1".to_string(),
                    body: "done".to_string(),
                }],
                ReviewOutcome::Approve,
            )
            .unwrap();
        assert_eq!(url, Some("https://example.com/review/1".parse().unwrap()));
        assert_eq!(
            mp.post_review(&head, None, &[], &[], ReviewOutcome::Comment)
                .unwrap(),
            None
        );
        type Call = (
            Vec<u8>,
            Option<String>,
            Vec<(
                String,
                u32,
                String,
                String,
                Option<String>,
                Option<u32>,
                Option<u32>,
            )>,
            Vec<(String, String)>,
            String,
        );
        let calls: Vec<Call> =
            Python::attach(|py| obj.getattr(py, "calls").unwrap().extract(py).unwrap());
        assert_eq!(
            calls,
            vec![
                (
                    b"git-v1:aaaa".to_vec(),
                    Some("Looks good".to_string()),
                    vec![(
                        "foo".to_string(),
                        3,
                        "old".to_string(),
                        "why?".to_string(),
                        Some("bar".to_string()),
                        Some(3),
                        None
                    )],
                    vec![("t1".to_string(), "done".to_string())],
                    "approve".to_string(),
                ),
                (
                    b"git-v1:aaaa".to_vec(),
                    None,
                    vec![],
                    vec![],
                    "comment".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn test_post_review_head_changed() {
        use super::ReviewOutcome;
        use crate::error::Error;
        use crate::revisionid::RevisionId;
        if !has_review_types() {
            return;
        }
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "ReviewableProposal")));
        let old = RevisionId::from(b"git-v1:bbbb".to_vec());
        match mp.post_review(&old, None, &[], &[], ReviewOutcome::Approve) {
            Err(Error::HeadChanged(expected, Some(actual))) => {
                assert_eq!(expected, old);
                assert_eq!(actual, RevisionId::from(b"git-v1:aaaa".to_vec()));
            }
            r => panic!("unexpected result {:?}", r),
        }
    }

    #[test]
    fn test_post_review_unsupported() {
        use super::ReviewOutcome;
        use crate::error::Error;
        use crate::revisionid::RevisionId;
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "OldProposal")));
        let head = RevisionId::from(b"git-v1:aaaa".to_vec());
        assert!(matches!(
            mp.post_review(&head, None, &[], &[], ReviewOutcome::Approve),
            Err(Error::NotImplemented)
        ));
    }

    #[test]
    fn test_merge() {
        let obj = Python::attach(|py| fake(py, "MergeableProposal"));
        let mp = MergeProposal::from(Python::attach(|py| obj.clone_ref(py)));
        mp.merge(true).unwrap();
        mp.merge(false).unwrap();
        let calls: Vec<(Option<String>, bool)> =
            Python::attach(|py| obj.getattr(py, "calls").unwrap().extract(py).unwrap());
        assert_eq!(calls, vec![(None, true), (None, false)]);
    }

    #[test]
    fn test_determine_title() {
        let description = "This is a test description";
        let title = super::determine_title(description).unwrap();
        assert_eq!(title, "This is a test description");
    }

    #[test]
    fn test_determine_title_invalid() {
        let description = "";
        assert_eq!(
            "ValueError: ",
            super::determine_title(description).unwrap_err()
        );
    }

    #[test]
    fn test_merge_with() {
        let obj = Python::attach(|py| fake(py, "MergeStatusProposal"));
        let mp = MergeProposal::from(Python::attach(|py| obj.clone_ref(py)));
        let head = RevisionId::from(b"git-v1:aaaa".to_vec());
        mp.merge_with(Some("Merge"), false, Some(&head)).unwrap();
        let calls: Vec<(Option<String>, bool, Option<RevisionId>)> =
            Python::attach(|py| obj.getattr(py, "calls").unwrap().extract(py).unwrap());
        assert_eq!(calls, vec![(Some("Merge".to_string()), false, Some(head))]);
    }

    #[test]
    fn test_merge_with_head_changed() {
        // Older versions of Breezy lack HeadChanged.
        if Python::attach(|py| {
            py.import("breezy.forge")?
                .getattr("HeadChanged")
                .map(|_| ())
        })
        .is_err()
        {
            return;
        }
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "MergeStatusProposal")));
        let old = RevisionId::from(b"git-v1:bbbb".to_vec());
        match mp.merge_with(None, true, Some(&old)) {
            Err(Error::HeadChanged(expected, actual)) => {
                assert_eq!(expected, old);
                assert_eq!(actual, Some(RevisionId::from(b"git-v1:aaaa".to_vec())));
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_get_merge_status() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "MergeStatusProposal")));
        assert_eq!(
            mp.get_merge_status().unwrap(),
            MergeStatus {
                head_revision: RevisionId::from(b"git-v1:aaaa".to_vec()),
                mergeable: false,
                as_admin_only: false,
                reason: Some("ci must pass".to_string()),
                auto_merge: AutoMergeStatus::Available,
            }
        );
    }

    #[test]
    fn test_get_merge_status_unsupported() {
        for name in ["OldProposal", "UnsupportedProposal"] {
            let mp = MergeProposal::from(Python::attach(|py| fake(py, name)));
            assert!(
                matches!(mp.get_merge_status(), Err(Error::NotImplemented)),
                "{name}"
            );
        }
    }

    #[test]
    fn test_rebase() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "RebaseProposal")));
        assert!(!mp.rebase(None).unwrap());
        let head = RevisionId::from(b"git-v1:aaaa".to_vec());
        assert!(mp.rebase(Some(&head)).unwrap());
    }

    #[test]
    fn test_rebase_head_changed() {
        // Older versions of Breezy lack HeadChanged.
        let supported = Python::attach(|py| {
            py.import("breezy.forge")
                .and_then(|m| m.hasattr("HeadChanged"))
                .unwrap()
        });
        if !supported {
            return;
        }
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "RebaseProposal")));
        let old = RevisionId::from(b"git-v1:bbbb".to_vec());
        match mp.rebase(Some(&old)) {
            Err(Error::HeadChanged(expected, actual)) => {
                assert_eq!(expected, old);
                assert_eq!(actual, Some(RevisionId::from(b"git-v1:aaaa".to_vec())));
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_rebase_unsupported() {
        for name in ["OldProposal", "UnsupportedProposal"] {
            let mp = MergeProposal::from(Python::attach(|py| fake(py, name)));
            assert!(matches!(mp.rebase(None), Err(Error::NotImplemented)));
        }
    }
}
