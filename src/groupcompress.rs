//! Group compression versioned files implementation

#![allow(missing_docs)]

use crate::error::Error;
use crate::graph::Key;
use crate::versionedfiles::PyVersionedFiles;
use pyo3::prelude::*;

pub struct GroupCompressVersionedFiles(Py<PyAny>);

impl GroupCompressVersionedFiles {
    pub fn new(py_obj: Py<PyAny>) -> Self {
        Self(py_obj)
    }

    pub fn from_transport(
        py: Python,
        transport: &crate::transport::Transport,
        index: Option<Py<PyAny>>,
        delta: bool,
        _is_locked: impl Fn() -> bool + 'static,
        track_external_parent_refs: bool,
        track_anomalous_cross_references: bool,
        use_chk_index: bool,
    ) -> PyResult<Self> {
        let gc_mod = py.import("breezy.bzr.groupcompress")?;
        let gcvf_cls = gc_mod.getattr("GroupCompressVersionedFiles")?;

        let kwargs = pyo3::types::PyDict::new(py);
        kwargs.set_item("delta", delta)?;
        // For testing, we can pass None for is_locked and let Python handle it
        kwargs.set_item("is_locked", py.None())?;
        kwargs.set_item("track_external_parent_refs", track_external_parent_refs)?;
        kwargs.set_item(
            "track_anomalous_cross_references",
            track_anomalous_cross_references,
        )?;
        kwargs.set_item("use_chk_index", use_chk_index)?;

        let transport_obj = transport.as_pyobject().clone_ref(py);
        let args = if let Some(idx) = index {
            (transport_obj, idx)
        } else {
            (transport_obj, py.None())
        };

        let obj = gcvf_cls.call(args, Some(&kwargs))?;
        Ok(GroupCompressVersionedFiles(obj.unbind()))
    }

    pub fn without_fallbacks(&self) -> Result<Self, Error> {
        Python::attach(|py| {
            let obj = self.0.call_method0(py, "without_fallbacks")?;
            Ok(GroupCompressVersionedFiles(obj))
        })
    }

    pub fn get_missing_compression_parent_keys(&self) -> Result<Vec<Key>, Error> {
        Python::attach(|py| {
            let result = self
                .0
                .call_method0(py, "get_missing_compression_parent_keys")?;
            let keys_iter = result
                .cast_bound::<pyo3::types::PyIterator>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected iterator"))?;

            let mut keys = Vec::new();
            for key_py in keys_iter {
                let key = key_py?.extract::<Key>()?;
                keys.push(key);
            }
            Ok(keys)
        })
    }
}

impl Clone for GroupCompressVersionedFiles {
    fn clone(&self) -> Self {
        Python::attach(|py| GroupCompressVersionedFiles(self.0.clone_ref(py)))
    }
}

impl PyVersionedFiles for GroupCompressVersionedFiles {
    fn to_object(&self, py: Python) -> Py<PyAny> {
        self.0.clone_ref(py)
    }
}

impl<'py> IntoPyObject<'py> for GroupCompressVersionedFiles {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for GroupCompressVersionedFiles {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GroupCompressVersionedFiles(ob.to_owned().unbind()))
    }
}

pub struct GroupCompressor(Py<PyAny>);

impl GroupCompressor {
    pub fn new(py: Python) -> PyResult<Self> {
        let gc_mod = py.import("breezy.bzr.groupcompress")?;
        let gc_cls = gc_mod.getattr("GroupCompressor")?;
        let obj = gc_cls.call0()?;
        Ok(GroupCompressor(obj.unbind()))
    }

    /// Create a test instance that properly handles chunks
    #[cfg(test)]
    pub fn new_for_testing(py: Python) -> PyResult<Self> {
        Self::new(py)
    }

    pub fn compress(
        &self,
        key: &Key,
        lines: Vec<&str>,
        expected_sha: Option<&str>,
        soft: bool,
    ) -> Result<(Option<String>, usize, Option<CompressorRecord>), Error> {
        Python::attach(|py| {
            // Convert lines to bytes as GroupCompressor expects bytes
            let lines_bytes: Vec<_> = lines
                .iter()
                .map(|line| pyo3::types::PyBytes::new(py, line.as_bytes()))
                .collect();
            let lines_list = pyo3::types::PyList::new(py, &lines_bytes)?;

            // Calculate total length
            let length: usize = lines.iter().map(|l| l.len()).sum();

            let expected_sha_arg: Py<PyAny> = if let Some(sha) = expected_sha {
                pyo3::types::PyBytes::new(py, sha.as_bytes())
                    .unbind()
                    .into()
            } else {
                py.None()
            };

            let result = self.0.call_method1(
                py,
                "compress",
                (
                    key.clone().into_pyobject(py)?,
                    lines_list,
                    length,
                    expected_sha_arg,
                    soft,
                ),
            )?;

            let tuple = result
                .cast_bound::<pyo3::types::PyTuple>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected tuple"))?;

            // compress returns (sha1, start_offset, end_offset, type)
            let sha1 = if tuple.get_item(0)?.is_none() {
                None
            } else {
                let item0 = tuple.get_item(0)?;
                let sha_bytes = item0
                    .cast::<pyo3::types::PyBytes>()
                    .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected bytes"))?;
                Some(
                    std::str::from_utf8(sha_bytes.as_bytes())
                        .map_err(|_| {
                            pyo3::exceptions::PyValueError::new_err("Invalid UTF-8 in SHA1")
                        })?
                        .to_string(),
                )
            };

            let _start_offset = tuple.get_item(1)?.extract::<usize>()?;
            let _end_offset = tuple.get_item(2)?.extract::<usize>()?;
            let _type = tuple.get_item(3)?.extract::<String>()?;

            // GroupCompressor doesn't return a record from compress, only from flush
            // Return the input length since that's what was compressed
            Ok((sha1, length, None))
        })
    }

    pub fn flush(&self) -> Result<CompressorRecord, Error> {
        Python::attach(|py| {
            let record = self.0.call_method0(py, "flush")?;
            Ok(CompressorRecord(record))
        })
    }
}

impl Clone for GroupCompressor {
    fn clone(&self) -> Self {
        Python::attach(|py| GroupCompressor(self.0.clone_ref(py)))
    }
}

pub struct CompressorRecord(Py<PyAny>);

impl CompressorRecord {
    pub fn to_chunks(&self) -> Result<(usize, Vec<Vec<u8>>), Error> {
        Python::attach(|py| {
            let result = self.0.call_method0(py, "to_chunks")?;
            let tuple = result
                .cast_bound::<pyo3::types::PyTuple>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected tuple"))?;

            let total_bytes = tuple.get_item(0)?.extract::<usize>()?;
            let chunks_item = tuple.get_item(1)?;
            let chunks_list = chunks_item
                .cast::<pyo3::types::PyList>()
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected list"))?;

            let mut chunks = Vec::new();
            for chunk_py in chunks_list {
                let chunk_bytes = chunk_py
                    .cast::<pyo3::types::PyBytes>()
                    .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected bytes"))?;
                chunks.push(chunk_bytes.as_bytes().to_vec());
            }
            Ok((total_bytes, chunks))
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        Python::attach(|py| {
            let result = self.0.call_method0(py, "to_bytes")?;
            let bytes = result
                .cast_bound::<pyo3::types::PyBytes>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected bytes"))?;
            Ok(bytes.as_bytes().to_vec())
        })
    }
}

impl Clone for CompressorRecord {
    fn clone(&self) -> Self {
        Python::attach(|py| CompressorRecord(self.0.clone_ref(py)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_compressor_basic() {
        crate::init();
        crate::init_bzr();

        pyo3::Python::attach(|py| {
            let compressor = GroupCompressor::new_for_testing(py).unwrap();

            let key = Key::from(vec!["file1".to_string()]);
            let lines = vec!["line1\n", "line2\n", "line3\n"];
            let expected_length: usize = lines.iter().map(|l| l.len()).sum();

            let (sha1, length, record) = compressor.compress(&key, lines, None, false).unwrap();

            assert!(sha1.is_some());
            assert_eq!(length, expected_length);
            assert!(record.is_none()); // compress doesn't return a record
        });
    }

    #[test]
    fn test_group_compressor_simple() {
        crate::init();
        crate::init_bzr();

        pyo3::Python::attach(|py| {
            let compressor = GroupCompressor::new_for_testing(py).unwrap();

            let key = Key::from(vec!["file2".to_string()]);
            let content = "test content\n";
            let lines = vec![content];

            let (sha1, length, record) = compressor.compress(&key, lines, None, false).unwrap();

            assert!(sha1.is_some());
            assert_eq!(length, content.len());
            assert!(record.is_none()); // compress doesn't return a record

            // Test flush to get the record
            let flush_record = compressor.flush().unwrap();
            let (total_bytes, chunks) = flush_record.to_chunks().unwrap();
            assert!(total_bytes > 0);
            assert!(!chunks.is_empty());
        });
    }
}
