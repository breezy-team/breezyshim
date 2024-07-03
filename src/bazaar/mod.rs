use pyo3::prelude::*;

pub mod tree;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileId(Vec<u8>);

impl Default for FileId {
    fn default() -> Self {
        Self::new()
    }
}

impl FileId {
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl From<&str> for FileId {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<String> for FileId {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<&[u8]> for FileId {
    fn from(s: &[u8]) -> Self {
        Self(s.to_vec())
    }
}

impl From<Vec<u8>> for FileId {
    fn from(s: Vec<u8>) -> Self {
        Self(s)
    }
}

impl From<FileId> for Vec<u8> {
    fn from(s: FileId) -> Self {
        s.0
    }
}

impl From<FileId> for String {
    fn from(s: FileId) -> Self {
        String::from_utf8(s.0).unwrap()
    }
}

impl pyo3::ToPyObject for FileId {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl pyo3::FromPyObject<'_> for FileId {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let bytes = ob.extract::<Vec<u8>>()?;
        Ok(Self(bytes))
    }
}

impl pyo3::IntoPy<pyo3::PyObject> for FileId {
    fn into_py(self, py: Python) -> pyo3::PyObject {
        self.0.to_object(py)
    }
}

pub fn gen_revision_id(username: &str, timestamp: Option<usize>) -> Vec<u8> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.bzr.generate_ids").unwrap();
        let gen_revision_id = m.getattr("gen_revision_id").unwrap();
        gen_revision_id.call1((username, timestamp)).unwrap().extract().unwrap()
    })
}

#[test]
fn test_gen_revision_id() {
    gen_revision_id("user", None);
}

pub fn gen_file_id(name: &str) -> Vec<u8> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.bzr.generate_ids").unwrap();
        let gen_file_id = m.getattr("gen_file_id").unwrap();
        gen_file_id.call1((name, )).unwrap().extract().unwrap()
    })
}

#[test]
fn test_file_id() {
    gen_file_id("somename");
}
