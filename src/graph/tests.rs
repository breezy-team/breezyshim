use crate::graph::{Graph, GraphNode, Key};
use crate::revisionid::RevisionId;
use pyo3::prelude::*;

#[test]
fn test_graph_node_trait_for_revision_id() {
    Python::attach(|py| {
        // Test RevisionId GraphNode implementation
        let rev_id = RevisionId::from(b"test-revision-id".to_vec());
        let py_obj = rev_id.to_pyobject(py).unwrap();

        // Should be bytes
        assert!(py_obj.is_instance_of::<pyo3::types::PyBytes>());

        // Round trip
        let rev_id2 = RevisionId::from_pyobject(&py_obj).unwrap();
        assert_eq!(rev_id, rev_id2);
    });
}

#[test]
fn test_graph_node_trait_for_key() {
    Python::attach(|py| {
        // Test Key GraphNode implementation
        let key = Key::from(vec!["file.txt".to_string(), "rev1".to_string()]);
        let py_obj = key.to_pyobject(py).unwrap();

        // Should be a tuple
        assert!(py_obj.is_instance_of::<pyo3::types::PyTuple>());

        // Round trip
        let key2 = Key::from_pyobject(&py_obj).unwrap();
        assert_eq!(key, key2);
    });
}

fn create_test_graph() -> Graph {
    Python::attach(|py| {
        // Create a mock graph for testing
        let graph_module = py.import("breezy.graph").unwrap();
        let dict_parents_provider = graph_module.getattr("DictParentsProvider").unwrap();

        // Create a simple graph structure:
        // null -> rev1 -> rev2 -> rev3
        //              \-> rev4
        let parents_dict = pyo3::types::PyDict::new(py);
        parents_dict
            .set_item(b"rev1".as_slice(), pyo3::types::PyTuple::empty(py))
            .unwrap();
        parents_dict
            .set_item(
                b"rev2".as_slice(),
                pyo3::types::PyTuple::new(py, &[pyo3::types::PyBytes::new(py, b"rev1")]).unwrap(),
            )
            .unwrap();
        parents_dict
            .set_item(
                b"rev3".as_slice(),
                pyo3::types::PyTuple::new(py, &[pyo3::types::PyBytes::new(py, b"rev2")]).unwrap(),
            )
            .unwrap();
        parents_dict
            .set_item(
                b"rev4".as_slice(),
                pyo3::types::PyTuple::new(py, &[pyo3::types::PyBytes::new(py, b"rev1")]).unwrap(),
            )
            .unwrap();

        let parents_provider = dict_parents_provider.call1((parents_dict,)).unwrap();
        let graph_class = graph_module.getattr("Graph").unwrap();
        let graph = graph_class.call1((parents_provider,)).unwrap();

        Graph::from(graph.unbind())
    })
}

#[test]
fn test_is_ancestor() {
    crate::init();
    let graph = create_test_graph();

    let rev1 = RevisionId::from(b"rev1".to_vec());
    let rev2 = RevisionId::from(b"rev2".to_vec());
    let rev3 = RevisionId::from(b"rev3".to_vec());
    let rev4 = RevisionId::from(b"rev4".to_vec());

    // Test ancestor relationships
    assert!(graph.is_ancestor(&rev1, &rev2).unwrap());
    assert!(graph.is_ancestor(&rev1, &rev3).unwrap());
    assert!(graph.is_ancestor(&rev2, &rev3).unwrap());
    assert!(graph.is_ancestor(&rev1, &rev4).unwrap());

    // Test non-ancestor relationships
    assert!(!graph.is_ancestor(&rev2, &rev1).unwrap());
    assert!(!graph.is_ancestor(&rev3, &rev1).unwrap());
    assert!(!graph.is_ancestor(&rev4, &rev2).unwrap());
    assert!(!graph.is_ancestor(&rev2, &rev4).unwrap());
}

#[test]
fn test_is_between() {
    crate::init();
    let graph = create_test_graph();

    let rev1 = RevisionId::from(b"rev1".to_vec());
    let rev2 = RevisionId::from(b"rev2".to_vec());
    let rev3 = RevisionId::from(b"rev3".to_vec());

    // rev2 is between rev1 and rev3
    assert!(graph.is_between(&rev2, &rev1, &rev3).unwrap());

    // rev1 is not between rev2 and rev3
    assert!(!graph.is_between(&rev1, &rev2, &rev3).unwrap());
}

#[test]
fn test_iter_lefthand_ancestry() {
    crate::init();
    let graph = create_test_graph();

    let rev3 = RevisionId::from(b"rev3".to_vec());
    let rev1 = RevisionId::from(b"rev1".to_vec());

    // Get ancestry from rev3
    let ancestry: Vec<_> = graph
        .iter_lefthand_ancestry(&rev3, None)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Should contain rev3, rev2, rev1 in that order
    assert_eq!(ancestry.len(), 3);
    assert_eq!(ancestry[0], RevisionId::from(b"rev3".to_vec()));
    assert_eq!(ancestry[1], RevisionId::from(b"rev2".to_vec()));
    assert_eq!(ancestry[2], RevisionId::from(b"rev1".to_vec()));

    // Test with stop revision
    let ancestry_with_stop: Vec<_> = graph
        .iter_lefthand_ancestry(&rev3, Some(&[rev1]))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Should stop before rev1
    assert_eq!(ancestry_with_stop.len(), 2);
    assert_eq!(ancestry_with_stop[0], RevisionId::from(b"rev3".to_vec()));
    assert_eq!(ancestry_with_stop[1], RevisionId::from(b"rev2".to_vec()));
}

#[test]
fn test_heads() {
    crate::init();
    let graph = create_test_graph();

    let rev1 = RevisionId::from(b"rev1".to_vec());
    let rev2 = RevisionId::from(b"rev2".to_vec());
    let rev3 = RevisionId::from(b"rev3".to_vec());
    let rev4 = RevisionId::from(b"rev4".to_vec());

    // Heads of [rev1, rev2, rev3] should be [rev3]
    let heads = graph.heads(&[rev1.clone(), rev2, rev3.clone()]).unwrap();
    assert_eq!(heads.len(), 1);
    assert!(heads.contains(&rev3));

    // Heads of [rev1, rev3, rev4] should be [rev3, rev4]
    let heads2 = graph.heads(&[rev1, rev3.clone(), rev4.clone()]).unwrap();
    assert_eq!(heads2.len(), 2);
    assert!(heads2.contains(&rev3));
    assert!(heads2.contains(&rev4));
}

#[test]
fn test_get_parent_map() {
    crate::init();
    let graph = create_test_graph();

    let rev1 = RevisionId::from(b"rev1".to_vec());
    let rev2 = RevisionId::from(b"rev2".to_vec());
    let rev3 = RevisionId::from(b"rev3".to_vec());
    let rev4 = RevisionId::from(b"rev4".to_vec());

    let parent_map = graph
        .get_parent_map(&[rev1.clone(), rev2.clone(), rev3.clone(), rev4.clone()])
        .unwrap();

    // Check parent relationships
    assert_eq!(parent_map.get(&rev1).unwrap().len(), 0); // rev1 has no parents
    assert_eq!(parent_map.get(&rev2).unwrap(), &vec![rev1.clone()]);
    assert_eq!(parent_map.get(&rev3).unwrap(), &vec![rev2]);
    assert_eq!(parent_map.get(&rev4).unwrap(), &vec![rev1]);
}
