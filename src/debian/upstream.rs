use crate::error::Error;
use crate::RevisionId;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A source for upstream versions (uscan, debian/rules, etc).
pub struct UpstreamSource(PyObject);

impl From<PyObject> for UpstreamSource {
    fn from(obj: PyObject) -> Self {
        UpstreamSource(obj)
    }
}

impl ToPyObject for UpstreamSource {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

pub struct Tarball {
    pub filename: String,
    pub component: Option<String>,
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

impl UpstreamSource {
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
        current_version: &str,
    ) -> Result<(String, String), Error> {
        Python::with_gil(|py| {
            Ok(self
                .0
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
    ) -> impl Iterator<Item = (String, String)> {
        let mut ret = vec![];
        Python::with_gil(|py| -> PyResult<()> {
            let recent_versions =
                self.0
                    .call_method1(py, "get_recent_versions", (package, since_version))?;

            while let Ok(Some((version, mangled_version))) =
                recent_versions.call_method0(py, "__next__")?.extract(py)
            {
                ret.push((version, mangled_version));
            }
            Ok(())
        })
        .unwrap();
        ret.into_iter()
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
    ) -> Result<HashMap<Option<String>, (RevisionId, String)>, Error> {
        Python::with_gil(|py| {
            Ok(self
                .0
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
                .0
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
        components: Option<&[String]>,
    ) -> Result<Vec<PathBuf>, Error> {
        Python::with_gil(|py| {
            Ok(self
                .0
                .call_method1(
                    py,
                    "fetch_tarballs",
                    (package, version, target_dir, components.map(|x| x.to_vec())),
                )?
                .extract(py)?)
        })
    }
}
