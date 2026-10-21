//! Configuration handling.
//!
//! This module provides access to the Breezy configuration system.
//! It allows reading and writing configuration values, and provides
//! access to credential stores.
use crate::Result;
use pyo3::conversion::IntoPyObjectExt;
use pyo3::prelude::*;

/// Parse a username string into name and email components.
///
/// # Parameters
///
/// * `e` - The username string to parse, typically in the format "Name <email@example.com>".
///
/// # Returns
///
/// A tuple containing the name and email address. If no email address is present,
/// the second element will be an empty string.
pub fn parse_username(e: &str) -> (String, String) {
    if let Some((_, username, email)) =
        lazy_regex::regex_captures!(r"(.*?)\s*<?([\[\]\w+.-]+@[\w+.-]+)>?", e)
    {
        (username.to_string(), email.to_string())
    } else {
        (e.to_string(), "".to_string())
    }
}

/// Extract an email address from a username string.
///
/// # Parameters
///
/// * `e` - The username string to extract an email address from.
///
/// # Returns
///
/// The email address, or None if no email address is present.
pub fn extract_email_address(e: &str) -> Option<String> {
    let (_name, email) = parse_username(e);

    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

/// Trait for values that can be stored in a configuration.
///
/// This trait is implemented for common types like strings, integers, and booleans,
/// and can be implemented for other types that need to be stored in a configuration.
pub trait ConfigValue: for<'py> IntoPyObject<'py> {}

impl ConfigValue for String {}
impl ConfigValue for &str {}
impl ConfigValue for i64 {}
impl ConfigValue for bool {}

/// Configuration for a branch.
///
/// This struct wraps a Python branch configuration object and provides methods for
/// accessing and modifying branch-specific configuration options.
pub struct BranchConfig(Py<PyAny>);

impl Clone for BranchConfig {
    fn clone(&self) -> Self {
        Python::attach(|py| -> Self { Self(self.0.clone_ref(py)) })
    }
}

impl<'py> IntoPyObject<'py> for BranchConfig {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> std::result::Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl BranchConfig {
    /// Create a new BranchConfig from a Python object.
    ///
    /// # Parameters
    ///
    /// * `o` - A Python object representing a branch configuration.
    ///
    /// # Returns
    ///
    /// A new BranchConfig instance.
    pub fn new(o: Py<PyAny>) -> Self {
        Self(o)
    }

    /// Set a user option in this branch configuration.
    ///
    /// # Parameters
    ///
    /// * `key` - The option key to set.
    /// * `value` - The value to set the option to.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the option could not be set.
    pub fn set_user_option<T: ConfigValue>(&self, key: &str, value: T) -> Result<()> {
        Python::attach(|py| -> Result<()> {
            let py_value = value.into_py_any(py)?;
            self.0
                .call_method1(py, "set_user_option", (key, py_value))?;
            Ok(())
        })?;
        Ok(())
    }

    /// Get a user option from this branch configuration, as a string.
    ///
    /// Returns `None` if the option is unset.
    pub fn get_user_option(&self, key: &str) -> Result<Option<String>> {
        Python::attach(|py| -> Result<Option<String>> {
            let value = self.0.call_method1(py, "get_user_option", (key,))?;
            if value.is_none(py) {
                Ok(None)
            } else {
                Ok(Some(value.extract(py)?))
            }
        })
    }

    /// Get a user option as a bool. Breezy interprets any of the
    /// strings "true", "yes", "1", "on", "enable" (case-insensitive)
    /// as true; "false", "no", "0", "off", "disable" as false. Unset
    /// keys, empty strings, and unrecognised values fall back to
    /// `default`.
    pub fn get_user_option_as_bool(&self, key: &str, default: bool) -> Result<bool> {
        Python::attach(|py| -> Result<bool> {
            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("default", default)?;
            let value = self
                .0
                .call_method(py, "get_user_option_as_bool", (key,), Some(&kwargs))?;
            Ok(value.extract(py)?)
        })
    }

    /// Get the branch's nickname as reported by Breezy, falling back to the
    /// directory name if no explicit nickname is set.
    pub fn get_nickname(&self) -> Result<String> {
        Python::attach(|py| -> Result<String> {
            Ok(self.0.call_method0(py, "get_nickname")?.extract(py)?)
        })
    }
}

/// A stack of configuration sources.
///
/// This struct represents a stack of configuration sources, where more specific
/// sources (like branch-specific configuration) override more general sources
/// (like global configuration).
pub struct ConfigStack(Py<PyAny>);

impl ConfigStack {
    /// Create a new ConfigStack from a Python object.
    ///
    /// # Parameters
    ///
    /// * `o` - A Python object representing a configuration stack.
    ///
    /// # Returns
    ///
    /// A new ConfigStack instance.
    pub fn new(o: Py<PyAny>) -> Self {
        Self(o)
    }

    /// Get a configuration value from this stack.
    ///
    /// # Parameters
    ///
    /// * `key` - The configuration key to get.
    ///
    /// # Returns
    ///
    /// The configuration value, or None if the key is not present.
    pub fn get(&self, key: &str) -> Result<Option<Py<PyAny>>> {
        Python::attach(|py| -> Result<Option<Py<PyAny>>> {
            let value = self.0.call_method1(py, "get", (key,))?;
            if value.is_none(py) {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        })
    }

    /// Set a configuration value in this stack.
    ///
    /// # Parameters
    ///
    /// * `key` - The configuration key to set.
    /// * `value` - The value to set the configuration to.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the configuration could not be set.
    pub fn set<T: ConfigValue>(&self, key: &str, value: T) -> Result<()> {
        Python::attach(|py| -> Result<()> {
            let py_value = value.into_py_any(py)?;
            self.0.call_method1(py, "set", (key, py_value))?;
            Ok(())
        })?;
        Ok(())
    }
}

/// Get the global configuration stack.
///
/// # Returns
///
/// The global configuration stack, or an error if it could not be retrieved.
pub fn global_stack() -> Result<ConfigStack> {
    Python::attach(|py| -> Result<ConfigStack> {
        let m = py.import("breezy.config")?;
        let stack = m.call_method0("GlobalStack")?;
        Ok(ConfigStack::new(stack.unbind()))
    })
}

/// Credentials for accessing a remote service.
///
/// This struct contains the credentials for accessing a remote service,
/// such as username, password, host, port, etc.
pub struct Credentials {
    /// The scheme of the service, like "https", "ftp", etc.
    pub scheme: Option<String>,
    /// The username for authenticating with the service.
    pub username: Option<String>,
    /// The password for authenticating with the service.
    pub password: Option<String>,
    /// The hostname of the service.
    pub host: Option<String>,
    /// The port number of the service.
    pub port: Option<i64>,
    /// The path on the service.
    pub path: Option<String>,
    /// The authentication realm of the service.
    pub realm: Option<String>,
    /// Whether to verify SSL certificates when connecting to the service.
    pub verify_certificates: Option<bool>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for Credentials {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let get = |key: &str| ob.call_method1("get", (key,));
        let scheme = get("scheme")?.extract()?;
        let username = get("user")?.extract()?;
        let password = get("password")?.extract()?;
        let host = get("host")?.extract()?;
        let port = get("port")?.extract()?;
        let path = get("path")?.extract()?;
        let realm = get("realm")?.extract()?;
        let verify_certificates = get("verify_certificates")?.extract()?;

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

impl<'py> IntoPyObject<'py> for Credentials {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> std::result::Result<Self::Output, Self::Error> {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("scheme", &self.scheme).unwrap();
        dict.set_item("user", &self.username).unwrap();
        dict.set_item("password", &self.password).unwrap();
        dict.set_item("host", &self.host).unwrap();
        dict.set_item("port", self.port).unwrap();
        dict.set_item("path", &self.path).unwrap();
        dict.set_item("realm", &self.realm).unwrap();
        dict.set_item("verify_certificates", self.verify_certificates)
            .unwrap();
        Ok(dict.into_any())
    }
}

// IntoPy is replaced by IntoPyObject in PyO3 0.25
// The IntoPyObject implementation above handles the conversion

/// A store for retrieving credentials.
///
/// This trait defines the interface for a credential store, which can be used to
/// retrieve credentials for accessing remote services. Implementations of this trait
/// can store credentials in different ways, like in a keychain, a config file, etc.
pub trait CredentialStore: Send + Sync {
    /// Get credentials for accessing a remote service.
    ///
    /// # Parameters
    ///
    /// * `scheme` - The scheme of the service, like "https", "ftp", etc.
    /// * `host` - The hostname of the service.
    /// * `port` - The port number of the service, or None for the default port.
    /// * `user` - The username to use, or None to use the default username.
    /// * `path` - The path on the service, or None for the root path.
    /// * `realm` - The authentication realm, or None for the default realm.
    ///
    /// # Returns
    ///
    /// The credentials for accessing the service, None if this store has no
    /// credentials for it, or an error if the credentials could not be retrieved.
    fn get_credentials(
        &self,
        scheme: &str,
        host: &str,
        port: Option<i64>,
        user: Option<&str>,
        path: Option<&str>,
        realm: Option<&str>,
    ) -> Result<Option<Credentials>>;
}

struct PyCredentialStore(Py<PyAny>);

impl CredentialStore for PyCredentialStore {
    fn get_credentials(
        &self,
        scheme: &str,
        host: &str,
        port: Option<i64>,
        user: Option<&str>,
        path: Option<&str>,
        realm: Option<&str>,
    ) -> Result<Option<Credentials>> {
        Python::attach(|py| -> Result<Option<Credentials>> {
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
/// A wrapper for a `CredentialStore` that can be exposed to Python.
///
/// This struct wraps a `CredentialStore` implementation and exposes it to Python
/// through the Pyo3 type system.
pub struct CredentialStoreWrapper(Box<dyn CredentialStore + Send + Sync>);

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
    ) -> PyResult<Option<Credentials>> {
        self.0
            .get_credentials(scheme, host, port, user, path, realm)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string()))
    }
}

/// A registry of credential stores.
///
/// This struct wraps a Python credential store registry, which can be used to
/// register and retrieve credential stores.
pub struct CredentialStoreRegistry(Py<PyAny>);

impl CredentialStoreRegistry {
    /// Create a new, empty `CredentialStoreRegistry`.
    ///
    /// This is separate from Breezy's global registry; use
    /// [`credential_store_registry`] to register stores that Breezy should use.
    ///
    /// # Returns
    ///
    /// A new `CredentialStoreRegistry` instance.
    pub fn new() -> Self {
        Python::attach(|py| -> Self {
            let m = py.import("breezy.config").unwrap();
            let registry = m.call_method0("CredentialStoreRegistry").unwrap();
            Self(registry.unbind())
        })
    }

    /// Get a credential store from this registry.
    ///
    /// # Parameters
    ///
    /// * `encoding` - The encoding of the credential store, or None for the default encoding.
    ///
    /// # Returns
    ///
    /// The credential store, or None if no credential store was found for the specified encoding.
    pub fn get_credential_store(
        &self,
        encoding: Option<&str>,
    ) -> Result<Option<Box<dyn CredentialStore>>> {
        Python::attach(|py| -> Result<Option<Box<dyn CredentialStore>>> {
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

    /// Get fallback credentials for accessing a remote service.
    ///
    /// # Parameters
    ///
    /// * `scheme` - The scheme of the service, like "https", "ftp", etc.
    /// * `host` - The hostname of the service.
    /// * `port` - The port number of the service, or None for the default port.
    /// * `user` - The username to use, or None to use the default username.
    /// * `path` - The path on the service, or None for the root path.
    /// * `realm` - The authentication realm, or None for the default realm.
    ///
    /// # Returns
    ///
    /// The fallback credentials for accessing the service, None if no fallback
    /// store has credentials for it, or an error if the credentials could not
    /// be retrieved.
    pub fn get_fallback_credentials(
        &self,
        scheme: &str,
        host: &str,
        port: Option<i64>,
        user: Option<&str>,
        path: Option<&str>,
        realm: Option<&str>,
    ) -> Result<Option<Credentials>> {
        Python::attach(|py| -> Result<Option<Credentials>> {
            let creds = self.0.call_method1(
                py,
                "get_fallback_credentials",
                (scheme, host, port, user, path, realm),
            )?;
            Ok(creds.extract(py)?)
        })
    }

    /// Register a credential store with this registry.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to register the credential store under.
    /// * `store` - The credential store to register.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the store could not be registered.
    pub fn register(&self, key: &str, store: Box<dyn CredentialStore>) -> Result<()> {
        Python::attach(|py| -> Result<()> {
            self.0
                .call_method1(py, "register", (key, CredentialStoreWrapper(store)))?;
            Ok(())
        })?;
        Ok(())
    }

    /// Register a fallback credential store with this registry.
    ///
    /// Fallback stores are queried when authentication.conf has no matching section.
    ///
    /// # Parameters
    ///
    /// * `key` - The key to register the credential store under.
    /// * `store` - The credential store to register as a fallback.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the store could not be registered.
    pub fn register_fallback(&self, key: &str, store: Box<dyn CredentialStore>) -> Result<()> {
        Python::attach(|py| -> Result<()> {
            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("fallback", true)?;
            self.0.call_method(
                py,
                "register",
                (key, CredentialStoreWrapper(store)),
                Some(&kwargs),
            )?;
            Ok(())
        })?;
        Ok(())
    }
}

impl Default for CredentialStoreRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Return Breezy's global credential store registry.
///
/// Stores registered here are used by Breezy when it looks up credentials.
pub fn credential_store_registry() -> CredentialStoreRegistry {
    Python::attach(|py| {
        let m = py.import("breezy.config").unwrap();
        CredentialStoreRegistry(m.getattr("credential_store_registry").unwrap().unbind())
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_credential_store() {
        fn takes_config_value<T: crate::config::ConfigValue>(_t: T) {}

        takes_config_value("foo");
        takes_config_value(1);
        takes_config_value(true);
        takes_config_value("foo".to_string());
    }

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

    struct HostStore;

    impl CredentialStore for HostStore {
        fn get_credentials(
            &self,
            scheme: &str,
            host: &str,
            port: Option<i64>,
            _user: Option<&str>,
            path: Option<&str>,
            realm: Option<&str>,
        ) -> Result<Option<Credentials>> {
            if host != "creds.example.com" {
                return Ok(None);
            }
            Ok(Some(Credentials {
                scheme: Some(scheme.to_string()),
                username: Some("worker".to_string()),
                password: Some("secret".to_string()),
                host: Some(host.to_string()),
                port,
                path: path.map(|p| p.to_string()),
                realm: realm.map(|r| r.to_string()),
                verify_certificates: Some(true),
            }))
        }
    }

    #[test]
    fn test_register_fallback_global() {
        crate::init();
        credential_store_registry()
            .register_fallback("test-host-store", Box::new(HostStore))
            .unwrap();
        Python::attach(|py| {
            let auth = py
                .import("breezy.config")
                .unwrap()
                .call_method0("AuthenticationConfig")
                .unwrap();
            let user: Option<String> = auth
                .call_method1("get_user", ("https", "creds.example.com"))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(user.as_deref(), Some("worker"));
            let password: String = auth
                .call_method1("get_password", ("https", "creds.example.com", "worker"))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(password, "secret");
            let user: Option<String> = auth
                .call_method1("get_user", ("https", "other.example.com"))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(user, None);
        });
    }

    #[test]
    fn test_get_fallback_credentials() {
        crate::init();
        let registry = CredentialStoreRegistry::new();
        assert!(registry
            .get_fallback_credentials("https", "creds.example.com", None, None, None, None)
            .unwrap()
            .is_none());
        registry
            .register_fallback("host-store", Box::new(HostStore))
            .unwrap();
        let creds = registry
            .get_fallback_credentials("https", "creds.example.com", None, None, None, None)
            .unwrap()
            .unwrap();
        assert_eq!(creds.username.as_deref(), Some("worker"));
        assert_eq!(creds.host.as_deref(), Some("creds.example.com"));
        assert!(registry
            .get_fallback_credentials("https", "other.example.com", None, None, None, None)
            .unwrap()
            .is_none());
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
}
