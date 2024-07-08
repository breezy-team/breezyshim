use crate::controldir::Prober;
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

pub struct RemoteCVSProber(PyObject);

impl RemoteCVSProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import_bound("breezy.plugins.cvs") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.cvs");
                    }
                }
            };
            let cvsprober = m.getattr("CVSProber").expect("Failed to get CVSProber");
            Some(Self(cvsprober.to_object(py)))
        })
    }
}

impl FromPyObject<'_> for RemoteCVSProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_object(obj.py())))
    }
}

impl ToPyObject for RemoteCVSProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Prober for RemoteCVSProber {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_cvs_prober() {
        let _ = RemoteCVSProber::new();
    }
}
