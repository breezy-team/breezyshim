use pyo3::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestEnv {
    pub temp_dir: TempDir,
    pub working_dir: PathBuf,
    pub home_dir: PathBuf,
    pub old_cwd: PathBuf,
    pub old_home: Option<String>,
    pub old_email: Option<String>,
    pub old_brz_home: Option<String>,
}

impl TestEnv {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().join("test");
        fs::create_dir(&working_dir).unwrap();
        let home_dir = temp_dir.path().join("home");
        fs::create_dir(&home_dir).unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        let old_home = std::env::var("HOME").ok();
        let old_email = std::env::var("BRZ_EMAIL").ok();
        let old_brz_home = std::env::var("BRZ_HOME").ok();
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
            old_home,
            old_email,
            old_brz_home,
        }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        if let Some(dir) = self.old_home.as_ref() {
            std::env::set_var("HOME", dir);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(email) = self.old_email.as_ref() {
            std::env::set_var("BRZ_EMAIL", email);
        } else {
            std::env::remove_var("BRZ_EMAIL");
        }
        if let Some(dir) = self.old_brz_home.as_ref() {
            std::env::set_var("BRZ_HOME", dir);
        } else {
            std::env::remove_var("BRZ_HOME");
        }
        let _ = std::env::set_current_dir(&self.old_cwd);
        pyo3::Python::with_gil(|py| {
            let os = py.import_bound("os").unwrap();
            os.call_method1("chdir", (self.old_cwd.to_str().unwrap(),))
                .unwrap();
            if let Some(dir) = self.old_home.as_ref() {
                os.call_method1("putenv", ("HOME", dir)).unwrap();
            } else {
                os.call_method1("unsetenv", ("HOME",)).unwrap();
            }
            if let Some(email) = self.old_email.as_ref() {
                os.call_method1("putenv", ("BRZ_EMAIL", email)).unwrap();
            } else {
                os.call_method1("unsetenv", ("BRZ_EMAIL",)).unwrap();
            }
            if let Some(dir) = self.old_brz_home.as_ref() {
                os.call_method1("putenv", ("BRZ_HOME", dir)).unwrap();
            } else {
                os.call_method1("unsetenv", ("BRZ_HOME",)).unwrap();
            }
        });
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}
