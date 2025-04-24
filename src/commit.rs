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

impl Default for NullCommitReporter {
    fn default() -> Self {
        Self::new()
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

pub trait PyCommitReporter: ToPyObject + std::any::Any + std::fmt::Debug {}

pub trait CommitReporter: std::fmt::Debug {}

impl<T: PyCommitReporter> CommitReporter for T {}

pub struct GenericCommitReporter(PyObject);

impl ToPyObject for GenericCommitReporter {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl FromPyObject<'_> for GenericCommitReporter {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(GenericCommitReporter(obj.to_object(obj.py())))
    }
}

impl PyCommitReporter for GenericCommitReporter {}

impl GenericCommitReporter {
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }
}

impl std::fmt::Debug for GenericCommitReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("GenericCommitReporter({:?})", self.0))
    }
}

impl PyCommitReporter for NullCommitReporter {}

impl std::fmt::Debug for NullCommitReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("NullCommitReporter({:?})", self.0))
    }
}

pub struct ReportCommitToLog(PyObject);

impl ReportCommitToLog {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py.import_bound("breezy.commit").unwrap();
            let rctl = m.getattr("ReportCommitToLog").unwrap();
            ReportCommitToLog(rctl.call0().unwrap().into())
        })
    }
}

impl From<PyObject> for ReportCommitToLog {
    fn from(obj: PyObject) -> Self {
        ReportCommitToLog(obj)
    }
}

impl ToPyObject for ReportCommitToLog {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl PyCommitReporter for ReportCommitToLog {}

impl std::fmt::Debug for ReportCommitToLog {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("ReportCommitToLog({:?})", self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_commit_reporter() {
        NullCommitReporter::new();
    }

    #[test]
    fn test_report_commit_to_log() {
        ReportCommitToLog::new();
    }
}
