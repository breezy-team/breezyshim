//! OS-specific utilities.
use std::path::Path;

/// Check if a file is inside a directory.
///
/// # Arguments
///
/// * `dir` - The directory to check
/// * `fname` - The file path to check
///
/// # Returns
///
/// `true` if the file is inside the directory, `false` otherwise
pub fn is_inside(dir: &Path, fname: &Path) -> bool {
    fname.starts_with(dir)
}

/// Check if a file is inside any of the directories in a list.
///
/// # Arguments
///
/// * `dir_list` - The list of directories to check
/// * `fname` - The file path to check
///
/// # Returns
///
/// `true` if the file is inside any of the directories, `false` otherwise
pub fn is_inside_any(dir_list: &[&Path], fname: &Path) -> bool {
    for dirname in dir_list {
        if is_inside(dirname, fname) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_inside_basic() {
        let dir = Path::new("/home/user");
        let file = Path::new("/home/user/document.txt");
        assert!(is_inside(dir, file));
    }

    #[test]
    fn test_is_inside_not_inside() {
        let dir = Path::new("/home/user");
        let file = Path::new("/home/other/document.txt");
        assert!(!is_inside(dir, file));
    }

    #[test]
    fn test_is_inside_nested() {
        let dir = Path::new("/home/user");
        let file = Path::new("/home/user/subdir/document.txt");
        assert!(is_inside(dir, file));
    }

    #[test]
    fn test_is_inside_same_path() {
        let dir = Path::new("/home/user");
        let file = Path::new("/home/user");
        assert!(is_inside(dir, file));
    }

    #[test]
    fn test_is_inside_relative_paths() {
        let dir = Path::new("user");
        let file = Path::new("user/document.txt");
        assert!(is_inside(dir, file));
    }

    #[test]
    fn test_is_inside_any_found() {
        let dirs = vec![
            Path::new("/home/user1"),
            Path::new("/home/user2"),
            Path::new("/home/user3"),
        ];
        let dir_refs: Vec<&Path> = dirs.iter().map(|p| *p).collect();
        let file = Path::new("/home/user2/document.txt");
        assert!(is_inside_any(&dir_refs, file));
    }

    #[test]
    fn test_is_inside_any_not_found() {
        let dirs = vec![
            Path::new("/home/user1"),
            Path::new("/home/user2"),
            Path::new("/home/user3"),
        ];
        let dir_refs: Vec<&Path> = dirs.iter().map(|p| *p).collect();
        let file = Path::new("/home/other/document.txt");
        assert!(!is_inside_any(&dir_refs, file));
    }

    #[test]
    fn test_is_inside_any_empty_list() {
        let dirs: Vec<&Path> = vec![];
        let file = Path::new("/home/user/document.txt");
        assert!(!is_inside_any(&dirs, file));
    }

    #[test]
    fn test_is_inside_any_first_match() {
        let dirs = vec![Path::new("/home/user"), Path::new("/home/user/subdir")];
        let dir_refs: Vec<&Path> = dirs.iter().map(|p| *p).collect();
        let file = Path::new("/home/user/subdir/document.txt");
        // Should match the first one that matches
        assert!(is_inside_any(&dir_refs, file));
    }
}
