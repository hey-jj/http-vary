//! The mutator: read a target's `Vary` header, append fields, write it back.

use std::borrow::Cow;

use crate::append::append;
use crate::error::VaryError;
use crate::field::Field;

/// A response-like value that holds a `Vary` header.
///
/// [`vary`] reads the current value, appends to it, and writes the result
/// back. Implement this trait to drive [`vary`] against your own response or
/// header map. A blanket call against [`HeaderStore`] is provided.
pub trait VaryTarget {
    /// Return the current `Vary` value, or `None` when it is unset.
    ///
    /// A multi-valued header is joined into one string with `", "` so it reads
    /// like a single `Vary` value. The return is a [`Cow`] so an implementer
    /// backed by a stored string can borrow it, and one that must join several
    /// values can return the owned join. [`vary`] reads it without forcing a
    /// clone.
    fn vary(&self) -> Option<Cow<'_, str>>;

    /// Set the `Vary` header to `value`, replacing any current value.
    fn set_vary(&mut self, value: String);
}

/// Append `field` to the target's `Vary` header.
///
/// The current value is read, merged with `field` through [`append`], and
/// written back. The header is only written when the merged value is non-empty.
/// An empty list with no existing header leaves the header unset, matching the
/// merge result of an empty string.
///
/// # Errors
///
/// Returns the same errors as [`append`]. A bad field name leaves the target
/// untouched.
///
/// # Examples
///
/// ```
/// use http_vary::{vary, HeaderStore};
///
/// let mut res = HeaderStore::new();
/// vary(&mut res, "Origin").unwrap();
/// vary(&mut res, "User-Agent").unwrap();
/// assert_eq!(res.vary(), Some("Origin, User-Agent"));
/// ```
pub fn vary<T: VaryTarget + ?Sized>(res: &mut T, field: impl Into<Field>) -> Result<(), VaryError> {
    let header = res.vary().unwrap_or(Cow::Borrowed(""));
    let val = append(&header, field)?;
    if !val.is_empty() {
        res.set_vary(val);
    }
    Ok(())
}

/// A minimal `Vary` header store for callers without their own header map.
///
/// It holds at most one `Vary` value. Build it from a single value or from a
/// multi-valued list to mirror a server that set the header either way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderStore {
    vary: Option<String>,
}

impl HeaderStore {
    /// Create a store with no `Vary` header.
    pub fn new() -> Self {
        HeaderStore { vary: None }
    }

    /// Create a store whose `Vary` header starts at `value`.
    pub fn with_vary(value: impl Into<String>) -> Self {
        HeaderStore {
            vary: Some(value.into()),
        }
    }

    /// Create a store whose `Vary` header starts as a multi-valued header.
    ///
    /// The parts are joined with `", "`, matching how a multi-valued header
    /// reads as a single `Vary` value.
    pub fn with_vary_list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let joined = values
            .into_iter()
            .map(|s| s.as_ref().to_owned())
            .collect::<Vec<_>>()
            .join(", ");
        HeaderStore { vary: Some(joined) }
    }

    /// Return the current `Vary` value, or `None` when it is unset.
    pub fn vary(&self) -> Option<&str> {
        self.vary.as_deref()
    }
}

impl VaryTarget for HeaderStore {
    fn vary(&self) -> Option<Cow<'_, str>> {
        self.vary.as_deref().map(Cow::Borrowed)
    }

    fn set_vary(&mut self, value: String) {
        self.vary = Some(value);
    }
}
