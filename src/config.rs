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

impl ToPyObject for BranchConfig {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.clone_ref(py)
    }
}

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

pub struct Credentials {
    pub scheme: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub path: Option<String>,
    pub realm: Option<String>,
    pub verify_certificates: Option<bool>,
}

impl FromPyObject<'_> for Credentials {
    fn extract_bound(ob: &Bound<PyAny>) -> PyResult<Self> {
        let scheme = ob.get_item("scheme")?.extract()?;
        let username = ob.get_item("username")?.extract()?;
        let password = ob.get_item("password")?.extract()?;
        let host = ob.get_item("host")?.extract()?;
        let port = ob.get_item("port")?.extract()?;
        let path = ob.get_item("path")?.extract()?;
        let realm = ob.get_item("realm")?.extract()?;
        let verify_certificates = ob.get_item("verify_certificates")?.extract()?;

        Ok(Credentials {
            scheme,
            username,
            password,
            host,
            port,
            path,
            realm,
            verify_certificates,
        })
    }
}

impl ToPyObject for Credentials {
    fn to_object(&self, py: Python) -> PyObject {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("scheme", &self.scheme).unwrap();
        dict.set_item("username", &self.username).unwrap();
        dict.set_item("password", &self.password).unwrap();
        dict.set_item("host", &self.host).unwrap();
        dict.set_item("port", &self.port).unwrap();
        dict.set_item("path", &self.path).unwrap();
        dict.set_item("realm", &self.realm).unwrap();
        dict.set_item("verify_certificates", &self.verify_certificates)
            .unwrap();
        dict.into()
    }
}

impl IntoPy<PyObject> for Credentials {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

pub trait CredentialStore: Send {
    fn get_credentials(
        &self,
        scheme: &str,
        host: &str,
        port: Option<i64>,
        user: Option<&str>,
        path: Option<&str>,
        realm: Option<&str>,
    ) -> Result<Credentials>;
}

struct PyCredentialStore(PyObject);

impl CredentialStore for PyCredentialStore {
    fn get_credentials(
        &self,
        scheme: &str,
        host: &str,
        port: Option<i64>,
        user: Option<&str>,
        path: Option<&str>,
        realm: Option<&str>,
    ) -> Result<Credentials> {
        Python::with_gil(|py| -> Result<Credentials> {
            let creds = self.0.call_method1(
                py,
                "get_credentials",
                (scheme, host, port, user, path, realm),
            )?;
            Ok(creds.extract(py)?)
        })
    }
}

#[pyclass]
pub struct CredentialStoreWrapper(Box<dyn CredentialStore>);

#[pymethods]
impl CredentialStoreWrapper {
    #[pyo3(signature = (scheme, host, port=None, user=None, path=None, realm=None))]
    fn get_credentials(
        &self,
        scheme: &str,
        host: &str,
        port: Option<i64>,
        user: Option<&str>,
        path: Option<&str>,
        realm: Option<&str>,
    ) -> PyResult<Credentials> {
        self.0
            .get_credentials(scheme, host, port, user, path, realm)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string()))
    }
}

pub struct CredentialStoreRegistry(PyObject);

impl CredentialStoreRegistry {
    pub fn new() -> Self {
        Python::with_gil(|py| -> Self {
            let m = py.import_bound("breezy.config").unwrap();
            let registry = m.call_method0("CredentialStoreRegistry").unwrap();
            Self(registry.to_object(py))
        })
    }

    pub fn get_credential_store(
        &self,
        encoding: Option<&str>,
    ) -> Result<Option<Box<dyn CredentialStore>>> {
        Python::with_gil(|py| -> Result<Option<Box<dyn CredentialStore>>> {
            let store = match self.0.call_method1(py, "get_credential_store", (encoding,)) {
                Ok(store) => store,
                Err(e) if e.is_instance_of::<pyo3::exceptions::PyKeyError>(py) => {
                    return Ok(None);
                }
                Err(e) => {
                    return Err(e.into());
                }
            };
            Ok(Some(Box::new(PyCredentialStore(store))))
        })
    }

    pub fn get_fallback_credentials(
        &self,
        scheme: &str,
        port: Option<i64>,
        user: Option<&str>,
        path: Option<&str>,
        realm: Option<&str>,
    ) -> Result<Credentials> {
        Python::with_gil(|py| -> Result<Credentials> {
            let creds = self.0.call_method1(
                py,
                "get_fallback_credentials",
                (scheme, port, user, path, realm),
            )?;
            Ok(creds.extract(py)?)
        })
    }

    pub fn register(&self, key: &str, store: Box<dyn CredentialStore>) -> Result<()> {
        Python::with_gil(|py| -> Result<()> {
            self.0
                .call_method1(py, "register", (key, CredentialStoreWrapper(store)))?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn register_fallback(&self, store: Box<dyn CredentialStore>) -> Result<()> {
        Python::with_gil(|py| -> Result<()> {
            let kwargs = pyo3::types::PyDict::new_bound(py);
            kwargs.set_item("fallback", true)?;
            self.0.call_method_bound(
                py,
                "register_fallback",
                (CredentialStoreWrapper(store),),
                Some(&kwargs),
            )?;
            Ok(())
        })?;
        Ok(())
    }
}

lazy_static::lazy_static! {
    pub static ref CREDENTIAL_STORE_REGISTRY: CredentialStoreRegistry =
        CredentialStoreRegistry::new()
    ;
}

#[test]
fn test_credential_store() {
    let env = crate::testing::TestEnv::new();
    let store = CREDENTIAL_STORE_REGISTRY
        .get_credential_store(None)
        .unwrap();
    assert!(store.is_none());
    std::mem::drop(env);
}
