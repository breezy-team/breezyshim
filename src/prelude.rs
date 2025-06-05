//! Prelude for the breezyshim crate
//!
//! This module re-exports commonly used items from the crate,
pub use crate::branch::Branch;
pub use crate::controldir::ControlDir;
pub use crate::error::Error as BrzError;
pub use crate::repository::Repository;
pub use crate::revisionid::RevisionId;
pub use crate::tree::{MutableTree, Tree};
pub use crate::workingtree::WorkingTree;
