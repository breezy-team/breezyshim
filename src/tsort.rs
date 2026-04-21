//! Topological sorting helpers.
//!
//! Wraps the [`breezy.tsort`](https://www.breezy-vcs.org/doc/en/api/breezy.tsort.html)
//! module. Only `merge_sort` is exposed for now — that is what revision
//! viewers like loggerhead need to lay out a branch history as a DAG with
//! dotted revision numbers.

use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use std::collections::HashMap;

use crate::graph::GraphNode;

/// One entry returned by [`merge_sort`].
///
/// Mirrors the 5-tuple yielded by `breezy.tsort.merge_sort` when called with
/// `generate_revno=True`.
#[derive(Debug, Clone)]
pub struct MergeSortEntry<T: GraphNode> {
    /// Global sequence number of this entry in the merge-sorted list.
    pub sequence: usize,
    /// Revision id (or other graph node) this entry describes.
    pub node: T,
    /// Merge depth: `0` is mainline, larger numbers are nested merges.
    pub merge_depth: usize,
    /// Dotted revision number components, e.g. `[2, 1, 3]` → `"2.1.3"`.
    pub revno: Vec<u32>,
    /// True iff this is the last entry at the current merge depth.
    pub end_of_merge: bool,
}

impl<T: GraphNode> MergeSortEntry<T> {
    /// Format [`Self::revno`] as a dotted revision number string.
    pub fn revno_str(&self) -> String {
        let mut out = String::new();
        for (i, n) in self.revno.iter().enumerate() {
            if i > 0 {
                out.push('.');
            }
            out.push_str(&n.to_string());
        }
        out
    }
}

/// Topologically sort `graph` with merge ordering and dotted revnos, as
/// `breezy.tsort.merge_sort` does. `graph` is a map from node to its parents.
///
/// `branch_tip` is the node to start from (typically the branch's last
/// revision). Ghost parents must already be filtered out of `graph`.
pub fn merge_sort<T: GraphNode>(
    graph: &HashMap<T, Vec<T>>,
    branch_tip: &T,
) -> Result<Vec<MergeSortEntry<T>>, crate::error::Error> {
    Python::attach(|py| {
        // `merge_sort` lived in `breezy.tsort` historically; newer breezy
        // (the dromedary rewrite) relocated it to `vcsgraph.tsort`. Try both.
        let tsort = py
            .import("breezy.tsort")
            .or_else(|_| py.import("vcsgraph.tsort"))?;
        let py_graph = pyo3::types::PyDict::new(py);
        for (node, parents) in graph {
            let py_parents = PyList::empty(py);
            for parent in parents {
                py_parents.append(parent.to_pyobject(py)?)?;
            }
            py_graph.set_item(node.to_pyobject(py)?, py_parents)?;
        }
        let kwargs = pyo3::types::PyDict::new(py);
        kwargs.set_item("generate_revno", true)?;
        let result = tsort.call_method(
            "merge_sort",
            (py_graph, branch_tip.to_pyobject(py)?),
            Some(&kwargs),
        )?;
        let mut out = Vec::new();
        for item in result.try_iter()? {
            let item = item?;
            let tup = item.cast::<PyTuple>().map_err(PyErr::from)?;
            if tup.len() != 5 {
                return Err(crate::error::Error::from(
                    pyo3::exceptions::PyValueError::new_err(
                        "Expected 5-tuple from merge_sort(generate_revno=True)",
                    ),
                ));
            }
            let sequence: usize = tup.get_item(0)?.extract()?;
            let node = T::from_pyobject(&tup.get_item(1)?)?;
            let merge_depth: usize = tup.get_item(2)?.extract()?;
            let revno: Vec<u32> = tup.get_item(3)?.extract()?;
            let end_of_merge: bool = tup.get_item(4)?.extract()?;
            out.push(MergeSortEntry {
                sequence,
                node,
                merge_depth,
                revno,
                end_of_merge,
            });
        }
        Ok(out)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revisionid::RevisionId;

    #[test]
    fn test_merge_sort_empty_like() {
        crate::init();
        let tip = RevisionId::from(b"tip".to_vec());
        let mut graph = HashMap::new();
        graph.insert(tip.clone(), vec![]);
        let sorted = merge_sort(&graph, &tip).unwrap();
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].node, tip);
        assert_eq!(sorted[0].merge_depth, 0);
        assert_eq!(sorted[0].revno, vec![1]);
        assert!(sorted[0].end_of_merge);
        assert_eq!(sorted[0].revno_str(), "1");
    }

    #[test]
    fn test_merge_sort_linear() {
        crate::init();
        let r1 = RevisionId::from(b"a".to_vec());
        let r2 = RevisionId::from(b"b".to_vec());
        let r3 = RevisionId::from(b"c".to_vec());
        let mut graph = HashMap::new();
        graph.insert(r1.clone(), vec![]);
        graph.insert(r2.clone(), vec![r1.clone()]);
        graph.insert(r3.clone(), vec![r2.clone()]);
        let sorted = merge_sort(&graph, &r3).unwrap();
        let nodes: Vec<_> = sorted.iter().map(|e| e.node.clone()).collect();
        assert_eq!(nodes, vec![r3, r2, r1]);
        let revnos: Vec<_> = sorted.iter().map(|e| e.revno_str()).collect();
        assert_eq!(revnos, vec!["3", "2", "1"]);
    }
}
