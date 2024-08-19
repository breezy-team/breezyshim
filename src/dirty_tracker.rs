//! Fast and efficient detection of files that have been modified in a directory tree.
use crate::tree::WorkingTree;
use dirty_tracker::DirtyTracker;
pub use dirty_tracker::State;

pub struct DirtyTreeTracker {
    tracker: DirtyTracker,
    tree: WorkingTree,
    base: std::path::PathBuf,
}

impl DirtyTreeTracker {
    /// Create a new DirtyTreeTracker for the given WorkingTree.
    pub fn new(tree: WorkingTree) -> Self {
        let base = tree.basedir();
        let tracker = DirtyTracker::new(&base).unwrap();
        Self {
            tracker,
            tree,
            base,
        }
    }

    pub fn new_in_subpath(tree: WorkingTree, subpath: &std::path::Path) -> Self {
        let base = tree.basedir();
        let tracker = DirtyTracker::new(&base.join(subpath)).unwrap();
        Self {
            tracker,
            tree,
            base,
        }
    }

    /// Get the current state.
    pub fn state(&mut self) -> State {
        let relpaths = self.relpaths();

        if relpaths.is_none() {
            return State::Unknown;
        }

        if relpaths.unwrap().into_iter().next().is_some() {
            State::Dirty
        } else {
            State::Clean
        }
    }

    /// Get the relative paths of the dirty files.
    pub fn relpaths(&mut self) -> Option<std::collections::HashSet<std::path::PathBuf>> {
        self.tracker.paths().map(|ps| {
            ps.iter()
                .map(|p| p.strip_prefix(&self.base).unwrap())
                .filter(|p| !self.tree.is_control_filename(p))
                .map(|p| p.to_path_buf())
                .collect()
        })
    }

    /// Get the absolute paths of the dirty files.
    pub fn paths(&mut self) -> Option<std::collections::HashSet<std::path::PathBuf>> {
        self.relpaths()
            .map(|ps| ps.iter().map(|p| self.tree.abspath(p).unwrap()).collect())
    }

    /// Mark the tree as clean.
    pub fn mark_clean(&mut self) {
        self.tracker.mark_clean()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::create_standalone_workingtree;
    use crate::controldir::ControlDirFormat;

    #[test]
    fn test_unchanged_tree() {
        let td = tempfile::tempdir().unwrap();

        let tree = create_standalone_workingtree(td.path(), &ControlDirFormat::default()).unwrap();
        let mut tracker = DirtyTreeTracker::new(tree);

        assert_eq!(tracker.state(), State::Clean);
        assert_eq!(tracker.relpaths(), Some(std::collections::HashSet::new()));
        assert_eq!(tracker.paths(), Some(std::collections::HashSet::new()));
    }

    #[test]
    fn test_unversioned_file() {
        let td = tempfile::tempdir().unwrap();

        let tree = create_standalone_workingtree(td.path(), &ControlDirFormat::default()).unwrap();
        let mut tracker = DirtyTreeTracker::new(tree);
        std::fs::write(td.path().join("foo"), "bar").unwrap();
        assert_eq!(
            tracker.relpaths(),
            Some(maplit::hashset! { std::path::PathBuf::from("foo") })
        );
        assert_eq!(
            tracker.paths(),
            Some(maplit::hashset! { td.path().join("foo") })
        );
        assert_eq!(tracker.state(), State::Dirty);
    }

    #[test]
    fn test_control_file_change() {
        let td = tempfile::tempdir().unwrap();

        let tree = create_standalone_workingtree(td.path(), &ControlDirFormat::default()).unwrap();
        let mut tracker = DirtyTreeTracker::new(tree.clone());
        tree.commit(
            "Dummy",
            Some(true),
            Some("Joe Example <joe@example.com>"),
            None,
        )
        .unwrap();
        assert_eq!(tracker.relpaths(), Some(std::collections::HashSet::new()));
        assert_eq!(tracker.state(), State::Clean);
        assert_eq!(tracker.paths(), Some(std::collections::HashSet::new()));
    }

    #[test]
    fn test_in_subpath() {
        let td = tempfile::tempdir().unwrap();

        let tree = create_standalone_workingtree(td.path(), &ControlDirFormat::default()).unwrap();
        let subdir = td.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        let mut tracker =
            DirtyTreeTracker::new_in_subpath(tree.clone(), std::path::Path::new("subdir"));
        std::fs::write(subdir.join("foo"), "bar").unwrap();
        assert_eq!(
            tracker.relpaths(),
            Some(maplit::hashset! { std::path::PathBuf::from("subdir/foo") })
        );
        assert_eq!(
            tracker.paths(),
            Some(maplit::hashset! { subdir.join("foo") })
        );
        assert_eq!(tracker.state(), State::Dirty);
    }

    #[test]
    fn test_outside_subpath() {
        let td = tempfile::tempdir().unwrap();

        let tree = create_standalone_workingtree(td.path(), &ControlDirFormat::default()).unwrap();
        let subdir = td.path().join("subdir");
        std::fs::create_dir(subdir).unwrap();
        let mut tracker =
            DirtyTreeTracker::new_in_subpath(tree.clone(), std::path::Path::new("subdir"));
        std::fs::write(td.path().join("foo"), "bar").unwrap();
        assert_eq!(tracker.relpaths(), Some(std::collections::HashSet::new()));
        assert_eq!(tracker.paths(), Some(std::collections::HashSet::new()));
        assert_eq!(tracker.state(), State::Clean);
    }

    #[test]
    fn test_in_subpath_control_only() {
        let td = tempfile::tempdir().unwrap();

        let tree = create_standalone_workingtree(td.path(), &ControlDirFormat::default()).unwrap();
        let subdir = td.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        let mut tracker =
            DirtyTreeTracker::new_in_subpath(tree.clone(), std::path::Path::new("subdir"));
        tree.commit(
            "Dummy",
            Some(true),
            Some("Joe Example <joe@example.com>)"),
            None,
        )
        .unwrap();
        assert_eq!(tracker.relpaths(), Some(std::collections::HashSet::new()));
        assert_eq!(tracker.state(), State::Clean);
        assert_eq!(tracker.paths(), Some(std::collections::HashSet::new()));
    }
}
