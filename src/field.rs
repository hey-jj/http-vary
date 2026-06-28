//! The field argument accepted by [`append`](crate::append) and [`vary`](crate::vary).

/// One or more field names to add to a `Vary` header.
///
/// The two shapes parse differently, matching how a `Vary` value is built.
///
/// - [`Field::One`] holds a single string. It is split on commas and spaces,
///   so it can carry a whole `Vary` value like `"Accept, Accept-Encoding"`.
/// - [`Field::List`] holds field names that are already separated. Each entry
///   is taken as one field and is never split again. An entry with a comma or
///   space is therefore invalid.
///
/// Build a `Field` with [`From`]. A `&str` or `String` becomes [`Field::One`].
/// A `Vec<&str>` or `Vec<String>` becomes [`Field::List`]. Construct the variant
/// directly for any other shape.
///
/// The two variants are the complete set of field shapes. A `Vary` value is
/// either one already-formatted string or a list of separate names. No third
/// shape is planned, so the enum stays open to direct matching without a
/// wildcard arm.
///
/// ```
/// use http_vary::{append, Field};
///
/// // A single string, split on the comma.
/// assert_eq!(append("", "Accept, Origin").unwrap(), "Accept, Origin");
///
/// // A list, taken verbatim.
/// assert_eq!(append("", vec!["Accept", "Origin"]).unwrap(), "Accept, Origin");
///
/// // Build the enum directly when you need to.
/// let f = Field::List(vec!["Accept".into(), "Origin".into()]);
/// assert_eq!(append("", f).unwrap(), "Accept, Origin");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field {
    /// A single string, parsed as a comma and space separated `Vary` value.
    One(String),
    /// Already separated field names, each used verbatim.
    List(Vec<String>),
}

impl From<&str> for Field {
    fn from(value: &str) -> Self {
        Field::One(value.to_owned())
    }
}

impl From<String> for Field {
    fn from(value: String) -> Self {
        Field::One(value)
    }
}

impl From<Vec<String>> for Field {
    fn from(value: Vec<String>) -> Self {
        Field::List(value)
    }
}

impl From<Vec<&str>> for Field {
    fn from(value: Vec<&str>) -> Self {
        Field::List(value.into_iter().map(str::to_owned).collect())
    }
}
