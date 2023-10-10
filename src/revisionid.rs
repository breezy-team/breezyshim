use pyo3::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct RevisionId(Vec<u8>);

impl std::fmt::Debug for RevisionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = String::from_utf8(self.0.clone()).unwrap();
        write!(f, "{}", s)
    }
}

impl RevisionId {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_null(&self) -> bool {
        self.0 == NULL_REVISION
    }

    pub fn is_reserved(&self) -> bool {
        self.0.starts_with(b":")
    }

    pub fn null() -> Self {
        Self(NULL_REVISION.to_vec())
    }
}

impl From<Vec<u8>> for RevisionId {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Serialize for RevisionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(String::from_utf8(self.0.clone()).unwrap().as_str())
    }
}

impl<'de> Deserialize<'de> for RevisionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|s| Self(s.into_bytes()))
    }
}

impl FromPyObject<'_> for RevisionId {
    fn extract(ob: &'_ PyAny) -> PyResult<Self> {
        let bytes = ob.extract::<Vec<u8>>()?;
        Ok(Self(bytes))
    }
}

impl ToPyObject for &RevisionId {
    fn to_object(&self, py: Python) -> PyObject {
        pyo3::types::PyBytes::new(py, &self.0).to_object(py)
    }
}

impl IntoPy<PyObject> for RevisionId {
    fn into_py(self, py: Python) -> PyObject {
        pyo3::types::PyBytes::new(py, self.0.as_slice()).to_object(py)
    }
}

impl std::fmt::Display for RevisionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = String::from_utf8(self.0.clone()).unwrap();
        write!(f, "{}", s)
    }
}

pub const CURRENT_REVISION: &[u8] = b"current:";
pub const NULL_REVISION: &[u8] = b"null:";
