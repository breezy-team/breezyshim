use crate::delta::TreeDelta;
use crate::tree::Tree;
use pyo3::prelude::*;

pub struct InterTree(PyObject);

pub fn get(source: &dyn Tree, target: &dyn Tree) -> InterTree {
    Python::with_gil(|py| {
        let source = source.to_object(py);
        let target = target.to_object(py);

        let intertree_cls = py
            .import("breezy.tree")
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
