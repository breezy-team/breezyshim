pub mod vcs_up_to_date;

use crate::Branch;
use crate::WorkingTree;

use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Debug)]
pub enum BuildError {
    Other(pyo3::PyErr),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BuildError::Other(e) => write!(f, "Python error: {}", e),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<pyo3::PyErr> for BuildError {
    fn from(e: pyo3::PyErr) -> Self {
        BuildError::Other(e)
    }
}

pub fn build_helper(
    local_tree: &WorkingTree,
    subpath: &std::path::Path,
    branch: &dyn Branch,
    target_dir: &std::path::Path,
    builder: &str,
) -> Result<(), BuildError> {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| -> PyResult<()> {
        let locals = PyDict::new_bound(py);
        locals.set_item("local_tree", local_tree)?;
        locals.set_item("subpath", subpath)?;
        locals.set_item("branch", branch)?;
        locals.set_item("target_dir", target_dir)?;
        locals.set_item("builder", builder)?;

        py.import_bound("breezy.plugins.debian.cmds")?
            .call_method1("build_helper", (locals,))?;

        Ok(())
    })?;

    Ok(())
}
