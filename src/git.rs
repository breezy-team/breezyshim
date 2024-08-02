use crate::controldir::Prober;
use crate::error::Error;
use pyo3::exceptions::PyModuleNotFoundError;
use pyo3::prelude::*;
use std::collections::HashMap;

pub struct RemoteGitProber(PyObject);

impl RemoteGitProber {
    pub fn new() -> Option<Self> {
        Python::with_gil(|py| {
            let m = match py.import_bound("breezy.git") {
                Ok(m) => m,
                Err(e) => {
                    if e.is_instance_of::<PyModuleNotFoundError>(py) {
                        return None;
                    } else {
                        e.print_and_set_sys_last_vars(py);
                        panic!("Failed to import breezy.git");
                    }
                }
            };
            let prober = m
                .getattr("RemoteGitProber")
                .expect("Failed to get GitProber");
            Some(Self(prober.to_object(py)))
        })
    }
}

impl FromPyObject<'_> for RemoteGitProber {
    fn extract_bound(obj: &Bound<PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_object(obj.py())))
    }
}

impl ToPyObject for RemoteGitProber {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Prober for RemoteGitProber {}

pub struct BareLocalGitControlDirFormat(PyObject);

impl BareLocalGitControlDirFormat {
    pub fn new() -> Self {
        Python::with_gil(|py| {
            let m = py
                .import_bound("breezy.git")
                .expect("Failed to import breezy.git");
            let format = m
                .getattr("BareLocalGitControlDirFormat")
                .expect("Failed to get BareLocalGitControlDirFormat");

            Self(
                format
                    .call0()
                    .expect("Failed to create BareLocalGitControlDirFormat")
                    .to_object(py),
            )
        })
    }
}

impl ToPyObject for BareLocalGitControlDirFormat {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl crate::controldir::AsFormat for BareLocalGitControlDirFormat {
    fn as_format(&self) -> Option<crate::controldir::ControlDirFormat> {
        Some(crate::controldir::ControlDirFormat::from(self.0.clone()))
    }
}

pub static ZERO_SHA: &[u8; 40] = b"0000000000000000000000000000000000000000";

pub trait InterGitRepository: crate::interrepository::InterRepository {
    fn fetch_refs(
        &self,
        get_changed_refs: std::sync::Mutex<
            Box<dyn FnMut(&HashMap<String, Vec<u8>>) -> HashMap<String, Vec<u8>> + Send>,
        >,
        lossy: bool,
        overwrite: bool,
    ) -> Result<(), Error> {
        Python::with_gil(|py| {
            let get_changed_refs = pyo3::types::PyCFunction::new_closure_bound(
                py,
                None,
                None,
                move |args, _kwargs| {
                    let refs = args.extract::<(HashMap<String, Vec<u8>>,)>().unwrap().0;
                    // Call get_changed_refs
                    if let Ok(mut get_changed_refs) = get_changed_refs.lock() {
                        get_changed_refs(&refs)
                    } else {
                        refs
                    }
                },
            )
            .unwrap();
            self.to_object(py).call_method1(
                py,
                "fetch_refs",
                (get_changed_refs, lossy, overwrite),
            )?;
            Ok(())
        })
    }
}
