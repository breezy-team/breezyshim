//! URL manipulation utilities.
use pyo3::prelude::*;

pub fn join_segment_parameters(
    url: &url::Url,
    parameters: std::collections::HashMap<String, String>,
) -> url::Url {
    pyo3::Python::with_gil(|py| {
        let urlutils = py.import_bound("breezy.urlutils").unwrap();
        urlutils
            .call_method1("join_segment_parameters", (url.to_string(), parameters))
            .unwrap()
            .extract::<String>()
            .map(|s| url::Url::parse(s.as_str()).unwrap())
            .unwrap()
    })
}

#[test]
fn test_join_segment_parameters() {
    let url = url::Url::parse("http://example.com/blah").unwrap();
    let mut parameters = std::collections::HashMap::new();
    parameters.insert("a".to_string(), "1".to_string());
    parameters.insert("b".to_string(), "2".to_string());
    let result = join_segment_parameters(&url, parameters);
    assert_eq!(
        result,
        url::Url::parse("http://example.com/blah,a=1,b=2").unwrap()
    );
}

pub fn split_segment_parameters(
    url: &url::Url,
) -> (url::Url, std::collections::HashMap<String, String>) {
    pyo3::Python::with_gil(|py| {
        let urlutils = py.import_bound("breezy.urlutils").unwrap();
        urlutils
            .call_method1("split_segment_parameters", (url.to_string(),))
            .unwrap()
            .extract::<(String, std::collections::HashMap<String, String>)>()
            .map(|(s, m)| (url::Url::parse(s.as_str()).unwrap(), m))
            .unwrap()
    })
}

#[test]
fn test_split_segment_parameters() {
    let url = url::Url::parse("http://example.com/blah,a=1,b=2").unwrap();
    let (result_url, result_parameters) = split_segment_parameters(&url);
    assert_eq!(
        result_url,
        url::Url::parse("http://example.com/blah").unwrap()
    );
    let mut expected_parameters = std::collections::HashMap::new();
    expected_parameters.insert("a".to_string(), "1".to_string());
    expected_parameters.insert("b".to_string(), "2".to_string());
    assert_eq!(result_parameters, expected_parameters);
}
