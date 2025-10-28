//! Commit-related functionality.
//!
//! This module provides types for reporting commit information and handling
//! commit operations in version control systems.

use pyo3::prelude::*;

/// A commit reporter that doesn't report anything.
///
/// This is useful when you want to perform a commit operation but don't want
/// to output any information about the commit.
pub struct NullCommitReporter(PyObject);

impl NullCommitReporter {
    /// Create a new NullCommitReporter.
    ///
    /// # Returns
    ///
    /// A new NullCommitReporter instance.
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

impl From<PyObject> for NullCommitReporter {
    fn from(obj: PyObject) -> Self {
        NullCommitReporter(obj)
    }
}

impl<'py> IntoPyObject<'py> for NullCommitReporter {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

/// Trait for Python commit reporters.
///
/// This trait is implemented by commit reporters that wrap Python objects.
pub trait PyCommitReporter: std::any::Any + std::fmt::Debug {
    /// Get the underlying Python object for this commit reporter.
    fn to_object(&self, py: Python) -> PyObject;
}

/// Trait for commit reporters.
///
/// This trait represents objects that report information about commits.
pub trait CommitReporter: std::fmt::Debug {}

impl<T: PyCommitReporter> CommitReporter for T {}

/// A generic commit reporter that wraps any Python commit reporter.
pub struct GenericCommitReporter(PyObject);

impl<'py> IntoPyObject<'py> for GenericCommitReporter {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for GenericCommitReporter {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(GenericCommitReporter(obj.to_owned().unbind()))
    }
}

impl PyCommitReporter for GenericCommitReporter {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl GenericCommitReporter {
    /// Create a new GenericCommitReporter from a Python object.
    ///
    /// # Parameters
    ///
    /// * `obj` - A Python object that implements the commit reporter interface.
    ///
    /// # Returns
    ///
    /// A new GenericCommitReporter instance that wraps the provided Python object.
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }
}

impl std::fmt::Debug for GenericCommitReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("GenericCommitReporter({:?})", self.0))
    }
}

impl PyCommitReporter for NullCommitReporter {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

impl std::fmt::Debug for NullCommitReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("NullCommitReporter({:?})", self.0))
    }
}

/// A commit reporter that reports commit information to the log.
///
/// This reporter outputs information about commits to the logging system.
pub struct ReportCommitToLog(PyObject);

impl ReportCommitToLog {
    /// Create a new ReportCommitToLog instance.
    ///
    /// # Returns
    ///
    /// A new ReportCommitToLog instance.
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py.import("breezy.commit").unwrap();
            let rctl = m.getattr("ReportCommitToLog").unwrap();
            ReportCommitToLog(rctl.call0().unwrap().into())
        })
    }
}

impl Default for ReportCommitToLog {
    fn default() -> Self {
        Self::new()
    }
}

impl From<PyObject> for ReportCommitToLog {
    fn from(obj: PyObject) -> Self {
        ReportCommitToLog(obj)
    }
}

impl<'py> IntoPyObject<'py> for ReportCommitToLog {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl PyCommitReporter for ReportCommitToLog {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

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
