use crate::branch::{py_tag_selector, Branch, BranchOpenError, RegularBranch};
use crate::transport::Transport;
use crate::tree::WorkingTree;

use crate::location::AsLocation;

use pyo3::prelude::*;
use pyo3::types::PyDict;

pub struct Prober(PyObject);

impl ToPyObject for Prober {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl FromPyObject<'_> for Prober {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        Ok(Prober(obj.to_object(obj.py())))
    }
}

impl Prober {
    pub fn new(obj: PyObject) -> Self {
        Prober(obj)
    }
}

pub struct ControlDir(PyObject);

impl ToPyObject for ControlDir {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl FromPyObject<'_> for ControlDir {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        Ok(ControlDir(obj.to_object(obj.py())))
    }
}

impl ControlDir {
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }

    pub fn get_format(&self) -> ControlDirFormat {
        Python::with_gil(|py| {
            let result = self.to_object(py).getattr(py, "_format")?;
            Ok::<_, PyErr>(ControlDirFormat(result))
        })
        .unwrap()
    }

    #[deprecated]
    pub fn create_branch_convenience(base: &url::Url) -> Result<Box<dyn Branch>, CreateError> {
        create_branch_convenience(base)
    }

    #[deprecated]
    pub fn create_standalone_workingtree(
        base: &std::path::Path,
        format: impl AsFormat,
    ) -> Result<WorkingTree, CreateError> {
        create_standalone_workingtree(base, format)
    }

    pub fn cloning_metadir(&self) -> ControlDirFormat {
        Python::with_gil(|py| {
            let result = self.to_object(py).call_method0(py, "cloning_metadir")?;
            Ok::<_, PyErr>(ControlDirFormat(result))
        })
        .unwrap()
    }

    #[deprecated]
    pub fn open_tree_or_branch(
        location: &url::Url,
        name: Option<&str>,
    ) -> Result<(Option<WorkingTree>, Box<dyn Branch>), BranchOpenError> {
        open_tree_or_branch(location, name, None)
    }

    #[deprecated]
    pub fn open(url: &url::Url) -> Result<ControlDir, OpenError> {
        open(url, None)
    }

    #[deprecated]
    pub fn open_containing_from_transport(
        transport: &Transport,
        probers: Option<&[Prober]>,
    ) -> Result<(ControlDir, String), OpenError> {
        open_containing_from_transport(transport, probers)
    }

    #[deprecated]
    pub fn open_from_transport(
        transport: &Transport,
        probers: Option<&[Prober]>,
    ) -> Result<ControlDir, OpenError> {
        open_from_transport(transport, probers)
    }

    pub fn create_branch(&self, name: Option<&str>) -> Result<Box<dyn Branch>, CreateError> {
        Python::with_gil(|py| {
            let branch = self
                .to_object(py)
                .call_method(py, "create_branch", (name,), None)?
                .extract(py)?;
            Ok(Box::new(RegularBranch::new(branch)) as Box<dyn Branch>)
        })
    }

    pub fn open_branch(
        &self,
        branch_name: Option<&str>,
    ) -> Result<Box<dyn Branch>, BranchOpenError> {
        Python::with_gil(|py| {
            let branch = self
                .to_object(py)
                .call_method(py, "open_branch", (branch_name,), None)?
                .extract(py)?;
            Ok(Box::new(RegularBranch::new(branch)) as Box<dyn Branch>)
        })
    }

    pub fn push_branch(
        &self,
        source_branch: &dyn Branch,
        to_branch_name: Option<&str>,
        stop_revision: Option<&crate::RevisionId>,
        overwrite: Option<bool>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> PyResult<Box<dyn Branch>> {
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
                kwargs.set_item("stop_revision", stop_revision.to_object(py))?;
            }
            let result = self.to_object(py).call_method(
                py,
                "push_branch",
                (&source_branch.to_object(py),),
                Some(kwargs),
            )?;
            Ok(
                Box::new(RegularBranch::new(result.getattr(py, "target_branch")?))
                    as Box<dyn Branch>,
            )
        })
    }

    pub fn sprout(
        &self,
        target: url::Url,
        source_branch: Option<&dyn Branch>,
        create_tree_if_local: Option<bool>,
        stacked: Option<bool>,
        revision_id: Option<&crate::RevisionId>,
    ) -> ControlDir {
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
                    .set_item("source_branch", &source_branch.to_object(py))
                    .unwrap();
            }
            if let Some(revision_id) = revision_id {
                kwargs
                    .set_item("revision_id", revision_id.to_object(py))
                    .unwrap();
            }

            let cd = self
                .0
                .call_method(py, "sprout", (target.to_string(),), Some(kwargs))
                .unwrap();
            ControlDir(cd)
        })
    }

    pub fn has_workingtree(&self) -> bool {
        Python::with_gil(|py| {
            let result = self
                .to_object(py)
                .call_method0(py, "has_workingtree")
                .unwrap();
            result.extract(py).unwrap()
        })
    }

    pub fn open_workingtree(&self) -> PyResult<WorkingTree> {
        Python::with_gil(|py| {
            let wt = self.0.call_method0(py, "open_workingtree")?.extract(py)?;
            Ok(WorkingTree(wt))
        })
    }

    pub fn branch_names(&self) -> PyResult<Vec<String>> {
        Python::with_gil(|py| {
            let names = self
                .0
                .call_method0(py, "branch_names")?
                .extract::<Vec<String>>(py)?;
            Ok(names)
        })
    }
}

impl std::fmt::Debug for ControlDir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("ControlDir({:?})", self.0))
    }
}

pub struct ControlDirFormat(PyObject);

impl ToPyObject for ControlDirFormat {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl Clone for ControlDirFormat {
    fn clone(&self) -> Self {
        Python::with_gil(|py| ControlDirFormat(self.0.clone_ref(py)))
    }
}

impl From<PyObject> for ControlDirFormat {
    fn from(obj: PyObject) -> Self {
        ControlDirFormat(obj)
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

#[test]
fn test_control_dir_format_default() {
    let d = ControlDirFormat::default();
    d.get_format_string();
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
}

#[derive(Debug)]
pub enum OpenError {
    Python(PyErr),
    NotFound(String),
    UnknownFormat,
}

impl From<PyErr> for OpenError {
    fn from(err: PyErr) -> Self {
        pyo3::import_exception!(breezy.errors, NotBranchError);
        pyo3::import_exception!(breezy.errors, UnknownFormatError);

        pyo3::Python::with_gil(|py| {
            if err.is_instance_of::<NotBranchError>(py) {
                OpenError::NotFound(err.value(py).getattr("path").unwrap().extract().unwrap())
            } else if err.is_instance_of::<UnknownFormatError>(py) {
                OpenError::UnknownFormat
            } else {
                OpenError::Python(err)
            }
        })
    }
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpenError::Python(err) => err.fmt(f),
            OpenError::NotFound(name) => write!(f, "Not found: {}", name),
            OpenError::UnknownFormat => write!(f, "Unknown format"),
        }
    }
}

impl std::error::Error for OpenError {}

#[derive(Debug)]
pub enum CreateError {
    Python(PyErr),
    AlreadyExists,
    UnknownFormat,
}

impl From<PyErr> for CreateError {
    fn from(err: PyErr) -> Self {
        pyo3::import_exception!(breezy.errors, AlreadyControlDirError);
        pyo3::import_exception!(breezy.errors, UnknownFormatError);

        pyo3::Python::with_gil(|py| {
            if err.is_instance_of::<AlreadyControlDirError>(py) {
                CreateError::AlreadyExists
            } else if err.is_instance_of::<UnknownFormatError>(py) {
                CreateError::UnknownFormat
            } else {
                CreateError::Python(err)
            }
        })
    }
}

impl std::fmt::Display for CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CreateError::Python(err) => err.fmt(f),
            CreateError::AlreadyExists => write!(f, "Already exists"),
            CreateError::UnknownFormat => write!(f, "Unknown format"),
        }
    }
}

impl std::error::Error for CreateError {}

pub fn open_tree_or_branch(
    location: impl AsLocation,
    name: Option<&str>,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<(Option<WorkingTree>, Box<dyn Branch>), BranchOpenError> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;

        let kwargs = PyDict::new(py);
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item("possible_transports", possible_transports.to_object(py))?;
        }

        let ret = cd.to_object(py).call_method(
            py,
            "open_tree_or_branch",
            (location.as_location(), name),
            Some(kwargs),
        )?;

        let (tree, branch) = ret.extract::<(Option<PyObject>, PyObject)>(py)?;
        let branch = Box::new(RegularBranch::new(branch)) as Box<dyn Branch>;
        let tree = tree.map(WorkingTree);
        Ok((tree, branch))
    })
}

#[test]
fn test_open_tree_or_branch() {
    let tmp_dir = tempfile::tempdir().unwrap();
    create_branch_convenience(&url::Url::from_directory_path(tmp_dir.path()).unwrap()).unwrap();
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

pub fn open(
    url: impl AsLocation,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<ControlDir, OpenError> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item("possible_transports", possible_transports.to_object(py))?;
        }
        let controldir = cd.call_method("open", (url.as_location(),), Some(kwargs))?;
        Ok(ControlDir(controldir.to_object(py)))
    })
}

#[test]
fn test_open() {
    let tmp_dir = tempfile::tempdir().unwrap();

    let e = open(
        &url::Url::from_directory_path(tmp_dir.path()).unwrap(),
        None,
    )
    .unwrap_err();

    assert!(matches!(e, OpenError::NotFound(_)),);

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

pub fn create(
    url: impl AsLocation,
    format: impl AsFormat,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<ControlDir, CreateError> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(format) = format.as_format() {
            kwargs.set_item("format", format.to_object(py))?;
        }
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item("possible_transports", possible_transports.to_object(py))?;
        }
        let controldir = cd.call_method("create", (url.as_location(),), Some(kwargs))?;
        Ok(ControlDir(controldir.to_object(py)))
    })
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

pub fn create_on_transport(
    transport: &Transport,
    format: impl AsFormat,
) -> Result<ControlDir, CreateError> {
    Python::with_gil(|py| {
        let format = format.as_format().unwrap().0;
        Ok(ControlDir(format.call_method(
            py,
            "initialize_on_transport",
            (&transport.to_object(py),),
            None,
        )?))
    })
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

pub fn open_containing_from_transport(
    transport: &Transport,
    probers: Option<&[Prober]>,
) -> Result<(ControlDir, String), OpenError> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(probers) = probers {
            kwargs.set_item("probers", probers.iter().map(|p| &p.0).collect::<Vec<_>>())?;
        }

        let (controldir, subpath): (PyObject, String) = cd
            .call_method(
                "open_containing_from_transport",
                (&transport.to_object(py),),
                Some(kwargs),
            )?
            .extract()?;
        Ok((ControlDir(controldir.to_object(py)), subpath))
    })
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
    assert!(matches!(e, OpenError::NotFound(_)),);
}

pub fn open_from_transport(
    transport: &Transport,
    probers: Option<&[Prober]>,
) -> Result<ControlDir, OpenError> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(probers) = probers {
            kwargs.set_item("probers", probers.iter().map(|p| &p.0).collect::<Vec<_>>())?;
        }
        let controldir = cd.call_method(
            "open_from_transport",
            (&transport.to_object(py),),
            Some(kwargs),
        )?;
        Ok(ControlDir(controldir.to_object(py)))
    })
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
    assert!(matches!(e, OpenError::NotFound(_)),);
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
            Some(ControlDirFormat(format.to_object(py)))
        })
    }
}

impl AsFormat for &ControlDirFormat {
    fn as_format(&self) -> Option<ControlDirFormat> {
        Some(ControlDirFormat(self.0.clone()))
    }
}

pub fn create_branch_convenience(base: &url::Url) -> Result<Box<dyn Branch>, CreateError> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let branch = cd.call_method("create_branch_convenience", (base.to_string(),), None)?;
        Ok(Box::new(RegularBranch::new(branch.to_object(py))) as Box<dyn Branch>)
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
) -> Result<WorkingTree, CreateError> {
    let base = base.to_str().unwrap();
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let format = format.as_format();
        let wt = cd.call_method(
            "create_standalone_workingtree",
            (base, format.unwrap_or_default().to_object(py)),
            None,
        )?;
        Ok(WorkingTree(wt.to_object(py)))
    })
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
    let branch =
        create_branch_convenience(&url::Url::from_directory_path(tmp_dir.path()).unwrap()).unwrap();

    assert_eq!(
        branch.get_user_url(),
        url::Url::from_directory_path(tmp_dir.path()).unwrap()
    );
}
