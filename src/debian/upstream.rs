use crate::debian::error::Error;
use crate::controldir::ControlDir;
use crate::debian::VersionKind;
use crate::debian::TarballKind;
use crate::RevisionId;
use pyo3::types::PyDict;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct PristineTarSource(PyObject);

impl From<PyObject> for PristineTarSource {
    fn from(obj: PyObject) -> Self {
        PristineTarSource(obj)
    }
}

impl ToPyObject for PristineTarSource {
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

impl ToPyObject for UpstreamBranchSource {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

pub struct Tarball {
    pub filename: String,
    pub component: TarballKind,
    pub md5: String,
}

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

impl ToPyObject for Tarball {
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

pub trait UpstreamSource: ToPyObject {
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
        package: &str,
        current_version: Option<&str>,
    ) -> Result<Option<(String, String)>, Error> {
        Python::with_gil(|py| {
            Ok(self
                .to_object(py)
                .call_method1(py, "get_latest_version", (package, current_version))?
                .extract(py)?)
        })
    }

    /// Retrieve recent version strings.
    ///
    /// # Arguments
    /// * `package`: Name of the package
    /// * `version`: Last upstream version since which to retrieve versions
    fn get_recent_versions(
        &self,
        package: &str,
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
        package: &str,
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

    /// Check whether this upstream source contains a particular package.
    ///
    /// # Arguments
    /// * `package` - Package name
    /// * `version` - Version string
    /// * `tarballs` - Tarballs list
    fn has_version(
        &self,
        package: &str,
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
        package: &str,
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

impl UpstreamSource for UpstreamBranchSource {}

impl UpstreamBranchSource {
    pub fn upstream_branch(&self) -> Box<dyn crate::branch::Branch> {
        let o = Python::with_gil(|py| self.to_object(py).getattr(py, "upstream_branch").unwrap());
        Box::new(crate::branch::RegularBranch::new(o))
    }

    pub fn version_as_revision(
        &self,
        package: &str,
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

    pub fn from_branch(upstream_branch: &dyn crate::branch::Branch, version_kind: Option<VersionKind>, local_dir: &ControlDir) -> Result<Self, Error> {
        Python::with_gil(|py| {
            let m = py
                .import_bound("breezy.plugins.debian.org.upstream.branch")
                .unwrap();
            let cls = m.getattr("UpstreamBranchSource").unwrap();
            let upstream_branch = upstream_branch.to_object(py);
            let kwargs = PyDict::new_bound(py);
            kwargs.set_item("version_kind", version_kind.unwrap_or_default())?;
            kwargs.set_item("local_dir", local_dir.to_object(py))?;
            Ok(UpstreamBranchSource(
                cls.call_method("from_branch", (upstream_branch,), Some(&kwargs))?.into(),
            ))
        })
    }
}

impl UpstreamSource for PristineTarSource {}

impl PristineTarSource {
    /// Check whether this upstream source contains a particular package.
    ///
    /// # Arguments
    /// * `package` - Package name
    /// * `version` - Version string
    /// * `tarballs` - Tarballs list
    pub fn has_version(
        &self,
        package: &str,
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
    upstream_branch: &dyn crate::branch::Branch,
    version_string: &str,
    revid: &RevisionId,
    sep: Option<&str>,
) -> Result<String, Error> {
    let sep = sep.unwrap_or("+");
    Python::with_gil(|py| {
        let m = py
            .import_bound("breezy.plugins.debian.org.upstream.branch")
            .unwrap();
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
