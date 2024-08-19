//! Working trees
use crate::branch::{Branch, RegularBranch};
use crate::controldir::ControlDir;
use crate::error::Error;
use crate::tree::{MutableTree, RevisionTree, Tree};
use crate::RevisionId;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

pub struct WorkingTree(pub PyObject);

impl Clone for WorkingTree {
    fn clone(&self) -> Self {
        Python::with_gil(|py| WorkingTree(self.0.clone_ref(py)))
    }
}

impl ToPyObject for WorkingTree {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl WorkingTree {
    pub fn is_control_filename(&self, path: &Path) -> bool {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "is_control_filename", (path,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Return the base path for this working tree.
    pub fn basedir(&self) -> PathBuf {
        Python::with_gil(|py| {
            self.to_object(py)
                .getattr(py, "basedir")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Return the branch for this working tree.
    pub fn branch(&self) -> Box<dyn Branch> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch").unwrap();
            Box::new(RegularBranch::new(branch)) as Box<dyn Branch>
        })
    }

    /// Return the control directory for this working tree.
    pub fn controldir(&self) -> ControlDir {
        Python::with_gil(|py| {
            let controldir = self.to_object(py).getattr(py, "controldir").unwrap();
            ControlDir::new(controldir)
        })
    }

    #[deprecated = "Use ::open instead"]
    pub fn open(path: &Path) -> Result<WorkingTree, Error> {
        open(path)
    }

    #[deprecated = "Use ::open_containing instead"]
    pub fn open_containing(path: &Path) -> Result<(WorkingTree, PathBuf), Error> {
        open_containing(path)
    }

    pub fn basis_tree(&self) -> Result<crate::tree::RevisionTree, Error> {
        Python::with_gil(|py| {
            let tree = self.to_object(py).call_method0(py, "basis_tree")?;
            Ok(RevisionTree(tree))
        })
    }

    pub fn revision_tree(&self, revision_id: &RevisionId) -> Result<Box<RevisionTree>, Error> {
        Python::with_gil(|py| {
            let tree = self.to_object(py).call_method1(
                py,
                "revision_tree",
                (revision_id.to_object(py),),
            )?;
            Ok(Box::new(RevisionTree(tree)))
        })
    }

    pub fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, Error> {
        Python::with_gil(|py| {
            let branch = self.to_object(py).getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, "get_tag_dict")?;
            tag_dict.extract(py)
        })
        .map_err(|e: PyErr| -> Error { e.into() })
    }

    pub fn abspath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "abspath", (path,))?
                .extract(py)?)
        })
    }

    pub fn relpath(&self, path: &Path) -> Result<PathBuf, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "relpath", (path,))?
                .extract(py)?)
        })
    }

    pub fn supports_setting_file_ids(&self) -> bool {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "supports_setting_file_ids")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    pub fn add(&self, paths: &[&Path]) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "add", (paths.to_vec(),))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    pub fn smart_add(&self, paths: &[&Path]) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "smart_add", (paths.to_vec(),))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }

    pub fn commit(
        &self,
        message: &str,
        allow_pointless: Option<bool>,
        committer: Option<&str>,
        specific_files: Option<&[&Path]>,
    ) -> Result<RevisionId, Error> {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new_bound(py);
            if let Some(committer) = committer {
                kwargs.set_item("committer", committer).unwrap();
            }
            if let Some(specific_files) = specific_files {
                kwargs.set_item("specific_files", specific_files).unwrap();
            }
            if let Some(allow_pointless) = allow_pointless {
                kwargs.set_item("allow_pointless", allow_pointless).unwrap();
            }

            let null_commit_reporter = py
                .import_bound("breezy.commit")
                .unwrap()
                .getattr("NullCommitReporter")
                .unwrap()
                .call0()
                .unwrap();
            kwargs.set_item("reporter", null_commit_reporter).unwrap();

            Ok(self
                .to_object(py)
                .call_method_bound(py, "commit", (message,), Some(&kwargs))
                .map_err(|e| -> Error { e.into() })?
                .extract(py)
                .unwrap())
        })
    }

    pub fn last_revision(&self) -> Result<RevisionId, Error> {
        Python::with_gil(|py| {
            let last_revision = self.to_object(py).call_method0(py, "last_revision")?;
            Ok(RevisionId::from(last_revision.extract::<Vec<u8>>(py)?))
        })
    }

    pub fn pull(
        &self,
        source: &dyn crate::branch::Branch,
        overwrite: Option<bool>,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
            let kwargs = {
                let kwargs = pyo3::types::PyDict::new_bound(py);
                if let Some(overwrite) = overwrite {
                    kwargs.set_item("overwrite", overwrite).unwrap();
                }
                kwargs
            };
            self.to_object(py)
                .call_method_bound(py, "pull", (source.to_object(py),), Some(&kwargs))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }
}

pub fn open(path: &Path) -> Result<WorkingTree, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.workingtree")?;
        let c = m.getattr("WorkingTree")?;
        let wt = c.call_method1("open", (path,))?;
        Ok(WorkingTree(wt.to_object(py)))
    })
}

pub fn open_containing(path: &Path) -> Result<(WorkingTree, PathBuf), Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.workingtree")?;
        let c = m.getattr("WorkingTree")?;
        let (wt, p): (Bound<PyAny>, String) =
            c.call_method1("open_containing", (path,))?.extract()?;
        Ok((WorkingTree(wt.to_object(py)), PathBuf::from(p)))
    })
}

impl From<PyObject> for WorkingTree {
    fn from(obj: PyObject) -> Self {
        WorkingTree(obj)
    }
}

impl Tree for WorkingTree {}

impl MutableTree for WorkingTree {}
