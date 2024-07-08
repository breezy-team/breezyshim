use crate::controldir::Prober;
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

pub struct RemoteGitProber(PyObject);

impl RemoteGitProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import_bound("breezy.git") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.git");
                    }
                }
            };
            let prober = m
                .getattr("RemoteGitProber")
                .expect("Failed to get GitProber");
            Some(Self(prober.to_object(py)))
        })
    }
}

impl FromPyObject<'_> for RemoteGitProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_object(obj.py())))
    }
}

impl ToPyObject for RemoteGitProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Prober for RemoteGitProber {}
