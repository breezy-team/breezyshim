use pyo3::prelude::*;

crate::wrapped_py!(NullCommitReporter);

impl NullCommitReporter {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py.import("breezy.commit").unwrap();
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

crate::wrapped_py!(ReportCommitToLog);

impl ReportCommitToLog {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py.import("breezy.commit").unwrap();
            let rctl = m.getattr("ReportCommitToLog").unwrap();
            ReportCommitToLog(rctl.call0().unwrap().into())
        })
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
