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

pub trait CommitReporter: ToPyObject {}

impl CommitReporter for NullCommitReporter {}

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

impl CommitReporter for ReportCommitToLog {}

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
