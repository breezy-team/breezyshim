use pyo3::prelude::*;
use pyo3::types::PyDict;

pub struct Transport(PyObject);

impl Transport {
    pub fn new(obj: PyObject) -> Self {
        Transport(obj)
    }
}

impl FromPyObject<'_> for Transport {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        Ok(Transport(obj.to_object(obj.py())))
    }
}

impl ToPyObject for Transport {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

pub fn get_transport(
    url: &url::Url,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Transport {
    pyo3::Python::with_gil(|py| {
        let urlutils = py.import("breezy.transport").unwrap();
        let kwargs = PyDict::new(py);
        kwargs
            .set_item(
                "possible_transports",
                possible_transports
                    .map(|t| t.iter().map(|t| t.0.clone()).collect::<Vec<PyObject>>()),
            )
            .unwrap();
        let o = urlutils
            .call_method("get_transport", (url.to_string(),), Some(kwargs))
            .unwrap();
        Transport(o.to_object(py))
    })
}
