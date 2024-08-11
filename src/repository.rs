use crate::controldir::ControlDir;
use crate::delta::TreeDelta;
use crate::graph::Graph;
use crate::location::AsLocation;
use crate::revisionid::RevisionId;
use crate::tree::{PyRevisionTree, RevisionTree};
use chrono::DateTime;
use chrono::TimeZone;
use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub struct RepositoryFormat(PyObject);

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

pub struct PyRepository(PyObject);

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

impl ToPyObject for Revision {
    fn to_object(&self, py: Python) -> PyObject {
        let kwargs = PyDict::new_bound(py);
        kwargs.set_item("message", self.message.clone()).unwrap();
        kwargs
            .set_item("committer", self.committer.clone())
            .unwrap();
        kwargs.set_item("timestamp", self.timestamp).unwrap();
        kwargs.set_item("timezone", self.timezone).unwrap();
        kwargs.set_item("revision_id", &self.revision_id).unwrap();
        kwargs
            .set_item("parent_ids", self.parent_ids.iter().collect::<Vec<_>>())
            .unwrap();
        py.import_bound("breezy.revision")
            .unwrap()
            .getattr("Revision")
            .unwrap()
            .call((), Some(&kwargs))
            .unwrap()
            .to_object(py)
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

impl ToPyObject for PyRepository {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl From<PyObject> for PyRepository {
    fn from(o: PyObject) -> Self {
        PyRepository(o)
    }
}

impl Repository for PyRepository {
    fn as_repository(&self) -> Box<dyn Repository> {
        Python::with_gil(|py| Box::new(PyRepository::from(self.to_object(py).clone_ref(py))))
    }
}


pub trait Repository: ToPyObject {
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

    fn fetch(
        &self,
        other_repository: &dyn Repository,
        stop_revision: Option<&RevisionId>,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            self.to_object(py).call_method1(
                py,
                "fetch",
                (
                    other_repository.to_object(py),
                    stop_revision.map(|r| r.to_object(py)),
                ),
            )?;
            Ok(())
        })
    }

    fn revision_tree(&self, revid: &RevisionId) -> Result<Box<dyn RevisionTree>, crate::error::Error> {
        Python::with_gil(|py| {
            let o = self.to_object(py).call_method1(py, "revision_tree", (revid.clone(),))?;
            Ok(Box::new(PyRevisionTree::from(o)) as Box<dyn RevisionTree>)
        })
    }

    fn get_graph(&self) -> Graph {
        Python::with_gil(|py| Graph::from(self.to_object(py).call_method0(py, "get_graph").unwrap()))
    }

    fn controldir(&self) -> ControlDir {
        Python::with_gil(|py| ControlDir::new(self.to_object(py).getattr(py, "controldir").unwrap()))
    }

    fn format(&self) -> RepositoryFormat {
        Python::with_gil(|py| RepositoryFormat(self.to_object(py).getattr(py, "_format").unwrap()))
    }

    fn iter_revisions(
        &self,
        revision_ids: Vec<RevisionId>,
    ) -> Box<dyn Iterator<Item = (RevisionId, Option<Revision>)>> {
        Python::with_gil(|py| {
            let o = self
                .to_object(py)
                .call_method1(py, "iter_revisions", (revision_ids,))
                .unwrap();
            Box::new(RevisionIterator(o)) as Box<dyn Iterator<Item = (RevisionId, Option<Revision>)>>
        })
    }

    fn get_revision_deltas(
        &self,
        revs: &[Revision],
        specific_files: Option<&[&std::path::Path]>,
    ) -> Box<dyn Iterator<Item = TreeDelta>> {
        Python::with_gil(|py| {
            let revs = revs.iter().map(|r| r.to_object(py)).collect::<Vec<_>>();
            let specific_files = specific_files
                .map(|files| files.iter().map(|f| f.to_path_buf()).collect::<Vec<_>>());
            let o = self
                .to_object(py)
                .call_method1(py, "get_revision_deltas", (revs, specific_files))
                .unwrap();
            Box::new(DeltaIterator(o)) as Box<dyn Iterator<Item = TreeDelta>>
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

    fn as_repository(&self) -> Box<dyn Repository>;
}

pub fn open(base: impl AsLocation) -> Result<Box<dyn Repository>, crate::error::Error> {
    Python::with_gil(|py| {
        let o = py
            .import_bound("breezy.repository")?
            .getattr("Repository")?
            .call_method1("open", (base.as_location(),))?;
        Ok(Box::new(PyRepository::from(o.to_object(py))) as Box<dyn Repository>)
    })
}

#[cfg(test)]
mod repository_tests {
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
        let repo = crate::repository::open(td.path()).unwrap();
        let _repo2 = repo.as_repository();
    }
}
