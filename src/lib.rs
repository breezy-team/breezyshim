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
//! use breezyshim::prelude::*;
//! use breezyshim::branch::open as open_branch;
//! breezyshim::plugin::load_plugins();
//! let b = open_branch(&"https://code.launchpad.net/brz".parse().unwrap()).unwrap();
//! println!("Last revision: {:?}", b.last_revision());
//! ```

#![deny(missing_docs)]
// Necessary for pyo3, which uses the gil-refs feature in macros
// which is not defined in breezyshim
#![allow(unexpected_cfgs)]
// TODO: Fix large error enum variants by boxing large fields
#![allow(clippy::result_large_err)]

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
/// Group compression versioned files implementation
pub mod groupcompress;
pub mod hooks;
pub mod interrepository;
pub mod intertree;
/// Knit versioned files implementation
pub mod knit;
#[cfg(feature = "launchpad")]
pub mod launchpad;
pub mod location;
pub mod lock;
pub mod mercurial;
pub mod merge;
pub mod osutils;
pub mod patches;
pub mod plugin;
pub mod prelude;
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
/// Versioned files API for storing file content history
pub mod versionedfiles;
/// Weave versioned files implementation
pub mod weave;
pub mod workingtree;
pub mod workspace;

#[cfg(feature = "debian")]
pub mod debian;

// Re-export core types and functions
/// Branch trait representing a branch in a version control system
pub use branch::Branch;
/// Control directory traits and types
pub use controldir::{ControlDir, Prober};
/// Forge related types and functions for interacting with source code hosting services
pub use forge::{get_forge, Forge, MergeProposal, MergeProposalStatus};
/// Lock type for managing access to resources
pub use lock::Lock;
use pyo3::exceptions::PyImportError;
use pyo3::prelude::*;
/// Revision identifier type
pub use revisionid::RevisionId;
use std::sync::Once;
/// Transport functions and types for accessing remote repositories
pub use transport::{get_transport, Transport};
/// Tree-related traits and types
pub use tree::{RevisionTree, Tree, WorkingTree};
/// URL utility functions
pub use urlutils::{join_segment_parameters, split_segment_parameters};
/// Workspace functions
pub use workspace::reset_tree;

/// Initialize Git support in Breezy.
///
/// This function imports the breezy.git module to ensure Git functionality is available.
pub fn init_git() {
    pyo3::Python::attach(|py| {
        py.import("breezy.git").unwrap();
    })
}

/// Initialize Bazaar support in Breezy.
///
/// This function imports the breezy.bzr module to ensure Bazaar functionality is available.
pub fn init_bzr() {
    pyo3::Python::attach(|py| {
        py.import("breezy.bzr").unwrap();
    })
}

#[cfg(feature = "auto-initialize")]
/// Initialize
#[ctor::ctor]
fn ensure_initialized() {
    init();
}

/// The minimum supported Breezy version.
const MINIMUM_VERSION: (usize, usize, usize) = (3, 3, 6);

/// Initialization lock to ensure Breezy is only initialized once.
static INIT_BREEZY: Once = Once::new();

/// Initialize the Breezy library and Python interpreter.
///
/// This function ensures Python is initialized and Breezy is loaded.
/// It is safe to call multiple times.
///
/// This function ensures that Breezy is properly initialized, checking version
/// compatibility and loading required modules. It should be called before
/// using any other functionality in this crate unless the "auto-initialize"
/// feature is enabled.
///
/// # Panics
///
/// - If Breezy is not installed
/// - If the installed Breezy version is too old
pub fn init() {
    INIT_BREEZY.call_once(|| {
        pyo3::Python::initialize();
        let (major, minor, micro) = pyo3::Python::attach(|py| match py.import("breezy") {
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
                    panic!("{}", e);
                }
            }
        });

        if (major, minor, micro) < MINIMUM_VERSION {
            panic!(
                "Breezy version {}.{}.{} is too old, please upgrade to at least {}.{}.{}.",
                major, minor, micro, MINIMUM_VERSION.0, MINIMUM_VERSION.1, MINIMUM_VERSION.2
            );
        }

        if major >= 4 {
            log::warn!("Support for Breezy 4.0 is experimental and incomplete.");
        }

        init_git();
        init_bzr();

        // Work around a breezy bug
        pyo3::Python::attach(|py| {
            let m = py.import("breezy.controldir").unwrap();
            let f = m.getattr("ControlDirFormat").unwrap();
            f.call_method0("known_formats").unwrap();
        });

        // Prevent race conditions
        pyo3::Python::attach(|py| {
            let m = py.import("breezy.config").unwrap();
            m.call_method0("GlobalStack").unwrap();
            m.call_method1("LocationStack", ("file:///",)).unwrap();
        });
    });
}

/// Shorthand for the standard result type used throughout this crate.
pub type Result<R> = std::result::Result<R, crate::error::Error>;
