//! Mercurial prober.
//!
//! This allows detecting Mercurial repositories, but does not provide any
//! functionality to interact with them.
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;

crate::wrapped_py!(SmartHgProber);

impl SmartHgProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import("breezy.plugins.hg") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.plugins.hg");
                    }
                }
            };
            let prober = m
                .getattr("SmartHgProber")
                .expect("Failed to get SmartHgProber");
            Some(Self(prober.unbind()))
        })
    }
}

impl crate::controldir::PyProber for SmartHgProber {}

impl std::fmt::Debug for SmartHgProber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("SmartHgProber({:?})", self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_hg_prober() {
        let _ = SmartHgProber::new();
    }
}
