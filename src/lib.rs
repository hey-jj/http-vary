//! Append and de-duplicate field names on an HTTP `Vary` response header.
//!
//! The `Vary` header tells caches which request headers a response depends on
//! (RFC 7231 section 7.1.4). This crate builds that value. It appends one or
//! more field names to an existing `Vary` value, drops duplicates without
//! regard to case while keeping the case as written, validates each name as an
//! RFC 7230 token, and honors the `*` wildcard.
//!
//! Two entry points cover both uses:
//!
//! - [`append`] is the pure core. Given a current `Vary` string and a field,
//!   it returns the new string. No I/O, no shared state.
//! - [`vary`] is the mutator. It reads the `Vary` header off a target, appends
//!   to it, and writes the result back. Drive it against [`HeaderStore`] or any
//!   type that implements [`VaryTarget`].
//!
//! # Examples
//!
//! ```
//! use http_vary::append;
//!
//! // Add a field to an existing value.
//! assert_eq!(append("Accept", "Origin").unwrap(), "Accept, Origin");
//!
//! // Duplicates are dropped, case is kept.
//! assert_eq!(append("Accept", "accEPT").unwrap(), "Accept");
//!
//! // The wildcard collapses the whole value.
//! assert_eq!(append("Accept, Accept-Encoding", "*").unwrap(), "*");
//! ```
//!
//! ```
//! use http_vary::{vary, HeaderStore};
//!
//! let mut res = HeaderStore::with_vary("Accept");
//! vary(&mut res, "Origin").unwrap();
//! assert_eq!(res.vary(), Some("Accept, Origin"));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod append;
mod error;
mod field;
mod parse;
mod target;

pub use append::append;
pub use error::VaryError;
pub use field::Field;
pub use target::{vary, HeaderStore, VaryTarget};
