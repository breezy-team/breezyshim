use crate::error::Error;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::Path;

pub fn clean_tree(
    directory: &Path,
    unknown: bool,
    ignored: bool,
    detritus: bool,
    dry_run: bool,
    no_prompt: bool,
) -> Result<(), Error> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.clean_tree")?;
        let f = m.getattr("clean_tree")?;
        let kwargs = PyDict::new_bound(py);
        kwargs.set_item("directory", directory.to_str().unwrap())?;
        kwargs.set_item("unknown", unknown)?;
        kwargs.set_item("ignored", ignored)?;
        kwargs.set_item("detritus", detritus)?;
        kwargs.set_item("dry_run", dry_run)?;
        kwargs.set_item("no_prompt", no_prompt)?;
        f.call((), Some(&kwargs))?;
        Ok(())
    })
}
