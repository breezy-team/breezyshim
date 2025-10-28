//! Versioned files API for storing file content history

#![allow(missing_docs)]

use crate::error::Error;
use crate::graph::{Key, KnownGraph};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyIterator, PyList, PyTuple};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FulltextContentFactory {
    pub sha1: Option<String>,
    pub storage_kind: String,
    pub key: Key,
    pub parents: Option<Vec<Key>>,
}

impl FulltextContentFactory {
    pub fn new(
        sha1: Option<String>,
        storage_kind: String,
        key: Key,
        parents: Option<Vec<Key>>,
    ) -> Self {
        Self {
            sha1,
            storage_kind,
            key,
            parents,
        }
    }
}

impl<'py> IntoPyObject<'py> for FulltextContentFactory {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let factory_mod = py.import("breezy.bzr.versionedfile")?;
        let factory_cls = factory_mod.getattr("FulltextContentFactory")?;

        let parents_py = if let Some(parents) = self.parents {
            let parent_tuples: Vec<_> = parents
                .into_iter()
                .map(|p| p.into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            Some(PyTuple::new(py, parent_tuples)?)
        } else {
            None
        };

        let kwargs = PyDict::new(py);
        if let Some(sha1) = self.sha1 {
            kwargs.set_item("sha1", PyBytes::new(py, sha1.as_bytes()))?;
        }
        kwargs.set_item("storage_kind", self.storage_kind)?;
        kwargs.set_item("key", self.key.into_pyobject(py)?)?;
        if let Some(parents) = parents_py {
            kwargs.set_item("parents", parents)?;
        }

        factory_cls.call((), Some(&kwargs))
    }
}

#[derive(Debug, Clone)]
pub struct AbsentContentFactory {
    pub key: Key,
    pub parents: Vec<Key>,
}

impl AbsentContentFactory {
    pub fn new(key: Key, parents: Vec<Key>) -> Self {
        Self { key, parents }
    }
}

impl<'py> IntoPyObject<'py> for AbsentContentFactory {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let factory_mod = py.import("breezy.bzr.versionedfile")?;
        let factory_cls = factory_mod.getattr("AbsentContentFactory")?;

        let parent_tuples: Vec<_> = self
            .parents
            .into_iter()
            .map(|p| p.into_pyobject(py))
            .collect::<Result<Vec<_>, _>>()?;
        let parents_py = PyTuple::new(py, parent_tuples)?;

        factory_cls.call1((self.key.into_pyobject(py)?, parents_py))
    }
}

pub trait VersionedFiles: Clone + Send + Sync {
    fn add_lines(
        &self,
        key: &Key,
        parents: &[Key],
        lines: Vec<&str>,
        parent_texts: Option<HashMap<Key, Vec<String>>>,
        left_matching_blocks: Option<Vec<(usize, usize, usize)>>,
        nostore_sha: Option<&str>,
        random_id: bool,
        check_content: bool,
    ) -> Result<(String, usize), Error>;

    fn get_record_stream(
        &self,
        keys: Vec<Key>,
        ordering: RecordOrdering,
        include_delta_closure: bool,
    ) -> Result<RecordStream, Error>;

    fn get_sha1s(&self, keys: Vec<Key>) -> Result<HashMap<Key, String>, Error>;

    fn insert_record_stream(&self, stream: RecordStream) -> Result<(), Error>;

    fn keys(&self) -> Result<Vec<Key>, Error>;

    fn make_mpdiffs(&self, keys: Vec<Key>) -> Result<Vec<MultiParentDiff>, Error>;

    fn get_parent_map(&self, keys: Vec<Key>) -> Result<HashMap<Key, Vec<Key>>, Error>;

    fn get_known_graph_ancestry(&self, keys: Vec<Key>) -> Result<KnownGraph, Error>;

    fn get_record_stream_for_keys(
        &self,
        keys: Vec<Key>,
        ordering: RecordOrdering,
    ) -> Result<RecordStream, Error> {
        self.get_record_stream(keys, ordering, false)
    }

    fn has_key(&self, key: &Key) -> Result<bool, Error> {
        let parent_map = self.get_parent_map(vec![key.clone()])?;
        Ok(parent_map.contains_key(key))
    }
}

pub trait PyVersionedFiles: VersionedFiles {
    fn to_object(&self, py: Python) -> PyObject;
}

impl<T: PyVersionedFiles> VersionedFiles for T {
    fn add_lines(
        &self,
        key: &Key,
        parents: &[Key],
        lines: Vec<&str>,
        parent_texts: Option<HashMap<Key, Vec<String>>>,
        left_matching_blocks: Option<Vec<(usize, usize, usize)>>,
        nostore_sha: Option<&str>,
        random_id: bool,
        check_content: bool,
    ) -> Result<(String, usize), Error> {
        Python::with_gil(|py| {
            let parents_py: Vec<_> = parents
                .iter()
                .map(|p| p.clone().into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            let parents_tuple = PyTuple::new(py, parents_py)?;

            let lines_py = PyList::new(py, lines)?;

            let kwargs = PyDict::new(py);

            if let Some(parent_texts) = parent_texts {
                let parent_texts_dict = PyDict::new(py);
                for (k, v) in parent_texts {
                    let lines_list = PyList::new(py, v)?;
                    parent_texts_dict.set_item(k.into_pyobject(py)?, lines_list)?;
                }
                kwargs.set_item("parent_texts", parent_texts_dict)?;
            }

            if let Some(blocks) = left_matching_blocks {
                let blocks_list = PyList::new(py, blocks.iter().map(|(a, b, c)| (*a, *b, *c)))?;
                kwargs.set_item("left_matching_blocks", blocks_list)?;
            }

            if let Some(sha) = nostore_sha {
                kwargs.set_item("nostore_sha", PyBytes::new(py, sha.as_bytes()))?;
            }

            kwargs.set_item("random_id", random_id)?;
            kwargs.set_item("check_content", check_content)?;

            let result = self.to_object(py).call_method(
                py,
                "add_lines",
                (key.clone().into_pyobject(py)?, parents_tuple, lines_py),
                Some(&kwargs),
            )?;

            let tuple = result
                .downcast_bound::<PyTuple>(py)
                .map_err(|_| PyValueError::new_err("Expected tuple"))?;
            let item0 = tuple.get_item(0)?;
            let sha1_bytes = item0
                .downcast::<PyBytes>()
                .map_err(|_| PyValueError::new_err("Expected bytes"))?;
            let sha1 = std::str::from_utf8(sha1_bytes.as_bytes())
                .map_err(|_| PyValueError::new_err("Invalid UTF-8 in SHA1"))?
                .to_string();
            let length = tuple.get_item(1)?.extract::<usize>()?;

            Ok((sha1, length))
        })
    }

    fn get_record_stream(
        &self,
        keys: Vec<Key>,
        ordering: RecordOrdering,
        include_delta_closure: bool,
    ) -> Result<RecordStream, Error> {
        Python::with_gil(|py| {
            let keys_py: Vec<_> = keys
                .into_iter()
                .map(|k| k.into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            let keys_list = PyList::new(py, keys_py)?;

            let ordering_str = match ordering {
                RecordOrdering::Unordered => "unordered",
                RecordOrdering::Topological => "topological",
                RecordOrdering::GroupedByKey => "groupcompress",
            };

            let stream_obj = self.to_object(py).call_method1(
                py,
                "get_record_stream",
                (keys_list, ordering_str, include_delta_closure),
            )?;

            Ok(RecordStream(stream_obj))
        })
    }

    fn get_sha1s(&self, keys: Vec<Key>) -> Result<HashMap<Key, String>, Error> {
        Python::with_gil(|py| {
            let keys_py: Vec<_> = keys
                .into_iter()
                .map(|k| k.into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            let keys_list = PyList::new(py, keys_py)?;

            let result_dict = self
                .to_object(py)
                .call_method1(py, "get_sha1s", (keys_list,))?;

            let dict = result_dict
                .downcast_bound::<PyDict>(py)
                .map_err(|_| PyValueError::new_err("Expected dict"))?;
            let mut sha1s = HashMap::new();

            for (key_py, sha_py) in dict {
                let key = key_py.extract::<Key>()?;
                let sha_bytes = sha_py
                    .downcast::<PyBytes>()
                    .map_err(|_| PyValueError::new_err("Expected bytes"))?;
                let sha = std::str::from_utf8(sha_bytes.as_bytes())
                    .map_err(|_| PyValueError::new_err("Invalid UTF-8 in SHA1"))?
                    .to_string();
                sha1s.insert(key, sha);
            }

            Ok(sha1s)
        })
    }

    fn insert_record_stream(&self, stream: RecordStream) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "insert_record_stream", (stream.0,))?;
            Ok(())
        })
    }

    fn keys(&self) -> Result<Vec<Key>, Error> {
        Python::with_gil(|py| {
            let keys_iter = self.to_object(py).call_method0(py, "keys")?;

            let mut keys = Vec::new();
            for key_py in keys_iter
                .downcast_bound::<PyIterator>(py)
                .map_err(|_| PyValueError::new_err("Expected iterator"))?
            {
                let key = key_py?.extract::<Key>()?;
                keys.push(key);
            }

            Ok(keys)
        })
    }

    fn make_mpdiffs(&self, keys: Vec<Key>) -> Result<Vec<MultiParentDiff>, Error> {
        Python::with_gil(|py| {
            let keys_py: Vec<_> = keys
                .into_iter()
                .map(|k| k.into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            let keys_list = PyList::new(py, keys_py)?;

            let result = self
                .to_object(py)
                .call_method1(py, "make_mpdiffs", (keys_list,))?;

            let mut diffs = Vec::new();
            for diff_py in result
                .downcast_bound::<PyIterator>(py)
                .map_err(|_| PyValueError::new_err("Expected iterator"))?
            {
                let diff = diff_py?.extract::<MultiParentDiff>()?;
                diffs.push(diff);
            }

            Ok(diffs)
        })
    }

    fn get_parent_map(&self, keys: Vec<Key>) -> Result<HashMap<Key, Vec<Key>>, Error> {
        Python::with_gil(|py| {
            let keys_py: Vec<_> = keys
                .into_iter()
                .map(|k| k.into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            let keys_list = PyList::new(py, keys_py)?;

            let result_dict =
                self.to_object(py)
                    .call_method1(py, "get_parent_map", (keys_list,))?;

            let dict = result_dict
                .downcast_bound::<PyDict>(py)
                .map_err(|_| PyValueError::new_err("Expected dict"))?;
            let mut parent_map = HashMap::new();

            for (key_py, parents_py) in dict {
                let key = key_py.extract::<Key>()?;
                let mut parents = Vec::new();
                for parent_py in parents_py
                    .downcast::<PyTuple>()
                    .map_err(|_| PyValueError::new_err("Expected tuple"))?
                {
                    let parent = parent_py.extract::<Key>()?;
                    parents.push(parent);
                }
                parent_map.insert(key, parents);
            }

            Ok(parent_map)
        })
    }

    fn get_known_graph_ancestry(&self, keys: Vec<Key>) -> Result<KnownGraph, Error> {
        Python::with_gil(|py| {
            let keys_py: Vec<_> = keys
                .into_iter()
                .map(|k| k.into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            let keys_list = PyList::new(py, keys_py)?;

            let graph_obj =
                self.to_object(py)
                    .call_method1(py, "get_known_graph_ancestry", (keys_list,))?;

            Ok(KnownGraph::new(graph_obj))
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RecordOrdering {
    Unordered,
    Topological,
    GroupedByKey,
}

pub struct RecordStream(PyObject);

impl RecordStream {
    pub fn iter(&self) -> Result<RecordStreamIterator, Error> {
        Python::with_gil(|py| {
            let iter = self.0.call_method0(py, "__iter__")?;
            Ok(RecordStreamIterator(iter))
        })
    }
}

pub struct RecordStreamIterator(PyObject);

impl Iterator for RecordStreamIterator {
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| match self.0.call_method0(py, "__next__") {
            Ok(record_py) => Some(record_py.bind(py).extract::<Record>().map_err(Into::into)),
            Err(e) if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) => None,
            Err(e) => Some(Err(e.into())),
        })
    }
}

#[derive(Debug)]
pub struct Record {
    pub key: Key,
    pub storage_kind: String,
    pub sha1: Option<String>,
    pub parents: Vec<Key>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for Record {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let key = ob.getattr("key")?.extract::<Key>()?;
        let storage_kind = ob.getattr("storage_kind")?.extract::<String>()?;

        let sha1 = if let Ok(sha_bytes) = ob.getattr("sha1") {
            if !sha_bytes.is_none() {
                let bytes = sha_bytes
                    .downcast::<PyBytes>()
                    .map_err(|_| PyValueError::new_err("Expected bytes"))?;
                Some(
                    std::str::from_utf8(bytes.as_bytes())
                        .map_err(|_| PyValueError::new_err("Invalid UTF-8 in SHA1"))?
                        .to_string(),
                )
            } else {
                None
            }
        } else {
            None
        };

        let parents = ob.getattr("parents")?.extract::<Vec<Key>>()?;

        Ok(Record {
            key,
            storage_kind,
            sha1,
            parents,
        })
    }
}

#[derive(Debug)]
pub struct MultiParentDiff {
    pub key: Key,
    pub parents: Vec<Key>,
    pub hunks: Vec<DiffHunk>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for MultiParentDiff {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let tuple = ob
            .downcast::<PyTuple>()
            .map_err(|_| PyValueError::new_err("Expected tuple"))?;
        let key = tuple.get_item(0)?.extract::<Key>()?;
        let parents = tuple.get_item(1)?.extract::<Vec<Key>>()?;

        let hunks_py = tuple.get_item(2)?;
        let mut hunks = Vec::new();
        for hunk_py in hunks_py
            .downcast::<PyList>()
            .map_err(|_| PyValueError::new_err("Expected list"))?
        {
            hunks.push(hunk_py.extract::<DiffHunk>()?);
        }

        Ok(MultiParentDiff {
            key,
            parents,
            hunks,
        })
    }
}

#[derive(Debug)]
pub enum DiffHunk {
    NewText(Vec<String>),
    ParentText {
        parent: usize,
        start: usize,
        end: usize,
    },
}

impl<'a, 'py> FromPyObject<'a, 'py> for DiffHunk {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let tuple = ob
            .downcast::<PyTuple>()
            .map_err(|_| PyValueError::new_err("Expected tuple"))?;
        let hunk_type = tuple.get_item(0)?.extract::<String>()?;

        match hunk_type.as_str() {
            "new" => {
                let lines = tuple.get_item(1)?.extract::<Vec<String>>()?;
                Ok(DiffHunk::NewText(lines))
            }
            "parent" => {
                let parent = tuple.get_item(1)?.extract::<usize>()?;
                let start = tuple.get_item(2)?.extract::<usize>()?;
                let end = tuple.get_item(3)?.extract::<usize>()?;
                Ok(DiffHunk::ParentText { parent, start, end })
            }
            _ => Err(PyValueError::new_err(format!(
                "Unknown hunk type: {}",
                hunk_type
            ))),
        }
    }
}

pub struct GenericVersionedFiles(PyObject);

impl GenericVersionedFiles {
    pub fn new(py_obj: PyObject) -> Self {
        Self(py_obj)
    }
}

impl Clone for GenericVersionedFiles {
    fn clone(&self) -> Self {
        Python::with_gil(|py| GenericVersionedFiles(self.0.clone_ref(py)))
    }
}

impl PyVersionedFiles for GenericVersionedFiles {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl<'py> IntoPyObject<'py> for GenericVersionedFiles {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for GenericVersionedFiles {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GenericVersionedFiles(ob.to_owned().unbind()))
    }
}

#[cfg(test)]
mod tests;
