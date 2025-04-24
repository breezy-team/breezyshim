//! Code hosting services and merge proposals.
use crate::branch::{py_tag_selector, Branch, GenericBranch, PyBranch};
use crate::error::Error;
use crate::revisionid::RevisionId;
use pyo3::conversion::ToPyObject;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::hash::Hash;

pub struct Forge(PyObject);

impl Clone for Forge {
    fn clone(&self) -> Self {
        Forge(Python::with_gil(|py| self.0.clone_ref(py)))
    }
}

impl From<PyObject> for Forge {
    fn from(obj: PyObject) -> Self {
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MergeProposalStatus {
    All,
    Open,
    Closed,
    Merged,
}

impl MergeProposalStatus {
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

impl ToPyObject for MergeProposalStatus {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_string().to_object(py)
    }
}

impl FromPyObject<'_> for MergeProposalStatus {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
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

pub struct MergeProposal(PyObject);

impl Clone for MergeProposal {
    fn clone(&self) -> Self {
        MergeProposal(Python::with_gil(|py| self.0.clone_ref(py)))
    }
}

impl From<PyObject> for MergeProposal {
    fn from(obj: PyObject) -> Self {
        MergeProposal(obj)
    }
}

impl MergeProposal {
    pub fn from_url(url: &url::Url) -> Result<Self, Error> {
        get_proposal_by_url(url)
    }

    pub fn reopen(&self) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method0(py, "reopen")?;
            Ok(())
        })
    }

    pub fn close(&self) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method0(py, "close")?;
            Ok(())
        })
    }

    pub fn url(&self) -> Result<url::Url, crate::error::Error> {
        Python::with_gil(|py| {
            let url = self.0.getattr(py, "url")?;
            Ok(url.extract::<String>(py)?.parse().unwrap())
        })
    }

    pub fn is_merged(&self) -> Result<bool, crate::error::Error> {
        Python::with_gil(|py| {
            let is_merged = self.0.call_method0(py, "is_merged")?;
            is_merged.extract(py).map_err(|e| e.into())
        })
    }

    pub fn is_closed(&self) -> Result<bool, crate::error::Error> {
        Python::with_gil(|py| {
            let is_closed = self.0.call_method0(py, "is_closed")?;
            is_closed.extract(py).map_err(|e| e.into())
        })
    }

    pub fn get_title(&self) -> Result<Option<String>, crate::error::Error> {
        Python::with_gil(|py| {
            let title = self.0.call_method0(py, "get_title")?;
            title.extract(py).map_err(|e| e.into())
        })
    }

    pub fn set_title(&self, title: Option<&str>) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(py, "set_title", (title,))?;
            Ok(())
        })
    }

    pub fn get_commit_message(&self) -> Result<Option<String>, crate::error::Error> {
        Python::with_gil(|py| {
            let commit_message = self.0.call_method0(py, "get_commit_message")?;
            commit_message.extract(py).map_err(|e| e.into())
        })
    }

    pub fn set_commit_message(
        &self,
        commit_message: Option<&str>,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "set_commit_message", (commit_message,))?;
            Ok(())
        })
    }

    pub fn get_target_branch_url(&self) -> Result<Option<url::Url>, crate::error::Error> {
        Python::with_gil(|py| {
            let target_branch_url = self.0.call_method0(py, "get_target_branch_url")?;
            target_branch_url
                .extract::<String>(py)?
                .parse::<url::Url>()
                .map(Some)
                .map_err(|e| e.into())
        })
    }

    pub fn get_source_branch_url(&self) -> Result<Option<url::Url>, crate::error::Error> {
        Python::with_gil(|py| {
            let source_branch_url = self.0.call_method0(py, "get_source_branch_url")?;
            source_branch_url
                .extract::<String>(py)?
                .parse::<url::Url>()
                .map(Some)
                .map_err(|e| e.into())
        })
    }

    pub fn get_description(&self) -> Result<Option<String>, crate::error::Error> {
        Python::with_gil(|py| {
            let description = self.0.call_method0(py, "get_description")?;
            description.extract(py).map_err(|e| e.into())
        })
    }

    pub fn set_description(&self, description: Option<&str>) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.0.call_method1(py, "set_description", (description,))?;
            Ok(())
        })
    }

    pub fn can_be_merged(&self) -> Result<bool, crate::error::Error> {
        Python::with_gil(|py| {
            let can_be_merged = self.0.call_method0(py, "can_be_merged")?;
            can_be_merged.extract(py).map_err(|e| e.into())
        })
    }

    pub fn supports_auto_merge(&self) -> bool {
        Python::with_gil(|py| {
            self.0
                .getattr(py, "supports_auto_merge")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    pub fn merge(&self, auto: bool) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.0.call_method1(py, "merge", (auto,))?;
            Ok(())
        })
    }

    pub fn get_web_url(&self) -> Result<url::Url, crate::error::Error> {
        Python::with_gil(|py| {
            let web_url = self.0.call_method0(py, "get_web_url")?;
            web_url
                .extract::<String>(py)?
                .parse::<url::Url>()
                .map_err(|e| e.into())
        })
    }

    pub fn get_merged_by(&self) -> Result<Option<String>, crate::error::Error> {
        Python::with_gil(|py| {
            let merged_by = self.0.call_method0(py, "get_merged_by")?;
            merged_by.extract(py).map_err(|e| e.into())
        })
    }

    pub fn get_merged_at(
        &self,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, crate::error::Error> {
        Python::with_gil(|py| {
            let merged_at = self.0.call_method0(py, "get_merged_at")?;
            merged_at
                .extract::<Option<chrono::DateTime<chrono::Utc>>>(py)
                .map_err(|e| e.into())
        })
    }
}

#[pyclass]
pub struct ProposalBuilder(PyObject, Py<PyDict>);

impl ProposalBuilder {
    pub fn description(self, description: &str) -> Self {
        Python::with_gil(|py| {
            self.1
                .bind(py)
                .set_item("description", description)
                .unwrap();
        });
        self
    }

    pub fn labels(self, labels: &[&str]) -> Self {
        Python::with_gil(|py| {
            self.1.bind(py).set_item("labels", labels).unwrap();
        });
        self
    }

    pub fn reviewers(self, reviewers: &[&str]) -> Self {
        Python::with_gil(|py| {
            self.1.bind(py).set_item("reviewers", reviewers).unwrap();
        });
        self
    }

    pub fn allow_collaboration(self, allow_collaboration: bool) -> Self {
        Python::with_gil(|py| {
            self.1
                .bind(py)
                .set_item("allow_collaboration", allow_collaboration)
                .unwrap();
        });
        self
    }

    pub fn title(self, title: &str) -> Self {
        Python::with_gil(|py| {
            self.1.bind(py).set_item("title", title).unwrap();
        });
        self
    }

    pub fn commit_message(self, commit_message: &str) -> Self {
        Python::with_gil(|py| {
            self.1
                .bind(py)
                .set_item("commit_message", commit_message)
                .unwrap();
        });
        self
    }

    pub fn build(self) -> Result<MergeProposal, crate::error::Error> {
        Python::with_gil(|py| {
            let kwargs = self.1;
            let proposal =
                self.0
                    .call_method_bound(py, "create_proposal", (), Some(kwargs.bind(py)))?;
            Ok(MergeProposal::from(proposal))
        })
    }
}

impl Forge {
    pub fn get_proposal_by_url(
        &self,
        url: &url::Url,
    ) -> Result<MergeProposal, crate::error::Error> {
        Python::with_gil(|py| {
            let proposal =
                self.to_object(py)
                    .call_method1(py, "get_proposal_by_url", (url.as_str(),))?;
            Ok(MergeProposal::from(proposal))
        })
    }

    pub fn get_web_url<B: PyBranch>(&self, branch: &B) -> Result<url::Url, crate::error::Error> {
        Python::with_gil(|py| {
            let url = self
                .to_object(py)
                .call_method1(py, "get_web_url", (&branch.to_object(py),))?
                .extract::<String>(py)
                .unwrap();
            Ok(url.parse::<url::Url>().unwrap())
        })
    }

    pub fn base_url(&self) -> url::Url {
        Python::with_gil(|py| {
            let base_url = self.to_object(py).getattr(py, "base_url").unwrap();
            base_url.extract::<String>(py).unwrap().parse().unwrap()
        })
    }

    pub fn forge_kind(&self) -> String {
        Python::with_gil(|py| {
            self.to_object(py)
                .bind(py)
                .get_type()
                .name()
                .unwrap()
                .to_string()
        })
    }

    pub fn forge_name(&self) -> String {
        Python::with_gil(|py| {
            self.to_object(py)
                .bind(py)
                .get_type()
                .name()
                .unwrap()
                .to_string()
        })
    }

    pub fn merge_proposal_description_format(&self) -> String {
        Python::with_gil(|py| {
            let merge_proposal_description_format = self
                .to_object(py)
                .getattr(py, "merge_proposal_description_format")
                .unwrap();
            merge_proposal_description_format.extract(py).unwrap()
        })
    }

    pub fn supports_merge_proposal_commit_message(&self) -> bool {
        Python::with_gil(|py| {
            let supports_merge_proposal_commit_message = self
                .to_object(py)
                .getattr(py, "supports_merge_proposal_commit_message")
                .unwrap();
            supports_merge_proposal_commit_message.extract(py).unwrap()
        })
    }

    pub fn supports_merge_proposal_title(&self) -> bool {
        Python::with_gil(|py| {
            let supports_merge_proposal_title = self
                .to_object(py)
                .getattr(py, "supports_merge_proposal_title")
                .unwrap();
            supports_merge_proposal_title.extract(py).unwrap()
        })
    }

    pub fn supports_merge_proposal_labels(&self) -> bool {
        Python::with_gil(|py| {
            let supports_merge_proposal_labels = self
                .to_object(py)
                .getattr(py, "supports_merge_proposal_labels")
                .unwrap();
            supports_merge_proposal_labels.extract(py).unwrap()
        })
    }

    pub fn get_proposer<B1: PyBranch, B2: PyBranch>(
        &self,
        from_branch: &B1,
        to_branch: &B2,
    ) -> Result<ProposalBuilder, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(ProposalBuilder(
                self.0.call_method1(
                    py,
                    "get_proposer",
                    (from_branch.to_object(py), to_branch.to_object(py)),
                )?,
                PyDict::new_bound(py).into(),
            ))
        })
    }

    pub fn iter_my_proposals(
        &self,
        status: Option<MergeProposalStatus>,
        author: Option<String>,
    ) -> Result<impl Iterator<Item = MergeProposal>, Error> {
        let ret: Vec<MergeProposal> =
            Python::with_gil(|py| -> Result<Vec<MergeProposal>, Error> {
                Ok(self
                    .to_object(py)
                    .call_method_bound(
                        py,
                        "iter_my_proposals",
                        (status.to_object(py), author),
                        None,
                    )?
                    .bind(py)
                    .iter()
                    .unwrap()
                    .map(|proposal| MergeProposal::from(proposal.unwrap().to_object(py)))
                    .collect())
            })?;
        Ok(ret.into_iter())
    }

    pub fn get_derived_branch<B: PyBranch>(
        &self,
        main_branch: &B,
        name: &str,
        owner: Option<&str>,
        preferred_schemes: Option<&[&str]>,
    ) -> Result<Box<dyn Branch>, crate::error::Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new_bound(py);
            if let Some(owner) = owner {
                kwargs.set_item("owner", owner)?;
            }
            if let Some(preferred_schemes) = preferred_schemes {
                kwargs.set_item("preferred_schemes", preferred_schemes)?;
            }
            let branch = self.to_object(py).call_method_bound(
                py,
                "get_derived_branch",
                (main_branch.to_object(py), name),
                Some(&kwargs),
            )?;
            Ok(Box::new(GenericBranch::new(branch)) as Box<dyn Branch>)
        })
    }

    pub fn iter_proposals(
        &self,
        source_branch: &dyn PyBranch,
        target_branch: &dyn PyBranch,
        status: MergeProposalStatus,
    ) -> Result<impl Iterator<Item = MergeProposal>, crate::error::Error> {
        Python::with_gil(move |py| {
            let kwargs = PyDict::new_bound(py);
            kwargs.set_item("status", status.to_string())?;
            let proposal_iter: PyObject = self
                .0
                .call_method_bound(
                    py,
                    "iter_proposals",
                    (&source_branch.to_object(py), &target_branch.to_object(py)),
                    Some(&kwargs),
                )?
                .extract(py)?;

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
        Python::with_gil(|py| {
            let kwargs = PyDict::new_bound(py);
            kwargs.set_item("local_branch", local_branch.to_object(py))?;
            kwargs.set_item("base_branch", base_branch.to_object(py))?;
            kwargs.set_item("name", name)?;
            if let Some(overwrite) = overwrite {
                kwargs.set_item("overwrite", overwrite)?;
            }
            if let Some(owner) = owner {
                kwargs.set_item("owner", owner)?;
            }
            if let Some(revision_id) = revision_id {
                kwargs.set_item("revision_id", revision_id)?;
            }
            if let Some(tag_selector) = tag_selector {
                kwargs.set_item("tag_selector", py_tag_selector(py, tag_selector)?)?;
            }
            let (b, u): (PyObject, String) = self
                .to_object(py)
                .call_method_bound(py, "publish_derived", (), Some(&kwargs))?
                .extract(py)?;
            Ok((
                Box::new(GenericBranch::new(b)) as Box<dyn Branch>,
                u.parse::<url::Url>().unwrap(),
            ))
        })
    }

    pub fn get_push_url(&self, branch: &dyn PyBranch) -> url::Url {
        Python::with_gil(|py| {
            let url = self
                .to_object(py)
                .call_method1(py, "get_push_url", (&branch.to_object(py),))
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            url.parse::<url::Url>().unwrap()
        })
    }

    pub fn get_user_url(&self, user: &str) -> Result<url::Url, crate::error::Error> {
        Python::with_gil(|py| {
            let url = self
                .to_object(py)
                .call_method1(py, "get_user_url", (user,))
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            Ok(url.parse::<url::Url>().unwrap())
        })
    }

    pub fn get_current_user(&self) -> Result<Option<String>, crate::error::Error> {
        Python::with_gil(|py| {
            let user = self
                .to_object(py)
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

impl FromPyObject<'_> for Forge {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Forge(ob.to_object(ob.py())))
    }
}

impl ToPyObject for Forge {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

pub fn get_forge(branch: &dyn PyBranch) -> Result<Forge, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.forge").unwrap();
        let forge = m.call_method1("get_forge", (branch.to_object(py),))?;
        Ok(Forge(forge.to_object(py)))
    })
}

pub fn get_forge_by_hostname(hostname: &str) -> Result<Forge, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.forge").unwrap();
        let forge = m.call_method1("get_forge_by_hostname", (hostname,))?;
        Ok(Forge(forge.to_object(py)))
    })
}

pub fn determine_title(description: &str) -> Result<String, String> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.forge").unwrap();
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

pub fn iter_forge_instances() -> impl Iterator<Item = Forge> {
    let ret = Python::with_gil(|py| {
        let m = py.import_bound("breezy.forge").unwrap();
        let f = m.getattr("iter_forge_instances").unwrap();
        let instances = f.call0().unwrap();
        instances
            .iter()
            .unwrap()
            .map(|i| Forge(i.unwrap().to_object(py)))
            .collect::<Vec<_>>()
    });
    ret.into_iter()
}

pub fn create_project(name: &str, summary: Option<&str>) -> Result<(), Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.forge").unwrap();
        m.call_method1("create_project", (name, summary))?;
        Ok(())
    })
}

pub fn get_proposal_by_url(url: &url::Url) -> Result<MergeProposal, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.forge").unwrap();
        let proposal = m.call_method1("get_proposal_by_url", (url.to_string(),))?;
        Ok(MergeProposal::from(proposal.to_object(py)))
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
