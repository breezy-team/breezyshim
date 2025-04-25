use pyo3::prelude::*;

/// Convert a Git URL from a Debian VCS field to a Breezy URL.
///
/// # Arguments
/// * `url` - The Git URL to convert
///
/// # Returns
/// The converted URL
pub fn vcs_git_url_to_bzr_url(url: &str) -> url::Url {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.plugins.debian.directory").unwrap();
        m.call_method1("vcs_git_url_to_bzr_url", (url,))
            .unwrap()
            .extract::<String>()
            .unwrap()
            .parse()
            .unwrap()
    })
}
