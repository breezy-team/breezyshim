use crate::error::Error;
use debian_control::apt::{Package, Source};
use debversion::Version;
use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;

struct SourceIterator(PyObject);

impl Iterator for SourceIterator {
    type Item = Source;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| {
            let next = self.0.call_method0(py, "__next__");
            if let Ok(o) = next.as_ref() {
                println!("{}", o.call_method0(py, "__str__").unwrap());
            }
            match next {
                Ok(next) => Some(next.extract(py).unwrap()),
                Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
                Err(e) => panic!("error iterating: {:?}", e),
            }
        })
    }
}

struct PackageIterator(PyObject);

impl Iterator for PackageIterator {
    type Item = Package;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| {
            let next = self.0.call_method0(py, "__next__");
            if let Ok(o) = next.as_ref() {
                println!("{}", o.call_method0(py, "__str__").unwrap());
            }
            match next {
                Ok(next) => Some(next.extract(py).unwrap()),
                Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
                Err(e) => panic!("error iterating: {:?}", e),
            }
        })
    }
}

pub trait Apt: ToPyObject {
    fn retrieve_orig(
        &self,
        source_name: &str,
        target_directory: &std::path::Path,
        orig_version: Option<&Version>,
    ) -> Result<(), Error> {
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

    fn retrieve_source(
        &self,
        source_name: &str,
        target_directory: &std::path::Path,
        source_version: Option<&Version>,
    ) -> Result<(), Error> {
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

    fn iter_sources(&self) -> impl Iterator<Item = Source> {
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            let iter = apt.call_method0(py, "iter_sources").unwrap();
            SourceIterator(iter)
        })
    }

    fn iter_binaries(&self) -> impl Iterator<Item = Package> {
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            let iter = apt.call_method0(py, "iter_binaries").unwrap();
            PackageIterator(iter)
        })
    }

    fn iter_source_by_name(&self, name: &str) -> impl Iterator<Item = Source> {
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            let iter = apt
                .call_method1(py, "iter_source_by_name", (name,))
                .unwrap();
            SourceIterator(iter)
        })
    }

    fn iter_binary_by_name(&self, name: &str) -> impl Iterator<Item = Package> {
        Python::with_gil(|py| {
            let apt = self.to_object(py);
            let iter = apt
                .call_method1(py, "iter_binary_by_name", (name,))
                .unwrap();
            PackageIterator(iter)
        })
    }
}

pub struct LocalApt(PyObject);

impl Apt for LocalApt {}

impl ToPyObject for LocalApt {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl LocalApt {
    pub fn new(rootdir: Option<&std::path::Path>) -> Result<Self, Error> {
        Python::with_gil(|py| {
            let m = PyModule::import_bound(py, "breezy.plugins.debian.apt_repo")?;
            let apt = m.getattr("LocalApt")?;
            let apt = apt.call1((rootdir,))?;

            apt.call_method0("__enter__")?;
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
                .call_method1(py, "__exit__", (py.None(), py.None(), py.None()))
                .unwrap();
        });
    }
}

pub struct RemoteApt(PyObject);

impl ToPyObject for RemoteApt {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl RemoteApt {
    pub fn new(
        mirror_uri: &url::Url,
        distribution: Option<&str>,
        components: Option<Vec<String>>,
        key_path: Option<&std::path::Path>,
    ) -> Result<Self, Error> {
        Python::with_gil(|py| {
            let m = PyModule::import_bound(py, "breezy.plugins.debian.apt_repo")?;
            let apt = m.getattr("RemoteApt")?;
            let apt = apt.call1((mirror_uri.as_str(), distribution, components, key_path))?;
            apt.call_method0("__enter__")?;
            Ok(Self(apt.to_object(py)))
        })
    }
}

impl Apt for RemoteApt {}

impl Drop for RemoteApt {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "__exit__", (py.None(), py.None(), py.None()))
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
    fn test_local_apt() {
        // Note that LocalApt appears to crash if initialized
        // concurrently by other tests.
        let apt = LocalApt::new(None).unwrap();
        let package = apt.iter_binaries().next().unwrap();
        assert!(package.name().is_some());
        assert!(package.version().is_some());
        let source = apt.iter_sources().next().unwrap();
        assert!(source.package().is_some());
        let source = apt.iter_source_by_name("dpkg").next().unwrap();
        assert_eq!(source.package().unwrap(), "dpkg");
        let package = apt.iter_binary_by_name("dpkg").next().unwrap();
        assert_eq!(package.name().unwrap(), "dpkg");
    }
}
