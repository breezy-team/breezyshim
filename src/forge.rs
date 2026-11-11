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
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("auto", auto)?;
            self.0.call_method(py, "merge", (), Some(&kwargs))?;
            Ok(())
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
        Python::attach(|py| {
            let mp = self.0.bind(py);
            if !mp.hasattr("get_author")? {
                return Err(crate::error::Error::NotImplemented);
            }
            mp.call_method0("get_author")?.extract().map_err(Into::into)
        })
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
            let merged_at = self.0.call_method0(py, "get_merged_at")?;
            if merged_at.is_none(py) {
                return Ok(None);
            }
            // Try timezone-aware first; fall back to naive (treat as UTC).
            if let Ok(dt) = merged_at.extract::<chrono::DateTime<chrono::Utc>>(py) {
                return Ok(Some(dt));
            }
            merged_at
                .extract::<chrono::NaiveDateTime>(py)
                .map(|dt| Some(dt.and_utc()))
                .map_err(Into::into)
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
}

/// Side of a diff that a line number refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    use super::{Forge, MergeProposal};
    use pyo3::prelude::*;

    const FAKES: &std::ffi::CStr = cr#"
class FakeForge:
    def __init__(self):
        self.calls = []

    def iter_my_forks(self, owner=None):
        self.calls.append(("iter_my_forks", owner))
        for name in ["one", "two"]:
            yield "%s/%s" % (owner or "me", name)

    def delete_project(self, name):
        self.calls.append(("delete_project", name))


class BrokenForge:
    def iter_my_forks(self, owner=None):
        yield "me/one"
        raise RuntimeError("no forks")

    def delete_project(self, name):
        raise RuntimeError("no such project")

    def get_current_user(self):
        raise RuntimeError("bad credentials")

    def get_user_url(self, user):
        raise RuntimeError("bad credentials")

    def get_web_url(self, branch):
        return "not a url"

    def publish_derived(self, **kwargs):
        return (None, "not a url")


class FakeProposal:
    def get_source_project(self):
        return "me/one"

    def get_target_project(self):
        return None

    def get_author(self):
        return "me"


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


class MergeableProposal:
    def __init__(self):
        self.calls = []

    def merge(self, commit_message=None, auto=False):
        self.calls.append((commit_message, auto))


class BrokenProposal:
    url = "not a url"

    def get_author(self):
        raise RuntimeError("no author")

    def get_source_project(self):
        raise RuntimeError("no source project")

    def get_target_project(self):
        raise RuntimeError("no target project")
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
        use crate::error::Error;
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
    fn test_get_author_error() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(mp.get_author().is_err());
    }

    #[test]
    fn test_get_source_and_target_project_error() {
        let mp = MergeProposal::from(Python::attach(|py| fake(py, "BrokenProposal")));
        assert!(mp.get_source_project().is_err());
        assert!(mp.get_target_project().is_err());
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
}
