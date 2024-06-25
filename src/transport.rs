use pyo3::prelude::*;
use pyo3::types::PyDict;

pub struct Transport(PyObject);

impl Transport {
    pub fn new(obj: PyObject) -> Self {
        Transport(obj)
    }

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

#[derive(Debug)]
pub enum Error {
    Python(PyErr),
}

impl From<PyErr> for Error {
    fn from(e: PyErr) -> Self {
        Error::Python(e)
    }
}

impl From<Error> for PyErr {
    fn from(e: Error) -> Self {
        match e {
            Error::Python(e) => e,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Python(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

pub fn get_transport(
    url: &url::Url,
    possible_transports: Option<&mut Vec<Transport>>,
) -> Result<Transport, Error> {
    pyo3::Python::with_gil(|py| {
        let urlutils = py.import_bound("breezy.transport").unwrap();
        let kwargs = PyDict::new_bound(py);
        kwargs.set_item(
            "possible_transports",
            possible_transports.map(|t| t.iter().map(|t| t.0.clone()).collect::<Vec<PyObject>>()),
        )?;
        let o = urlutils.call_method("get_transport", (url.to_string(),), Some(&kwargs))?;
        Ok(Transport(o.to_object(py)))
    })
}
