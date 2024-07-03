use pyo3::prelude::*;
use pyo3::types::{PyList,PyBytes};
use crate::transform::TreeTransform;
use patchkit::patch::{UnifiedPatch, HunkLine};

fn py_patches(iter_patches: impl Iterator<Item = UnifiedPatch>) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let m = py.import_bound("breezy.patches")?;
        let patchc = m.getattr("Patch")?;
        let hunkc = m.getattr("Hunk")?;
        let insertlinec = m.getattr("InsertLine")?;
        let removelinec = m.getattr("RemoveLine")?;
        let contextlinec = m.getattr("ContextLine")?;
        let mut ret = vec![];
        for patch in iter_patches {
            let pypatch = patchc.call1((patch.orig_name, patch.mod_name, patch.orig_ts, patch.mod_ts))?;
            let pyhunks = pypatch.getattr("hunks")?;

            for hunk in patch.hunks {
                let pyhunk = hunkc.call1((hunk.orig_pos, hunk.orig_range, hunk.mod_pos, hunk.mod_range, hunk.tail))?;
                pyhunks.call_method1("append", (&pyhunk, ))?;

                let pylines = pyhunk.getattr("lines")?;

                for line in hunk.lines {
                    pylines.call_method1("append", (
                        match line {
                            HunkLine::ContextLine(l) => contextlinec.call1((PyBytes::new_bound(py, l.as_slice()), ))?,
                            HunkLine::InsertLine(l) => insertlinec.call1((PyBytes::new_bound(py, l.as_slice()), ))?,
                            HunkLine::RemoveLine(l) => removelinec.call1((PyBytes::new_bound(py, l.as_slice()), ))?
                        }, ))?;
                }
            }
            ret.push(pypatch);
        }
        Ok(PyList::new_bound(py, ret.iter()).into_py(py))
    })
}

/// Apply patches to a TreeTransform.
///
/// # Arguments
/// * `tt`: TreeTransform instance
/// * `patches`: List of patches
/// * `prefix`: Number leading path segments to strip
pub fn apply_patches(tt: &TreeTransform, patches: impl Iterator<Item = UnifiedPatch>, prefix: Option<usize>) -> crate::Result<()> {
    Python::with_gil(|py| {
        let patches = py_patches(patches)?;
        let m = py.import_bound("breezy.tree")?;
        let apply_patches = m.getattr("apply_patches")?;
        apply_patches.call1((tt.to_object(py), patches, prefix))?;
        Ok(())
    })
}
