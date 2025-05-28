//! Support for detecting CVS repositories.
//!
//! This module provides a prober for detecting CVS repositories, but
//! does not provide any support for interacting with them.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

crate::wrapped_py!(CVSProber);

impl CVSProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.plugins.cvs") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.cvs");
                    }
                }
            };
            let cvsprober = m.getattr("CVSProber").expect("Failed to get CVSProber");
            Some(Self::from(cvsprober))
        })
    }
}

impl std::fmt::Debug for CVSProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("CVSProber({:?})", self.0))
    }
}

impl crate::controldir::PyProber for CVSProber {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_cvs_prober() {
        let _ = CVSProber::new();
    }
}
