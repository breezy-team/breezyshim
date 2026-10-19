//! Chroot transport servers (`breezy.transport.chroot`, `dromedary.chroot` in Breezy 3.4).
use crate::error::Error;
use crate::transport::Transport;
use pyo3::prelude::*;

/// A chroot transport server over a backing transport; serve it with a root client path such as `/`.
pub struct ChrootServer(Py<PyAny>);

impl ChrootServer {
    /// Create a chroot server for `backing`; call [`ChrootServer::start_server`] to use it.
    pub fn new(backing: &Transport) -> Result<Self, Error> {
        Python::attach(|py| {
            let m = crate::import_first(py, &["breezy.transport.chroot", "dromedary.chroot"])?;
            let server = m.call_method1("ChrootServer", (backing.as_pyobject(),))?;
            Ok(ChrootServer(server.unbind()))
        })
    }

    /// Register the `chroot-*:///` URL scheme of this server.
    pub fn start_server(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.0.call_method0(py, "start_server")?;
            Ok(())
        })
    }

    /// Return the URL of the root of the chroot.
    pub fn get_url(&self) -> Result<url::Url, Error> {
        Python::attach(|py| {
            let url = self.0.call_method0(py, "get_url")?.extract::<String>(py)?;
            Ok(url.parse()?)
        })
    }

    /// Unregister the URL scheme of this server.
    pub fn stop_server(&self) -> Result<(), Error> {
        Python::attach(|py| {
            self.0.call_method0(py, "stop_server")?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ChrootServer;
    use crate::transport::get_transport;

    #[test]
    fn test_chroot_server() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir(td.path().join("root")).unwrap();
        std::fs::write(td.path().join("root/README"), b"").unwrap();
        let backing_url = url::Url::from_directory_path(td.path().join("root")).unwrap();
        let backing = get_transport(&backing_url, None).unwrap();

        let server = ChrootServer::new(&backing).unwrap();
        server.start_server().unwrap();
        let url = server.get_url().unwrap();
        let chroot = get_transport(&url, None).unwrap();
        assert_eq!(chroot.base(), url);
        assert!(chroot.has("README").unwrap());

        server.stop_server().unwrap();
        assert!(get_transport(&url, None).is_err());
    }
}
