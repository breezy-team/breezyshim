//! This crate contains a rust wrapper for the Breezy API, which is written in Python.
//!
//! Breezy itself is being ported to Rust, but until that port has completed, this crate allows
//! access to the most important Breezy APIs via Rust.
//!
//! The Rust API here will follow the Breezy 4.0 Rust API as much as possible, to make porting
//! easier.
//!
//! # Example
//!
//! ```
//! use breezyshim::branch::open as open_branch;
//! breezyshim::plugin::load_plugins();
//! let b = open_branch(&"lp:brz".parse().unwrap()).unwrap();
//! println!("Last revision: {:?}", b.last_revision());
//! ```

pub mod bazaar;
pub mod branch;
pub mod controldir;
pub mod delta;
pub mod diff;
pub mod dirty_tracker;
pub mod export;
pub mod forge;
pub mod github;
pub mod gpg;
pub mod graph;
pub mod hooks;
pub mod intertree;
pub mod location;
pub mod lock;
pub mod merge;
pub mod plugin;
pub mod rename_map;
pub mod repository;
pub mod revisionid;
pub mod status;
pub mod tags;
pub mod transform;
pub mod transport;
pub mod tree;
pub mod urlutils;
pub mod version;
pub mod workspace;

#[cfg(feature = "debian")]
pub mod debian;

pub use branch::Branch;
pub use controldir::{ControlDir, Prober};
pub use dirty_tracker::DirtyTracker;
pub use forge::{get_forge, Forge, MergeProposal, MergeProposalStatus};
pub use lock::Lock;
use pyo3::exceptions::PyImportError;
pub use revisionid::RevisionId;
use std::sync::Once;
pub use transport::{get_transport, Transport};
pub use tree::{RevisionTree, Tree, WorkingTree};
pub use urlutils::{join_segment_parameters, split_segment_parameters};
pub use workspace::reset_tree;

pub fn init_git() {
    pyo3::Python::with_gil(|py| {
        py.import("breezy.git").unwrap();
    })
}

pub fn init_bzr() {
    pyo3::Python::with_gil(|py| {
        py.import("breezy.bzr").unwrap();
    })
}

#[derive(Debug)]
pub struct BreezyNotInstalled {
    pub message: String,
}

impl std::fmt::Display for BreezyNotInstalled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Breezy is not installed: {}", self.message)
    }
}

static INIT_BREEZY: Once = Once::new();

pub fn init() -> Result<(), BreezyNotInstalled> {
    INIT_BREEZY.call_once(|| {
        pyo3::Python::with_gil(|py| match py.import("breezy") {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.is_instance_of::<PyImportError>(py) {
                    Err(BreezyNotInstalled {
                        message: e.to_string(),
                    })
                } else {
                    Err::<(), pyo3::PyErr>(e).unwrap();
                    unreachable!()
                }
            }
        })
        .expect("Breezy is not installed");

        init_git();
        init_bzr();

        // Work around a breezy bug
        pyo3::Python::with_gil(|py| {
            let m = py.import("breezy.controldir").unwrap();
            let f = m.getattr("ControlDirFormat").unwrap();
            f.call_method0("known_formats").unwrap();
        });
    });
    Ok(())
}

pub fn load_plugins() {
    pyo3::Python::with_gil(|py| {
        let m = py.import("breezy.plugin").unwrap();
        m.call_method0("load_plugins").unwrap();
    })
}
