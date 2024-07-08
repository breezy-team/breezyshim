use crate::controldir::Prober;
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

pub struct SmartHgProber(PyObject);

impl SmartHgProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import_bound("breezy.plugins.hg") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.hg");
                    }
                }
            };
            let prober = m
                .getattr("SmartHgProber")
                .expect("Failed to get SmartHgProber");
            Some(Self(prober.to_object(py)))
        })
    }
}

impl FromPyObject<'_> for SmartHgProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_object(obj.py())))
    }
}

impl ToPyObject for SmartHgProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Prober for SmartHgProber {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_hg_prober() {
        let _ = SmartHgProber::new();
    }
}
