//! Inventory trees
use crate::error::Error;
use crate::tree::{MutableTree, Path};
use pyo3::prelude::*;

pub trait MutableInventoryTree: MutableTree {
    fn add(&self, paths: &[&Path], file_ids: &[crate::bazaar::FileId]) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.to_object(py)
                .call_method1(py, "add", (paths.to_vec(), file_ids.to_vec()))
        })
        .map_err(|e| e.into())
        .map(|_| ())
    }
}

impl MutableInventoryTree for crate::tree::WorkingTree {}
