//! The pure core: append fields to a `Vary` header string.

use crate::error::VaryError;
use crate::field::Field;
use crate::parse::parse;

/// The `tchar` set from RFC 7230 section 3.2.6.
///
/// ```text
/// field-name = token
/// token      = 1*tchar
/// tchar      = "!" / "#" / "$" / "%" / "&" / "'" / "*"
///            / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
///            / DIGIT / ALPHA
/// ```
///
/// A valid field name is one or more of these bytes and nothing else.
fn is_token_byte(b: u8) -> bool {
    matches!(b,
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*'
        | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
        | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

/// Return true when `name` is a valid RFC 7230 field name.
///
/// The name must be non-empty and every byte must be a `tchar`. The empty
/// string fails because the grammar needs at least one character.
fn is_valid_field_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(is_token_byte)
}

/// Append `field` to the `Vary` header value `header` and return the result.
///
/// The merge keeps existing entries in place, drops duplicates without regard
/// to case, and preserves the case of each name as written. The `*` wildcard
/// collapses the whole value to `"*"`.
///
/// `header` is the current `Vary` value and may be empty. `field` is one name,
/// a comma separated list of names, or a slice of names. See [`Field`] for how
/// each shape is read.
///
/// # Steps
///
/// 1. Validate every field name as an RFC 7230 token.
/// 2. If `header` is exactly `"*"`, return `"*"`.
/// 3. If any incoming field or any existing entry is `"*"`, return `"*"`.
/// 4. Append each new name once, comparing case-insensitively against what is
///    already present, including names added earlier in this same call.
///
/// Validation runs before the wildcard checks, so an invalid name fails even
/// when a `"*"` is also present.
///
/// # Errors
///
/// Returns [`VaryError::FieldRequired`] when a single field is the empty
/// string. Returns [`VaryError::InvalidFieldName`] when any field name is not a
/// valid token. An empty list is a no-op and returns `header` unchanged.
///
/// # Examples
///
/// ```
/// use http_vary::append;
///
/// assert_eq!(append("", "Origin").unwrap(), "Origin");
/// assert_eq!(append("Accept", "Origin").unwrap(), "Accept, Origin");
/// assert_eq!(append("Accept", "accEPT").unwrap(), "Accept");
/// assert_eq!(append("Accept, Accept-Encoding", "*").unwrap(), "*");
/// ```
pub fn append(header: &str, field: impl Into<Field>) -> Result<String, VaryError> {
    let field = field.into();

    // Collect the field names. A single string is split on commas and spaces.
    // A list is used verbatim, so its entries are never split again.
    let fields: Vec<&str> = match &field {
        Field::One(s) => {
            // An empty single field has no name to add.
            if s.is_empty() {
                return Err(VaryError::FieldRequired);
            }
            parse(s)
        }
        Field::List(list) => list.iter().map(String::as_str).collect(),
    };

    // Reject any field name that is not a valid token. This runs first, so a
    // bad name fails even when the value would otherwise collapse to "*".
    for name in &fields {
        if !is_valid_field_name(name) {
            return Err(VaryError::InvalidFieldName);
        }
    }

    // An existing unspecified vary stays unspecified.
    if header == "*" {
        return Ok(String::from("*"));
    }

    // Lowercased current entries, used only for case-insensitive dedup.
    let lower_header = header.to_ascii_lowercase();
    let mut vals: Vec<String> = parse(&lower_header).iter().map(|s| s.to_string()).collect();

    // A wildcard on either side collapses the whole value.
    if fields.contains(&"*") || vals.iter().any(|v| v == "*") {
        return Ok(String::from("*"));
    }

    // Append each new name once, preserving its original case.
    let mut val = header.to_owned();
    for name in &fields {
        let low = name.to_ascii_lowercase();
        if !vals.contains(&low) {
            vals.push(low);
            if val.is_empty() {
                val.push_str(name);
            } else {
                val.push_str(", ");
                val.push_str(name);
            }
        }
    }

    Ok(val)
}
