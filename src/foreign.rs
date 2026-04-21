//! Support for foreign version control systems.
//!
//! This module provides types and traits for interacting with various
//! version control systems supported by Breezy.

use crate::revisionid::RevisionId;
use pyo3::prelude::*;

/// Type of version control system.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VcsType {
    /// Bazaar version control system.
    Bazaar,
    /// Git version control system.
    Git,
    /// Mercurial version control system.
    Hg,
    /// Subversion version control system.
    Svn,
    /// Fossil version control system.
    Fossil,
    /// Darcs version control system.
    Darcs,
    /// CVS version control system.
    Cvs,
    /// GNU Arch version control system.
    Arch,
    /// SVK version control system.
    Svk,
}

/// Parsed foreign-revision-id: the abbreviation of the VCS
/// (e.g. `"git"`, `"hg"`) and a display-friendly string form of
/// the foreign revid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignInfo {
    /// Short VCS name, e.g. `"git"`, `"hg"`, `"svn"`.
    pub abbreviation: String,
    /// Human-readable foreign revid. For git this is the SHA-1.
    pub foreign_revid: String,
}

/// If `revid` is a foreign revision id (as produced by one of the
/// bzr-git / bzr-hg / bzr-svn plugins), parse and return the
/// foreign-VCS abbreviation and revid form. Returns `None` for
/// native bzr revids or when the foreign plugin isn't loaded.
pub fn parse_foreign_revid(revid: &RevisionId) -> Option<ForeignInfo> {
    // Fast path: a foreign revid always contains a `:` separator;
    // native bzr ones are `<author>-<timestamp>-<slug>`. Bail out
    // quickly on the common case.
    if !revid.as_bytes().contains(&b':') {
        return None;
    }
    Python::attach(|py| -> Option<ForeignInfo> {
        let foreign = py.import("breezy.foreign").ok()?;
        let registry = foreign.getattr("foreign_vcs_registry").ok()?;
        let result = registry
            .call_method1("parse_revision_id", (revid.as_bytes().to_vec(),))
            .ok()?;
        let foreign_revid = result.get_item(0).ok()?;
        let mapping = result.get_item(1).ok()?;
        let vcs = mapping.getattr("vcs").ok()?;
        let abbreviation: String = vcs.getattr("abbreviation").ok()?.extract().ok()?;
        // show_foreign_revid returns a dict like {"git commit": "<sha>"};
        // take the first (only) value.
        let shown = vcs
            .call_method1("show_foreign_revid", (foreign_revid,))
            .ok()?;
        let values = shown
            .call_method0("values")
            .ok()?
            .try_iter()
            .ok()?
            .next()?
            .ok()?;
        let foreign_revid_str: String = values.extract().ok()?;
        Some(ForeignInfo {
            abbreviation,
            foreign_revid: foreign_revid_str,
        })
    })
}
