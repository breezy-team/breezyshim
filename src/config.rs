pub fn parse_username(e: &str) -> (String, String) {
    if let Some((_, username, email)) =
        lazy_regex::regex_captures!(r"(.*?)\s*<?([\[\]\w+.-]+@[\w+.-]+)>?", e)
    {
        (username.to_string(), email.to_string())
    } else {
        (e.to_string(), "".to_string())
    }
}

pub fn extract_email_address(e: &str) -> Option<String> {
    let (_name, email) = parse_username(e);

    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

#[test]
fn test_parse_username() {
    assert_eq!(
        parse_username("John Doe <joe@example.com>"),
        ("John Doe".to_string(), "joe@example.com".to_string())
    );
    assert_eq!(
        parse_username("John Doe"),
        ("John Doe".to_string(), "".to_string())
    );
}

#[test]
fn test_extract_email_address() {
    assert_eq!(
        extract_email_address("John Doe <joe@example.com>"),
        Some("joe@example.com".to_string())
    );
    assert_eq!(extract_email_address("John Doe"), None);
}
