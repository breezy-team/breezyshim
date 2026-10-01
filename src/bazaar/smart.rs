//! Bazaar smart protocol server bits.
//!
//! Wraps `breezy.bzr.smart.medium` for callers that need to serve
//! smart protocol requests themselves (e.g. an HTTP-fronted bzr
//! smart server). The usual flow is:
//!
//! 1. Call [`detect_protocol_factory`] on the incoming request
//!    bytes; it returns the right [`SmartProtocolFactory`] for the
//!    protocol version and any bytes that still need feeding into
//!    the protocol.
//! 2. Call [`SmartProtocolFactory::build`] with the backing
//!    transport, the write function, and the jail root. The
//!    returned [`SmartServerRequestProtocol`] is a stateful
//!    protocol handler.
//! 3. Call [`SmartServerRequestProtocol::accept_bytes`] with the
//!    unused bytes from step 1, then check
//!    [`SmartServerRequestProtocol::next_read_size`]. A non-zero
//!    size means the client's request is incomplete.
//!
//! The protocol handler writes the response bytes to the Python
//! write callable supplied at factory construction time; callers
//! typically wire this to a `BytesIO` and read its `.getvalue()`
//! back at the end.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use crate::error::Error;
use crate::transport::Transport;

/// A write sink for a [`SmartServerRequestProtocol`].
///
/// The smart protocol writes response bytes to a Python callable;
/// wrapping an `io.BytesIO` here lets callers collect the output
/// without touching PyO3 themselves.
pub struct ResponseBuffer(Py<PyAny>);

impl ResponseBuffer {
    /// Allocate a fresh `io.BytesIO` to collect protocol output.
    pub fn new() -> Result<Self, Error> {
        Python::attach(|py| -> PyResult<Self> {
            let io = py.import("io")?;
            let buf = io.call_method0("BytesIO")?;
            Ok(ResponseBuffer(buf.unbind()))
        })
        .map_err(Into::into)
    }

    /// Return the `write` bound method, ready to pass to
    /// [`SmartProtocolFactory::build`] as the `write_func`.
    pub fn write_func(&self) -> Result<Py<PyAny>, Error> {
        Python::attach(|py| -> PyResult<Py<PyAny>> {
            Ok(self.0.bind(py).getattr("write")?.unbind())
        })
        .map_err(Into::into)
    }

    /// Drain the buffer and return everything the protocol wrote so
    /// far. Consumes the buffer.
    pub fn into_bytes(self) -> Result<Vec<u8>, Error> {
        Python::attach(|py| -> PyResult<Vec<u8>> {
            self.0.bind(py).call_method0("getvalue")?.extract()
        })
        .map_err(Into::into)
    }
}

/// A protocol factory returned by [`detect_protocol_factory`].
///
/// Wraps the Python callable `(transport, write_func, root_client_path, *, jail_root=...) -> SmartServerRequestProtocol`.
pub struct SmartProtocolFactory(Py<PyAny>);

/// A smart protocol request handler.
///
/// Wraps `breezy.bzr.smart.protocol.SmartServerRequestProtocol`
/// (version-agnostic — the concrete type is picked by Breezy from
/// the incoming request bytes).
pub struct SmartServerRequestProtocol(Py<PyAny>);

/// Inspect `bytes` and return the protocol factory for whatever
/// smart-protocol version is in use, plus any bytes that haven't
/// been consumed by the version detection and must be fed to the
/// server protocol via [`SmartServerRequestProtocol::accept_bytes`].
///
/// Equivalent to:
///
/// ```python
/// from breezy.bzr.smart.medium import _get_protocol_factory_for_bytes
/// factory, unused = _get_protocol_factory_for_bytes(bytes)
/// ```
pub fn detect_protocol_factory(bytes: &[u8]) -> Result<(SmartProtocolFactory, Vec<u8>), Error> {
    Python::attach(|py| -> PyResult<(SmartProtocolFactory, Vec<u8>)> {
        let medium = py.import("breezy.bzr.smart.medium")?;
        let f = medium.getattr("_get_protocol_factory_for_bytes")?;
        let request = PyBytes::new(py, bytes);
        let pair = f.call1((request,))?;
        let factory: Py<PyAny> = pair.get_item(0)?.unbind();
        let unused: Vec<u8> = pair.get_item(1)?.extract()?;
        Ok((SmartProtocolFactory(factory), unused))
    })
    .map_err(Into::into)
}

impl SmartProtocolFactory {
    /// Build a [`SmartServerRequestProtocol`] bound to `backing`
    /// (the transport the protocol writes against), `write_func`
    /// (a Python callable that accepts bytes), `root_client_path`
    /// (typically `"."`), and `jail_root` (the top-level transport
    /// that bounds client access).
    ///
    /// `write_func` is passed through as a `Py<PyAny>` so callers
    /// can supply `BytesIO.write`, a bound method, or any other
    /// Python callable without further ceremony.
    pub fn build(
        &self,
        backing: &Transport,
        write_func: Py<PyAny>,
        root_client_path: &str,
        jail_root: &Transport,
    ) -> Result<SmartServerRequestProtocol, Error> {
        Python::attach(|py| -> PyResult<SmartServerRequestProtocol> {
            let kwargs = PyDict::new(py);
            kwargs.set_item("jail_root", jail_root.as_pyobject())?;
            let obj = self.0.bind(py).call(
                (backing.as_pyobject(), write_func, root_client_path),
                Some(&kwargs),
            )?;
            Ok(SmartServerRequestProtocol(obj.unbind()))
        })
        .map_err(Into::into)
    }
}

impl SmartServerRequestProtocol {
    /// Feed `bytes` into the protocol. For the one-shot HTTP
    /// request/response flow typically used by an HTTP smart
    /// server, this is called once with the "unused_bytes" from
    /// [`detect_protocol_factory`].
    pub fn accept_bytes(&self, bytes: &[u8]) -> Result<(), Error> {
        Python::attach(|py| -> PyResult<()> {
            let b = PyBytes::new(py, bytes);
            self.0.bind(py).call_method1("accept_bytes", (b,))?;
            Ok(())
        })
        .map_err(Into::into)
    }

    /// Return the number of bytes the protocol expects to read
    /// next. Zero means the request is complete; a non-zero value
    /// means the client's request was truncated or the protocol
    /// version is newer than this server understands.
    pub fn next_read_size(&self) -> Result<usize, Error> {
        Python::attach(|py| -> PyResult<usize> {
            let r = self.0.bind(py).call_method0("next_read_size")?;
            r.extract()
        })
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `_get_protocol_factory_for_bytes` returns some factory even
    /// when the request is empty — the point of this test is that
    /// the Python call round-trips without panicking.
    #[test]
    fn detect_protocol_factory_empty_input() {
        crate::init();
        // Empty input might legitimately raise; what we're checking
        // is that we don't panic on the Python call.
        let _ = detect_protocol_factory(b"");
    }

    /// A well-formed v1 (fallback) request parses into a factory
    /// plus the full input as `unused_bytes` — Breezy inspects just
    /// enough of the stream to pick the right protocol class, then
    /// hands the rest back for `accept_bytes`.
    #[test]
    fn detect_protocol_factory_v1_fallback_returns_input_unchanged() {
        crate::init();
        let input: &[u8] = b"bzr request 3\n";
        let (_factory, unused) = detect_protocol_factory(input).expect("factory should resolve");
        assert_eq!(unused, input.to_vec());
    }

    /// Explicit v3 marker `bzr message 3 (bzr 1.6)\n` is one of the
    /// shapes `_get_protocol_factory_for_bytes` dispatches on, so
    /// it round-trips too.
    #[test]
    fn detect_protocol_factory_v3_marker() {
        crate::init();
        let input: &[u8] = b"bzr message 3 (bzr 1.6)\n";
        let r = detect_protocol_factory(input);
        assert!(r.is_ok(), "v3 marker should parse: {:?}", r.err());
    }

    /// Full round-trip: detect factory, build a protocol, feed it a
    /// malformed body, and check `next_read_size`. The point is that
    /// the Rust/Python plumbing between all four methods stays glued
    /// together across breezy upgrades.
    #[test]
    fn full_roundtrip_via_response_buffer() {
        crate::init();
        let tmp = tempfile::tempdir().unwrap();
        let url = url::Url::from_file_path(tmp.path()).unwrap();
        let transport = crate::transport::get_transport(&url, None).unwrap();
        let jail = crate::transport::get_transport(&url, None).unwrap();

        let buffer = ResponseBuffer::new().unwrap();
        let write_func = buffer.write_func().unwrap();

        let input: &[u8] = b"bzr request 3\n";
        let (factory, unused) = detect_protocol_factory(input).unwrap();
        let proto = factory.build(&transport, write_func, ".", &jail).unwrap();
        proto.accept_bytes(&unused).unwrap();

        // Any sensible value is fine — the point is that the full
        // Rust API round-trips without panicking.
        let _ = proto.next_read_size().unwrap();
        let _ = buffer.into_bytes().unwrap();
    }
}
