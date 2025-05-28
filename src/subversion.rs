//! Subversion repository prober.
//!
//! This module provides a prober for Subversion repositories, but no actual
//! implementation is provided.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

crate::wrapped_py!(SvnRepositoryProber);

impl SvnRepositoryProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.plugins.svn") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.svn");
                    }
                }
            };
            let prober = m
                .getattr("SvnRepositoryProber")
                .expect("Failed to get SvnRepositoryProber");
            Some(Self::from(prober))
        })
    }
}

impl crate::controldir::PyProber for SvnRepositoryProber {}

impl std::fmt::Debug for SvnRepositoryProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("SvnRepositoryProber({:?})", self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let _ = SvnRepositoryProber::new();
    }
}
