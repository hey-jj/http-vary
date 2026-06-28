//! Argument and validation cases for `append`. Each row asserts the error
//! variant or that the call succeeds.

use http_vary::{append, VaryError};

#[test]
fn empty_string_field_is_required() {
    assert_eq!(append("", ""), Err(VaryError::FieldRequired));
}

#[test]
fn rejects_colon_separator() {
    assert_eq!(
        append("", "invalid:header"),
        Err(VaryError::InvalidFieldName)
    );
}

#[test]
fn rejects_space_separator() {
    assert_eq!(
        append("", "invalid header"),
        Err(VaryError::InvalidFieldName)
    );
}

#[test]
fn rejects_newline() {
    assert_eq!(
        append("", "invalid\nheader"),
        Err(VaryError::InvalidFieldName)
    );
}

#[test]
fn rejects_high_byte() {
    assert_eq!(
        append("", "invalid\u{0080}header"),
        Err(VaryError::InvalidFieldName)
    );
}

#[test]
fn rejects_tab() {
    assert_eq!(append("", "a\tb"), Err(VaryError::InvalidFieldName));
}

#[test]
fn rejects_del_control() {
    assert_eq!(append("", "a\u{007f}b"), Err(VaryError::InvalidFieldName));
}

#[test]
fn rejects_bare_comma() {
    // The comma splits into two empty tokens, both of which fail the grammar.
    assert_eq!(append("", ","), Err(VaryError::InvalidFieldName));
}

#[test]
fn rejects_whitespace_only_field() {
    assert_eq!(append("", " "), Err(VaryError::InvalidFieldName));
}

#[test]
fn rejects_array_element_with_separator() {
    // A list entry is taken verbatim, so a comma plus space inside it is not a
    // valid token.
    assert_eq!(append("", vec!["a, b"]), Err(VaryError::InvalidFieldName));
}

#[test]
fn validation_runs_before_wildcard() {
    // A bad name fails even when a "*" is present in the same list.
    assert_eq!(
        append("", vec!["*", "bad header"]),
        Err(VaryError::InvalidFieldName)
    );
}

// These mirror the does-not-throw cases. Each must succeed.

#[test]
fn accepts_single_string() {
    assert!(append("", "foo").is_ok());
}

#[test]
fn accepts_vary_header_string() {
    assert!(append("", "foo, bar").is_ok());
}

#[test]
fn accepts_array_of_strings() {
    assert!(append("", vec!["foo", "bar"]).is_ok());
}

#[test]
fn error_messages_match_the_grammar_rules() {
    assert_eq!(
        VaryError::FieldRequired.to_string(),
        "field argument is required"
    );
    assert_eq!(
        VaryError::InvalidFieldName.to_string(),
        "field argument contains an invalid header name"
    );
}
