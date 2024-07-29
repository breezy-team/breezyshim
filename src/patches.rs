use crate::transform::TreeTransform;
use crate::tree::Tree;
use patchkit::patch::{HunkLine, UnifiedPatch};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};

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
            let pypatch = patchc.call1((
                PyBytes::new_bound(py, &patch.orig_name),
                PyBytes::new_bound(py, &patch.mod_name),
                patch.orig_ts,
                patch.mod_ts,
            ))?;
            let pyhunks = pypatch.getattr("hunks")?;

            for hunk in patch.hunks {
                let pyhunk = hunkc.call1((
                    hunk.orig_pos,
                    hunk.orig_range,
                    hunk.mod_pos,
                    hunk.mod_range,
                    hunk.tail,
                ))?;
                pyhunks.call_method1("append", (&pyhunk,))?;

                let pylines = pyhunk.getattr("lines")?;

                for line in hunk.lines {
                    pylines.call_method1(
                        "append",
                        (match line {
                            HunkLine::ContextLine(l) => {
                                contextlinec.call1((PyBytes::new_bound(py, l.as_slice()),))?
                            }
                            HunkLine::InsertLine(l) => {
                                insertlinec.call1((PyBytes::new_bound(py, l.as_slice()),))?
                            }
                            HunkLine::RemoveLine(l) => {
                                removelinec.call1((PyBytes::new_bound(py, l.as_slice()),))?
                            }
                        },),
                    )?;
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
pub fn apply_patches(
    tt: &TreeTransform,
    patches: impl Iterator<Item = UnifiedPatch>,
    prefix: Option<usize>,
) -> crate::Result<()> {
    Python::with_gil(|py| {
        let patches = py_patches(patches)?;
        let m = py.import_bound("breezy.tree")?;
        let apply_patches = m.getattr("apply_patches")?;
        apply_patches.call1((tt.to_object(py), patches, prefix))?;
        Ok(())
    })
}

pub struct AppliedPatches(PyObject, PyObject);

impl AppliedPatches {
    pub fn new(
        tree: &dyn Tree,
        patches: Vec<UnifiedPatch>,
        prefix: Option<usize>,
    ) -> crate::Result<Self> {
        let (ap, tree) = Python::with_gil(|py| -> Result<_, PyErr> {
            let patches = py_patches(patches.into_iter())?;
            let m = py.import_bound("breezy.patches")?;
            let c = m.getattr("AppliedPatches")?;
            let ap = c.call1((tree.to_object(py), patches, prefix))?;
            let tree = ap.call_method0("__enter__")?;
            Ok((ap.to_object(py), tree.to_object(py)))
        })?;
        Ok(Self(tree, ap))
    }
}

impl Drop for AppliedPatches {
    fn drop(&mut self) {
        Python::with_gil(|py| -> Result<(), PyErr> {
            self.1
                .call_method1(py, "__exit__", (py.None(), py.None(), py.None()))?;
            Ok(())
        })
        .unwrap();
    }
}

impl ToPyObject for AppliedPatches {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl Tree for AppliedPatches {}

#[cfg(test)]
mod applied_patches_tests {
    use crate::controldir::ControlDirFormat;
    use crate::tree::Tree;
    use patchkit::patch::UnifiedPatch;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_apply_simple() {
        let env = crate::testing::TestEnv::new();
        let td = tempfile::tempdir().unwrap();
        let tree = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        std::fs::write(td.path().join("a"), "a\n").unwrap();
        tree.add(&[std::path::Path::new("a")]).unwrap();
        tree.commit("Add a", None, None, None).unwrap();
        let patch = UnifiedPatch::parse_patch(patchkit::parse::splitlines(
            br#"--- a/a
+++ b/a
@@ -1 +1 @@
-a
+b
"#,
        ))
        .unwrap();

        let newtree = crate::patches::AppliedPatches::new(&tree, vec![patch], None).unwrap();
        assert_eq!(
            b"b\n".to_vec(),
            newtree.get_file_text(std::path::Path::new("a")).unwrap()
        );
        std::mem::drop(newtree);
        std::mem::drop(env);
    }

    #[test]
    #[serial]
    fn test_apply_delete() {
        let env = crate::testing::TestEnv::new();
        let td = tempfile::tempdir().unwrap();
        let tree = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        std::fs::write(td.path().join("a"), "a\n").unwrap();
        tree.add(&[std::path::Path::new("a")]).unwrap();
        tree.commit("Add a", None, None, None).unwrap();
        let patch = patchkit::patch::UnifiedPatch::parse_patch(patchkit::parse::splitlines(
            br#"--- a/a
+++ /dev/null
@@ -1 +0,0 @@
-a
"#,
        ))
        .unwrap();
        let newtree = crate::patches::AppliedPatches::new(&tree, vec![patch], None).unwrap();
        assert!(!newtree.has_filename(std::path::Path::new("a")));
        std::mem::drop(env);
    }

    #[test]
    #[serial]
    fn test_apply_add() {
        let env = crate::testing::TestEnv::new();
        let td = tempfile::tempdir().unwrap();
        let tree = crate::controldir::create_standalone_workingtree(
            td.path(),
            &ControlDirFormat::default(),
        )
        .unwrap();
        std::fs::write(td.path().join("a"), "a\n").unwrap();
        tree.add(&[std::path::Path::new("a")]).unwrap();
        tree.commit("Add a", None, None, None).unwrap();
        let patch = patchkit::patch::UnifiedPatch::parse_patch(patchkit::parse::splitlines(
            br#"--- /dev/null
+++ b/b
@@ -0,0 +1 @@
+b
"#,
        ))
        .unwrap();
        let newtree = crate::patches::AppliedPatches::new(&tree, vec![patch], None).unwrap();
        assert_eq!(
            b"b\n".to_vec(),
            newtree.get_file_text(std::path::Path::new("b")).unwrap()
        );
        std::mem::drop(env);
    }
}
