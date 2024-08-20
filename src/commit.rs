use pyo3::prelude::*;

pub struct NullCommitReporter(PyObject);

impl NullCommitReporter {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.commit").unwrap();
            let ncr = m.getattr("NullCommitReporter").unwrap();
            NullCommitReporter(ncr.call0().unwrap().into())
        })
    }
}

impl From<PyObject> for NullCommitReporter {
    fn from(obj: PyObject) -> Self {
        NullCommitReporter(obj)
    }
}

impl ToPyObject for NullCommitReporter {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

pub trait CommitReporter: ToPyObject {
}

impl CommitReporter for NullCommitReporter {
}
