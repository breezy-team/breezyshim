//! Operations between two trees.
use crate::delta::TreeDelta;
use pyo3::prelude::*;

crate::wrapped_py!(InterTree);

pub fn get<S: crate::tree::PyTree, T: crate::tree::PyTree>(source: &S, target: &T) -> InterTree {
    Python::with_gil(|py| {
        let intertree_cls = py
            .import("breezy.tree")
            .unwrap()
            .getattr("InterTree")
            .unwrap();

        InterTree::from(
            intertree_cls
                .call_method1("get", (source, target))
                .unwrap()
        )
    })
}

impl InterTree {
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
