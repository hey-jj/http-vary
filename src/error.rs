//! Error type for [`append`](crate::append) and [`vary`](crate::vary).

use core::fmt;

/// What went wrong while building a `Vary` header value.
///
/// Each variant carries one of the exact messages the validation rules
/// produce, so callers can match on the cause or print it directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VaryError {
    /// The field argument held no field name.
    ///
    /// Returned when a single field is the empty string. An empty list is a
    /// no-op, not an error, so it never produces this.
    FieldRequired,
    /// A field name failed the RFC 7230 `token` grammar.
    ///
    /// Returned when a field holds a character outside the `tchar` set, for
    /// example `:`, a space, a newline, or any byte at or above `0x80`. An
    /// empty token reached through a stray comma or space also lands here.
    InvalidFieldName,
}

impl fmt::Display for VaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            VaryError::FieldRequired => "field argument is required",
            VaryError::InvalidFieldName => "field argument contains an invalid header name",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for VaryError {}
