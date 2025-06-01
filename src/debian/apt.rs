//! APT repository access.
use crate::error::Error;
use debian_control::apt::{Package, Source};
use debversion::Version;
use pyo3::exceptions::PyStopIteration;
use pyo3::intern;
use pyo3::prelude::*;

pyo3::import_exception!(breezy.plugins.debian.apt_repo, NoAptSources);

lazy_static::lazy_static! {
    static ref apt_mutex: std::sync::Mutex<()> = std::sync::Mutex::new(());
}

struct SourceIterator(PyObject);

impl Iterator for SourceIterator {
    type Item = Source;

    fn next(&mut self) -> Option<Self::Item> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let next = self.0.call_method0(py, "__next__");
            match next {
                Ok(next) => Some(next.extract(py).unwrap()),
                Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
                Err(e) if e.is_instance_of::<NoAptSources>(py) => None,
                Err(e) => panic!("error iterating: {:?}", e),
            }
        })
    }
}

struct PackageIterator(PyObject);

impl Iterator for PackageIterator {
    type Item = Package;

    fn next(&mut self) -> Option<Self::Item> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let next = self.0.call_method0(py, "__next__");
            match next {
                Ok(next) => Some(next.extract(py).unwrap()),
                Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
                Err(e) => panic!("error iterating: {:?}", e),
            }
        })
    }
}

/// Interface for interacting with APT repositories.
///
/// This trait defines methods for retrieving packages and other information
/// from APT repositories, both local and remote.
pub trait Apt: ToPyObject {
    // Retrieve the orig tarball from the repository.
    //
    // # Arguments
    // * `source_name` - The name of the source package to retrieve.
    // * `target_directory` - The directory to store the orig tarball in.
    // * `orig_version` - The version of the orig tarball to retrieve.
    //
    // # Returns
    // * `Ok(())` - If the orig tarball was successfully retrieved.
    /// Retrieve the orig tarball from the repository.
    ///
    /// # Arguments
    /// * `source_name` - The name of the source package to retrieve
    /// * `target_directory` - The directory to store the orig tarball in
    /// * `orig_version` - The version of the orig tarball to retrieve
    ///
    /// # Returns
    /// * `Ok(())` - If the orig tarball was successfully retrieved
    fn retrieve_orig(
        &self,
        source_name: &str,
        target_directory: &std::path::Path,
        orig_version: Option<&Version>,
    ) -> Result<(), Error> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            apt.call_method1(
                py,
                "retrieve_orig",
                (source_name, target_directory, orig_version.to_object(py)),
            )?;
            Ok(())
        })
    }

    /// Retrieve the source package from the repository.
    ///
    /// # Arguments
    /// * `source_name` - The name of the source package to retrieve.
    /// * `target_directory` - The directory to store the source package in.
    /// * `source_version` - The version of the source package to retrieve.
    ///
    /// # Returns
    /// * `Ok(())` - If the source package was successfully retrieved.
    fn retrieve_source(
        &self,
        source_name: &str,
        target_directory: &std::path::Path,
        source_version: Option<&Version>,
    ) -> Result<(), Error> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            apt.call_method1(
                py,
                "retrieve_source",
                (source_name, target_directory, source_version.to_object(py)),
            )?;
            Ok(())
        })
    }

    /// Retrieve the binary package from the repository.
    fn iter_sources(&self) -> Box<dyn Iterator<Item = Source>> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            let iter = apt.call_method0(py, "iter_sources").unwrap();
            Box::new(SourceIterator(iter))
        })
    }

    /// Retrieve the binary package from the repository.
    fn iter_binaries(&self) -> Box<dyn Iterator<Item = Package>> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            let iter = apt.call_method0(py, "iter_binaries").unwrap();
            Box::new(PackageIterator(iter))
        })
    }

    /// Retrieve source package by name.
    fn iter_source_by_name(&self, name: &str) -> Box<dyn Iterator<Item = Source>> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            let iter = apt
                .call_method1(py, "iter_source_by_name", (name,))
                .unwrap();
            Box::new(SourceIterator(iter))
        })
    }

    /// Retrieve binary package by name.
    fn iter_binary_by_name(&self, name: &str) -> Box<dyn Iterator<Item = Package>> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            let iter = apt
                .call_method1(py, "iter_binary_by_name", (name,))
                .unwrap();
            Box::new(PackageIterator(iter))
        })
    }
}

/// Interface to a local APT repository.
///
/// This struct provides access to the APT repositories configured on the local system.
pub struct LocalApt(PyObject);

impl Apt for LocalApt {}

impl<'py> IntoPyObject<'py> for LocalApt {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl LocalApt {
    /// Create a new LocalApt instance.
    ///
    /// # Arguments
    /// * `rootdir` - Optional root directory for the APT configuration
    ///
    /// # Returns
    /// A new LocalApt instance or an error
    pub fn new(rootdir: Option<&std::path::Path>) -> Result<Self, Error> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let m = PyModule::import(py, "breezy.plugins.debian.apt_repo")?;
            let apt = m.getattr("LocalApt")?;
            let apt = apt.call1((rootdir,))?;

            apt.call_method0(intern!(py, "__enter__"))?;
            Ok(Self(apt.to_object(py)))
        })
    }
}

impl Default for LocalApt {
    fn default() -> Self {
        LocalApt::new(None).unwrap()
    }
}

impl Drop for LocalApt {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            self.0
                .call_method1(
                    py,
                    intern!(py, "__exit__"),
                    (py.None(), py.None(), py.None()),
                )
                .unwrap();
        });
    }
}

/// Interface to a remote APT repository.
///
/// This struct provides access to APT repositories on remote servers.
pub struct RemoteApt(PyObject);

impl<'py> IntoPyObject<'py> for RemoteApt {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl RemoteApt {
    /// Create a new RemoteApt instance.
    ///
    /// # Arguments
    /// * `mirror_uri` - URI of the APT mirror
    /// * `distribution` - Optional distribution name (e.g., "unstable")
    /// * `components` - Optional list of components (e.g., "main", "contrib")
    /// * `key_path` - Optional path to the GPG key file
    ///
    /// # Returns
    /// A new RemoteApt instance or an error
    pub fn new(
        mirror_uri: &url::Url,
        distribution: Option<&str>,
        components: Option<Vec<String>>,
        key_path: Option<&std::path::Path>,
    ) -> Result<Self, Error> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let m = PyModule::import(py, "breezy.plugins.debian.apt_repo")?;
            let apt = m.getattr("RemoteApt")?;
            let apt = apt.call1((mirror_uri.as_str(), distribution, components, key_path))?;
            apt.call_method0(intern!(py, "__enter__"))?;
            Ok(Self(apt.to_object(py)))
        })
    }

    /// Create a new RemoteApt instance from an APT sources.list entry string.
    ///
    /// # Arguments
    /// * `text` - Text from a sources.list entry
    /// * `key_path` - Optional path to the GPG key file
    ///
    /// # Returns
    /// A new RemoteApt instance or an error
    pub fn from_string(text: &str, key_path: Option<&std::path::Path>) -> Result<Self, Error> {
        let _mutex = apt_mutex.lock().unwrap();
        Python::with_gil(|py| {
            let m = PyModule::import(py, "breezy.plugins.debian.apt_repo")?;
            let apt = m.getattr("RemoteApt")?;
            let apt = apt.call_method1("from_string", (text, key_path))?;
            apt.call_method0(intern!(py, "__enter__"))?;
            Ok(Self(apt.to_object(py)))
        })
    }
}

impl Apt for RemoteApt {}

impl Drop for RemoteApt {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            self.0
                .call_method1(
                    py,
                    intern!(py, "__exit__"),
                    (py.None(), py.None(), py.None()),
                )
                .unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_apt_retrieve_orig() {
        let apt = LocalApt::new(None).unwrap();
        let td = tempfile::tempdir().unwrap();

        match apt.retrieve_orig("apt", td.path(), None) {
            Ok(_) => {
                // Verify the orig file is there
                let entries = td.path().read_dir().unwrap().collect::<Vec<_>>();
                assert_eq!(entries.len(), 1);
                let entry = entries[0].as_ref().unwrap();
                assert!(entry.file_name().to_str().unwrap().starts_with("apt_"),);
                assert!(entry
                    .file_name()
                    .to_str()
                    .unwrap()
                    .ends_with(".orig.tar.gz"),);
            }
            Err(Error::NotImplemented) => {
                // This is expected, LocalApt does not implement this method
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    #[ignore] // Sometimes hangs
    fn test_local_apt() {
        let apt = LocalApt::new(None).unwrap();
        let package = apt.iter_binaries().next().unwrap();
        assert!(package.name().is_some());
        assert!(package.version().is_some());
        let mut sources = apt.iter_sources();
        if let Some(source) = sources.next() {
            assert!(source.package().is_some());
            let source = apt.iter_source_by_name("dpkg").next().unwrap();
            assert_eq!(source.package().unwrap(), "dpkg");
            let package = apt.iter_binary_by_name("dpkg").next().unwrap();
            assert_eq!(package.name().unwrap(), "dpkg");
        }
    }
}
