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
