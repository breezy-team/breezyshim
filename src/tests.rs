use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestEnv {
    pub temp_dir: TempDir,
    pub working_dir: PathBuf,
    pub home_dir: PathBuf,
    pub old_cwd: PathBuf,
    pub old_env: HashMap<String, Option<String>>,
}

impl TestEnv {
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
            let os = py.import_bound("os").unwrap();
            os.call_method1("chdir", (working_dir.to_str().unwrap(),))
                .unwrap();
            os.call_method1("putenv", ("HOME", home_dir.to_str().unwrap()))
                .unwrap();
            os.call_method1("putenv", ("BRZ_EMAIL", brz_email)).unwrap();
            os.call_method1("putenv", ("BRZ_HOME", breezy_home.to_str().unwrap()))
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
                let os = py.import_bound("os").unwrap();
                if let Some(value) = value {
                    os.call_method1("putenv", (key, value)).unwrap();
                } else {
                    os.call_method1("unsetenv", (key,)).unwrap();
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
