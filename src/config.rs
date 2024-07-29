use crate::Result;
use pyo3::prelude::*;

pub fn parse_username(e: &str) -> (String, String) {
    if let Some((_, username, email)) =
        lazy_regex::regex_captures!(r"(.*?)\s*<?([\[\]\w+.-]+@[\w+.-]+)>?", e)
    {
        (username.to_string(), email.to_string())
    } else {
        (e.to_string(), "".to_string())
    }
}

pub fn extract_email_address(e: &str) -> Option<String> {
    let (_name, email) = parse_username(e);

    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

#[test]
fn test_parse_username() {
    assert_eq!(
        parse_username("John Doe <joe@example.com>"),
        ("John Doe".to_string(), "joe@example.com".to_string())
    );
    assert_eq!(
        parse_username("John Doe"),
        ("John Doe".to_string(), "".to_string())
    );
}

#[test]
fn test_extract_email_address() {
    assert_eq!(
        extract_email_address("John Doe <joe@example.com>"),
        Some("joe@example.com".to_string())
    );
    assert_eq!(extract_email_address("John Doe"), None);
}

pub trait ConfigValue: ToPyObject {}

impl ConfigValue for String {}
impl ConfigValue for str {}
impl ConfigValue for i64 {}
impl ConfigValue for bool {}

#[derive(Clone)]
pub struct BranchConfig(PyObject);

impl BranchConfig {
    pub fn new(o: PyObject) -> Self {
        Self(o)
    }

    pub fn set_user_option(&self, key: &str, value: &impl ConfigValue) -> Result<()> {
        Python::with_gil(|py| -> Result<()> {
            self.0
                .call_method1(py, "set_user_option", (key, value.to_object(py)))?;
            Ok(())
        })?;
        Ok(())
    }
}

pub struct ConfigStack(PyObject);

impl ConfigStack {
    pub fn new(o: PyObject) -> Self {
        Self(o)
    }

    pub fn get(&self, key: &str) -> Result<Option<PyObject>> {
        Python::with_gil(|py| -> Result<Option<PyObject>> {
            let value = self.0.call_method1(py, "get", (key,))?;
            if value.is_none(py) {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        })
    }
}

pub fn global_stack() -> Result<ConfigStack> {
    Python::with_gil(|py| -> Result<ConfigStack> {
        let m = py.import_bound("breezy.config")?;
        let stack = m.call_method0("GlobalStack")?;
        Ok(ConfigStack::new(stack.to_object(py)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_config_stack() {
        let env = crate::testing::TestEnv::new();
        let stack = global_stack().unwrap();
        stack.get("email").unwrap();
        std::mem::drop(env);
    }
}
