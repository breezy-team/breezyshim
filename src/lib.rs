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
//! ```no_run
//! use breezyshim::branch::open as open_branch;
//! breezyshim::plugin::load_plugins();
//! let b = open_branch(&"https://code.launchpad.net/brz".parse().unwrap()).unwrap();
//! println!("Last revision: {:?}", b.last_revision());
//! ```

// Necessary for pyo3, which uses the gil-refs feature in macros
// which is not defined in breezyshim
#![allow(unexpected_cfgs)]

pub mod bazaar;
pub mod branch;
pub mod clean_tree;
pub mod commit;
pub mod config;
pub mod controldir;
pub mod cvs;
pub mod darcs;
pub mod delta;
pub mod diff;
#[cfg(feature = "dirty-tracker")]
pub mod dirty_tracker;
pub mod error;
pub mod export;
pub mod foreign;
pub mod forge;
pub mod fossil;
pub mod git;
pub mod github;
pub mod gitlab;
pub mod gpg;
pub mod graph;
pub mod hooks;
pub mod interrepository;
pub mod intertree;
pub mod launchpad;
pub mod location;
pub mod lock;
pub mod mercurial;
pub mod merge;
pub mod osutils;
pub mod patches;
pub mod plugin;
pub mod rename_map;
pub mod repository;
pub mod revisionid;
pub mod status;
pub mod subversion;
pub mod tags;
pub mod testing;
pub mod transform;
pub mod transport;
pub mod tree;
pub mod ui;
pub mod urlutils;
pub mod version;
pub mod workingtree;
pub mod workspace;

#[cfg(feature = "debian")]
pub mod debian;

pub use branch::Branch;
pub use controldir::{ControlDir, Prober};
pub use forge::{get_forge, Forge, MergeProposal, MergeProposalStatus};
pub use lock::Lock;
use pyo3::exceptions::PyImportError;
use pyo3::prelude::*;
pub use revisionid::RevisionId;
use std::sync::Once;
pub use transport::{get_transport, Transport};
pub use tree::{RevisionTree, Tree, WorkingTree};
pub use urlutils::{join_segment_parameters, split_segment_parameters};
pub use workspace::reset_tree;

pub fn init_git() {
    pyo3::Python::with_gil(|py| {
        py.import_bound("breezy.git").unwrap();
    })
}

pub fn init_bzr() {
    pyo3::Python::with_gil(|py| {
        py.import_bound("breezy.bzr").unwrap();
    })
}

#[cfg(feature = "auto-initialize")]
#[ctor::ctor]
fn ensure_initialized() {
    init();
}

const MINIMUM_VERSION: (usize, usize, usize) = (3, 3, 6);

static INIT_BREEZY: Once = Once::new();

pub fn init() {
    INIT_BREEZY.call_once(|| {
        pyo3::prepare_freethreaded_python();
        let (major, minor, micro) = pyo3::Python::with_gil(|py| match py.import_bound("breezy") {
            Ok(breezy) => {
                let (major, minor, micro, _releaselevel, _serial): (
                    usize,
                    usize,
                    usize,
                    String,
                    usize,
                ) = breezy.getattr("version_info").unwrap().extract().unwrap();
                (major, minor, micro)
            }
            Err(e) => {
                if e.is_instance_of::<PyImportError>(py) {
                    panic!("Breezy is not installed. Please install Breezy first.");
                } else {
                    Err::<(), pyo3::PyErr>(e).unwrap();
                    unreachable!()
                }
            }
        });

        if (major, minor, micro) < MINIMUM_VERSION {
            panic!(
                "Breezy version {} is too old, please upgrade to at least {}.",
                format!("{}.{}.{}", major, minor, micro),
                format!(
                    "{}.{}.{}",
                    MINIMUM_VERSION.0, MINIMUM_VERSION.1, MINIMUM_VERSION.2
                )
            );
        }

        if major >= 4 {
            log::warn!("Support for Breezy 4.0 is experimental and incomplete.");
        }

        init_git();
        init_bzr();

        // Work around a breezy bug
        pyo3::Python::with_gil(|py| {
            let m = py.import_bound("breezy.controldir").unwrap();
            let f = m.getattr("ControlDirFormat").unwrap();
            f.call_method0("known_formats").unwrap();
        });

        // Prevent race conditions
        pyo3::Python::with_gil(|py| {
            let m = py.import_bound("breezy.config").unwrap();
            m.call_method0("GlobalStack").unwrap();
            m.call_method1("LocationStack", ("file:///",)).unwrap();
        });
    });
}

pub type Result<R> = std::result::Result<R, crate::error::Error>;
