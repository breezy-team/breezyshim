//! Operations between two trees.
use crate::delta::TreeDelta;
use pyo3::prelude::*;

/// Represents operations between two trees.
///
/// InterTree allows comparing and performing operations between two trees,
/// such as finding differences or applying changes from one tree to another.
pub struct InterTree(PyObject);

/// Get an InterTree for operations between two trees.
///
/// # Arguments
///
/// * `source` - The source tree
/// * `target` - The target tree
///
/// # Returns
///
/// An InterTree object that can be used to perform operations between the trees
pub fn get<S: crate::tree::PyTree, T: crate::tree::PyTree>(source: &S, target: &T) -> InterTree {
    Python::with_gil(|py| {
        let source = source.to_object(py);
        let target = target.to_object(py);

        let intertree_cls = py
            .import_bound("breezy.tree")
            .unwrap()
            .getattr("InterTree")
            .unwrap();

        InterTree(
            intertree_cls
                .call_method1("get", (source, target))
                .unwrap()
                .to_object(py),
        )
    })
}

impl InterTree {
    /// Compare the source and target trees.
    ///
    /// # Returns
    ///
    /// A TreeDelta representing the differences between the source and target trees
    pub fn compare(&self) -> TreeDelta {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "compare")
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }
}
