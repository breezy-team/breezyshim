//! Repository handling
//!
//! A repository is a collection of revisions and their associated data.
use crate::controldir::{ControlDir, GenericControlDir};
use crate::delta::TreeDelta;
use crate::foreign::VcsType;
use crate::graph::Graph;
use crate::location::AsLocation;
use crate::lock::Lock;
use crate::revisionid::RevisionId;
use crate::tree::RevisionTree;
use chrono::DateTime;
use chrono::TimeZone;
use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;
use pyo3::types::PyDict;

crate::wrapped_py!(RepositoryFormat);

impl Clone for RepositoryFormat {
    fn clone(&self) -> Self {
        Python::with_gil(|py| RepositoryFormat(self.0.clone_ref(py)))
    }
}

impl RepositoryFormat {
    pub fn supports_chks(&self) -> bool {
        Python::with_gil(|py| {
            self.0
                .getattr(py, "supports_chks")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}

pub trait Repository {
    fn vcs_type(&self) -> VcsType;
    fn get_user_url(&self) -> url::Url;
    fn user_transport(&self) -> crate::transport::Transport;
    fn control_transport(&self) -> crate::transport::Transport;
    fn fetch<R: PyRepository>(
        &self,
        other_repository: &R,
        stop_revision: Option<&RevisionId>,
    ) -> Result<(), crate::error::Error>;
    fn revision_tree(&self, revid: &RevisionId) -> Result<RevisionTree, crate::error::Error>;
    fn get_graph(&self) -> Graph;
    fn controldir(&self) -> Box<dyn ControlDir>;
    fn format(&self) -> RepositoryFormat;
    fn iter_revisions(
        &self,
        revision_ids: Vec<RevisionId>,
    ) -> impl Iterator<Item = (RevisionId, Option<Revision>)>;
    fn get_revision_deltas(
        &self,
        revs: &[Revision],
        specific_files: Option<&[&std::path::Path]>,
    ) -> impl Iterator<Item = TreeDelta>;
    fn get_revision(&self, revision_id: &RevisionId) -> Result<Revision, crate::error::Error>;
    fn lookup_bzr_revision_id(
        &self,
        revision_id: &RevisionId,
    ) -> Result<(Vec<u8>,), crate::error::Error>;
    fn lookup_foreign_revision_id(
        &self,
        foreign_revid: &[u8],
    ) -> Result<RevisionId, crate::error::Error>;
    fn lock_read(&self) -> Result<Lock, crate::error::Error>;
    fn lock_write(&self) -> Result<Lock, crate::error::Error>;
}

pub trait PyRepository: ToPyObject + std::any::Any {}

crate::wrapped_py!(GenericRepository);

impl Clone for GenericRepository {
    fn clone(&self) -> Self {
        Python::with_gil(|py| GenericRepository(self.0.clone_ref(py)))
    }
}

#[derive(Debug)]
pub struct Revision {
    pub revision_id: RevisionId,
    pub parent_ids: Vec<RevisionId>,
    pub message: String,
    pub committer: String,
    pub timestamp: f64,
    pub timezone: i32,
}

impl Revision {
    pub fn datetime(&self) -> DateTime<chrono::FixedOffset> {
        let tz = chrono::FixedOffset::east_opt(self.timezone).unwrap();
        tz.timestamp_opt(self.timestamp as i64, 0).unwrap()
    }
}

impl<'py> IntoPyObject<'py> for &Revision {
    type Target = PyDict;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("message", self.message.clone())?;
        kwargs
            .set_item("committer", self.committer.clone())
            ?;
        kwargs.set_item("timestamp", self.timestamp)?;
        kwargs.set_item("timezone", self.timezone)?;
        kwargs.set_item("revision_id", &self.revision_id)?;
        kwargs
            .set_item("parent_ids", self.parent_ids.iter().collect::<Vec<_>>())
            ?;
        py.import("breezy.revision")
            ?
            .getattr("Revision")
            ?
            .call((), Some(&kwargs))
    }
}

impl FromPyObject<'_> for Revision {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Revision {
            revision_id: ob.getattr("revision_id")?.extract()?,
            parent_ids: ob.getattr("parent_ids")?.extract()?,
            message: ob.getattr("message")?.extract()?,
            committer: ob.getattr("committer")?.extract()?,
            timestamp: ob.getattr("timestamp")?.extract()?,
            timezone: ob.getattr("timezone")?.extract()?,
        })
    }
}

pub struct RevisionIterator(PyObject);

impl Iterator for RevisionIterator {
    type Item = (RevisionId, Option<Revision>);

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| match self.0.call_method0(py, "__next__") {
            Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
            Ok(o) => Some(o.extract(py).unwrap()),
            Err(e) => panic!("Error in revision iterator: {}", e),
        })
    }
}

pub struct DeltaIterator(PyObject);

impl Iterator for DeltaIterator {
    type Item = TreeDelta;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| match self.0.call_method0(py, "__next__") {
            Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
            Ok(o) => Some(o.extract(py).unwrap()),
            Err(e) => panic!("Error in delta iterator: {}", e),
        })
    }
}

impl PyRepository for GenericRepository {}

impl<T: PyRepository> Repository for T {
    fn vcs_type(&self) -> VcsType {
        Python::with_gil(|py| {
            if self.to_object(py).getattr(py, "_git").is_ok() {
                VcsType::Git
            } else {
                VcsType::Bazaar
            }
        })
    }

    fn get_user_url(&self) -> url::Url {
        Python::with_gil(|py| {
            self.to_object(py)
                .getattr(py, "user_url")
                .unwrap()
                .extract::<String>(py)
                .unwrap()
                .parse()
                .unwrap()
        })
    }

    fn user_transport(&self) -> crate::transport::Transport {
        Python::with_gil(|py| {
            crate::transport::Transport::new(
                self.to_object(py).getattr(py, "user_transport").unwrap(),
            )
        })
    }

    fn control_transport(&self) -> crate::transport::Transport {
        Python::with_gil(|py| {
            crate::transport::Transport::new(
                self.to_object(py).getattr(py, "control_transport").unwrap(),
            )
        })
    }

    fn fetch<R: PyRepository>(
        &self,
        other_repository: &R,
        stop_revision: Option<&RevisionId>,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py).call_method1(
                py,
                "fetch",
                (
                    other_repository,
                    stop_revision,
                ),
            )?;
            Ok(())
        })
    }

    fn revision_tree(&self, revid: &RevisionId) -> Result<RevisionTree, crate::error::Error> {
        Python::with_gil(|py| {
            let o = self.0.call_method1(py, "revision_tree", (revid,))?;
            Ok(RevisionTree::from(o))
        })
    }

    fn get_graph(&self) -> Graph {
        Python::with_gil(|py| {
            Graph::from(self.to_object(py).call_method0(py, "get_graph").unwrap())
        })
    }

    fn controldir(&self) -> Box<dyn ControlDir> {
        Python::with_gil(|py| {
            Box::new(GenericControlDir::new(
                self.to_object(py).getattr(py, "controldir").unwrap(),
            )) as Box<dyn ControlDir>
        })
    }

    fn format(&self) -> RepositoryFormat {
        Python::with_gil(|py| RepositoryFormat::from(self.to_object(py).getattr(py, "_format").unwrap()))
    }

    fn iter_revisions(
        &self,
        revision_ids: Vec<RevisionId>,
    ) -> impl Iterator<Item = (RevisionId, Option<Revision>)> {
        Python::with_gil(|py| {
            let o = self
                .to_object(py)
                .call_method1(py, "iter_revisions", (revision_ids,))
                .unwrap();
            RevisionIterator(o)
        })
    }

    fn get_revision_deltas(
        &self,
        revs: &[Revision],
        specific_files: Option<&[&std::path::Path]>,
    ) -> impl Iterator<Item = TreeDelta> {
        Python::with_gil(|py| {
            let specific_files = specific_files
                .map(|files| files.iter().map(|f| f.to_path_buf()).collect::<Vec<_>>());
            let o = self
                .to_object(py)
                .call_method1(py, "get_revision_deltas", (revs, specific_files))
                .unwrap();
            DeltaIterator(o)
        })
    }

    fn get_revision(&self, revision_id: &RevisionId) -> Result<Revision, crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "get_revision", (revision_id.clone(),))?
                .extract(py)
        })
        .map_err(|e| e.into())
    }

    // TODO: This should really be on ForeignRepository
    fn lookup_bzr_revision_id(
        &self,
        revision_id: &RevisionId,
    ) -> Result<(Vec<u8>,), crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "lookup_bzr_revision_id", (revision_id.clone(),))?
                .extract::<(Vec<u8>, PyObject)>(py)
        })
        .map_err(|e| e.into())
        .map(|(v, _m)| (v,))
    }

    fn lookup_foreign_revision_id(
        &self,
        foreign_revid: &[u8],
    ) -> Result<RevisionId, crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "lookup_foreign_revision_id", (foreign_revid,))?
                .extract(py)
        })
        .map_err(|e| e.into())
    }

    fn lock_read(&self) -> Result<Lock, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(Lock::from(
                self.to_object(py).call_method0(py, "lock_read")?,
            ))
        })
    }

    fn lock_write(&self) -> Result<Lock, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(Lock::from(
                self.to_object(py).call_method0(py, "lock_write")?,
            ))
        })
    }
}

pub fn open(base: impl AsLocation) -> Result<GenericRepository, crate::error::Error> {
    Python::with_gil(|py| {
        let o = py
            .import("breezy.repository")?
            .getattr("Repository")?
            .call_method1("open", (base.as_location(),))?;
        Ok(GenericRepository::from(o))
    })
}

#[cfg(test)]
mod repository_tests {
    use super::GenericRepository;
    use crate::controldir::ControlDirFormat;

    #[test]
    fn test_simple() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let _repo = crate::repository::open(td.path()).unwrap();
    }

    #[test]
    fn test_clone() {
        let td = tempfile::tempdir().unwrap();
        let _dir = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        let repo: GenericRepository = crate::repository::open(td.path()).unwrap();
        let _repo2 = repo.clone();
    }
}
