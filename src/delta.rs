//! Detection of changes between trees.
use crate::osutils::is_inside_any;
use crate::tree::TreeChange;
use pyo3::prelude::*;

/// Describes changes from one tree to another.
///
/// Contains seven lists with TreeChange objects.
///
/// added
/// removed
/// renamed
/// copied
/// kind_changed
/// modified
/// unchanged
/// unversioned
///
/// Each id is listed only once.
///
/// Files that are both modified and renamed or copied are listed only in
/// renamed or copied, with the text_modified flag true. The text_modified
/// applies either to the content of the file or the target of the
/// symbolic link, depending of the kind of file.
///
/// Files are only considered renamed if their name has changed or
/// their parent directory has changed.  Renaming a directory
/// does not count as renaming all its contents.
///
/// The lists are normally sorted when the delta is created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeDelta {
    /// Files that were added between the trees.
    pub added: Vec<TreeChange>,
    /// Files that were removed between the trees.
    pub removed: Vec<TreeChange>,
    /// Files that were renamed between the trees.
    pub renamed: Vec<TreeChange>,
    /// Files that were copied between the trees.
    pub copied: Vec<TreeChange>,
    /// Files that changed kind between the trees.
    pub kind_changed: Vec<TreeChange>,
    /// Files that were modified between the trees.
    pub modified: Vec<TreeChange>,
    /// Files that were unchanged between the trees.
    pub unchanged: Vec<TreeChange>,
    /// Files that are unversioned in the trees.
    pub unversioned: Vec<TreeChange>,
    /// Files that are missing in the trees.
    pub missing: Vec<TreeChange>,
}

impl TreeDelta {
    /// Check if there are any changes in this delta.
    pub fn has_changed(&self) -> bool {
        !self.added.is_empty()
            || !self.removed.is_empty()
            || !self.renamed.is_empty()
            || !self.copied.is_empty()
            || !self.kind_changed.is_empty()
            || !self.modified.is_empty()
    }
}
impl FromPyObject<'_> for TreeDelta {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let added = ob.getattr("added")?.extract()?;
        let removed = ob.getattr("removed")?.extract()?;
        let renamed = ob.getattr("renamed")?.extract()?;
        let copied = ob.getattr("copied")?.extract()?;
        let kind_changed = ob.getattr("kind_changed")?.extract()?;
        let modified = ob.getattr("modified")?.extract()?;
        let unchanged = ob.getattr("unchanged")?.extract()?;
        let unversioned = ob.getattr("unversioned")?.extract()?;
        let missing = ob.getattr("missing")?.extract()?;
        Ok(TreeDelta {
            added,
            removed,
            renamed,
            copied,
            kind_changed,
            modified,
            unchanged,
            unversioned,
            missing,
        })
    }
}

/// Filter out excluded paths from a list of tree changes.
///
/// This function filters out tree changes that are in excluded paths.
///
/// # Arguments
/// * `iter_changes` - Iterator of tree changes
/// * `exclude` - List of paths to exclude
///
/// # Returns
/// Iterator of tree changes that aren't in excluded paths
pub fn filter_excluded<'a>(
    iter_changes: impl Iterator<Item = TreeChange> + 'a,
    exclude: &'a [&'a std::path::Path],
) -> impl Iterator<Item = TreeChange> + 'a {
    iter_changes.filter(|change| {
        let new_excluded = if let Some(p) = change.path.1.as_ref() {
            is_inside_any(exclude, p.as_path())
        } else {
            false
        };

        let old_excluded = if let Some(p) = change.path.0.as_ref() {
            is_inside_any(exclude, p.as_path())
        } else {
            false
        };

        if old_excluded && new_excluded {
            false
        } else if old_excluded || new_excluded {
            // TODO(jelmer): Perhaps raise an error here instead?
            false
        } else {
            true
        }
    })
}
