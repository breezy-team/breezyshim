//! Topological sorting helpers.
//!
//! Thin wrapper around [`vcs_graph::tsort::merge_sort`] that presents the
//! result as [`MergeSortEntry`] values. Only `merge_sort` is exposed for
//! now — that is what revision viewers like loggerhead need to lay out a
//! branch history as a DAG with dotted revision numbers.

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
pub fn merge_sort<T: GraphNode + std::fmt::Debug>(
    graph: &HashMap<T, Vec<T>>,
    branch_tip: &T,
) -> Result<Vec<MergeSortEntry<T>>, crate::error::Error> {
    let rows = vcs_graph::tsort::merge_sort(graph.clone(), Some(branch_tip.clone()), None, true)
        .map_err(|e| {
            crate::error::Error::from(pyo3::exceptions::PyValueError::new_err(format!("{:?}", e)))
        })?;
    let mut out = Vec::with_capacity(rows.len());
    for (sequence, node, merge_depth, revno, end_of_merge) in rows {
        let revno = revno
            .expect("generate_revno=true should always yield a revno")
            .into_iter()
            .map(|n| n as u32)
            .collect();
        out.push(MergeSortEntry {
            sequence,
            node,
            merge_depth,
            revno,
            end_of_merge,
        });
    }
    Ok(out)
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
