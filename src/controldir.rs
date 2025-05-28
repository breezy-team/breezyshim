//! The `ControlDir` class provides a high-level interface to control directories,
//! e.g. ".bzr" or ".git" directories.
use crate::branch::{py_tag_selector, Branch, GenericBranch, PyBranch};
use crate::error::Error;
use crate::repository::GenericRepository;
use crate::transport::Transport;
use crate::tree::WorkingTree;

use crate::location::AsLocation;

use pyo3::prelude::*;
use pyo3::types::PyDict;

pub trait PyProber: ToPyObject + std::any::Any + std::fmt::Debug {}

pub trait Prober: std::fmt::Debug {
    fn probe_transport(&self, transport: &Transport) -> Result<bool, Error>;
    fn probe(&self, url: &url::Url) -> Result<bool, Error>;
}

impl<T: PyProber> Prober for T {
    fn probe_transport(&self, transport: &Transport) -> Result<bool, Error> {
        Python::with_gil(|py| {
            let result = self.to_object(py).call_method1(
                py,
                "probe_transport",
                (&transport.to_object(py),),
            )?;
            Ok(result.extract(py)?)
        })
    }

    fn probe(&self, url: &url::Url) -> Result<bool, Error> {
        Python::with_gil(|py| {
            let result = self
                .to_object(py)
                .call_method1(py, "probe", (url.to_string(),))?;
            Ok(result.extract(py)?)
        })
    }
}

pub trait PyControlDir: ToPyObject + std::any::Any + std::fmt::Debug {}

pub trait ControlDir: std::fmt::Debug {
    fn get_user_url(&self) -> url::Url;
    fn get_format(&self) -> ControlDirFormat;
    fn user_transport(&self) -> Transport;
    fn control_transport(&self) -> Transport;
    fn open_repository(&self) -> Result<GenericRepository, Error>;
    fn find_repository(&self) -> Result<GenericRepository, Error>;
    fn cloning_metadir(&self) -> ControlDirFormat;
    fn create_branch(&self, name: Option<&str>) -> Result<Box<dyn Branch>, Error>;
    fn create_repository(&self, shared: Option<bool>) -> Result<GenericRepository, Error>;
    fn open_branch(&self, branch_name: Option<&str>) -> Result<Box<dyn Branch>, Error>;
    fn create_workingtree(&self) -> crate::Result<WorkingTree>;
    fn set_branch_reference(&self, branch: &dyn PyBranch, name: Option<&str>) -> crate::Result<()>;
    fn push_branch(
        &self,
        source_branch: &dyn PyBranch,
        to_branch_name: Option<&str>,
        stop_revision: Option<&crate::RevisionId>,
        overwrite: Option<bool>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> crate::Result<Box<dyn Branch>>;
    fn sprout(
        &self,
        target: url::Url,
        source_branch: Option<&dyn PyBranch>,
        create_tree_if_local: Option<bool>,
        stacked: Option<bool>,
        revision_id: Option<&crate::RevisionId>,
    ) -> Result<Box<dyn ControlDir>, Error>;
    fn has_workingtree(&self) -> bool;
    fn open_workingtree(&self) -> crate::Result<WorkingTree>;
    fn branch_names(&self) -> crate::Result<Vec<String>>;
}

crate::wrapped_py!(GenericControlDir);

impl PyControlDir for GenericControlDir {}

impl GenericControlDir {
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }
}

impl<T: PyControlDir + ?Sized> ControlDir for T {
    fn get_user_url(&self) -> url::Url {
        Python::with_gil(|py| {
            let result = self.into_pyobject(py).unwrap().getattr( "user_url").unwrap();
            url::Url::parse(&result.extract::<String>().unwrap()).unwrap()
        })
    }

    fn get_format(&self) -> ControlDirFormat {
        Python::with_gil(|py| {
            let result = self.into_pyobject(py).unwrap().getattr("_format")?;
            Ok::<_, PyErr>(ControlDirFormat(result.unbind()))
        })
        .unwrap()
    }

    fn user_transport(&self) -> Transport {
        Python::with_gil(|py| {
            let result = self.into_pyobject(py).unwrap().getattr("user_transport").unwrap();
            crate::transport::Transport::new(result.unbind())
        })
    }

    fn control_transport(&self) -> Transport {
        Python::with_gil(|py| {
            let result = self.into_pyobject(py).unwrap().getattr("control_transport").unwrap();
            crate::transport::Transport::new(result.unbind())
        })
    }

    fn open_repository(&self) -> Result<GenericRepository, Error> {
        Python::with_gil(|py| {
            let result = self.into_pyobject(py).unwrap().call_method0("open_repository")?;
            Ok(GenericRepository::new(result))
        })
    }

    fn find_repository(&self) -> Result<GenericRepository, Error> {
        Python::with_gil(|py| {
            let result = self.into_pyobject(py).unwrap().call_method0("find_repository")?;
            Ok(GenericRepository::new(result))
        })
    }

    fn cloning_metadir(&self) -> ControlDirFormat {
        Python::with_gil(|py| {
            let result = self.into_pyobject(py).unwrap().call_method0("cloning_metadir")?;
            Ok::<_, PyErr>(ControlDirFormat(result.unbind()))
        })
        .unwrap()
    }

    fn create_branch(&self, name: Option<&str>) -> Result<Box<dyn Branch>, Error> {
        Python::with_gil(|py| {
            let branch = self
                .into_pyobject(py).unwrap()
                .call_method("create_branch", (name,), None)?
                .extract()?;
            Ok(Box::new(GenericBranch::from(branch)) as Box<dyn Branch>)
        })
    }

    fn create_repository(&self, shared: Option<bool>) -> Result<GenericRepository, Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            if let Some(shared) = shared {
                kwargs.set_item("shared", shared)?;
            }
            let repository = self
                .into_pyobject(py).unwrap()
                .call_method("create_repository", (), Some(&kwargs))?
                .extract()?;
            Ok(GenericRepository::new(repository))
        })
    }

    fn open_branch(&self, branch_name: Option<&str>) -> Result<Box<dyn Branch>, Error> {
        Python::with_gil(|py| {
            let branch: Bound<PyAny> = self
                .into_pyobject(py).unwrap()
                .call_method("open_branch", (branch_name,), None)?
                .extract()?;
            Ok(Box::new(GenericBranch::from(branch.unbind())) as Box<dyn Branch>)
        })
    }

    fn create_workingtree(&self) -> crate::Result<WorkingTree> {
        Python::with_gil(|py| {
            let wt = self
                .into_pyobject(py).unwrap()
                .call_method0("create_workingtree")?
                .extract()?;
            Ok(WorkingTree::from(wt))
        })
    }

    fn set_branch_reference(&self, branch: &dyn PyBranch, name: Option<&str>) -> crate::Result<()> {
        Python::with_gil(|py| {
            self.into_pyobject(py).unwrap().call_method1(
                "set_branch_reference",
                (&branch, name),
            )?;
            Ok(())
        })
    }

    fn push_branch(
        &self,
        source_branch: &dyn PyBranch,
        to_branch_name: Option<&str>,
        stop_revision: Option<&crate::RevisionId>,
        overwrite: Option<bool>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> crate::Result<Box<dyn Branch>> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            if let Some(to_branch_name) = to_branch_name {
                kwargs.set_item("name", to_branch_name)?;
            }
            if let Some(tag_selector) = tag_selector {
                kwargs.set_item("tag_selector", py_tag_selector(py, tag_selector)?)?;
            }
            if let Some(overwrite) = overwrite {
                kwargs.set_item("overwrite", overwrite)?;
            }
            if let Some(stop_revision) = stop_revision {
                kwargs.set_item("stop_revision", &stop_revision)?;
            }
            let result = self.into_pyobject(py).unwrap().call_method(
                "push_branch",
                (&source_branch,),
                Some(&kwargs),
            )?;
            Ok(
                Box::new(GenericBranch::new(result.getattr("target_branch")?.unbind()))
                    as Box<dyn Branch>,
            )
        })
    }

    fn sprout(
        &self,
        target: url::Url,
        source_branch: Option<&dyn PyBranch>,
        create_tree_if_local: Option<bool>,
        stacked: Option<bool>,
        revision_id: Option<&crate::RevisionId>,
    ) -> Result<Box<dyn ControlDir>, Error> {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            if let Some(create_tree_if_local) = create_tree_if_local {
                kwargs
                    .set_item("create_tree_if_local", create_tree_if_local)
                    .unwrap();
            }
            if let Some(stacked) = stacked {
                kwargs.set_item("stacked", stacked).unwrap();
            }
            if let Some(source_branch) = source_branch {
                kwargs
                    .set_item("source_branch", source_branch)
                    .unwrap();
            }
            if let Some(revision_id) = revision_id {
                kwargs
                    .set_item("revision_id", revision_id)
                    .unwrap();
            }

            let cd =
                self.0
                    .call_method(py, "sprout", (target.to_string(),), Some(&kwargs))?;

            Ok(Box::new(GenericControlDir(cd)) as Box<dyn ControlDir>)
        })
    }

    fn has_workingtree(&self) -> bool {
        Python::with_gil(|py| {
            let result = self
                .to_object(py)
                .call_method0(py, "has_workingtree")
                .unwrap();
            result.extract(py).unwrap()
        })
    }

    fn open_workingtree(&self) -> crate::Result<WorkingTree> {
        Python::with_gil(|py| {
            let wt = self
                .to_object(py)
                .call_method0(py, "open_workingtree")?
                .extract(py)?;
            Ok(WorkingTree::from(wt))
        })
    }

    fn branch_names(&self) -> crate::Result<Vec<String>> {
        Python::with_gil(|py| {
            let names = self
                .to_object(py)
                .call_method0(py, "branch_names")?
                .extract::<Vec<String>>(py)?;
            Ok(names)
        })
    }
}

impl std::fmt::Debug for GenericControlDir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("ControlDir({:?})", self.0))
    }
}

crate::wrapped_py!(ControlDirFormat);

impl Clone for ControlDirFormat {
    fn clone(&self) -> Self {
        Python::with_gil(|py| ControlDirFormat(self.0.clone_ref(py)))
    }
}

impl Default for ControlDirFormat {
    fn default() -> Self {
        Python::with_gil(|py| {
            let breezy = PyModule::import(py, "breezy.controldir").unwrap();
            let cd_format = breezy.getattr("ControlDirFormat").unwrap();
            let obj = cd_format.call_method0("get_default_format").unwrap();
            assert!(!obj.is_none());
            ControlDirFormat(obj.into())
        })
    }
}

impl ControlDirFormat {
    pub fn get_format_string(&self) -> Vec<u8> {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "get_format_string")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    pub fn get_format_description(&self) -> String {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "get_format_description")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    pub fn is_control_filename(&self, filename: &str) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "is_control_filename", (filename,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    pub fn initialize_on_transport(
        &self,
        transport: &Transport,
    ) -> Result<Box<dyn ControlDir>, Error> {
        Python::with_gil(|py| {
            let cd =
                self.0
                    .call_method1(py, "initialize_on_transport", (&transport,))?;
            Ok(Box::new(GenericControlDir(cd)) as Box<dyn ControlDir>)
        })
    }

    pub fn initialize(&self, location: impl AsLocation) -> Result<Box<dyn ControlDir>, Error> {
        Python::with_gil(|py| {
            let cd = self
                .0
                .call_method1(py, "initialize", (location.as_location(),))?;
            Ok(Box::new(GenericControlDir(cd)) as Box<dyn ControlDir>)
        })
    }
}

pub fn open_tree_or_branch(
    location: impl AsLocation,
    name: Option<&str>,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<(Option<WorkingTree>, Box<dyn Branch>), Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;

        let kwargs = PyDict::new(py);
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item("possible_transports", possible_transports)?;
        }

        let ret = cd.to_object(py).call_method(
            py,
            "open_tree_or_branch",
            (location.as_location(), name),
            Some(&kwargs),
        )?;

        let (tree, branch) = ret.extract::<(Option<PyObject>, PyObject)>(py)?;
        let branch = Box::new(GenericBranch::new(branch)) as Box<dyn Branch>;
        let tree = tree.map(WorkingTree::from);
        Ok((tree, branch))
    })
}

pub fn open(
    url: impl AsLocation,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<Box<dyn ControlDir>, Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item("possible_transports", possible_transports)?;
        }
        let controldir = cd.call_method("open", (url.as_location(),), Some(&kwargs))?;
        Ok(Box::new(GenericControlDir(controldir.to_object(py))) as Box<dyn ControlDir>)
    })
}

pub fn create(
    url: impl AsLocation,
    format: impl AsFormat,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<Box<dyn ControlDir>, Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(format) = format.as_format() {
            kwargs.set_item("format", format)?;
        }
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item("possible_transports", possible_transports)?;
        }
        let controldir = cd.call_method("create", (url.as_location(),), Some(&kwargs))?;
        Ok(Box::new(GenericControlDir(controldir.to_object(py))) as Box<dyn ControlDir>)
    })
}

pub fn create_on_transport(
    transport: &Transport,
    format: impl AsFormat,
) -> Result<Box<dyn ControlDir>, Error> {
    Python::with_gil(|py| {
        let format = format.as_format().unwrap().0;
        Ok(Box::new(GenericControlDir(format.call_method_bound(
            py,
            "initialize_on_transport",
            (&transport,),
            None,
        )?)) as Box<dyn ControlDir>)
    })
}

pub fn open_containing_from_transport(
    transport: &Transport,
    probers: Option<&[&dyn PyProber]>,
) -> Result<(Box<dyn ControlDir>, String), Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(probers) = probers {
            kwargs.set_item(
                "probers",
                probers
            )?;
        }

        let (controldir, subpath): (PyObject, String) = cd
            .call_method(
                "open_containing_from_transport",
                (&transport,),
                Some(&kwargs),
            )?
            .extract()?;
        Ok((
            Box::new(GenericControlDir(controldir.to_object(py))) as Box<dyn ControlDir>,
            subpath,
        ))
    })
}

pub fn open_from_transport(
    transport: &Transport,
    probers: Option<&[&dyn PyProber]>,
) -> Result<Box<dyn ControlDir>, Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(probers) = probers {
            kwargs.set_item(
                "probers",
                probers
            )?;
        }
        let controldir = cd.call_method(
            "open_from_transport",
            (&transport,),
            Some(&kwargs),
        )?;
        Ok(Box::new(GenericControlDir(controldir.to_object(py))) as Box<dyn ControlDir>)
    })
}

pub trait AsFormat {
    fn as_format(&self) -> Option<ControlDirFormat>;
}

impl AsFormat for &str {
    fn as_format(&self) -> Option<ControlDirFormat> {
        Python::with_gil(|py| {
            let m = py.import("breezy.controldir").ok()?;
            let cd = m.getattr("format_registry").ok()?;
            let format = cd
                .call_method1("make_controldir", (self.to_string(),))
                .ok()?;
            Some(ControlDirFormat::from(format))
        })
    }
}

impl AsFormat for &ControlDirFormat {
    fn as_format(&self) -> Option<ControlDirFormat> {
        Some(Python::with_gil(|py| {
            ControlDirFormat(self.0.clone_ref(py))
        }))
    }
}

pub fn create_branch_convenience(
    base: &url::Url,
    force_new_tree: Option<bool>,
    format: impl AsFormat,
) -> Result<Box<dyn Branch>, Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let format = format.as_format();
        let kwargs = PyDict::new(py);
        if let Some(force_new_tree) = force_new_tree {
            kwargs.set_item("force_new_tree", force_new_tree)?;
        }
        if let Some(format) = format {
            kwargs.set_item("format", format)?;
        }
        let branch = cd.call_method(
            "create_branch_convenience",
            (base.to_string(),),
            Some(&kwargs),
        )?;
        Ok(Box::new(GenericBranch::from(branch)) as Box<dyn Branch>)
    })
}

/// Create a standalone working tree.
///
/// # Arguments
/// * `base` - The base directory for the working tree.
/// * `format` - The format of the working tree.
pub fn create_standalone_workingtree(
    base: &std::path::Path,
    format: impl AsFormat,
) -> Result<WorkingTree, Error> {
    let base = base.to_str().unwrap();
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let format = format.as_format();
        let wt = cd.call_method(
            "create_standalone_workingtree",
            (base, format.unwrap_or_default()),
            None,
        )?;
        Ok(WorkingTree::from(wt))
    })
}

crate::wrapped_py!(GenericProber);

impl PyProber for GenericProber {}

impl std::fmt::Debug for GenericProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("Prober({:?})", self.0))
    }
}

pub fn all_probers() -> Vec<Box<dyn Prober>> {
    Python::with_gil(|py| -> PyResult<Vec<Box<dyn Prober>>> {
        let m = py.import("breezy.controldir")?;
        let cdf = m.getattr("ControlDirFormat")?;
        let probers = cdf
            .call_method0("all_probers")?
            .extract::<Vec<PyObject>>()?;
        Ok(probers
            .into_iter()
            .map(|p| Box::new(GenericProber::new(p)) as Box<dyn Prober>)
            .collect::<Vec<_>>())
    })
    .unwrap()
}

pub struct ControlDirFormatRegistry(PyObject);

impl ControlDirFormatRegistry {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py.import("breezy.controldir").unwrap();
            let obj = m.getattr("format_registry").unwrap();
            ControlDirFormatRegistry(obj.into())
        })
    }

    pub fn make_controldir(&self, format: &str) -> Option<ControlDirFormat> {
        Python::with_gil(
            |py| match self.0.call_method1(py, "make_controldir", (format,)) {
                Ok(format) => Some(ControlDirFormat(format.to_object(py))),
                Err(e) if e.is_instance_of::<pyo3::exceptions::PyKeyError>(py) => None,
                Err(e) => panic!("{}", e),
            },
        )
    }
}

impl Default for ControlDirFormatRegistry {
    fn default() -> Self {
        ControlDirFormatRegistry::new()
    }
}

lazy_static::lazy_static! {
    pub static ref FORMAT_REGISTRY: ControlDirFormatRegistry = ControlDirFormatRegistry::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_dir_format_registry() {
        let registry = ControlDirFormatRegistry::new();
        let format = registry.make_controldir("2a").unwrap();
        let _ = format.get_format_string();
    }

    #[test]
    fn test_format_registry() {
        let format = FORMAT_REGISTRY.make_controldir("2a").unwrap();
        let _ = format.get_format_string();
    }

    #[test]
    fn test_all_probers() {
        let probers = all_probers();
        assert!(!probers.is_empty());
    }

    #[test]
    fn test_open_tree_or_branch() {
        let tmp_dir = tempfile::tempdir().unwrap();
        create_branch_convenience(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
            &ControlDirFormat::default(),
        )
        .unwrap();
        let (wt, branch) = open_tree_or_branch(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            wt.unwrap().basedir().canonicalize().unwrap(),
            tmp_dir.path().canonicalize().unwrap()
        );
        assert_eq!(
            branch.get_user_url(),
            url::Url::from_directory_path(tmp_dir.path()).unwrap()
        );
    }

    #[test]
    fn test_control_dir_format_default() {
        let d = ControlDirFormat::default();
        d.get_format_string();
    }

    #[test]
    fn test_open() {
        let tmp_dir = tempfile::tempdir().unwrap();

        let e = open(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
        )
        .unwrap_err();

        assert!(matches!(e, Error::NotBranchError(..)),);

        let cd = create(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            "2a",
            None,
        )
        .unwrap();

        let od = open(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(
            cd.get_format().get_format_string(),
            od.get_format().get_format_string()
        );
    }

    #[test]
    fn test_create() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let cd = create(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            "2a",
            None,
        )
        .unwrap();

        let od = open(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(
            cd.get_format().get_format_string(),
            od.get_format().get_format_string()
        );
    }

    #[test]
    fn test_create_on_transport() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let transport = crate::transport::get_transport(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
        )
        .unwrap();
        let _cd = create_on_transport(&transport, "2a").unwrap();
    }

    #[test]
    fn test_open_containing_from_transport() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let transport = crate::transport::get_transport(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
        )
        .unwrap();
        let e = open_containing_from_transport(&transport, None).unwrap_err();
        assert!(matches!(e, Error::NotBranchError(..)),);
    }

    #[test]
    fn test_open_from_transport() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let transport = crate::transport::get_transport(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
        )
        .unwrap();
        let e = open_from_transport(&transport, None).unwrap_err();
        assert!(matches!(e, Error::NotBranchError(..)),);
    }

    #[test]
    fn test_create_standalone_workingtree() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();

        assert_eq!(
            wt.basedir().canonicalize().unwrap(),
            tmp_dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn test_create_branch_convenience() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let branch = create_branch_convenience(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            None,
            &ControlDirFormat::default(),
        )
        .unwrap();

        assert_eq!(
            branch.get_user_url(),
            url::Url::from_directory_path(tmp_dir.path()).unwrap()
        );
    }

    #[test]
    fn test_create_repository() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let controldir = create(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            &ControlDirFormat::default(),
            None,
        )
        .unwrap();
        let _repo = controldir.create_repository(None).unwrap();
    }

    #[test]
    fn test_create_branch() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let controldir = create(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            &ControlDirFormat::default(),
            None,
        )
        .unwrap();
        assert!(matches!(
            controldir.create_branch(None),
            Err(Error::NoRepositoryPresent)
        ));
        let _repo = controldir.create_repository(None).unwrap();
        let _branch = controldir.create_branch(Some("foo")).unwrap();
    }

    #[test]
    fn test_create_workingtree() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let controldir = create(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            &ControlDirFormat::default(),
            None,
        )
        .unwrap();
        controldir.create_repository(None).unwrap();
        controldir.create_branch(None).unwrap();
        let _wt = controldir.create_workingtree().unwrap();
    }

    #[test]
    fn test_branch_names() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let controldir = create(
            &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
            &ControlDirFormat::default(),
            None,
        )
        .unwrap();
        controldir.create_repository(None).unwrap();
        controldir.create_branch(None).unwrap();
        assert_eq!(controldir.branch_names().unwrap(), vec!["".to_string()]);
    }
}
