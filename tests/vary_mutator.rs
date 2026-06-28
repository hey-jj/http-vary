//! Mutator cases for `vary`. Each builds a starting header state, calls `vary`,
//! and asserts the resulting `Vary` header or its absence.

use http_vary::{vary, HeaderStore, VaryError};

// arguments / field

#[test]
fn empty_string_field_errors() {
    let mut res = HeaderStore::new();
    assert_eq!(vary(&mut res, ""), Err(VaryError::FieldRequired));
    assert_eq!(res.vary(), None);
}

#[test]
fn accepts_string() {
    let mut res = HeaderStore::new();
    assert!(vary(&mut res, "foo").is_ok());
}

#[test]
fn accepts_array_of_string() {
    let mut res = HeaderStore::new();
    assert!(vary(&mut res, vec!["foo", "bar"]).is_ok());
}

#[test]
fn accepts_vary_header_string() {
    let mut res = HeaderStore::new();
    assert!(vary(&mut res, "foo, bar").is_ok());
}

#[test]
fn rejects_colon_separator() {
    let mut res = HeaderStore::new();
    assert_eq!(
        vary(&mut res, "invalid:header"),
        Err(VaryError::InvalidFieldName)
    );
}

#[test]
fn rejects_space_separator() {
    let mut res = HeaderStore::new();
    assert_eq!(
        vary(&mut res, "invalid header"),
        Err(VaryError::InvalidFieldName)
    );
}

#[test]
fn invalid_name_leaves_target_untouched() {
    // A failed merge must not write the header. The store stays as it was.
    let mut res = HeaderStore::with_vary("Accept");
    assert_eq!(
        vary(&mut res, "invalid:header"),
        Err(VaryError::InvalidFieldName)
    );
    assert_eq!(res.vary(), Some("Accept"));
}

#[test]
fn rejects_del_control_char() {
    let mut res = HeaderStore::new();
    assert_eq!(
        vary(&mut res, "a\u{007f}b"),
        Err(VaryError::InvalidFieldName)
    );
    assert_eq!(res.vary(), None);
}

#[test]
fn rejects_high_byte() {
    let mut res = HeaderStore::new();
    assert_eq!(
        vary(&mut res, "a\u{0080}b"),
        Err(VaryError::InvalidFieldName)
    );
    assert_eq!(res.vary(), None);
}

// when no Vary

#[test]
fn no_vary_sets_value() {
    let mut res = HeaderStore::new();
    vary(&mut res, "Origin").unwrap();
    assert_eq!(res.vary(), Some("Origin"));
}

#[test]
fn no_vary_sets_value_from_array() {
    let mut res = HeaderStore::new();
    vary(&mut res, vec!["Origin", "User-Agent"]).unwrap();
    assert_eq!(res.vary(), Some("Origin, User-Agent"));
}

#[test]
fn no_vary_preserves_case() {
    let mut res = HeaderStore::new();
    vary(&mut res, vec!["ORIGIN", "user-agent", "AccepT"]).unwrap();
    assert_eq!(res.vary(), Some("ORIGIN, user-agent, AccepT"));
}

#[test]
fn no_vary_empty_array_leaves_header_unset() {
    let mut res = HeaderStore::new();
    let empty: Vec<&str> = Vec::new();
    vary(&mut res, empty).unwrap();
    assert_eq!(res.vary(), None);
}

// when existing Vary

#[test]
fn existing_vary_sets_value() {
    let mut res = HeaderStore::with_vary("Accept");
    vary(&mut res, "Origin").unwrap();
    assert_eq!(res.vary(), Some("Accept, Origin"));
}

#[test]
fn existing_vary_sets_value_with_multiple_calls() {
    let mut res = HeaderStore::with_vary("Accept");
    vary(&mut res, "Origin").unwrap();
    vary(&mut res, "User-Agent").unwrap();
    assert_eq!(res.vary(), Some("Accept, Origin, User-Agent"));
}

#[test]
fn existing_vary_does_not_duplicate() {
    let mut res = HeaderStore::with_vary("Accept");
    vary(&mut res, "Accept").unwrap();
    assert_eq!(res.vary(), Some("Accept"));
}

#[test]
fn existing_vary_compares_case_insensitive() {
    let mut res = HeaderStore::with_vary("Accept");
    vary(&mut res, "accEPT").unwrap();
    assert_eq!(res.vary(), Some("Accept"));
}

#[test]
fn existing_vary_preserves_case() {
    let mut res = HeaderStore::with_vary("AccepT");
    vary(&mut res, vec!["accEPT", "ORIGIN"]).unwrap();
    assert_eq!(res.vary(), Some("AccepT, ORIGIN"));
}

// when existing Vary as array

#[test]
fn existing_array_sets_value() {
    let mut res = HeaderStore::with_vary_list(["Accept", "Accept-Encoding"]);
    vary(&mut res, "Origin").unwrap();
    assert_eq!(res.vary(), Some("Accept, Accept-Encoding, Origin"));
}

#[test]
fn existing_array_does_not_duplicate() {
    let mut res = HeaderStore::with_vary_list(["Accept", "Accept-Encoding"]);
    vary(&mut res, vec!["accept", "origin"]).unwrap();
    assert_eq!(res.vary(), Some("Accept, Accept-Encoding, origin"));
}

#[test]
fn existing_array_with_wildcard_collapses() {
    // A multi-valued header that already holds "*" collapses to "*" after the
    // join, the same as a single "*" header.
    let mut res = HeaderStore::with_vary_list(["Accept", "*"]);
    vary(&mut res, "Origin").unwrap();
    assert_eq!(res.vary(), Some("*"));
}

// when Vary: *

#[test]
fn wildcard_sets_value() {
    let mut res = HeaderStore::new();
    vary(&mut res, "*").unwrap();
    assert_eq!(res.vary(), Some("*"));
}

#[test]
fn wildcard_acts_as_if_all_set() {
    let mut res = HeaderStore::with_vary("*");
    vary(&mut res, vec!["Origin", "User-Agent"]).unwrap();
    assert_eq!(res.vary(), Some("*"));
}

#[test]
fn wildcard_eradicates_existing_values() {
    let mut res = HeaderStore::with_vary("Accept, Accept-Encoding");
    vary(&mut res, "*").unwrap();
    assert_eq!(res.vary(), Some("*"));
}

#[test]
fn wildcard_updates_bad_existing_header() {
    let mut res = HeaderStore::with_vary("Accept, Accept-Encoding, *");
    vary(&mut res, "Origin").unwrap();
    assert_eq!(res.vary(), Some("*"));
}

// when field is string

#[test]
fn field_string_sets_value() {
    let mut res = HeaderStore::new();
    vary(&mut res, "Accept").unwrap();
    assert_eq!(res.vary(), Some("Accept"));
}

#[test]
fn field_string_sets_vary_header() {
    let mut res = HeaderStore::new();
    vary(&mut res, "Accept, Accept-Encoding").unwrap();
    assert_eq!(res.vary(), Some("Accept, Accept-Encoding"));
}

#[test]
fn field_string_accepts_lws() {
    let mut res = HeaderStore::new();
    vary(&mut res, "  Accept     ,     Origin    ").unwrap();
    assert_eq!(res.vary(), Some("Accept, Origin"));
}

#[test]
fn field_string_handles_contained_wildcard() {
    let mut res = HeaderStore::new();
    vary(&mut res, "Accept,*").unwrap();
    assert_eq!(res.vary(), Some("*"));
}

// when field is array

#[test]
fn field_array_sets_value() {
    let mut res = HeaderStore::new();
    vary(&mut res, vec!["Accept", "Accept-Language"]).unwrap();
    assert_eq!(res.vary(), Some("Accept, Accept-Language"));
}

#[test]
fn field_array_ignores_double_entries() {
    let mut res = HeaderStore::new();
    vary(&mut res, vec!["Accept", "Accept"]).unwrap();
    assert_eq!(res.vary(), Some("Accept"));
}

#[test]
fn field_array_is_case_insensitive() {
    let mut res = HeaderStore::new();
    vary(&mut res, vec!["Accept", "ACCEPT"]).unwrap();
    assert_eq!(res.vary(), Some("Accept"));
}

#[test]
fn field_array_handles_contained_wildcard() {
    let mut res = HeaderStore::new();
    vary(&mut res, vec!["Origin", "User-Agent", "*", "Accept"]).unwrap();
    assert_eq!(res.vary(), Some("*"));
}

#[test]
fn field_array_handles_existing_values() {
    let mut res = HeaderStore::with_vary("Accept, Accept-Encoding");
    vary(&mut res, vec!["origin", "accept", "accept-charset"]).unwrap();
    assert_eq!(
        res.vary(),
        Some("Accept, Accept-Encoding, origin, accept-charset")
    );
}

// trait object usage, to confirm the unsized bound works

#[test]
fn works_through_trait_object() {
    use http_vary::VaryTarget;
    let mut store = HeaderStore::new();
    let res: &mut dyn VaryTarget = &mut store;
    vary(res, "Origin").unwrap();
    assert_eq!(store.vary(), Some("Origin"));
}
