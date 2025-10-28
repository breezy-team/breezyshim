#[cfg(test)]
mod tests {
    use crate::graph::Key;
    use crate::init;
    use crate::versionedfiles::{AbsentContentFactory, FulltextContentFactory, RecordOrdering};

    #[test]
    fn test_key_conversion() {
        let key = Key::from(vec!["file1".to_string(), "rev1".to_string()]);
        let vec: Vec<String> = key.clone().into();
        assert_eq!(vec, vec!["file1", "rev1"]);
    }

    #[test]
    fn test_fulltext_content_factory() {
        init();
        let key = Key::from(vec!["file1".to_string()]);
        let factory = FulltextContentFactory::new(
            Some("abc123".to_string()),
            "fulltext".to_string(),
            key,
            Some(vec![]),
        );
        assert_eq!(factory.sha1, Some("abc123".to_string()));
        assert_eq!(factory.storage_kind, "fulltext");
    }

    #[test]
    fn test_absent_content_factory() {
        let key = Key::from(vec!["file1".to_string()]);
        let parents = vec![Key::from(vec!["parent1".to_string()])];
        let factory = AbsentContentFactory::new(key, parents);
        assert_eq!(factory.parents.len(), 1);
    }

    #[test]
    fn test_weave_basic_operations() {
        init();
        pyo3::Python::attach(|py| {
            // Import the weave module
            py.import("breezy.bzr.weave").ok();
            let weave = crate::weave::Weave::new_empty(py).unwrap();

            // Add initial version
            weave
                .add_lines("v1", vec![], vec!["line1\n", "line2\n"])
                .unwrap();

            // Get text back
            let text = weave.get_text("v1").unwrap();
            assert_eq!(text, vec!["line1\n", "line2\n"]);

            // Add child version
            weave
                .add_lines("v2", vec!["v1"], vec!["line1\n", "line2\n", "line3\n"])
                .unwrap();

            // Check ancestry
            let ancestry = weave.get_ancestry(vec!["v2"]).unwrap();
            assert!(ancestry.contains(&"v1".to_string()));
            assert!(ancestry.contains(&"v2".to_string()));

            // Check version count
            assert_eq!(weave.numversions().unwrap(), 2);
        });
    }

    #[test]
    fn test_record_ordering() {
        let _unordered = RecordOrdering::Unordered;
        let _topological = RecordOrdering::Topological;
        let _grouped = RecordOrdering::GroupedByKey;
    }
}
