//! Optional integration with the `bzr-search` plugin
//! (`breezy.plugins.search`).
//!
//! The plugin is rarely installed. Callers should treat this module as
//! optional: [`is_available`] reports whether the plugin was importable,
//! and every other function returns a [`SearchUnavailable`] error if it
//! is not.

use crate::branch::PyBranch;
use crate::revisionid::RevisionId;
use pyo3::prelude::*;
use std::sync::OnceLock;

/// Error returned when the `breezy.plugins.search` plugin is not
/// installed or the branch hasn't been indexed.
#[derive(Debug)]
pub enum SearchError {
    /// The `breezy.plugins.search` plugin isn't importable.
    Unavailable,
    /// The branch exists but has no search index built for it.
    NoIndex,
    /// Any other error from the plugin bubbles up as-is.
    Other(crate::error::Error),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::Unavailable => write!(f, "breezy search plugin is not installed"),
            SearchError::NoIndex => write!(f, "branch has no search index"),
            SearchError::Other(e) => write!(f, "breezy error: {e}"),
        }
    }
}

impl std::error::Error for SearchError {}

impl From<crate::error::Error> for SearchError {
    fn from(e: crate::error::Error) -> Self {
        SearchError::Other(e)
    }
}

/// Returns true iff `breezy.plugins.search` can be imported.
pub fn is_available() -> bool {
    static CELL: OnceLock<bool> = OnceLock::new();
    *CELL.get_or_init(|| Python::attach(|py| py.import("breezy.plugins.search.index").is_ok()))
}

/// A single search result.
#[derive(Debug, Clone)]
pub enum Hit {
    /// Hit from commit-message / metadata text.
    Revision(RevisionId),
    /// Hit from file-text indexing.
    FileText {
        /// Revision in which the hit occurred.
        revision: RevisionId,
        /// Path (as a UTF-8 string; Breezy's file-id/path may contain
        /// non-UTF-8 bytes, which are replaced with U+FFFD).
        path: String,
    },
}

/// Suggest query terms matching the prefix in `query` (used for
/// autocomplete). Returns a sorted, deduplicated list.
pub fn suggest<B: PyBranch>(branch: &B, query: &str) -> Result<Vec<String>, SearchError> {
    if !is_available() {
        return Err(SearchError::Unavailable);
    }
    Python::attach(|py| -> PyResult<Vec<String>> {
        let m = py.import("breezy.plugins.search.index")?;
        let errors = py.import("breezy.plugins.search.errors")?;
        let open_index_branch = m.getattr("open_index_branch")?;
        let py_branch = branch.to_object(py);
        let index = match open_index_branch.call1((py_branch,)) {
            Ok(i) => i,
            Err(e) => {
                let no_index = errors.getattr("NoSearchIndex")?;
                if e.is_instance(py, &no_index) {
                    return Ok(Vec::new());
                }
                return Err(e);
            }
        };
        let terms: Vec<(String,)> = {
            let query_tuples: Vec<(String,)> =
                query.split_whitespace().map(|t| (t.to_string(),)).collect();
            let raw = index.call_method1("suggest", (query_tuples,))?;
            raw.extract()?
        };
        let mut out: Vec<String> = terms.into_iter().map(|(t,)| t).collect();
        out.sort();
        out.dedup();
        Ok(out)
    })
    .map_err(|e| SearchError::Other(e.into()))
    .and_then(|v| {
        if v.is_empty() {
            Err(SearchError::NoIndex)
        } else {
            Ok(v)
        }
    })
}

/// Run a full search against the branch's index, returning a
/// deduplicated list of hits.
pub fn search<B: PyBranch>(branch: &B, query: &str) -> Result<Vec<Hit>, SearchError> {
    if !is_available() {
        return Err(SearchError::Unavailable);
    }
    Python::attach(|py| -> PyResult<Vec<Hit>> {
        let m = py.import("breezy.plugins.search.index")?;
        let errors = py.import("breezy.plugins.search.errors")?;
        let open_index_branch = m.getattr("open_index_branch")?;
        let py_branch = branch.to_object(py);
        let index = match open_index_branch.call1((py_branch,)) {
            Ok(i) => i,
            Err(e) => {
                let no_index = errors.getattr("NoSearchIndex")?;
                if e.is_instance(py, &no_index) {
                    return Ok(Vec::new());
                }
                return Err(e);
            }
        };
        let file_hit = m.getattr("FileTextHit")?;
        let revision_hit = m.getattr("RevisionHit")?;
        let query_tuples: Vec<(String,)> =
            query.split_whitespace().map(|t| (t.to_string(),)).collect();
        let results = index.call_method1("search", (query_tuples,))?;
        let mut hits = Vec::new();
        for item in results.try_iter()? {
            let item = item?;
            if item.is_instance(&file_hit)? {
                // FileTextHit.text_key is (file_id, revid)
                let tk = item.getattr("text_key")?;
                let revision: RevisionId = tk.get_item(1)?.extract()?;
                let file_id_bytes: Vec<u8> = tk.get_item(0)?.extract()?;
                hits.push(Hit::FileText {
                    revision,
                    path: String::from_utf8_lossy(&file_id_bytes).into_owned(),
                });
            } else if item.is_instance(&revision_hit)? {
                // RevisionHit.revision_key is (revid,)
                let rk = item.getattr("revision_key")?;
                let revision: RevisionId = rk.get_item(0)?.extract()?;
                hits.push(Hit::Revision(revision));
            }
        }
        Ok(hits)
    })
    .map_err(|e| SearchError::Other(e.into()))
}
