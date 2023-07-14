use crate::lock::Lock;
use crate::revisionid::RevisionId;
use pyo3::prelude::*;

pub trait Tree {
    fn obj(&self) -> &PyObject;

    fn get_tag_dict(&self) -> Result<std::collections::HashMap<String, RevisionId>, PyErr> {
        Python::with_gil(|py| {
            let branch = self.obj().getattr(py, "branch")?;
            let tags = branch.getattr(py, "tags")?;
            let tag_dict = tags.call_method0(py, "get_tag_dict")?;
            tag_dict.extract(py)
        })
    }

    fn get_file(&self, path: &std::path::Path) -> PyResult<Box<dyn std::io::Read>> {
        Python::with_gil(|py| {
            let f = self.obj().call_method1(py, "get_file", (path,))?;

            let f = pyo3_file::PyFileLikeObject::with_requirements(f, true, false, false)?;

            Ok(Box::new(f) as Box<dyn std::io::Read>)
        })
    }

    fn lock_read(&self) -> PyResult<Lock> {
        Python::with_gil(|py| {
            let lock = self.obj().call_method0(py, "lock_read").unwrap();
            Ok(Lock(lock))
        })
    }

    fn has_filename(&self, path: &std::path::Path) -> bool {
        Python::with_gil(|py| {
            self.obj()
                .call_method1(py, "has_filename", (path,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_parent_ids(&self) -> PyResult<Vec<RevisionId>> {
        Python::with_gil(|py| {
            self.obj()
                .call_method0(py, "get_parent_ids")
                .unwrap()
                .extract(py)
        })
    }

    fn is_ignored(&self, path: &std::path::Path) -> bool {
        Python::with_gil(|py| {
            self.obj()
                .call_method1(py, "is_ignored", (path,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn is_versioned(&self, path: &std::path::Path) -> bool {
        Python::with_gil(|py| {
            self.obj()
                .call_method1(py, "is_versioned", (path,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn iter_changes(
        &self,
        other: &Box<dyn Tree>,
        specific_files: Option<&[&std::path::Path]>,
        want_unversioned: Option<bool>,
        require_versioned: Option<bool>,
    ) -> PyResult<Box<dyn Iterator<Item = PyResult<TreeChange>>>> {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
            if let Some(specific_files) = specific_files {
                kwargs.set_item("specific_files", specific_files)?;
            }
            if let Some(want_unversioned) = want_unversioned {
                kwargs.set_item("want_unversioned", want_unversioned)?;
            }
            if let Some(require_versioned) = require_versioned {
                kwargs.set_item("require_versioned", require_versioned)?;
            }
            struct TreeChangeIter(pyo3::PyObject);

            impl Iterator for TreeChangeIter {
                type Item = PyResult<TreeChange>;

                fn next(&mut self) -> Option<Self::Item> {
                    Python::with_gil(|py| {
                        let next = self.0.call_method0(py, "__next__").unwrap();
                        if next.is_none(py) {
                            None
                        } else {
                            Some(next.extract(py))
                        }
                    })
                }
            }

            Ok(Box::new(TreeChangeIter(self.obj().call_method(
                py,
                "iter_changes",
                (other.obj(),),
                Some(kwargs),
            )?))
                as Box<dyn Iterator<Item = PyResult<TreeChange>>>)
        })
    }

    fn has_versioned_directories(&self) -> bool {
        Python::with_gil(|py| {
            self.obj()
                .call_method0(py, "has_versioned_directories")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}

pub struct RevisionTree(pub PyObject);

impl Tree for RevisionTree {
    fn obj(&self) -> &PyObject {
        &self.0
    }
}

pub struct WorkingTree(pub PyObject);

impl WorkingTree {
    pub fn basis_tree(&self) -> Box<dyn Tree> {
        Python::with_gil(|py| {
            let tree = self.0.call_method0(py, "basis_tree").unwrap();
            Box::new(RevisionTree(tree))
        })
    }

    pub fn abspath(&self, path: &std::path::Path) -> PyResult<std::path::PathBuf> {
        Python::with_gil(|py| Ok(self.0.call_method1(py, "abspath", (path,))?.extract(py)?))
    }

    pub fn supports_setting_file_ids(&self) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "supports_setting_file_ids")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    pub fn add(&self, paths: &[&std::path::Path]) -> PyResult<()> {
        Python::with_gil(|py| {
            self.0.call_method1(py, "add", (paths.to_vec(),)).unwrap();
        });
        Ok(())
    }

    pub fn smart_add(&self, paths: &[&std::path::Path]) -> PyResult<()> {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "smart_add", (paths.to_vec(),))
                .unwrap();
        });
        Ok(())
    }

    pub fn commit(
        &self,
        message: &str,
        allow_pointless: Option<bool>,
        committer: Option<&str>,
        specific_files: Option<&[&std::path::Path]>,
    ) -> PyResult<RevisionId> {
        Python::with_gil(|py| {
            let kwargs = pyo3::types::PyDict::new(py);
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
                .import("breezy.commit")?
                .getattr("NullCommitReporter")?
                .call0()?;
            kwargs.set_item("reporter", null_commit_reporter).unwrap();

            self.0
                .call_method(py, "commit", (message,), Some(kwargs))
                .unwrap()
                .extract(py)
        })
    }

    pub fn last_revision(&self) -> Result<RevisionId, PyErr> {
        Python::with_gil(|py| {
            let last_revision = self.0.call_method0(py, "last_revision")?;
            Ok(RevisionId::from(last_revision.extract::<Vec<u8>>(py)?))
        })
    }
}

impl Tree for WorkingTree {
    fn obj(&self) -> &PyObject {
        &self.0
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TreeChange {
    pub path: (Option<std::path::PathBuf>, Option<std::path::PathBuf>),
    pub changed_content: bool,
    pub versioned: (bool, bool),
    pub name: (Option<std::ffi::OsString>, Option<std::ffi::OsString>),
    pub kind: (Option<String>, Option<String>),
    pub executable: (bool, bool),
    pub copied: (bool, bool),
}

impl ToPyObject for TreeChange {
    fn to_object(&self, py: Python) -> PyObject {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("path", &self.path).unwrap();
        dict.set_item("changed_content", self.changed_content)
            .unwrap();
        dict.set_item("versioned", self.versioned).unwrap();
        dict.set_item("name", &self.name).unwrap();
        dict.set_item("kind", &self.kind).unwrap();
        dict.set_item("executable", self.executable).unwrap();
        dict.set_item("copied", self.copied).unwrap();
        dict.into()
    }
}

impl FromPyObject<'_> for TreeChange {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        let path = obj.get_item("path")?;
        let changed_content = obj.get_item("changed_content")?;
        let versioned = obj.get_item("versioned")?;
        let name = obj.get_item("name")?;
        let kind = obj.get_item("kind")?;
        let executable = obj.get_item("executable")?;
        let copied = obj.get_item("copied")?;

        Ok(TreeChange {
            path: path.extract()?,
            changed_content: changed_content.extract()?,
            versioned: versioned.extract()?,
            name: name.extract()?,
            kind: kind.extract()?,
            executable: executable.extract()?,
            copied: copied.extract()?,
        })
    }
}