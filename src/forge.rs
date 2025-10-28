//! Code hosting services and merge proposals.
use crate::branch::{py_tag_selector, Branch, GenericBranch, PyBranch};
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
            Ok(url.extract::<String>(py)?.parse().unwrap())
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

    /// Returns the URL of the target branch for this merge proposal.
    pub fn get_target_branch_url(&self) -> Result<Option<url::Url>, crate::error::Error> {
        Python::attach(|py| {
            let target_branch_url = self.0.call_method0(py, "get_target_branch_url")?;
            target_branch_url
                .extract::<String>(py)?
                .parse::<url::Url>()
                .map(Some)
                .map_err(Into::into)
        })
    }

    /// Returns the URL of the source branch for this merge proposal.
    pub fn get_source_branch_url(&self) -> Result<Option<url::Url>, crate::error::Error> {
        Python::attach(|py| {
            let source_branch_url = self.0.call_method0(py, "get_source_branch_url")?;
            source_branch_url
                .extract::<String>(py)?
                .parse::<url::Url>()
                .map(Some)
                .map_err(Into::into)
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
            self.0.call_method1(py, "merge", (auto,))?;
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

    /// Returns the date and time when this proposal was merged, if it has been merged.
    pub fn get_merged_at(
        &self,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, crate::error::Error> {
        Python::attach(|py| {
            let merged_at = self.0.call_method0(py, "get_merged_at")?;
            merged_at
                .extract::<Option<chrono::DateTime<chrono::Utc>>>(py)
                .map_err(Into::into)
        })
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
                .extract::<String>(py)
                .unwrap();
            Ok(url.parse::<url::Url>().unwrap())
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
    pub fn iter_my_proposals(
        &self,
        status: Option<MergeProposalStatus>,
        author: Option<String>,
    ) -> Result<impl Iterator<Item = MergeProposal>, Error> {
        let ret: Vec<MergeProposal> = Python::attach(|py| -> Result<Vec<MergeProposal>, Error> {
            Ok(self
                .0
                .call_method(py, "iter_my_proposals", (status, author), None)?
                .bind(py)
                .try_iter()
                .unwrap()
                .map(|proposal| MergeProposal::from(proposal.unwrap().unbind()))
                .collect())
        })?;
        Ok(ret.into_iter())
    }

    /// Gets a branch derived from a main branch with the given name and optional owner.
    pub fn get_derived_branch(
        &self,
        main_branch: &dyn PyBranch,
        name: &str,
        owner: Option<&str>,
        preferred_schemes: Option<&[&str]>,
    ) -> Result<Box<dyn Branch>, crate::error::Error> {
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
            Ok(Box::new(GenericBranch::from(branch)) as Box<dyn Branch>)
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
    pub fn publish_derived(
        &self,
        local_branch: &dyn PyBranch,
        base_branch: &dyn PyBranch,
        name: &str,
        overwrite: Option<bool>,
        owner: Option<&str>,
        revision_id: Option<&RevisionId>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> Result<(Box<dyn Branch>, url::Url), crate::error::Error> {
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
            Ok((
                Box::new(GenericBranch::from(b)) as Box<dyn Branch>,
                u.parse::<url::Url>().unwrap(),
            ))
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
                .call_method1(py, "get_user_url", (user,))
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            Ok(url.parse::<url::Url>().unwrap())
        })
    }

    /// Returns the username of the currently authenticated user.
    pub fn get_current_user(&self) -> Result<Option<String>, crate::error::Error> {
        Python::attach(|py| {
            let user = self
                .to_object()
                .call_method0(py, "get_current_user")
                .unwrap()
                .extract::<Option<String>>(py)
                .unwrap();
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
