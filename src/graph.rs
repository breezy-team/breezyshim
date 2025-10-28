//! Graph traversal operations on revision graphs.
use crate::revisionid::RevisionId;
use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;
use pyo3::types::{PyFrozenSet, PyIterator, PyTuple};
use std::collections::HashMap;
use std::hash::Hash;

/// Trait for types that can be used as nodes in a graph.
///
/// This trait allows graph operations to work with any type that can be
/// converted to a Python object and compared for equality.
pub trait GraphNode: Eq + Hash + Clone {
    /// Convert this node to a Python object representation.
    fn to_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;

    /// Create a node from a Python object.
    fn from_pyobject(obj: &Bound<PyAny>) -> PyResult<Self>;
}

/// Represents a graph of revisions.
///
/// This struct provides methods for traversing and querying relationships
/// between revisions in a version control repository.
pub struct Graph(Py<PyAny>);

impl<'py> IntoPyObject<'py> for Graph {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Graph {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Graph(ob.to_owned().unbind()))
    }
}

impl From<Py<PyAny>> for Graph {
    fn from(ob: Py<PyAny>) -> Self {
        Graph(ob)
    }
}

/// Implement GraphNode for RevisionId
impl GraphNode for RevisionId {
    fn to_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Ok(self.as_bytes().into_pyobject(py)?.into_any())
    }

    fn from_pyobject(obj: &Bound<PyAny>) -> PyResult<Self> {
        let bytes: Vec<u8> = obj.extract()?;
        Ok(RevisionId::from(bytes))
    }
}

struct NodeIter<T: GraphNode>(Py<PyAny>, std::marker::PhantomData<T>);

impl<T: GraphNode> Iterator for NodeIter<T> {
    type Item = Result<T, crate::error::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Python::attach(|py| match self.0.call_method0(py, "__next__") {
            Ok(item) => match T::from_pyobject(item.bind(py)) {
                Ok(node) => Some(Ok(node)),
                Err(e) => Some(Err(e.into())),
            },
            Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
            Err(e) => Some(Err(e.into())),
        })
    }
}

struct TopoOrderIter<T: GraphNode>(Py<PyAny>, std::marker::PhantomData<T>);

impl<T: GraphNode> Iterator for TopoOrderIter<T> {
    type Item = Result<(usize, T, usize, bool), crate::error::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Python::attach(|py| match self.0.call_method0(py, "__next__") {
            Ok(item) => {
                let tuple = match item.bind(py).cast::<PyTuple>() {
                    Ok(t) => t,
                    Err(e) => return Some(Err(PyErr::from(e).into())),
                };
                if tuple.len() != 4 {
                    return Some(Err(pyo3::exceptions::PyValueError::new_err(
                        "Expected 4-tuple from iter_topo_order",
                    )
                    .into()));
                }
                match (
                    tuple.get_item(0).and_then(|i| i.extract::<usize>()),
                    tuple.get_item(1).and_then(|i| T::from_pyobject(&i)),
                    tuple.get_item(2).and_then(|i| i.extract::<usize>()),
                    tuple.get_item(3).and_then(|i| i.extract::<bool>()),
                ) {
                    (Ok(seq), Ok(node), Ok(depth), Ok(eom)) => Some(Ok((seq, node, depth, eom))),
                    _ => Some(Err(pyo3::exceptions::PyValueError::new_err(
                        "Failed to extract values from topo_order tuple",
                    )
                    .into())),
                }
            }
            Err(e) if e.is_instance_of::<PyStopIteration>(py) => None,
            Err(e) => Some(Err(e.into())),
        })
    }
}

impl Graph {
    /// Get the underlying Py<PyAny>.
    pub(crate) fn as_pyobject(&self) -> &Py<PyAny> {
        &self.0
    }

    /// Check if one node is an ancestor of another.
    ///
    /// # Arguments
    ///
    /// * `node1` - The potential ancestor node
    /// * `node2` - The potential descendant node
    ///
    /// # Returns
    ///
    /// `true` if `node1` is an ancestor of `node2`, `false` otherwise
    pub fn is_ancestor<T: GraphNode>(
        &self,
        node1: &T,
        node2: &T,
    ) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            let result = self.0.call_method1(
                py,
                "is_ancestor",
                (node1.to_pyobject(py)?, node2.to_pyobject(py)?),
            )?;
            Ok(result.extract(py)?)
        })
    }

    /// Iterate through the left-hand ancestry of a node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to start from
    /// * `stop_nodes` - Optional list of nodes where iteration should stop
    ///
    /// # Returns
    ///
    /// An iterator that yields nodes in the ancestry chain
    pub fn iter_lefthand_ancestry<T: GraphNode>(
        &self,
        node: &T,
        stop_nodes: Option<&[T]>,
    ) -> Result<impl Iterator<Item = Result<T, crate::error::Error>>, crate::error::Error> {
        Python::attach(|py| {
            let stop_py = if let Some(nodes) = stop_nodes {
                let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
                Some(py_nodes?)
            } else {
                None
            };

            let iter = self.0.call_method1(
                py,
                "iter_lefthand_ancestry",
                (node.to_pyobject(py)?, stop_py),
            )?;
            Ok(NodeIter(iter, std::marker::PhantomData))
        })
    }

    /// Find the least common ancestor(s) of a set of nodes.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A list of nodes to find the LCA for
    ///
    /// # Returns
    ///
    /// A vector of nodes that are the least common ancestors
    pub fn find_lca<T: GraphNode>(&self, nodes: &[T]) -> Result<Vec<T>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self.0.call_method1(py, "find_lca", (py_nodes?,))?;
            let py_set = result
                .cast_bound::<pyo3::types::PySet>(py)
                .map_err(PyErr::from)?;
            let mut lca_nodes = Vec::new();
            for item in py_set {
                lca_nodes.push(T::from_pyobject(&item)?)
            }
            Ok(lca_nodes)
        })
    }

    /// Get the heads from a set of nodes.
    ///
    /// Heads are nodes that are not ancestors of any other node in the set.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to find heads from
    ///
    /// # Returns
    ///
    /// A vector of nodes that are heads
    pub fn heads<T: GraphNode>(&self, nodes: &[T]) -> Result<Vec<T>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self.0.call_method1(py, "heads", (py_nodes?,))?;
            let py_set = result
                .cast_bound::<pyo3::types::PySet>(py)
                .map_err(PyErr::from)?;
            let mut head_nodes = Vec::new();
            for item in py_set {
                head_nodes.push(T::from_pyobject(&item)?)
            }
            Ok(head_nodes)
        })
    }

    /// Find the unique ancestors of one set of nodes that are not ancestors of another set.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to check
    /// * `common_nodes` - List of common nodes to exclude
    ///
    /// # Returns
    ///
    /// A vector of nodes that are unique ancestors
    pub fn find_unique_ancestors<T: GraphNode>(
        &self,
        nodes: &[T],
        common_nodes: &[T],
    ) -> Result<Vec<T>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let py_common: Result<Vec<_>, _> =
                common_nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result =
                self.0
                    .call_method1(py, "find_unique_ancestors", (py_nodes?, py_common?))?;
            let py_list = result
                .cast_bound::<pyo3::types::PyList>(py)
                .map_err(PyErr::from)?;
            let mut unique_ancestors = Vec::new();
            for item in py_list {
                unique_ancestors.push(T::from_pyobject(&item)?)
            }
            Ok(unique_ancestors)
        })
    }

    /// Find the difference between two sets of nodes.
    ///
    /// # Arguments
    ///
    /// * `left_nodes` - The left set of nodes
    /// * `right_nodes` - The right set of nodes
    ///
    /// # Returns
    ///
    /// A tuple of (nodes only in left, nodes only in right)
    pub fn find_difference<T: GraphNode>(
        &self,
        left_nodes: &[T],
        right_nodes: &[T],
    ) -> Result<(Vec<T>, Vec<T>), crate::error::Error> {
        Python::attach(|py| {
            let py_left: Result<Vec<_>, _> = left_nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let py_right: Result<Vec<_>, _> =
                right_nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self
                .0
                .call_method1(py, "find_difference", (py_left?, py_right?))?;
            let tuple = result.cast_bound::<PyTuple>(py).map_err(PyErr::from)?;

            let left_only = tuple.get_item(0)?;
            let right_only = tuple.get_item(1)?;

            let mut left_result = Vec::new();
            for item in left_only
                .cast::<pyo3::types::PySet>()
                .map_err(PyErr::from)?
            {
                left_result.push(T::from_pyobject(&item)?);
            }

            let mut right_result = Vec::new();
            for item in right_only
                .cast::<pyo3::types::PySet>()
                .map_err(PyErr::from)?
            {
                right_result.push(T::from_pyobject(&item)?);
            }

            Ok((left_result, right_result))
        })
    }

    /// Iterate through ancestry of given nodes.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to get ancestry for
    ///
    /// # Returns
    ///
    /// An iterator that yields nodes in the ancestry
    pub fn iter_ancestry<T: GraphNode>(
        &self,
        nodes: &[T],
    ) -> Result<impl Iterator<Item = Result<T, crate::error::Error>>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let iter = self.0.call_method1(py, "iter_ancestry", (py_nodes?,))?;
            Ok(NodeIter(iter, std::marker::PhantomData))
        })
    }

    /// Get the parent map for a set of nodes.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to get parents for
    ///
    /// # Returns
    ///
    /// A map from node to list of parent nodes
    pub fn get_parent_map<T: GraphNode>(
        &self,
        nodes: &[T],
    ) -> Result<HashMap<T, Vec<T>>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self.0.call_method1(py, "get_parent_map", (py_nodes?,))?;
            let py_dict = result
                .cast_bound::<pyo3::types::PyDict>(py)
                .map_err(PyErr::from)?;

            let mut parent_map = HashMap::new();
            for (key, value) in py_dict {
                let key_node = T::from_pyobject(&key)?;

                let mut parents = Vec::new();
                for parent in value.cast::<pyo3::types::PyTuple>().map_err(PyErr::from)? {
                    parents.push(T::from_pyobject(&parent)?);
                }
                parent_map.insert(key_node, parents);
            }
            Ok(parent_map)
        })
    }

    /// Check if a node is between two other nodes.
    ///
    /// # Arguments
    ///
    /// * `candidate` - The node to check
    /// * `ancestor` - The potential ancestor
    /// * `descendant` - The potential descendant
    ///
    /// # Returns
    ///
    /// `true` if `candidate` is between `ancestor` and `descendant`
    pub fn is_between<T: GraphNode>(
        &self,
        candidate: &T,
        ancestor: &T,
        descendant: &T,
    ) -> Result<bool, crate::error::Error> {
        Python::attach(|py| {
            let result = self.0.call_method1(
                py,
                "is_between",
                (
                    candidate.to_pyobject(py)?,
                    ancestor.to_pyobject(py)?,
                    descendant.to_pyobject(py)?,
                ),
            )?;
            Ok(result.extract(py)?)
        })
    }

    /// Iterate through nodes in topological order.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to order
    ///
    /// # Returns
    ///
    /// An iterator that yields (sequence_number, node, depth, end_of_merge)
    pub fn iter_topo_order<T: GraphNode>(
        &self,
        nodes: &[T],
    ) -> Result<
        impl Iterator<Item = Result<(usize, T, usize, bool), crate::error::Error>>,
        crate::error::Error,
    > {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let iter = self.0.call_method1(py, "iter_topo_order", (py_nodes?,))?;
            Ok(TopoOrderIter(iter, std::marker::PhantomData))
        })
    }

    /// Find all descendants of the given nodes.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to find descendants for
    ///
    /// # Returns
    ///
    /// A vector of nodes that are descendants
    pub fn find_descendants<T: GraphNode>(
        &self,
        nodes: &[T],
    ) -> Result<Vec<T>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self.0.call_method1(py, "find_descendants", (py_nodes?,))?;
            let py_set = result
                .cast_bound::<pyo3::types::PySet>(py)
                .map_err(PyErr::from)?;
            let mut descendants = Vec::new();
            for item in py_set {
                descendants.push(T::from_pyobject(&item)?);
            }
            Ok(descendants)
        })
    }

    /// Find the distance from nodes to null.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to find distance for
    ///
    /// # Returns
    ///
    /// A map from node to distance
    pub fn find_distance_to_null<T: GraphNode>(
        &self,
        nodes: &[T],
    ) -> Result<HashMap<T, usize>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self
                .0
                .call_method1(py, "find_distance_to_null", (py_nodes?,))?;
            let py_dict = result
                .cast_bound::<pyo3::types::PyDict>(py)
                .map_err(PyErr::from)?;

            let mut distance_map = HashMap::new();
            for (key, value) in py_dict {
                let key_node = T::from_pyobject(&key)?;
                let distance: usize = value.extract()?;
                distance_map.insert(key_node, distance);
            }
            Ok(distance_map)
        })
    }

    /// Find the unique least common ancestor.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to find unique LCA for
    /// * `count` - The number of heads to look for (optional)
    ///
    /// # Returns
    ///
    /// The unique LCA node or None if there isn't a unique one
    pub fn find_unique_lca<T: GraphNode>(
        &self,
        nodes: &[T],
        count: Option<usize>,
    ) -> Result<Option<T>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = if let Some(c) = count {
                self.0.call_method1(py, "find_unique_lca", (py_nodes?, c))?
            } else {
                self.0.call_method1(py, "find_unique_lca", (py_nodes?,))?
            };

            if result.is_none(py) {
                Ok(None)
            } else {
                Ok(Some(T::from_pyobject(result.bind(py))?))
            }
        })
    }

    /// Find merge order for nodes.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to find merge order for
    ///
    /// # Returns
    ///
    /// An ordered list of nodes
    pub fn find_merge_order<T: GraphNode>(
        &self,
        nodes: &[T],
    ) -> Result<Vec<T>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self.0.call_method1(py, "find_merge_order", (py_nodes?,))?;
            let py_list = result
                .cast_bound::<pyo3::types::PyList>(py)
                .map_err(PyErr::from)?;
            let mut merge_order = Vec::new();
            for item in py_list {
                merge_order.push(T::from_pyobject(&item)?);
            }
            Ok(merge_order)
        })
    }

    /// Find the lefthand merger of a node.
    ///
    /// # Arguments
    ///
    /// * `node` - Node to find merger for
    /// * `tip` - Optional tip node
    ///
    /// # Returns
    ///
    /// The lefthand merger node
    pub fn find_lefthand_merger<T: GraphNode>(
        &self,
        node: &T,
        tip: Option<&T>,
    ) -> Result<Option<T>, crate::error::Error> {
        Python::attach(|py| {
            let args = if let Some(t) = tip {
                (node.to_pyobject(py)?, t.to_pyobject(py)?)
            } else {
                (node.to_pyobject(py)?, py.None().into_bound(py))
            };
            let result = self.0.call_method1(py, "find_lefthand_merger", args)?;

            if result.is_none(py) {
                Ok(None)
            } else {
                Ok(Some(T::from_pyobject(result.bind(py))?))
            }
        })
    }

    /// Find lefthand distances for nodes.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to find distances for
    ///
    /// # Returns
    ///
    /// A map from node to distance
    pub fn find_lefthand_distances<T: GraphNode>(
        &self,
        nodes: &[T],
    ) -> Result<HashMap<T, usize>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self
                .0
                .call_method1(py, "find_lefthand_distances", (py_nodes?,))?;
            let py_dict = result
                .cast_bound::<pyo3::types::PyDict>(py)
                .map_err(PyErr::from)?;

            let mut distance_map = HashMap::new();
            for (key, value) in py_dict {
                let key_node = T::from_pyobject(&key)?;
                let distance: usize = value.extract()?;
                distance_map.insert(key_node, distance);
            }
            Ok(distance_map)
        })
    }

    /// Get the child map for a set of nodes.
    ///
    /// # Arguments
    ///
    /// * `nodes` - List of nodes to get children for
    ///
    /// # Returns
    ///
    /// A map from node to list of child nodes
    pub fn get_child_map<T: GraphNode>(
        &self,
        nodes: &[T],
    ) -> Result<HashMap<T, Vec<T>>, crate::error::Error> {
        Python::attach(|py| {
            let py_nodes: Result<Vec<_>, _> = nodes.iter().map(|n| n.to_pyobject(py)).collect();
            let result = self.0.call_method1(py, "get_child_map", (py_nodes?,))?;
            let py_dict = result
                .cast_bound::<pyo3::types::PyDict>(py)
                .map_err(PyErr::from)?;

            let mut child_map = HashMap::new();
            for (key, value) in py_dict {
                let key_node = T::from_pyobject(&key)?;

                let mut children = Vec::new();
                for child in value.cast::<pyo3::types::PyList>().map_err(PyErr::from)? {
                    children.push(T::from_pyobject(&child)?);
                }
                child_map.insert(key_node, children);
            }
            Ok(child_map)
        })
    }
}

/// A key identifying a specific version of a file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(Vec<String>);

impl From<Vec<String>> for Key {
    fn from(v: Vec<String>) -> Self {
        Key(v)
    }
}

impl From<Key> for Vec<String> {
    fn from(k: Key) -> Self {
        k.0
    }
}

impl<'py> IntoPyObject<'py> for Key {
    type Target = PyTuple;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyTuple::new(py, self.0)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Key {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let tuple = ob.cast::<PyTuple>()?;
        let mut items = Vec::new();
        for item in tuple.iter() {
            items.push(item.extract::<String>()?);
        }
        Ok(Key(items))
    }
}

/// Implement GraphNode for Key
impl GraphNode for Key {
    fn to_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Ok(PyTuple::new(py, &self.0)?.into_any())
    }

    fn from_pyobject(obj: &Bound<PyAny>) -> PyResult<Self> {
        obj.extract::<Key>()
    }
}

/// A known graph of file versions
pub struct KnownGraph(Py<PyAny>);

impl KnownGraph {
    /// Create a new KnownGraph from a Python object
    pub fn new(py_obj: Py<PyAny>) -> Self {
        Self(py_obj)
    }

    /// Get the heads of the given nodes
    pub fn heads<T: GraphNode>(&self, nodes: Vec<T>) -> Result<Vec<T>, crate::error::Error> {
        Python::attach(|py| {
            let nodes_py: Vec<_> = nodes
                .into_iter()
                .map(|n| n.to_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            let nodes_frozenset = PyFrozenSet::new(py, &nodes_py)?;

            let result = self.0.call_method1(py, "heads", (nodes_frozenset,))?;

            let mut heads = Vec::new();
            for head_py in result
                .cast_bound::<PyIterator>(py)
                .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Expected iterator"))?
            {
                let head = T::from_pyobject(&head_py?)?;
                heads.push(head);
            }

            Ok(heads)
        })
    }
}

impl Clone for KnownGraph {
    fn clone(&self) -> Self {
        Python::attach(|py| KnownGraph(self.0.clone_ref(py)))
    }
}

#[cfg(test)]
mod tests;
