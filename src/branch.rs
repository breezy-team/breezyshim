//! Branches are the primary way to interact with the history of a project in Breezy.
//!
//! A branch is a named sequence of revisions. Each revision is a snapshot of the project at a
//! particular point in time. Revisions are linked together in a chain, forming a history of the
//! project. The branch itself is a pointer to the most recent revision in the chain.
//! Branches can be pushed to and pulled from other branches, allowing changes to be shared between
//! different branches.
//!
//! Breezy supports several different types of branches, each with different capabilities and
//! constraints.
use crate::controldir::{ControlDir, GenericControlDir};
use crate::error::Error;
use crate::foreign::VcsType;
use crate::lock::Lock;
use crate::repository::{Repository, GenericRepository, PyRepository};
use crate::revisionid::RevisionId;
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub struct BranchFormat(PyObject);

impl Clone for BranchFormat {
    fn clone(&self) -> Self {
        Python::with_gil(|py| BranchFormat(self.0.clone_ref(py)))
    }
}

impl BranchFormat {
    pub fn supports_stacking(&self) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "supports_stacking")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}

pub trait Branch: ToPyObject {
    fn format(&self) -> BranchFormat;
    fn vcs_type(&self) -> VcsType;
    fn revno(&self) -> u32;
    fn lock_read(&self) -> Result<Lock, crate::error::Error>;
    fn lock_write(&self) -> Result<Lock, crate::error::Error>;
    fn tags(&self) -> Result<crate::tags::Tags, crate::error::Error>;
    fn repository(&self) -> GenericRepository;
    fn last_revision(&self) -> RevisionId;
    fn name(&self) -> Option<String>;
    fn basis_tree(&self) -> Result<crate::tree::RevisionTree, crate::error::Error>;
    fn get_user_url(&self) -> url::Url;
    fn controldir(&self) -> Box<dyn ControlDir>;

    fn push(
        &self,
        remote_branch: &dyn Branch,
        overwrite: bool,
        stop_revision: Option<&RevisionId>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> Result<(), crate::error::Error>;

    fn pull(&self, source_branch: &dyn Branch, overwrite: Option<bool>) -> Result<(), Error>;
    fn get_parent(&self) -> Option<String>;
    fn set_parent(&mut self, parent: &str);
    fn get_public_branch(&self) -> Option<String>;
    fn get_push_location(&self) -> Option<String>;
    fn get_submit_branch(&self) -> Option<String>;
    fn user_transport(&self) -> crate::transport::Transport;
    fn get_config(&self) -> crate::config::BranchConfig;
    fn get_config_stack(&self) -> crate::config::ConfigStack;

    fn sprout(&self, to_controldir: &dyn ControlDir, to_branch_name: &str) -> Result<(), Error>;
    fn create_checkout(
        &self,
        to_location: &std::path::Path,
    ) -> Result<crate::tree::WorkingTree, Error>;
    fn generate_revision_history(&self, last_revision: &RevisionId) -> Result<(), Error>;
}

pub trait PyBranch: ToPyObject + Send + std::any::Any {}

impl<T: PyBranch> Branch for T {
    fn format(&self) -> BranchFormat {
        Python::with_gil(|py| BranchFormat(self.to_object(py).getattr(py, "_format").unwrap()))
    }

    fn vcs_type(&self) -> VcsType {
        self.repository().vcs_type()
    }

    fn revno(&self) -> u32 {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "revno")
                .unwrap()
                .extract(py)
                .unwrap()
        })
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

    fn tags(&self) -> Result<crate::tags::Tags, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(crate::tags::Tags::from(
                self.to_object(py).getattr(py, "tags")?,
            ))
        })
    }

    fn repository(&self) -> GenericRepository {
        Python::with_gil(|py| {
            GenericRepository::new(self.to_object(py).getattr(py, "repository").unwrap())
        })
    }

    fn last_revision(&self) -> RevisionId {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "last_revision")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn name(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .getattr(py, "name")
                .unwrap()
                .extract::<Option<String>>(py)
                .unwrap()
        })
    }

    fn basis_tree(&self) -> Result<crate::tree::RevisionTree, crate::error::Error> {
        Python::with_gil(|py| {
            Ok(crate::tree::RevisionTree(
                self.to_object(py).call_method0(py, "basis_tree")?,
            ))
        })
    }

    fn get_user_url(&self) -> url::Url {
        Python::with_gil(|py| {
            let url = self
                .to_object(py)
                .getattr(py, "user_url")
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            url.parse::<url::Url>().unwrap()
        })
    }

    fn controldir(&self) -> Box<dyn ControlDir> {
        Python::with_gil(|py| {
            Box::new(GenericControlDir::new(self.to_object(py).getattr(py, "controldir").unwrap())) as Box<dyn ControlDir>
        })
    }

    fn push(
        &self,
        remote_branch: &dyn Branch,
        overwrite: bool,
        stop_revision: Option<&RevisionId>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> Result<(), crate::error::Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new_bound(py);
            kwargs.set_item("overwrite", overwrite)?;
            if let Some(stop_revision) = stop_revision {
                kwargs.set_item("stop_revision", stop_revision)?;
            }
            if let Some(tag_selector) = tag_selector {
                kwargs.set_item("tag_selector", py_tag_selector(py, tag_selector)?)?;
            }
            self.to_object(py).call_method_bound(
                py,
                "push",
                (&remote_branch.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn pull(&self, source_branch: &dyn Branch, overwrite: Option<bool>) -> Result<(), Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new_bound(py);
            if let Some(overwrite) = overwrite {
                kwargs.set_item("overwrite", overwrite)?;
            }
            self.to_object(py).call_method_bound(
                py,
                "pull",
                (&source_branch.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn get_parent(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_parent")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn set_parent(&mut self, parent: &str) {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "set_parent", (parent,))
                .unwrap();
        })
    }

    fn get_public_branch(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_public_branch")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_push_location(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_push_location")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn get_submit_branch(&self) -> Option<String> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method0(py, "get_submit_branch")
                .unwrap()
                .extract(py)
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

    fn get_config(&self) -> crate::config::BranchConfig {
        Python::with_gil(|py| {
            crate::config::BranchConfig::new(
                self.to_object(py).call_method0(py, "get_config").unwrap(),
            )
        })
    }

    fn get_config_stack(&self) -> crate::config::ConfigStack {
        Python::with_gil(|py| {
            crate::config::ConfigStack::new(
                self.to_object(py)
                    .call_method0(py, "get_config_stack")
                    .unwrap(),
            )
        })
    }

    fn sprout(&self, to_controldir: &dyn ControlDir, to_branch_name: &str) -> Result<(), Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new_bound(py);
            kwargs.set_item("name", to_branch_name)?;
            self.to_object(py).call_method_bound(
                py,
                "sprout",
                (to_controldir.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(())
        })
    }

    fn create_checkout(
        &self,
        to_location: &std::path::Path,
    ) -> Result<crate::tree::WorkingTree, Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(
                    py,
                    "create_checkout",
                    (to_location.to_string_lossy().to_string(),),
                )
                .map(crate::tree::WorkingTree)
                .map_err(|e| e.into())
        })
    }

    fn generate_revision_history(&self, last_revision: &RevisionId) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py).call_method1(
                py,
                "generate_revision_history",
                (last_revision.to_object(py),),
            )?;
            Ok(())
        })
    }
}

pub struct GenericBranch(PyObject);

impl Clone for GenericBranch {
    fn clone(&self) -> Self {
        Python::with_gil(|py| GenericBranch(self.0.clone_ref(py)))
    }
}

impl PyBranch for GenericBranch {}

impl ToPyObject for GenericBranch {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl GenericBranch {
    pub fn new(obj: PyObject) -> Self {
        GenericBranch(obj)
    }
}

impl FromPyObject<'_> for GenericBranch {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(GenericBranch(ob.to_object(ob.py())))
    }
}

pub struct MemoryBranch(PyObject);

impl Clone for MemoryBranch {
    fn clone(&self) -> Self {
        Python::with_gil(|py| MemoryBranch(self.0.clone_ref(py)))
    }
}

impl ToPyObject for MemoryBranch {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl PyBranch for MemoryBranch {}

impl MemoryBranch {
    pub fn new<R: PyRepository>(repository: &R, revno: Option<u32>, revid: &RevisionId) -> Self {
        Python::with_gil(|py| {
            let mb_cls = py
                .import_bound("breezy.memorybranch")
                .unwrap()
                .getattr("MemoryBranch")
                .unwrap();

            let o = mb_cls
                .call1((repository.to_object(py), (revno, revid.clone())))
                .unwrap();

            MemoryBranch(o.to_object(py))
        })
    }
}

pub(crate) fn py_tag_selector(
    py: Python,
    tag_selector: Box<dyn Fn(String) -> bool>,
) -> PyResult<PyObject> {
    #[pyclass(unsendable)]
    struct PyTagSelector(Box<dyn Fn(String) -> bool>);

    #[pymethods]
    impl PyTagSelector {
        fn __call__(&self, tag: String) -> bool {
            (self.0)(tag)
        }
    }
    Ok(PyTagSelector(tag_selector).into_py(py))
}

pub fn open(url: &url::Url) -> Result<Box<dyn Branch>, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();
        let r = c.call_method1("open", (url.to_string(),))?;
        Ok(Box::new(GenericBranch(r.to_object(py))) as Box<dyn Branch>)
    })
}

pub fn open_containing(url: &url::Url) -> Result<(Box<dyn Branch>, String), Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();

        let (b, p): (Bound<PyAny>, String) = c
            .call_method1("open_containing", (url.to_string(),))?
            .extract()?;

        Ok((
            Box::new(GenericBranch(b.to_object(py))) as Box<dyn Branch>,
            p,
        ))
    })
}

pub fn open_from_transport(
    transport: &crate::transport::Transport,
) -> Result<Box<dyn Branch>, Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.branch").unwrap();
        let c = m.getattr("Branch").unwrap();
        let r = c.call_method1("open_from_transport", (transport.to_object(py),))?;
        Ok(Box::new(GenericBranch(r.to_object(py))) as Box<dyn Branch>)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_clone() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_directory_path(td.path()).unwrap();
        let branch = crate::controldir::create_branch_convenience(
            &url,
            None,
            &crate::controldir::ControlDirFormat::default(),
        )
        .unwrap();

        assert_eq!(branch.revno(), 0);
        assert_eq!(branch.last_revision(), RevisionId::null());
    }

    #[test]
    fn test_create_and_clone_memory() {
        let td = tempfile::tempdir().unwrap();
        let url = url::Url::from_directory_path(td.path()).unwrap();
        let branch = crate::controldir::create_branch_convenience(
            &url,
            None,
            &crate::controldir::ControlDirFormat::default(),
        )
        .unwrap();
        let branch = MemoryBranch::new(&branch.repository(), None, &RevisionId::null());

        assert_eq!(branch.last_revision(), RevisionId::null());
    }
}
