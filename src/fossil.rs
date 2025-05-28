//! Support for detecting Fossil repositories.
//!
//! This module provides a prober for detecting Fossil repositories, but
//! currently does not provide any additional functionality.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

crate::wrapped_py!(RemoteFossilProber);

impl RemoteFossilProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.plugins.fossil") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.fossil");
                    }
                }
            };
            let prober = m
                .getattr("RemoteFossilProber")
                .expect("Failed to get RemoteFossilProber");
            Some(Self::from(prober))
        })
    }
}

impl crate::controldir::PyProber for RemoteFossilProber {}

impl std::fmt::Debug for RemoteFossilProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("RemoteFossilProber({:?})", self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_fossil_prober() {
        let _ = RemoteFossilProber::new();
    }
}
