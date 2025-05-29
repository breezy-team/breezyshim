use crate::branch::{Branch, PyBranch};
use crate::controldir::{ControlDir, PyControlDir};
use crate::debian::error::Error;
use crate::debian::TarballKind;
use crate::debian::VersionKind;
use crate::tree::{PyTree, Tree};
use crate::RevisionId;
use debversion::Version;
use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Source for pristine tarballs.
///
/// This struct represents a source for pristine tarballs stored
/// in a pristine-tar branch.
pub struct PristineTarSource(PyObject);

impl From<PyObject> for PristineTarSource {
    fn from(obj: PyObject) -> Self {
        PristineTarSource(obj)
    }
}

impl<'py> IntoPyObject<'py> for PristineTarSource {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl IntoPy<PyObject> for PristineTarSource {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

/// A source for upstream versions (uscan, debian/rules, etc).
pub struct UpstreamBranchSource(PyObject);

impl From<PyObject> for UpstreamBranchSource {
    fn from(obj: PyObject) -> Self {
        UpstreamBranchSource(obj)
    }
}

impl<'py> IntoPyObject<'py> for UpstreamBranchSource {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

/// Information about a tarball file.
///
/// This struct contains metadata about a tarball file, including its
/// filename, component kind, and MD5 hash.
pub struct Tarball {
    /// The filename of the tarball.
    pub filename: String,
    /// The kind of component this tarball represents.
    pub component: TarballKind,
    /// The MD5 hash of the tarball.
    pub md5: String,
}

/// A collection of tarballs.
pub type Tarballs = Vec<Tarball>;

impl FromPyObject<'_> for Tarball {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Tarball {
            filename: ob.get_item(0)?.extract()?,
            component: ob.get_item(1)?.extract()?,
            md5: ob.get_item(2)?.extract()?,
        })
    }
}

impl<'py> IntoPyObject<'py> for Tarball {
    fn to_object(&self, py: Python) -> PyObject {
        (
            self.filename.clone(),
            self.component.clone(),
            self.md5.clone(),
        )
            .to_object(py)
    }
}

impl IntoPy<PyObject> for Tarball {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

/// Trait for Python-based upstream sources.
///
/// This trait is implemented by wrappers around Python upstream source objects.
pub trait PyUpstreamSource: for<'py> IntoPyObject<'py> + std::any::Any + std::fmt::Debug {}

/// Trait for upstream sources.
///
/// This trait defines the interface for working with upstream sources,
/// which provide access to upstream versions of packages.
pub trait UpstreamSource: std::fmt::Debug {
    /// Check what the latest upstream version is.
    ///
    /// # Arguments
    /// * `package` - Name of the package
    /// * `version` - The current upstream version of the package.
    ///
    /// # Returns
    /// A tuple of the latest upstream version and the mangled version.
    fn get_latest_version(
        &self,
        package: Option<&str>,
        current_version: Option<&str>,
    ) -> Result<Option<(String, String)>, Error>;

    /// Retrieve recent version strings.
    ///
    /// # Arguments
    /// * `package`: Name of the package
    /// * `version`: Last upstream version since which to retrieve versions
    fn get_recent_versions(
        &self,
        package: Option<&str>,
        since_version: Option<&str>,
    ) -> Box<dyn Iterator<Item = (String, String)>>;

    /// Lookup the revision ids for a particular version.
    ///
    /// # Arguments
    /// * `package` - Package name
    /// * `version` - Version string
    ///
    /// # Returns
    /// A dictionary mapping component names to revision ids
    fn version_as_revisions(
        &self,
        package: Option<&str>,
        version: &str,
        tarballs: Option<Tarballs>,
    ) -> Result<HashMap<TarballKind, (RevisionId, PathBuf)>, Error>;

    /// Check whether this upstream source contains a particular package.
    ///
    /// # Arguments
    /// * `package` - Package name
    /// * `version` - Version string
    /// * `tarballs` - Tarballs list
    fn has_version(
        &self,
        package: Option<&str>,
        version: &str,
        tarballs: Option<Tarballs>,
    ) -> Result<bool, Error>;

    /// Fetch the source tarball for a particular version.
    ///
    /// # Arguments
    /// * `package` - Name of the package
    /// * `version` - Version string of the version to fetch
    /// * `target_dir` - Directory in which to store the tarball
    /// * `components` - List of component names to fetch; may be None,
    ///
    /// # Returns
    /// Paths of the fetched tarballs
    fn fetch_tarballs(
        &self,
        package: Option<&str>,
        version: &str,
        target_dir: &Path,
        components: Option<&[TarballKind]>,
    ) -> Result<Vec<PathBuf>, Error>;
}

impl<T: PyUpstreamSource> UpstreamSource for T {
    fn get_latest_version(
        &self,
        package: Option<&str>,
        current_version: Option<&str>,
    ) -> Result<Option<(String, String)>, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "get_latest_version", (package, current_version))?
                .extract(py)?)
        })
    }

    fn get_recent_versions(
        &self,
        package: Option<&str>,
        since_version: Option<&str>,
    ) -> Box<dyn Iterator<Item = (String, String)>> {
        let mut ret = vec![];
        Python::with_gil(|py| -> PyResult<()> {
            let recent_versions = self.to_object(py).call_method1(
                py,
                "get_recent_versions",
                (package, since_version),
            )?;

            while let Ok(Some((version, mangled_version))) =
                recent_versions.call_method0(py, "__next__")?.extract(py)
            {
                ret.push((version, mangled_version));
            }
            Ok(())
        })
        .unwrap();
        Box::new(ret.into_iter())
    }

    fn version_as_revisions(
        &self,
        package: Option<&str>,
        version: &str,
        tarballs: Option<Tarballs>,
    ) -> Result<HashMap<TarballKind, (RevisionId, PathBuf)>, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "version_as_revisions", (package, version, tarballs))?
                .extract(py)?)
        })
    }

    fn has_version(
        &self,
        package: Option<&str>,
        version: &str,
        tarballs: Option<Tarballs>,
    ) -> Result<bool, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "has_version", (package, version, tarballs))?
                .extract(py)?)
        })
    }

    fn fetch_tarballs(
        &self,
        package: Option<&str>,
        version: &str,
        target_dir: &Path,
        components: Option<&[TarballKind]>,
    ) -> Result<Vec<PathBuf>, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(
                    py,
                    "fetch_tarballs",
                    (package, version, target_dir, components.map(|x| x.to_vec())),
                )?
                .extract(py)?)
        })
    }
}

/// A generic wrapper around any Python upstream source object.
///
/// This struct provides a way to interact with any upstream source
/// from Python code, regardless of its specific implementation.
pub struct GenericUpstreamSource(PyObject);

impl<'py> IntoPyObject<'py> for GenericUpstreamSource {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl FromPyObject<'_> for GenericUpstreamSource {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(GenericUpstreamSource(obj.clone().unbind()))
    }
}

impl PyUpstreamSource for GenericUpstreamSource {}

impl GenericUpstreamSource {
    /// Create a new generic upstream source from a Python object.
    ///
    /// # Arguments
    /// * `obj` - The Python object representing an upstream source.
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }
}

impl std::fmt::Debug for GenericUpstreamSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("GenericUpstreamSource({:?})", self.0))
    }
}

impl PyUpstreamSource for UpstreamBranchSource {}

impl std::fmt::Debug for UpstreamBranchSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpstreamBranchSource").finish()
    }
}

impl UpstreamBranchSource {
    /// Get the upstream branch associated with this source.
    ///
    /// # Returns
    /// A branch object representing the upstream branch.
    pub fn upstream_branch(&self) -> Box<dyn crate::branch::Branch> {
        let o = Python::with_gil(|py| self.to_object(py).getattr(py, "upstream_branch").unwrap());
        Box::new(crate::branch::GenericBranch::from(o))
    }

    /// Get a revision tree for a specific upstream version.
    ///
    /// # Arguments
    /// * `source_name` - Optional name of the source package
    /// * `mangled_upstream_version` - The mangled version string of the upstream version
    ///
    /// # Returns
    /// A revision tree object or an error
    pub fn revision_tree(
        &self,
        source_name: Option<&str>,
        mangled_upstream_version: &str,
    ) -> Result<crate::tree::RevisionTree, Error> {
        Python::with_gil(|py| {
            Ok(crate::tree::RevisionTree(self.to_object(py).call_method1(
                py,
                "revision_tree",
                (source_name, mangled_upstream_version),
            )?))
        })
    }

    /// Get the revision ID for a specific upstream version.
    ///
    /// # Arguments
    /// * `package` - Optional name of the source package
    /// * `version` - Version string of the upstream version
    /// * `tarballs` - Optional list of tarballs
    ///
    /// # Returns
    /// A tuple containing the revision ID and path, or an error
    pub fn version_as_revision(
        &self,
        package: Option<&str>,
        version: &str,
        tarballs: Option<Tarballs>,
    ) -> Result<(RevisionId, PathBuf), Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "version_as_revision", (package, version, tarballs))?
                .extract(py)?)
        })
    }

    /// Create an upstream branch source from a branch.
    ///
    /// # Arguments
    /// * `upstream_branch` - The upstream branch to use
    /// * `version_kind` - Optional kind of version to use
    /// * `local_dir` - The local control directory
    /// * `create_dist` - Optional function to create a distribution
    ///
    /// # Returns
    /// A new upstream branch source or an error
    pub fn from_branch(
        upstream_branch: &dyn PyBranch,
        version_kind: Option<VersionKind>,
        local_dir: &dyn PyControlDir,
        create_dist: Option<
            impl Fn(&dyn PyTree, &str, &str, &Path, &Path) -> Result<OsString, Error>
                + Send
                + Sync
                + 'static,
        >,
    ) -> Result<Self, Error> {
        Python::with_gil(|py| {
            let m = py.import("breezy.plugins.debian.upstream.branch").unwrap();
            let cls = m.getattr("UpstreamBranchSource").unwrap();
            let upstream_branch = upstream_branch.to_object(py);
            let kwargs = PyDict::new(py);
            kwargs.set_item("version_kind", version_kind.unwrap_or_default())?;
            kwargs.set_item("local_dir", local_dir.to_object(py))?;
            if let Some(create_dist) = create_dist {
                let create_dist = move |args: &Bound<'_, PyTuple>,
                                        _kwargs: Option<&Bound<'_, PyDict>>|
                      -> PyResult<_> {
                    let args = args.extract::<(PyObject, String, String, PathBuf, PathBuf)>()?;
                    create_dist(
                        &crate::tree::RevisionTree(args.0),
                        &args.1,
                        &args.2,
                        &args.3,
                        &args.4,
                    )
                    .map(|x| x.to_string_lossy().into_owned())
                    .map_err(|e| e.into())
                };
                let create_dist = PyCFunction::new_closure_bound(py, None, None, create_dist)?;
                kwargs.set_item("create_dist", create_dist)?;
            }
            Ok(UpstreamBranchSource(
                cls.call_method("from_branch", (upstream_branch,), Some(&kwargs))?
                    .into(),
            ))
        })
    }
}

impl PyUpstreamSource for PristineTarSource {}

impl std::fmt::Debug for PristineTarSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("PristineTarSource({:?})", self.0))
    }
}

impl PristineTarSource {
    /// Check whether this upstream source contains a particular package.
    ///
    /// # Arguments
    /// * `package` - Package name
    /// * `version` - Version string
    /// * `tarballs` - Tarballs list
    pub fn has_version(
        &self,
        package: Option<&str>,
        version: &str,
        tarballs: Option<Tarballs>,
        try_hard: bool,
    ) -> Result<bool, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "has_version", (package, version, tarballs, try_hard))?
                .extract(py)?)
        })
    }
}

/// Update the revision in a upstream version string.
///
/// # Arguments
/// * `branch` - Branch in which the revision can be found
/// * `version_string` - Original version string
/// * `revid` - Revision id of the revision
/// * `sep` - Separator to use when adding snapshot
pub fn upstream_version_add_revision(
    upstream_branch: &dyn PyBranch,
    version_string: &str,
    revid: &RevisionId,
    sep: Option<&str>,
) -> Result<String, Error> {
    let sep = sep.unwrap_or("+");
    Python::with_gil(|py| {
        let m = py.import("breezy.plugins.debian.upstream.branch").unwrap();
        let upstream_version_add_revision = m.getattr("upstream_version_add_revision").unwrap();
        Ok(upstream_version_add_revision
            .call_method1(
                "upstream_version_add_revision",
                (
                    upstream_branch.to_object(py),
                    version_string,
                    revid.to_object(py),
                    sep,
                ),
            )?
            .extract()?)
    })
}

/// Get a pristine-tar source for a packaging branch.
///
/// # Arguments
/// * `packaging_tree` - The packaging tree
/// * `packaging_branch` - The packaging branch
///
/// # Returns
/// A pristine-tar source or an error
pub fn get_pristine_tar_source(
    packaging_tree: &dyn PyTree,
    packaging_branch: &dyn PyBranch,
) -> Result<PristineTarSource, Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.plugins.debian.upstream").unwrap();
        let cls = m.getattr("get_pristine_tar_source").unwrap();
        Ok(PristineTarSource(
            cls.call1((packaging_tree.to_object(py), packaging_branch.to_object(py)))?
                .into(),
        ))
    })
}

/// Run a distribution command to create a source tarball.
///
/// # Arguments
/// * `revtree` - The revision tree to run the command in
/// * `package` - Optional name of the package
/// * `version` - Version of the package
/// * `target_dir` - Directory to store the result in
/// * `dist_command` - Command to run to create the distribution
/// * `include_controldir` - Whether to include the control directory
/// * `subpath` - Subpath within the tree
///
/// # Returns
/// Whether the command succeeded or an error
pub fn run_dist_command(
    revtree: &dyn PyTree,
    package: Option<&str>,
    version: &Version,
    target_dir: &Path,
    dist_command: &str,
    include_controldir: bool,
    subpath: &Path,
) -> Result<bool, Error> {
    Python::with_gil(|py| {
        let m = py.import("breezy.plugins.debian.upstream").unwrap();
        let run_dist_command = m.getattr("run_dist_command").unwrap();
        let kwargs = PyDict::new(py);
        kwargs.set_item("revtree", revtree.to_object(py))?;
        kwargs.set_item("package", package)?;
        kwargs.set_item("version", version)?;
        kwargs.set_item(target_dir, target_dir)?;
        kwargs.set_item("dist_command", dist_command)?;
        kwargs.set_item("include_controldir", include_controldir)?;
        kwargs.set_item("subpath", subpath)?;

        Ok(run_dist_command.call((), Some(&kwargs))?.extract()?)
    })
}
