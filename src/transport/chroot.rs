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
