//! Revision tags
use crate::error::Error;
use crate::revisionid::RevisionId;
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

/// Represents a collection of revision tags.
///
/// Tags allow associating human-readable names with specific revision IDs.
/// This struct provides methods to manage and query these tags.
pub struct Tags(PyObject);

impl From<PyObject> for Tags {
    fn from(obj: PyObject) -> Self {
        Tags(obj)
    }
}

impl Tags {
    /// Get a mapping from revision IDs to sets of tags.
    ///
    /// # Returns
    ///
    /// A HashMap mapping each revision ID to a set of tag names that point to it,
    /// or an error if the operation fails
    pub fn get_reverse_tag_dict(
        &self,
    ) -> Result<HashMap<RevisionId, HashSet<String>>, crate::error::Error> {
        Python::with_gil(|py| self.0.call_method0(py, "get_reverse_tag_dict")?.extract(py))
            .map_err(Into::into)
    }

    /// Get a mapping from tag names to revision IDs.
    ///
    /// # Returns
    ///
    /// A HashMap mapping each tag name to the revision ID it points to,
    /// or an error if the operation fails
    pub fn get_tag_dict(&self) -> Result<HashMap<String, RevisionId>, crate::error::Error> {
        Python::with_gil(|py| self.0.call_method0(py, "get_tag_dict")?.extract(py))
            .map_err(Into::into)
    }

    /// Look up a revision ID by tag name.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag name to look up
    ///
    /// # Returns
    ///
    /// The revision ID the tag points to, or an error if the tag doesn't exist
    pub fn lookup_tag(&self, tag: &str) -> Result<RevisionId, Error> {
        Ok(Python::with_gil(|py| {
            self.0.call_method1(py, "lookup_tag", (tag,))?.extract(py)
        })?)
    }

    /// Check if a tag exists.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag name to check
    ///
    /// # Returns
    ///
    /// `true` if the tag exists, `false` otherwise
    pub fn has_tag(&self, tag: &str) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "has_tag", (tag,))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Set a tag to point to a specific revision.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag name to set
    /// * `revision_id` - The revision ID the tag should point to
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails
    pub fn set_tag(&self, tag: &str, revision_id: &RevisionId) -> Result<(), Error> {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "set_tag", (tag, revision_id.to_object(py)))
        })?;
        Ok(())
    }

    /// Delete a tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag name to delete
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails
    pub fn delete_tag(&self, tag: &str) -> Result<(), Error> {
        Python::with_gil(|py| self.0.call_method1(py, "delete_tag", (tag,)))?;
        Ok(())
    }
}
