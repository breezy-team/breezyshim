use crate::error::Error;
use debversion::Version;
use pyo3::prelude::*;

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
            Ok(Self(apt.to_object(py)))
        })
    }
}

impl Default for LocalApt {
    fn default() -> Self {
        LocalApt::new(None).unwrap()
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
            Ok(Self(apt.to_object(py)))
        })
    }
}

impl Apt for RemoteApt {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_local_apt() {
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
}
