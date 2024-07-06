use std::path::{Path, PathBuf};

pub fn is_inside(dir: &Path, fname: &Path) -> bool {
    fname.starts_with(dir)
}

pub fn is_inside_any(dir_list: &[&Path], fname: &Path) -> bool {
    for dirname in dir_list {
        if is_inside(dirname, fname) {
            return true;
        }
    }
    false
}
