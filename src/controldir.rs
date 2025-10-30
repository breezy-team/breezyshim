//! The `ControlDir` class provides a high-level interface to control directories,
//! e.g. ".bzr" or ".git" directories.
use crate::branch::{py_tag_selector, Branch, GenericBranch, PyBranch};
use crate::error::Error;
use crate::repository::{GenericRepository, Repository};
use crate::transport::Transport;
use crate::workingtree::GenericWorkingTree;

use crate::location::AsLocation;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Trait for Python probers that can detect control directories.
///
/// This trait is implemented by prober types that wrap Python probers,
/// which are used to detect the presence of control directories.
pub trait PyProber: std::any::Any + std::fmt::Debug {
    /// Get the underlying Python object for this prober.
    fn to_object(&self, py: Python) -> Py<PyAny>;
}

/// Trait for probers that can detect control directories.
///
/// This trait defines the interface for probers, which are used to detect
/// the presence of control directories (like .git or .bzr) in a location.
pub trait Prober: std::fmt::Debug {
    /// Check if a control directory exists at the location specified by a transport.
    ///
    /// # Parameters
    ///
    /// * `transport` - The transport to probe.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if a control directory exists, `Ok(false)` if not, or an error
    /// if the probe could not be completed.
    fn probe_transport(&self, transport: &Transport) -> Result<bool, Error>;
    /// Check if a control directory exists at the specified URL.
    ///
    /// # Parameters
    ///
    /// * `url` - The URL to probe.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if a control directory exists, `Ok(false)` if not, or an error
    /// if the probe could not be completed.
    fn probe(&self, url: &url::Url) -> Result<bool, Error>;
}

impl<T: PyProber> Prober for T {
    fn probe_transport(&self, transport: &Transport) -> Result<bool, Error> {
        Python::attach(|py| {
            let result = self.to_object(py).call_method1(
                py,
                "probe_transport",
                (transport.as_pyobject(),),
            )?;
            Ok(result.extract(py)?)
        })
    }

    fn probe(&self, url: &url::Url) -> Result<bool, Error> {
        Python::attach(|py| {
            let result = self
                .to_object(py)
                .call_method1(py, "probe", (url.to_string(),))?;
            Ok(result.extract(py)?)
        })
    }
}

/// Trait for Python control directories.
///
/// This trait is implemented by control directory types that wrap Python
/// control directory objects.
pub trait PyControlDir: std::any::Any + std::fmt::Debug {
    /// Get the underlying Python object for this control directory.
    fn to_object(&self, py: Python) -> Py<PyAny>;
}

/// Trait for control directories.
///
/// A control directory is a directory that contains version control metadata,
/// like .git or .bzr. This trait defines the interface for accessing and
/// manipulating control directories.
pub trait ControlDir: std::fmt::Debug {
    /// Get a reference to self as Any for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;
    /// The branch type associated with this control directory.
    type Branch: Branch + ?Sized;
    /// The repository type associated with this control directory.
    type Repository: Repository;
    /// The working tree type associated with this control directory.
    type WorkingTree: crate::workingtree::WorkingTree;
    /// Get the user-visible URL for this control directory.
    ///
    /// # Returns
    ///
    /// The URL that can be used to access this control directory.
    fn get_user_url(&self) -> url::Url;
    /// Get the format of this control directory.
    ///
    /// # Returns
    ///
    /// The format of this control directory.
    fn get_format(&self) -> ControlDirFormat;
    /// Get a transport for accessing this control directory's user files.
    ///
    /// # Returns
    ///
    /// A transport for accessing this control directory's user files.
    fn user_transport(&self) -> Transport;
    /// Get a transport for accessing this control directory's control files.
    ///
    /// # Returns
    ///
    /// A transport for accessing this control directory's control files.
    fn control_transport(&self) -> Transport;
    /// Open the repository in this control directory.
    ///
    /// # Returns
    ///
    /// The repository, or an error if the repository could not be opened.
    fn open_repository(&self) -> Result<Self::Repository, Error>;
    /// Find a repository in this control directory or its parents.
    ///
    /// # Returns
    ///
    /// The repository, or an error if no repository could be found.
    fn find_repository(&self) -> Result<GenericRepository, Error>;
    /// Get the format to use when cloning this control directory.
    ///
    /// # Returns
    ///
    /// The format to use when cloning this control directory.
    fn cloning_metadir(&self) -> ControlDirFormat;
    /// Create a new branch in this control directory.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the branch to create, or None for the default branch.
    ///
    /// # Returns
    ///
    /// The newly created branch, or an error if the branch could not be created.
    fn create_branch(&self, name: Option<&str>) -> Result<Box<Self::Branch>, Error>;
    /// Create a new repository in this control directory.
    ///
    /// # Parameters
    ///
    /// * `shared` - Whether the repository should be shared.
    ///
    /// # Returns
    ///
    /// The newly created repository, or an error if the repository could not be created.
    fn create_repository(&self, shared: Option<bool>) -> Result<GenericRepository, Error>;
    /// Open a branch in this control directory.
    ///
    /// # Parameters
    ///
    /// * `branch_name` - The name of the branch to open, or None for the default branch.
    ///
    /// # Returns
    ///
    /// The branch, or an error if the branch could not be opened.
    fn open_branch(&self, branch_name: Option<&str>) -> Result<Box<Self::Branch>, Error>;
    /// Create a working tree in this control directory.
    ///
    /// # Returns
    ///
    /// The newly created working tree, or an error if the working tree could not be created.
    fn create_workingtree(&self) -> crate::Result<GenericWorkingTree>;
    /// Set a branch reference in this control directory.
    ///
    /// # Parameters
    ///
    /// * `branch` - The branch to reference.
    /// * `name` - The name to use for the reference, or None for the default name.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the reference could not be set.
    fn set_branch_reference(&self, branch: &dyn PyBranch, name: Option<&str>) -> crate::Result<()>;
    /// Push a branch to this control directory.
    ///
    /// # Parameters
    ///
    /// * `source_branch` - The branch to push.
    /// * `to_branch_name` - The name of the branch to push to, or None for the default name.
    /// * `stop_revision` - The revision to stop pushing at, or None to push all revisions.
    /// * `overwrite` - Whether to overwrite the target branch if it has diverged.
    /// * `tag_selector` - A function that selects which tags to push, or None to push all tags.
    ///
    /// # Returns
    ///
    /// The target branch after the push, or an error if the push failed.
    fn push_branch(
        &self,
        source_branch: &dyn PyBranch,
        to_branch_name: Option<&str>,
        stop_revision: Option<&crate::RevisionId>,
        overwrite: Option<bool>,
        tag_selector: Option<Box<dyn Fn(String) -> bool>>,
    ) -> crate::Result<Box<Self::Branch>>;
    /// Create a new control directory based on this one (similar to clone).
    ///
    /// # Parameters
    ///
    /// * `target` - The URL of the new control directory.
    /// * `source_branch` - The branch to use as a source, or None to use the default branch.
    /// * `create_tree_if_local` - Whether to create a working tree if the target is local.
    /// * `stacked` - Whether the new branch should be stacked on this one.
    /// * `revision_id` - The revision to sprout from, or None to use the last revision.
    ///
    /// # Returns
    ///
    /// The new control directory, or an error if it could not be created.
    fn sprout(
        &self,
        target: url::Url,
        source_branch: Option<&dyn PyBranch>,
        create_tree_if_local: Option<bool>,
        stacked: Option<bool>,
        revision_id: Option<&crate::RevisionId>,
    ) -> Result<
        Box<
            dyn ControlDir<
                Branch = GenericBranch,
                Repository = GenericRepository,
                WorkingTree = crate::workingtree::GenericWorkingTree,
            >,
        >,
        Error,
    >;
    /// Check if this control directory has a working tree.
    ///
    /// # Returns
    ///
    /// `true` if this control directory has a working tree, `false` otherwise.
    fn has_workingtree(&self) -> bool;
    /// Open the working tree in this control directory.
    ///
    /// # Returns
    ///
    /// The working tree, or an error if the working tree could not be opened.
    fn open_workingtree(&self) -> crate::Result<GenericWorkingTree>;
    /// Get the names of all branches in this control directory.
    ///
    /// # Returns
    ///
    /// A list of branch names, or an error if the branch names could not be retrieved.
    fn branch_names(&self) -> crate::Result<Vec<String>>;

    /// Check if a branch with the given name exists in this control directory.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the branch to check, or None for the default branch.
    ///
    /// # Returns
    ///
    /// `true` if the branch exists, `false` otherwise.
    fn has_branch(&self, name: Option<&str>) -> bool;

    /// Create both a branch and repository in this control directory.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the branch to create, or None for the default branch.
    /// * `shared` - Whether the repository should be shared.
    ///
    /// # Returns
    ///
    /// The created branch, or an error if the branch could not be created.
    fn create_branch_and_repo(
        &self,
        name: Option<&str>,
        shared: Option<bool>,
    ) -> Result<Box<Self::Branch>, Error>;

    /// Get all branches in this control directory.
    ///
    /// # Returns
    ///
    /// A hashmap of branch names to branches, or an error if the branches could not be retrieved.
    fn get_branches(&self) -> crate::Result<std::collections::HashMap<String, Box<Self::Branch>>>;

    /// List all branches in this control directory.
    ///
    /// # Returns
    ///
    /// A list of branch names, or an error if the branches could not be listed.
    fn list_branches(&self) -> crate::Result<Vec<String>>;

    /// Find branches in the repository.
    ///
    /// # Parameters
    ///
    /// * `using` - Whether to use the repository's revisions.
    ///
    /// # Returns
    ///
    /// A vector of branches found, or an error if the branches could not be found.
    fn find_branches(&self, using: Option<bool>) -> crate::Result<Vec<Box<Self::Branch>>>;

    /// Get the reference location for a branch.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the branch, or None for the default branch.
    ///
    /// # Returns
    ///
    /// The branch reference location, or an error if the reference could not be found.
    fn get_branch_reference(&self, name: Option<&str>) -> crate::Result<String>;

    /// Check if this control directory can be converted to the given format.
    ///
    /// # Parameters
    ///
    /// * `format` - The format to check conversion to.
    ///
    /// # Returns
    ///
    /// `true` if conversion is possible, `false` otherwise.
    fn can_convert_format(&self, format: &ControlDirFormat) -> bool;

    /// Check if the target format is a valid conversion target.
    ///
    /// # Parameters
    ///
    /// * `target_format` - The format to check as a conversion target.
    ///
    /// # Returns
    ///
    /// An error if the target format is not valid for conversion.
    fn check_conversion_target(&self, target_format: &ControlDirFormat) -> crate::Result<()>;

    /// Check if this control directory needs format conversion.
    ///
    /// # Parameters
    ///
    /// * `format` - The format to check against.
    ///
    /// # Returns
    ///
    /// `true` if format conversion is needed, `false` otherwise.
    fn needs_format_conversion(&self, format: Option<&ControlDirFormat>) -> bool;

    /// Destroy the branch in this control directory.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the branch to destroy, or None for the default branch.
    ///
    /// # Returns
    ///
    /// An error if the branch could not be destroyed.
    fn destroy_branch(&self, name: Option<&str>) -> crate::Result<()>;

    /// Destroy the repository in this control directory.
    ///
    /// # Returns
    ///
    /// An error if the repository could not be destroyed.
    fn destroy_repository(&self) -> crate::Result<()>;

    /// Destroy the working tree in this control directory.
    ///
    /// # Returns
    ///
    /// An error if the working tree could not be destroyed.
    fn destroy_workingtree(&self) -> crate::Result<()>;

    /// Destroy the working tree metadata in this control directory.
    ///
    /// # Returns
    ///
    /// An error if the working tree metadata could not be destroyed.
    fn destroy_workingtree_metadata(&self) -> crate::Result<()>;

    /// Get the configuration for this control directory.
    ///
    /// # Returns
    ///
    /// A configuration stack for this control directory.
    fn get_config(&self) -> crate::Result<crate::config::ConfigStack>;
}

/// A generic wrapper for a Python control directory object.
///
/// This struct wraps a Python control directory object and provides access to it
/// through the ControlDir trait.
pub struct GenericControlDir(Py<PyAny>);

impl<'py> IntoPyObject<'py> for GenericControlDir {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for GenericControlDir {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GenericControlDir(obj.to_owned().unbind()))
    }
}

impl PyControlDir for GenericControlDir {
    fn to_object(&self, py: Python) -> Py<PyAny> {
        self.0.clone_ref(py)
    }
}

impl GenericControlDir {
    /// Create a new GenericControlDir from a Python control directory object.
    ///
    /// # Parameters
    ///
    /// * `obj` - A Python object representing a control directory.
    ///
    /// # Returns
    ///
    /// A new GenericControlDir instance.
    pub fn new(obj: Py<PyAny>) -> Self {
        Self(obj)
    }
}

impl<T: PyControlDir> ControlDir for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    type Branch = GenericBranch;
    type Repository = crate::repository::GenericRepository;
    type WorkingTree = crate::workingtree::GenericWorkingTree;
    fn get_user_url(&self) -> url::Url {
        Python::attach(|py| {
            let result = self.to_object(py).getattr(py, "user_url").unwrap();
            url::Url::parse(&result.extract::<String>(py).unwrap()).unwrap()
        })
    }

    fn get_format(&self) -> ControlDirFormat {
        Python::attach(|py| {
            let result = self.to_object(py).getattr(py, "_format")?;
            Ok::<_, PyErr>(ControlDirFormat(result))
        })
        .unwrap()
    }

    fn user_transport(&self) -> Transport {
        Python::attach(|py| {
            let result = self.to_object(py).getattr(py, "user_transport").unwrap();
            crate::transport::Transport::new(result)
        })
    }

    fn control_transport(&self) -> Transport {
        Python::attach(|py| {
            let result = self.to_object(py).getattr(py, "control_transport").unwrap();
            crate::transport::Transport::new(result)
        })
    }

    fn open_repository(&self) -> Result<GenericRepository, Error> {
        Python::attach(|py| {
            let result = self.to_object(py).call_method0(py, "open_repository")?;
            Ok(GenericRepository::new(result))
        })
    }

    fn find_repository(&self) -> Result<GenericRepository, Error> {
        Python::attach(|py| {
            let result = self.to_object(py).call_method0(py, "find_repository")?;
            Ok(GenericRepository::new(result))
        })
    }

    fn cloning_metadir(&self) -> ControlDirFormat {
        Python::attach(|py| {
            let result = self.to_object(py).call_method0(py, "cloning_metadir")?;
            Ok::<_, PyErr>(ControlDirFormat(result))
        })
        .unwrap()
    }

    fn create_branch(&self, name: Option<&str>) -> Result<Box<Self::Branch>, Error> {
        Python::attach(|py| {
            let branch: Py<PyAny> =
                self.to_object(py)
                    .call_method(py, "create_branch", (name,), None)?;
            Ok(Box::new(GenericBranch::from(branch)) as Box<Self::Branch>)
        })
    }

    fn create_repository(&self, shared: Option<bool>) -> Result<GenericRepository, Error> {
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            if let Some(shared) = shared {
                kwargs.set_item("shared", shared)?;
            }
            let repository =
                self.to_object(py)
                    .call_method(py, "create_repository", (), Some(&kwargs))?;
            Ok(GenericRepository::new(repository))
        })
    }

    fn open_branch(&self, branch_name: Option<&str>) -> Result<Box<Self::Branch>, Error> {
        Python::attach(|py| {
            let branch: Py<PyAny> =
                self.to_object(py)
                    .call_method(py, "open_branch", (branch_name,), None)?;
            Ok(Box::new(GenericBranch::from(branch)) as Box<Self::Branch>)
        })
    }

    fn create_workingtree(&self) -> crate::Result<GenericWorkingTree> {
        Python::attach(|py| {
            let wt = self.to_object(py).call_method0(py, "create_workingtree")?;
            Ok(GenericWorkingTree(wt))
        })
    }

    fn set_branch_reference(&self, branch: &dyn PyBranch, name: Option<&str>) -> crate::Result<()> {
        Python::attach(|py| {
            self.to_object(py).call_method1(
                py,
                "set_branch_reference",
                (&branch.to_object(py), name),
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
    ) -> crate::Result<Box<Self::Branch>> {
        Python::attach(|py| {
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
                kwargs.set_item("stop_revision", stop_revision.clone())?;
            }
            let result = self.to_object(py).call_method(
                py,
                "push_branch",
                (&source_branch.to_object(py),),
                Some(&kwargs),
            )?;
            Ok(
                Box::new(GenericBranch::from(result.getattr(py, "target_branch")?))
                    as Box<Self::Branch>,
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
    ) -> Result<
        Box<
            dyn ControlDir<
                Branch = GenericBranch,
                Repository = GenericRepository,
                WorkingTree = crate::workingtree::GenericWorkingTree,
            >,
        >,
        Error,
    > {
        Python::attach(|py| {
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
                    .set_item("source_branch", source_branch.to_object(py))
                    .unwrap();
            }
            if let Some(revision_id) = revision_id {
                kwargs.set_item("revision_id", revision_id.clone()).unwrap();
            }

            let cd = self.to_object(py).call_method(
                py,
                "sprout",
                (target.to_string(),),
                Some(&kwargs),
            )?;
            Ok(Box::new(GenericControlDir(cd))
                as Box<
                    dyn ControlDir<
                        Branch = GenericBranch,
                        Repository = GenericRepository,
                        WorkingTree = crate::workingtree::GenericWorkingTree,
                    >,
                >)
        })
    }

    fn has_workingtree(&self) -> bool {
        Python::attach(|py| {
            let result = self
                .to_object(py)
                .call_method0(py, "has_workingtree")
                .unwrap();
            result.extract(py).unwrap()
        })
    }

    fn open_workingtree(&self) -> crate::Result<GenericWorkingTree> {
        Python::attach(|py| {
            let wt = self.to_object(py).call_method0(py, "open_workingtree")?;
            Ok(GenericWorkingTree(wt))
        })
    }

    fn branch_names(&self) -> crate::Result<Vec<String>> {
        Python::attach(|py| {
            let names = self
                .to_object(py)
                .call_method0(py, "branch_names")?
                .extract::<Vec<String>>(py)?;
            Ok(names)
        })
    }

    fn has_branch(&self, name: Option<&str>) -> bool {
        Python::attach(|py| {
            let result = self
                .to_object(py)
                .call_method1(py, "has_branch", (name,))
                .unwrap();
            result.extract(py).unwrap()
        })
    }

    fn create_branch_and_repo(
        &self,
        name: Option<&str>,
        shared: Option<bool>,
    ) -> Result<Box<Self::Branch>, Error> {
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            if let Some(shared) = shared {
                kwargs.set_item("shared", shared)?;
            }
            let branch: Py<PyAny> = self.to_object(py).call_method(
                py,
                "create_branch_and_repo",
                (name,),
                Some(&kwargs),
            )?;
            Ok(Box::new(GenericBranch::from(branch)) as Box<Self::Branch>)
        })
    }

    fn get_branches(&self) -> crate::Result<std::collections::HashMap<String, Box<Self::Branch>>> {
        Python::attach(|py| {
            let branches_dict = self.to_object(py).call_method0(py, "get_branches")?;
            let mut branches = std::collections::HashMap::new();
            let dict: &Bound<PyDict> = branches_dict
                .cast_bound(py)
                .map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>("Expected a dict"))?;
            for (key, value) in dict.iter() {
                let name: String = key.extract()?;
                let branch = GenericBranch::from(value.unbind());
                branches.insert(name, Box::new(branch) as Box<Self::Branch>);
            }
            Ok(branches)
        })
    }

    fn list_branches(&self) -> crate::Result<Vec<String>> {
        Python::attach(|py| {
            let names = self
                .to_object(py)
                .call_method0(py, "list_branches")?
                .extract::<Vec<String>>(py)?;
            Ok(names)
        })
    }

    fn find_branches(&self, using: Option<bool>) -> crate::Result<Vec<Box<Self::Branch>>> {
        Python::attach(|py| {
            let kwargs = PyDict::new(py);
            if let Some(using) = using {
                kwargs.set_item("using", using)?;
            }
            let branches_list =
                self.to_object(py)
                    .call_method(py, "find_branches", (), Some(&kwargs))?;
            let mut branches = Vec::new();
            let list: &Bound<PyList> = branches_list
                .cast_bound(py)
                .map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>("Expected a list"))?;
            for item in list.iter() {
                let branch = GenericBranch::from(item.unbind());
                branches.push(Box::new(branch) as Box<Self::Branch>);
            }
            Ok(branches)
        })
    }

    fn get_branch_reference(&self, name: Option<&str>) -> crate::Result<String> {
        Python::attach(|py| {
            let reference = self
                .to_object(py)
                .call_method1(py, "get_branch_reference", (name,))?
                .extract::<String>(py)?;
            Ok(reference)
        })
    }

    fn can_convert_format(&self, format: &ControlDirFormat) -> bool {
        Python::attach(|py| {
            let result = self
                .to_object(py)
                .call_method1(py, "can_convert_format", (format.0.clone_ref(py),))
                .unwrap();
            result.extract(py).unwrap()
        })
    }

    fn check_conversion_target(&self, target_format: &ControlDirFormat) -> crate::Result<()> {
        Python::attach(|py| {
            self.to_object(py).call_method1(
                py,
                "check_conversion_target",
                (target_format.0.clone_ref(py),),
            )?;
            Ok(())
        })
    }

    fn needs_format_conversion(&self, format: Option<&ControlDirFormat>) -> bool {
        Python::attach(|py| {
            let result = if let Some(format) = format {
                self.to_object(py)
                    .call_method1(py, "needs_format_conversion", (format.0.clone_ref(py),))
                    .unwrap()
            } else {
                self.to_object(py)
                    .call_method0(py, "needs_format_conversion")
                    .unwrap()
            };
            result.extract(py).unwrap()
        })
    }

    fn destroy_branch(&self, name: Option<&str>) -> crate::Result<()> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method1(py, "destroy_branch", (name,))?;
            Ok(())
        })
    }

    fn destroy_repository(&self) -> crate::Result<()> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "destroy_repository")?;
            Ok(())
        })
    }

    fn destroy_workingtree(&self) -> crate::Result<()> {
        Python::attach(|py| {
            self.to_object(py).call_method0(py, "destroy_workingtree")?;
            Ok(())
        })
    }

    fn destroy_workingtree_metadata(&self) -> crate::Result<()> {
        Python::attach(|py| {
            self.to_object(py)
                .call_method0(py, "destroy_workingtree_metadata")?;
            Ok(())
        })
    }

    fn get_config(&self) -> crate::Result<crate::config::ConfigStack> {
        Python::attach(|py| {
            let config = self.to_object(py).call_method0(py, "get_config")?;
            Ok(crate::config::ConfigStack::new(config))
        })
    }
}

impl std::fmt::Debug for GenericControlDir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("ControlDir({:?})", self.0))
    }
}

/// The format of a control directory.
///
/// This struct represents the format of a control directory, which defines how
/// the control directory is stored on disk and what capabilities it has.
pub struct ControlDirFormat(Py<PyAny>);

impl<'py> IntoPyObject<'py> for ControlDirFormat {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl Clone for ControlDirFormat {
    fn clone(&self) -> Self {
        Python::attach(|py| ControlDirFormat(self.0.clone_ref(py)))
    }
}

impl From<Py<PyAny>> for ControlDirFormat {
    fn from(obj: Py<PyAny>) -> Self {
        ControlDirFormat(obj)
    }
}

impl Default for ControlDirFormat {
    fn default() -> Self {
        Python::attach(|py| {
            let breezy = PyModule::import(py, "breezy.controldir").unwrap();
            let cd_format = breezy.getattr("ControlDirFormat").unwrap();
            let obj = cd_format.call_method0("get_default_format").unwrap();
            assert!(!obj.is_none());
            ControlDirFormat(obj.into())
        })
    }
}

impl ControlDirFormat {
    /// Get the format string for this control directory format.
    ///
    /// # Returns
    ///
    /// The format string as a byte vector.
    pub fn get_format_string(&self) -> Vec<u8> {
        Python::attach(|py| {
            self.0
                .call_method0(py, "get_format_string")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Get a human-readable description of this control directory format.
    ///
    /// # Returns
    ///
    /// A string describing this control directory format.
    pub fn get_format_description(&self) -> String {
        Python::attach(|py| {
            self.0
                .call_method0(py, "get_format_description")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Check if a filename is a control filename in this format.
    ///
    /// # Parameters
    ///
    /// * `filename` - The filename to check.
    ///
    /// # Returns
    ///
    /// `true` if the filename is a control filename, `false` otherwise.
    pub fn is_control_filename(&self, filename: &str) -> bool {
        Python::attach(|py| {
            self.0
                .call_method1(py, "is_control_filename", (filename,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Initialize a control directory of this format on a transport.
    ///
    /// # Parameters
    ///
    /// * `transport` - The transport to initialize the control directory on.
    ///
    /// # Returns
    ///
    /// The initialized control directory, or an error if initialization failed.
    pub fn initialize_on_transport(
        &self,
        transport: &Transport,
    ) -> Result<
        Box<
            dyn ControlDir<
                Branch = GenericBranch,
                Repository = GenericRepository,
                WorkingTree = crate::workingtree::GenericWorkingTree,
            >,
        >,
        Error,
    > {
        Python::attach(|py| {
            let cd =
                self.0
                    .call_method1(py, "initialize_on_transport", (transport.as_pyobject(),))?;
            Ok(Box::new(GenericControlDir(cd))
                as Box<
                    dyn ControlDir<
                        Branch = GenericBranch,
                        Repository = GenericRepository,
                        WorkingTree = crate::workingtree::GenericWorkingTree,
                    >,
                >)
        })
    }

    /// Initialize a control directory of this format at a location.
    ///
    /// # Parameters
    ///
    /// * `location` - The location to initialize the control directory at.
    ///
    /// # Returns
    ///
    /// The initialized control directory, or an error if initialization failed.
    pub fn initialize(
        &self,
        location: impl AsLocation,
    ) -> Result<
        Box<
            dyn ControlDir<
                Branch = GenericBranch,
                Repository = GenericRepository,
                WorkingTree = crate::workingtree::GenericWorkingTree,
            >,
        >,
        Error,
    > {
        Python::attach(|py| {
            let cd = self
                .0
                .call_method1(py, "initialize", (location.as_location(),))?;
            Ok(Box::new(GenericControlDir(cd))
                as Box<
                    dyn ControlDir<
                        Branch = GenericBranch,
                        Repository = GenericRepository,
                        WorkingTree = crate::workingtree::GenericWorkingTree,
                    >,
                >)
        })
    }
}

/// Open a tree or branch at a location.
///
/// # Parameters
///
/// * `location` - The location to open.
/// * `name` - The name of the branch to open, or None for the default branch.
/// * `possible_transports` - Optional list of transports to try.
///
/// # Returns
///
/// A tuple with an optional working tree (if one exists) and a branch, or an
/// error if neither could be opened.
pub fn open_tree_or_branch(
    location: impl AsLocation,
    name: Option<&str>,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<(Option<GenericWorkingTree>, Box<dyn Branch>), Error> {
    Python::attach(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;

        let kwargs = PyDict::new(py);
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item(
                "possible_transports",
                possible_transports
                    .iter()
                    .map(|t| t.as_pyobject().clone_ref(py))
                    .collect::<Vec<Py<PyAny>>>(),
            )?;
        }

        let ret = cd.call_method(
            "open_tree_or_branch",
            (location.as_location(), name),
            Some(&kwargs),
        )?;

        let (tree, branch) = ret.extract::<(Option<Py<PyAny>>, Py<PyAny>)>()?;
        let branch = Box::new(GenericBranch::from(branch)) as Box<dyn Branch>;
        let tree = tree.map(GenericWorkingTree);
        Ok((tree, branch))
    })
}

/// Open a control directory at a location.
///
/// # Parameters
///
/// * `url` - The location to open.
/// * `possible_transports` - Optional list of transports to try.
///
/// # Returns
///
/// The control directory, or an error if one could not be opened.
pub fn open(
    url: impl AsLocation,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<
    Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = GenericRepository,
            WorkingTree = crate::workingtree::GenericWorkingTree,
        >,
    >,
    Error,
> {
    Python::attach(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item(
                "possible_transports",
                possible_transports
                    .iter()
                    .map(|t| t.as_pyobject().clone_ref(py))
                    .collect::<Vec<Py<PyAny>>>(),
            )?;
        }
        let controldir = cd.call_method("open", (url.as_location(),), Some(&kwargs))?;
        Ok(Box::new(GenericControlDir(controldir.unbind()))
            as Box<
                dyn ControlDir<
                    Branch = GenericBranch,
                    Repository = GenericRepository,
                    WorkingTree = crate::workingtree::GenericWorkingTree,
                >,
            >)
    })
}
/// Create a new control directory at a location.
///
/// # Parameters
///
/// * `url` - The location to create the control directory at.
/// * `format` - The format to use for the new control directory.
/// * `possible_transports` - Optional list of transports to try.
///
/// # Returns
///
/// The newly created control directory, or an error if it could not be created.
pub fn create(
    url: impl AsLocation,
    format: impl AsFormat,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<
    Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = GenericRepository,
            WorkingTree = crate::workingtree::GenericWorkingTree,
        >,
    >,
    Error,
> {
    Python::attach(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(format) = format.as_format() {
            kwargs.set_item("format", format.clone())?;
        }
        if let Some(possible_transports) = possible_transports {
            kwargs.set_item(
                "possible_transports",
                possible_transports
                    .iter()
                    .map(|t| t.as_pyobject().clone_ref(py))
                    .collect::<Vec<Py<PyAny>>>(),
            )?;
        }
        let controldir = cd.call_method("create", (url.as_location(),), Some(&kwargs))?;
        Ok(Box::new(GenericControlDir(controldir.unbind()))
            as Box<
                dyn ControlDir<
                    Branch = GenericBranch,
                    Repository = GenericRepository,
                    WorkingTree = crate::workingtree::GenericWorkingTree,
                >,
            >)
    })
}
/// Create a new control directory on a transport.
///
/// # Parameters
///
/// * `transport` - The transport to create the control directory on.
/// * `format` - The format to use for the new control directory.
///
/// # Returns
///
/// The newly created control directory, or an error if it could not be created.
pub fn create_on_transport(
    transport: &Transport,
    format: impl AsFormat,
) -> Result<
    Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = GenericRepository,
            WorkingTree = crate::workingtree::GenericWorkingTree,
        >,
    >,
    Error,
> {
    Python::attach(|py| {
        let format = format.as_format().unwrap().0;
        Ok(Box::new(GenericControlDir(format.call_method(
            py,
            "initialize_on_transport",
            (transport.as_pyobject(),),
            None,
        )?))
            as Box<
                dyn ControlDir<
                    Branch = GenericBranch,
                    Repository = GenericRepository,
                    WorkingTree = crate::workingtree::GenericWorkingTree,
                >,
            >)
    })
}

/// Find a control directory containing a location specified by a transport.
///
/// # Parameters
///
/// * `transport` - The transport to search from.
/// * `probers` - Optional list of probers to use to detect control directories.
///
/// # Returns
///
/// A tuple containing the control directory and the relative path from the control
/// directory to the location specified by the transport, or an error if no control
/// directory could be found.
pub fn open_containing_from_transport(
    transport: &Transport,
    probers: Option<&[&dyn PyProber]>,
) -> Result<
    (
        Box<
            dyn ControlDir<
                Branch = GenericBranch,
                Repository = GenericRepository,
                WorkingTree = crate::workingtree::GenericWorkingTree,
            >,
        >,
        String,
    ),
    Error,
> {
    Python::attach(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(probers) = probers {
            kwargs.set_item(
                "probers",
                probers.iter().map(|p| p.to_object(py)).collect::<Vec<_>>(),
            )?;
        }

        let (controldir, subpath): (Py<PyAny>, String) = cd
            .call_method(
                "open_containing_from_transport",
                (transport.as_pyobject(),),
                Some(&kwargs),
            )?
            .extract()?;
        Ok((
            Box::new(GenericControlDir(controldir))
                as Box<
                    dyn ControlDir<
                        Branch = GenericBranch,
                        Repository = GenericRepository,
                        WorkingTree = crate::workingtree::GenericWorkingTree,
                    >,
                >,
            subpath,
        ))
    })
}

/// Open a control directory from a transport.
///
/// # Parameters
///
/// * `transport` - The transport to open from.
/// * `probers` - Optional list of probers to use to detect control directories.
///
/// # Returns
///
/// The opened control directory, or an error if no control directory could be found.
pub fn open_from_transport(
    transport: &Transport,
    probers: Option<&[&dyn PyProber]>,
) -> Result<
    Box<
        dyn ControlDir<
            Branch = GenericBranch,
            Repository = GenericRepository,
            WorkingTree = crate::workingtree::GenericWorkingTree,
        >,
    >,
    Error,
> {
    Python::attach(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let kwargs = PyDict::new(py);
        if let Some(probers) = probers {
            kwargs.set_item(
                "probers",
                probers.iter().map(|p| p.to_object(py)).collect::<Vec<_>>(),
            )?;
        }
        let controldir = cd.call_method(
            "open_from_transport",
            (transport.as_pyobject(),),
            Some(&kwargs),
        )?;
        Ok(Box::new(GenericControlDir(controldir.unbind()))
            as Box<
                dyn ControlDir<
                    Branch = GenericBranch,
                    Repository = GenericRepository,
                    WorkingTree = crate::workingtree::GenericWorkingTree,
                >,
            >)
    })
}

/// Trait for types that can be converted to a control directory format.
///
/// This trait is implemented by types that can be converted to a control directory
/// format, like &str and &ControlDirFormat.
pub trait AsFormat {
    /// Convert to a control directory format.
    ///
    /// # Returns
    ///
    /// The control directory format, or None if the conversion failed.
    fn as_format(&self) -> Option<ControlDirFormat>;
}

impl AsFormat for &str {
    fn as_format(&self) -> Option<ControlDirFormat> {
        Python::attach(|py| {
            let m = py.import("breezy.controldir").ok()?;
            let cd = m.getattr("format_registry").ok()?;
            let format = cd
                .call_method1("make_controldir", (self.to_string(),))
                .ok()?;
            Some(ControlDirFormat(format.unbind()))
        })
    }
}

impl AsFormat for &ControlDirFormat {
    fn as_format(&self) -> Option<ControlDirFormat> {
        Some(Python::attach(|py| ControlDirFormat(self.0.clone_ref(py))))
    }
}

/// Create a branch conveniently (includes creating a repository if needed).
///
/// # Parameters
///
/// * `base` - The URL to create the branch at.
/// * `force_new_tree` - Whether to force the creation of a new working tree if
///   one already exists.
/// * `format` - The format to use for the new branch.
///
/// # Returns
///
/// The newly created branch, or an error if the branch could not be created.
pub fn create_branch_convenience(
    base: &url::Url,
    force_new_tree: Option<bool>,
    format: impl AsFormat,
) -> Result<Box<dyn Branch>, Error> {
    Python::attach(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let format = format.as_format();
        let kwargs = PyDict::new(py);
        if let Some(force_new_tree) = force_new_tree {
            kwargs.set_item("force_new_tree", force_new_tree)?;
        }
        if let Some(format) = format {
            kwargs.set_item("format", format.clone())?;
        }
        let branch = cd.call_method(
            "create_branch_convenience",
            (base.to_string(),),
            Some(&kwargs),
        )?;
        Ok(Box::new(GenericBranch::from(branch.unbind())) as Box<dyn Branch>)
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
) -> Result<GenericWorkingTree, Error> {
    let base = base.to_str().unwrap();
    Python::attach(|py| {
        let m = py.import("breezy.controldir")?;
        let cd = m.getattr("ControlDir")?;
        let format = format.as_format();
        let wt = cd.call_method(
            "create_standalone_workingtree",
            (base, format.unwrap_or_default()),
            None,
        )?;
        Ok(GenericWorkingTree(wt.unbind()))
    })
}

/// A generic prober for detecting control directories.
///
/// This struct wraps a Python prober object and provides access to it through
/// the Prober trait.
pub struct GenericProber(Py<PyAny>);

impl<'py> IntoPyObject<'py> for GenericProber {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for GenericProber {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GenericProber(obj.to_owned().unbind()))
    }
}

impl PyProber for GenericProber {
    fn to_object(&self, py: Python) -> Py<PyAny> {
        self.0.clone_ref(py)
    }
}

impl GenericProber {
    /// Create a new GenericProber from a Python prober object.
    ///
    /// # Parameters
    ///
    /// * `obj` - A Python object representing a prober.
    ///
    /// # Returns
    ///
    /// A new GenericProber instance.
    pub fn new(obj: Py<PyAny>) -> Self {
        Self(obj)
    }
}

/// Implementation of Debug for GenericProber.
impl std::fmt::Debug for GenericProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("Prober({:?})", self.0))
    }
}

/// Get all available probers.
///
/// # Returns
///
/// A list of all available probers.
pub fn all_probers() -> Vec<Box<dyn PyProber>> {
    Python::attach(|py| -> PyResult<Vec<Box<dyn PyProber>>> {
        let m = py.import("breezy.controldir")?;
        let cdf = m.getattr("ControlDirFormat")?;
        let probers = cdf
            .call_method0("all_probers")?
            .extract::<Vec<Py<PyAny>>>()?;
        Ok(probers
            .into_iter()
            .map(|p| Box::new(GenericProber::new(p)) as Box<dyn PyProber>)
            .collect::<Vec<_>>())
    })
    .unwrap()
}

/// A registry of control directory formats.
///
/// This struct wraps a Python registry of control directory formats,
/// which can be used to create control directory formats from names.
pub struct ControlDirFormatRegistry(Py<PyAny>);

impl ControlDirFormatRegistry {
    /// Create a new ControlDirFormatRegistry.
    ///
    /// # Returns
    ///
    /// A new ControlDirFormatRegistry instance.
    pub fn new() -> Self {
        Python::attach(|py| {
            let m = py.import("breezy.controldir").unwrap();
            let obj = m.getattr("format_registry").unwrap();
            ControlDirFormatRegistry(obj.into())
        })
    }

    /// Create a control directory format from a format name.
    ///
    /// # Parameters
    ///
    /// * `format` - The name of the format to create.
    ///
    /// # Returns
    ///
    /// The control directory format, or None if the format name is not recognized.
    pub fn make_controldir(&self, format: &str) -> Option<ControlDirFormat> {
        Python::attach(
            |py| match self.0.call_method1(py, "make_controldir", (format,)) {
                Ok(format) => Some(ControlDirFormat(format)),
                Err(e) if e.is_instance_of::<pyo3::exceptions::PyKeyError>(py) => None,
                Err(e) => panic!("{}", e),
            },
        )
    }
}

/// Implementation of Default for ControlDirFormatRegistry.
impl Default for ControlDirFormatRegistry {
    /// Creates a default ControlDirFormatRegistry.
    ///
    /// # Returns
    ///
    /// A new ControlDirFormatRegistry instance.
    fn default() -> Self {
        ControlDirFormatRegistry::new()
    }
}

lazy_static::lazy_static! {
    /// The global control directory format registry.
    ///
    /// This is a lazily initialized static reference to a `ControlDirFormatRegistry`
    /// instance, which can be used to access control directory formats.
    pub static ref FORMAT_REGISTRY: ControlDirFormatRegistry = ControlDirFormatRegistry::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workingtree::WorkingTree;

    #[test]
    fn test_controldir_to_pycontroldir_conversion() {
        // Test the pattern from the issue:
        // 1. Get a working tree
        // 2. Get its controldir as Box<dyn ControlDir>
        // 3. Downcast it to use as &dyn PyControlDir

        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();

        // Get controldir as Box<dyn ControlDir>
        let controldir = wt.controldir();

        // Now try to downcast it to GenericControlDir using as_any()
        if let Some(generic_controldir) = controldir.as_any().downcast_ref::<GenericControlDir>() {
            // Success! We can now use it as &dyn PyControlDir
            let py_controldir: &dyn PyControlDir = generic_controldir;
            // Verify we can call PyControlDir methods
            Python::attach(|py| {
                let _obj = py_controldir.to_object(py);
            });
        } else {
            panic!("Failed to downcast ControlDir to GenericControlDir");
        }
    }

    #[test]
    fn test_control_dir_format_registry() {
        crate::init();
        let registry = ControlDirFormatRegistry::new();
        let format = registry.make_controldir("2a").unwrap();
        let _ = format.get_format_string();
    }

    #[test]
    fn test_format_registry() {
        crate::init();
        let format = FORMAT_REGISTRY.make_controldir("2a").unwrap();
        let _ = format.get_format_string();
    }

    #[test]
    fn test_all_probers() {
        crate::init();
        let probers = all_probers();
        assert!(!probers.is_empty());
    }

    #[test]
    fn test_open_tree_or_branch() {
        crate::init();
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
        crate::init();
        let d = ControlDirFormat::default();
        d.get_format_string();
    }

    #[test]
    fn test_open() {
        crate::init();
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
        crate::init();
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
        crate::init();
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
        crate::init();
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
        crate::init();
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
        crate::init();
        let tmp_dir = tempfile::tempdir().unwrap();
        let wt = create_standalone_workingtree(tmp_dir.path(), "2a").unwrap();

        assert_eq!(
            wt.basedir().canonicalize().unwrap(),
            tmp_dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn test_create_branch_convenience() {
        crate::init();
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
        crate::init();
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
        crate::init();
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
        crate::init();
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
        crate::init();
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
