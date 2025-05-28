//! Transport module
use crate::error::Error;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::{Path, PathBuf};

crate::wrapped_py!(Transport);

impl Transport {
    pub fn base(&self) -> url::Url {
        pyo3::Python::with_gil(|py| {
            self.to_object(py)
                .getattr(py, "base")
                .unwrap()
                .extract::<String>(py)
                .unwrap()
                .parse()
                .unwrap()
        })
    }

    pub fn is_local(&self) -> bool {
        pyo3::import_exception!(breezy.errors, NotLocalUrl);
        pyo3::Python::with_gil(|py| {
            self.0
                .call_method1(py, "local_abspath", (".",))
                .map(|_| true)
                .or_else(|e| {
                    if e.is_instance_of::<NotLocalUrl>(py) {
                        Ok::<_, PyErr>(false)
                    } else {
                        panic!("Unexpected error: {:?}", e)
                    }
                })
                .unwrap()
        })
    }

    pub fn local_abspath(&self, path: &Path) -> Result<PathBuf, Error> {
        pyo3::Python::with_gil(|py| {
            Ok(self
                .0
                .call_method1(py, "local_abspath", (path,))
                .unwrap()
                .extract::<PathBuf>(py)
                .unwrap())
        })
    }

    pub fn has(&self, path: &str) -> Result<bool, Error> {
        pyo3::Python::with_gil(|py| {
            Ok(self
                .0
                .call_method1(py, "has", (path,))?
                .extract::<bool>(py)
                .unwrap())
        })
    }

    pub fn ensure_base(&self) -> Result<(), Error> {
        pyo3::Python::with_gil(|py| {
            self.0.call_method0(py, "ensure_base")?;
            Ok(())
        })
    }

    pub fn create_prefix(&self) -> Result<(), Error> {
        pyo3::Python::with_gil(|py| {
            self.0.call_method0(py, "create_prefix")?;
            Ok(())
        })
    }

    pub fn clone(&self, path: &str) -> Result<Transport, Error> {
        pyo3::Python::with_gil(|py| {
            let o = self.0.call_method1(py, "clone", (path,))?;
            Ok(Transport(o))
        })
    }
}

pub fn get_transport(
    url: &url::Url,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<Transport, Error> {
    pyo3::Python::with_gil(|py| {
        let urlutils = py.import("breezy.transport").unwrap();
        let kwargs = PyDict::new(py);
        kwargs.set_item(
            "possible_transports",
            possible_transports.map(|t| {
                t.iter()
                    .map(|t| t.0.clone_ref(py))
                    .collect::<Vec<PyObject>>()
            }),
        )?;
        let o = urlutils.call_method("get_transport", (url.to_string(),), Some(&kwargs))?;
        Ok(Transport(o.unbind()))
    })
}
