//! Operations between two trees.
use crate::delta::TreeDelta;
use pyo3::prelude::*;

pub struct InterTree(PyObject);

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
