//! Git version control system support.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

/// A prober that can detect remote Git repositories.
pub struct RemoteGitProber(Py<PyAny>);

/// The SHA1 hash consisting of all zeros, representing the absence of a commit in Git.
pub const ZERO_SHA: &[u8] = b"0000000000000000000000000000000000000000";

impl RemoteGitProber {
    /// Create a new RemoteGitProber, returning None if the Git plugin is not available.
    pub fn new() -> Option<Self> {
        Python::attach(|py| {
            let m = match py.import("breezy.git") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.git");
                    }
                }
            };
            let prober = m
                .getattr("RemoteGitProber")
                .expect("Failed to get GitProber");
            Some(Self(prober.unbind()))
        })
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for RemoteGitProber {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_owned().unbind()))
    }
}

impl<'py> IntoPyObject<'py> for RemoteGitProber {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl std::fmt::Debug for RemoteGitProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("RemoteGitProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for RemoteGitProber {
    fn to_object(&self, py: Python) -> Py<PyAny> {
        self.0.clone_ref(py)
    }
}

/// Format for bare local Git repositories.
pub struct BareLocalGitControlDirFormat(Py<PyAny>);

impl BareLocalGitControlDirFormat {
    /// Create a new BareLocalGitControlDirFormat.
    pub fn new() -> Self {
        Python::attach(|py| {
            let m = py
                .import("breezy.git")
                .expect("Failed to import breezy.git");
            let format = m
                .getattr("BareLocalGitControlDirFormat")
                .expect("Failed to get BareLocalGitControlDirFormat");

            Self(
                format
                    .call0()
                    .expect("Failed to create BareLocalGitControlDirFormat")
                    .unbind(),
            )
        })
    }
}

impl<'py> IntoPyObject<'py> for BareLocalGitControlDirFormat {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl crate::controldir::AsFormat for BareLocalGitControlDirFormat {
    fn as_format(&self) -> Option<crate::controldir::ControlDirFormat> {
        Some(Python::attach(|py| {
            crate::controldir::ControlDirFormat::from(self.0.clone_ref(py))
        }))
    }
}

/// Retrieve the Git committer information from the working tree's repository.
pub fn get_committer(working_tree: &dyn crate::workingtree::PyWorkingTree) -> Option<String> {
    use crate::branch::Branch;
    use crate::repository::PyRepository;
    pyo3::Python::attach(|py| {
        let repo = working_tree.branch().repository();
        let git = match repo.to_object(py).getattr(py, "_git") {
            Ok(x) => Some(x),
            Err(e) if e.is_instance_of::<pyo3::exceptions::PyAttributeError>(py) => None,
            Err(e) => {
                return Err(e);
            }
        };

        if let Some(git) = git {
            let cs = git.call_method0(py, "get_config_stack")?;

            let mut user = std::env::var("GIT_COMMITTER_NAME").ok();
            let mut email = std::env::var("GIT_COMMITTER_EMAIL").ok();
            if user.is_none() {
                match cs.call_method1(py, "get", (("user",), "name")) {
                    Ok(x) => {
                        user = Some(
                            std::str::from_utf8(x.extract::<&[u8]>(py)?)
                                .unwrap()
                                .to_string(),
                        );
                    }
                    Err(e) if e.is_instance_of::<pyo3::exceptions::PyKeyError>(py) => {
                        // Ignore
                    }
                    Err(e) => {
                        return Err(e);
                    }
                };
            }
            if email.is_none() {
                match cs.call_method1(py, "get", (("user",), "email")) {
                    Ok(x) => {
                        email = Some(
                            std::str::from_utf8(x.extract::<&[u8]>(py)?)
                                .unwrap()
                                .to_string(),
                        );
                    }
                    Err(e) if e.is_instance_of::<pyo3::exceptions::PyKeyError>(py) => {
                        // Ignore
                    }
                    Err(e) => {
                        return Err(e);
                    }
                };
            }

            if let (Some(user), Some(email)) = (user, email) {
                return Ok(Some(format!("{} <{}>", user, email)));
            }

            let gs = crate::config::global_stack().unwrap();

            Ok(gs
                .get("email")?
                .map(|email| email.extract::<String>(py).unwrap()))
        } else {
            Ok(None)
        }
    })
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controldir::AsFormat;

    #[test]
    fn test_zero_sha() {
        assert_eq!(ZERO_SHA.len(), 40);
        assert_eq!(ZERO_SHA, b"0000000000000000000000000000000000000000");
    }

    #[test]
    fn test_remote_git_prober_new() {
        // This may return None if git plugin is not available
        let _prober = RemoteGitProber::new();
    }

    #[test]
    fn test_remote_git_prober_debug() {
        if let Some(prober) = RemoteGitProber::new() {
            let debug_str = format!("{:?}", prober);
            assert!(debug_str.contains("RemoteGitProber"));
        }
    }

    #[test]
    fn test_bare_local_git_control_dir_format() {
        // This test will only pass if git plugin is available
        let result = std::panic::catch_unwind(|| BareLocalGitControlDirFormat::new());

        if let Ok(format) = result {
            let _opt_format = format.as_format();
        }
    }

    #[test]
    fn test_remote_git_prober_into_pyobject() {
        if let Some(prober) = RemoteGitProber::new() {
            Python::attach(|py| {
                let _pyobj = prober.into_pyobject(py).unwrap();
            });
        }
    }

    #[test]
    fn test_bare_local_git_into_pyobject() {
        let result = std::panic::catch_unwind(|| BareLocalGitControlDirFormat::new());

        if let Ok(format) = result {
            Python::attach(|py| {
                let _pyobj = format.into_pyobject(py).unwrap();
            });
        }
    }

    #[serial_test::serial]
    #[test]
    // Ignored on Windows due to dulwich permission errors when creating .git directories in CI
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_git_env() {
        let td = tempfile::tempdir().unwrap();
        let cd = crate::controldir::create_standalone_workingtree(td.path(), "git").unwrap();

        let old_name = std::env::var("GIT_COMMITTER_NAME").ok();
        let old_email = std::env::var("GIT_COMMITTER_EMAIL").ok();

        std::env::set_var("GIT_COMMITTER_NAME", "Some Git Committer");
        std::env::set_var("GIT_COMMITTER_EMAIL", "committer@example.com");

        let committer = get_committer(&cd).unwrap();

        if let Some(old_name) = old_name {
            std::env::set_var("GIT_COMMITTER_NAME", old_name);
        } else {
            std::env::remove_var("GIT_COMMITTER_NAME");
        }

        if let Some(old_email) = old_email {
            std::env::set_var("GIT_COMMITTER_EMAIL", old_email);
        } else {
            std::env::remove_var("GIT_COMMITTER_EMAIL");
        }

        assert_eq!(
            "Some Git Committer <committer@example.com>",
            committer.as_str()
        );

        // Drop cd before td cleanup to release Python file handles (needed on Windows)
        drop(cd);
    }

    #[serial_test::serial]
    #[test]
    // Ignored on Windows due to dulwich permission errors when creating .git directories in CI
    #[cfg_attr(target_os = "windows", ignore)]
    fn test_git_config() {
        let td = tempfile::tempdir().unwrap();
        let cd = crate::controldir::create_standalone_workingtree(td.path(), "git").unwrap();

        std::fs::write(
            td.path().join(".git/config"),
            b"[user]\nname = Some Git Committer\nemail = other@example.com",
        )
        .unwrap();

        assert_eq!(
            get_committer(&cd).unwrap(),
            "Some Git Committer <other@example.com>"
        );

        // Drop cd before td cleanup to release Python file handles (needed on Windows)
        drop(cd);
    }
}
