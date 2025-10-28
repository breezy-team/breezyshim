//! URL manipulation utilities.
use pyo3::prelude::*;

/// Join segment parameters to a URL.
///
/// This function adds the specified parameters to a URL as segment parameters.
///
/// # Parameters
///
/// * `url` - The URL to add parameters to.
/// * `parameters` - The parameters to add to the URL.
///
/// # Returns
///
/// A new URL with the specified parameters added.
pub fn join_segment_parameters(
    url: &url::Url,
    parameters: std::collections::HashMap<String, String>,
) -> url::Url {
    pyo3::Python::attach(|py| {
        let urlutils = py.import("breezy.urlutils").unwrap();
        urlutils
            .call_method1("join_segment_parameters", (url.to_string(), parameters))
            .unwrap()
            .extract::<String>()
            .map(|s| url::Url::parse(s.as_str()).unwrap())
            .unwrap()
    })
}

/// Split segment parameters from a URL.
///
/// This function extracts segment parameters from a URL.
///
/// # Parameters
///
/// * `url` - The URL to extract parameters from.
///
/// # Returns
///
/// A tuple containing the URL without parameters and a map of the parameters.
pub fn split_segment_parameters(
    url: &url::Url,
) -> (url::Url, std::collections::HashMap<String, String>) {
    pyo3::Python::attach(|py| {
        let urlutils = py.import("breezy.urlutils").unwrap();
        urlutils
            .call_method1("split_segment_parameters", (url.to_string(),))
            .unwrap()
            .extract::<(String, std::collections::HashMap<String, String>)>()
            .map(|(s, m)| (url::Url::parse(s.as_str()).unwrap(), m))
            .unwrap()
    })
}

fn char_is_safe(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~'
}

/// Escape a byte slice for use in a URL.
///
/// This function escapes bytes for use in a URL, preserving characters that
/// are considered safe.
///
/// # Parameters
///
/// * `relpath` - The byte slice to escape.
/// * `safe` - Additional characters to consider safe (not to escape).
///
/// # Returns
///
/// The escaped string.
pub fn escape(relpath: &[u8], safe: Option<&str>) -> String {
    let mut result = String::new();
    let safe = safe.unwrap_or("/~").as_bytes();
    for b in relpath {
        if char_is_safe(char::from(*b)) || safe.contains(b) {
            result.push(char::from(*b));
        } else {
            result.push_str(&format!("%{:02X}", *b));
        }
    }
    result
}

/// Escape a UTF-8 string for use in a URL.
///
/// This is a convenience function that converts the string to bytes and calls `escape`.
///
/// # Parameters
///
/// * `relpath` - The string to escape.
/// * `safe` - Additional characters to consider safe (not to escape).
///
/// # Returns
///
/// The escaped string.
pub fn escape_utf8(relpath: &str, safe: Option<&str>) -> String {
    escape(relpath.as_bytes(), safe)
}

/// Unescape a URL-encoded UTF-8 string.
///
/// This function decodes percent-encoded characters in a string.
///
/// # Parameters
///
/// * `url` - The URL-encoded string to decode.
///
/// # Returns
///
/// The decoded string.
pub fn unescape_utf8(url: &str) -> String {
    use percent_encoding::percent_decode_str;

    percent_decode_str(url)
        .decode_utf8()
        .map(|s| s.to_string())
        .unwrap_or_else(|_| url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(escape(b"blah", None), "blah");
        assert_eq!(escape(b"blah", Some("")), "blah");
        assert_eq!(escape(b"blah", Some("/~")), "blah");

        assert_eq!(escape(b"la/bla", None), "la/bla");
        assert_eq!(escape(b"la/bla", Some("")), "la%2Fbla");

        assert_eq!(escape_utf8("la/bla", Some("/")), "la/bla");
    }

    #[test]
    fn test_unescape() {
        assert_eq!(unescape_utf8("blah"), "blah");
        assert_eq!(unescape_utf8("la%2Fbla"), "la/bla");
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
}
