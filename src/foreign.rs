//! Support for foreign version control systems.
//!
//! This module provides types and traits for interacting with various
//! version control systems supported by Breezy.

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
