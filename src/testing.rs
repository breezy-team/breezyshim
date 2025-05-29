//! Test utilities for the Breezy Rust bindings.
use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Environment for running Breezy tests.
///
/// This struct sets up a temporary environment for running Breezy tests,
/// including temporary directories and environment variables, and cleans
/// up after itself when dropped.
pub struct TestEnv {
    /// The temporary directory that contains all test files.
    pub temp_dir: TempDir,
    /// The working directory where tests will run.
    pub working_dir: PathBuf,
    /// The home directory for the test environment.
    pub home_dir: PathBuf,
    /// The original working directory before the test environment was set up.
    pub old_cwd: PathBuf,
    /// The original environment variables before the test environment was set up.
    pub old_env: HashMap<String, Option<String>>,
}

impl TestEnv {
    /// Create a new testing environment.
    ///
    /// This sets up a temporary directory structure with a working directory
    /// and home directory, and configures environment variables for Breezy.
    ///
    /// # Returns
    ///
    /// A new TestEnv instance
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().join("test");
        fs::create_dir(&working_dir).unwrap();
        let home_dir = temp_dir.path().join("home");
        fs::create_dir(&home_dir).unwrap();
        let mut old_env = HashMap::new();
        let old_cwd = std::env::current_dir().unwrap();
        old_env.insert("HOME".to_string(), std::env::var("HOME").ok());
        old_env.insert("BRZ_EMAIL".to_string(), std::env::var("BRZ_EMAIL").ok());
        old_env.insert("BRZ_HOME".to_string(), std::env::var("BRZ_HOME").ok());
        let brz_email = "Joe Tester <joe@example.com>";
        let breezy_home = home_dir.join(".config/breezy");
        std::env::set_current_dir(&working_dir).unwrap();
        std::env::set_var("HOME", &home_dir);
        std::env::set_var("BRZ_EMAIL", brz_email);
        std::env::set_var("BRZ_HOME", &breezy_home);
        pyo3::Python::with_gil(|py| {
            let os = py.import("os").unwrap();
            os.call_method1("chdir", (working_dir.to_str().unwrap(),))
                .unwrap();
            let environ = os.getattr("environ").unwrap();
            environ
                .set_item("HOME", home_dir.to_str().unwrap())
                .unwrap();
            environ.set_item("BRZ_EMAIL", brz_email).unwrap();
            environ
                .set_item("BRZ_HOME", breezy_home.to_str().unwrap())
                .unwrap();
        });
        fs::create_dir_all(&breezy_home).unwrap();
        fs::write(
            breezy_home.join("breezy.conf"),
            r#"
[DEFAULT]
email = Joe Tester <joe@example.com>
"#,
        )
        .unwrap();
        Self {
            temp_dir,
            home_dir,
            working_dir,
            old_cwd,
            old_env,
        }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        for (key, value) in self.old_env.drain() {
            if let Some(value) = value.as_ref() {
                std::env::set_var(&key, value);
            } else {
                std::env::remove_var(&key);
            }
            Python::with_gil(|py| {
                let os = py.import("os").unwrap();
                let environ = os.getattr("environ").unwrap();
                if let Some(value) = value {
                    environ.set_item(key, value).unwrap();
                } else {
                    environ.del_item(key).unwrap();
                }
            });
        }
        let _ = std::env::set_current_dir(&self.old_cwd);
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_testenv() {
        let env = TestEnv::new();
        assert_eq!(env.home_dir, env.temp_dir.path().join("home"));
        assert_eq!(env.working_dir, env.temp_dir.path().join("test"));
        assert_eq!(
            std::env::current_dir().unwrap(),
            env.working_dir.canonicalize().unwrap()
        );
        assert_eq!(
            std::env::var("HOME").unwrap(),
            env.home_dir.to_str().unwrap()
        );
        assert_eq!(
            std::env::var("BRZ_EMAIL").unwrap(),
            "Joe Tester <joe@example.com>"
        );

        Python::with_gil(|py| {
            let os = py.import("os").unwrap();
            assert_eq!(
                os.call_method0("getcwd")
                    .unwrap()
                    .extract::<PathBuf>()
                    .unwrap(),
                env.working_dir.canonicalize().unwrap(),
            );
            assert_eq!(
                os.call_method1("getenv", ("HOME",))
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                env.home_dir.to_str().unwrap()
            );
            assert_eq!(
                os.call_method1("getenv", ("BRZ_EMAIL",))
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "Joe Tester <joe@example.com>"
            );
        });
    }
}
