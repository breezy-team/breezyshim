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

    pub fn create_branch_convenience(base: &url::Url) -> PyResult<Box<dyn Branch>> {
        Python::with_gil(|py| {
            let m = py.import("breezy.controldir")?;
            let cd = m.getattr("ControlDir")?;
            let branch = cd.call_method("create_branch_convenience", (base.to_string(),), None)?;
            Ok(Box::new(RegularBranch::new(branch.to_object(py))) as Box<dyn Branch>)
        })
    }

    pub fn create_standalone_workingtree(
        base: &std::path::Path,
        format: Option<impl AsFormat>,
    ) -> PyResult<WorkingTree> {
        let base = base.to_str().unwrap();
        Python::with_gil(|py| {
            let m = py.import("breezy.controldir")?;
            let cd = m.getattr("ControlDir")?;
            let format = format.map(|format| format.as_format().to_string());
            let wt = cd.call_method("create_standalone_workingtree", (base, format), None)?;
            Ok(WorkingTree(wt.to_object(py)))
        })
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
        open_tree_or_branch(location, name)
    }

    #[deprecated]
    pub fn open(url: &url::Url) -> PyResult<ControlDir> {
        open(url)
    }

    #[deprecated]
    pub fn open_containing_from_transport(
        transport: &Transport,
        probers: Option<&[Prober]>,
    ) -> PyResult<(ControlDir, String)> {
        open_containing_from_transport(transport, probers)
    }

    #[deprecated]
    pub fn open_from_transport(
        transport: &Transport,
        probers: Option<&[Prober]>,
    ) -> PyResult<ControlDir> {
        open_from_transport(transport, probers)
    }

    pub fn create_branch(&self, name: Option<&str>) -> PyResult<Box<dyn Branch>> {
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

            let cd = self
                .0
                .call_method(py, "sprout", (target.to_string(),), Some(kwargs))
                .unwrap();
            ControlDir(cd)
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

pub struct ControlDirFormat(PyObject);

impl From<PyObject> for ControlDirFormat {
    fn from(obj: PyObject) -> Self {
        ControlDirFormat(obj)
    }
}

pub fn open_tree_or_branch(
    location: impl AsLocation,
    name: Option<&str>,
) -> Result<(Option<WorkingTree>, Box<dyn Branch>), BranchOpenError> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;

        let ret = cd.to_object(py).call_method(
            py,
            "open_tree_or_branch",
            (location.as_location(), name),
            None,
        )?;

        let (tree, branch) = ret.extract::<(Option<PyObject>, PyObject)>(py)?;
        let branch = Box::new(RegularBranch::new(branch)) as Box<dyn Branch>;
        let tree = tree.map(WorkingTree);
        Ok((tree, branch))
    })
}

pub fn open(url: impl AsLocation) -> PyResult<ControlDir> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let controldir = cd.call_method("open", (url.as_location(),), None)?;
        Ok(ControlDir(controldir.to_object(py)))
    })
}

pub fn create(
    url: impl AsLocation,
    format: Option<impl AsFormat>,
    possible_transports: Option<&mut Transport>,
) -> PyResult<ControlDir> {
    Python::with_gil(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(format) = format {
            kwargs.set_item("format", format.as_format())?;
        }
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item("possible_transports", possible_transports.to_object(py))?;
        }
        let controldir = cd.call_method("create", (url.as_location(),), Some(kwargs))?;
        Ok(ControlDir(controldir.to_object(py)))
    })
}

pub fn open_containing_from_transport(
    transport: &Transport,
    probers: Option<&[Prober]>,
) -> PyResult<(ControlDir, String)> {
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

pub fn open_from_transport(
    transport: &Transport,
    probers: Option<&[Prober]>,
) -> PyResult<ControlDir> {
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

pub trait AsFormat {
    fn as_format(&self) -> &str;
}

impl AsFormat for str {
    fn as_format(&self) -> &str {
        self
    }
}
