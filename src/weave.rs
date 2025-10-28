//! Weave versioned files implementation

#![allow(missing_docs)]

use crate::error::Error;
use pyo3::prelude::*;

pub struct WeaveFile(Py<PyAny>);

impl WeaveFile {
    pub fn new(py_obj: Py<PyAny>) -> Self {
        Self(py_obj)
    }

    pub fn from_transport(
        py: Python,
        transport: &crate::transport::Transport,
        file_name: &str,
        mode: Option<&str>,
        create: bool,
    ) -> PyResult<Self> {
        let weave_mod = py.import("breezy.bzr.weave")?;
        let weave_cls = weave_mod.getattr("WeaveFile")?;

        let kwargs = pyo3::types::PyDict::new(py);
        if let Some(m) = mode {
            kwargs.set_item("mode", m)?;
        }
        kwargs.set_item("create", create)?;

        let obj = weave_cls.call(
            (file_name, transport.as_pyobject().clone_ref(py)),
            Some(&kwargs),
        )?;
        Ok(WeaveFile(obj.unbind()))
    }

    pub fn add_lines(
        &self,
        version_id: &str,
        parents: Vec<&str>,
        lines: Vec<&str>,
    ) -> Result<(), Error> {
        Python::attach(|py| {
            let parents_list = pyo3::types::PyList::new(py, parents)?;
            let lines_list = pyo3::types::PyList::new(py, lines)?;

            self.0
                .call_method1(py, "add_lines", (version_id, parents_list, lines_list))?;
            Ok(())
        })
    }

    pub fn get_lines(&self, version_id: &str) -> Result<Vec<String>, Error> {
        Python::attach(|py| {
            let result = self.0.call_method1(py, "get_lines", (version_id,))?;
            let lines_list = result
                .cast_bound::<pyo3::types::PyList>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected list"))?;

            let mut lines = Vec::new();
            for line in lines_list {
                lines.push(line.extract::<String>()?);
            }
            Ok(lines)
        })
    }

    pub fn get_ancestry(&self, version_ids: Vec<&str>) -> Result<Vec<String>, Error> {
        Python::attach(|py| {
            let ids_list = pyo3::types::PyList::new(py, version_ids)?;
            let result = self.0.call_method1(py, "get_ancestry", (ids_list,))?;
            let ancestry_list = result
                .cast_bound::<pyo3::types::PyList>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected list"))?;

            let mut ancestry = Vec::new();
            for id in ancestry_list {
                ancestry.push(id.extract::<String>()?);
            }
            Ok(ancestry)
        })
    }

    pub fn get_parent_map(
        &self,
        version_ids: Option<Vec<&str>>,
    ) -> Result<std::collections::HashMap<String, Vec<String>>, Error> {
        Python::attach(|py| {
            let ids_arg: Py<PyAny> = if let Some(ids) = version_ids {
                pyo3::types::PyList::new(py, ids)?.unbind().into()
            } else {
                py.None()
            };

            let result = self.0.call_method1(py, "get_parent_map", (ids_arg,))?;
            let parent_dict = result
                .cast_bound::<pyo3::types::PyDict>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected dict"))?;

            let mut parent_map = std::collections::HashMap::new();
            for (key, value) in parent_dict {
                let version_id = key.extract::<String>()?;
                let parents_list = value
                    .cast::<pyo3::types::PyList>()
                    .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected list"))?;

                let mut parents = Vec::new();
                for parent in parents_list {
                    parents.push(parent.extract::<String>()?);
                }
                parent_map.insert(version_id, parents);
            }
            Ok(parent_map)
        })
    }
}

impl Clone for WeaveFile {
    fn clone(&self) -> Self {
        Python::attach(|py| WeaveFile(self.0.clone_ref(py)))
    }
}

impl<'py> IntoPyObject<'py> for WeaveFile {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for WeaveFile {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(WeaveFile(ob.to_owned().unbind()))
    }
}

pub struct Weave(Py<PyAny>);

impl Weave {
    pub fn new(py_obj: Py<PyAny>) -> Self {
        Self(py_obj)
    }

    pub fn new_empty(py: Python) -> PyResult<Self> {
        let weave_mod = py.import("breezy.bzr.weave")?;
        let weave_cls = weave_mod.getattr("Weave")?;
        let obj = weave_cls.call0()?;
        Ok(Weave(obj.unbind()))
    }

    pub fn add_lines(&self, name: &str, parents: Vec<&str>, text: Vec<&str>) -> Result<(), Error> {
        Python::attach(|py| {
            let parents_list = pyo3::types::PyList::new(py, parents)?;
            // Convert text to bytes as required by weave
            let text_bytes: Vec<_> = text
                .iter()
                .map(|line| pyo3::types::PyBytes::new(py, line.as_bytes()))
                .collect();
            let text_list = pyo3::types::PyList::new(py, text_bytes)?;

            self.0
                .call_method1(py, "add_lines", (name, parents_list, text_list))?;
            Ok(())
        })
    }

    pub fn get_text(&self, name: &str) -> Result<Vec<String>, Error> {
        Python::attach(|py| {
            let result = self.0.call_method1(py, "get_text", (name,))?;
            let bytes_result = result
                .cast_bound::<pyo3::types::PyBytes>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected bytes"))?;

            let text = std::str::from_utf8(bytes_result.as_bytes())
                .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid UTF-8"))?;

            // Split into lines
            let lines: Vec<String> = text.lines().map(|line| format!("{}\n", line)).collect();
            Ok(lines)
        })
    }

    pub fn get_ancestry(&self, names: Vec<&str>) -> Result<Vec<String>, Error> {
        Python::attach(|py| {
            let names_list = pyo3::types::PyList::new(py, names)?;
            let result = self.0.call_method1(py, "get_ancestry", (names_list,))?;
            let ancestry_set = result
                .cast_bound::<pyo3::types::PySet>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected set"))?;

            let mut ancestry = Vec::new();
            for name in ancestry_set {
                ancestry.push(name.extract::<String>()?);
            }
            Ok(ancestry)
        })
    }

    pub fn numversions(&self) -> Result<usize, Error> {
        Python::attach(|py| {
            let result = self.0.call_method0(py, "num_versions")?;
            Ok(result.extract::<usize>(py)?)
        })
    }
}

impl Clone for Weave {
    fn clone(&self) -> Self {
        Python::attach(|py| Weave(self.0.clone_ref(py)))
    }
}

impl<'py> IntoPyObject<'py> for Weave {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Weave {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Weave(ob.to_owned().unbind()))
    }
}
