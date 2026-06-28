# http-vary

Append and de-duplicate field names on an HTTP `Vary` response header.

The `Vary` header tells caches which request headers a response depends on
(RFC 7231 section 7.1.4). This crate builds that value. It appends names to an
existing value, drops duplicates without regard to case while keeping the case
as written, validates each name as an RFC 7230 token, and honors the `*`
wildcard.

## Installation

```toml
[dependencies]
http-vary = "0.1"
```

## Usage

`append` is the pure core. Give it the current value and a field, get the new
value back.

```rust
use http_vary::append;

assert_eq!(append("Accept", "Origin").unwrap(), "Accept, Origin");

// A duplicate is dropped, the existing case is kept.
assert_eq!(append("Accept", "accEPT").unwrap(), "Accept");

// The wildcard collapses the whole value.
assert_eq!(append("Accept, Accept-Encoding", "*").unwrap(), "*");
```

A field can be one name, a comma separated list, or a `Vec` of names.

```rust
use http_vary::append;

assert_eq!(append("", "Accept, Origin").unwrap(), "Accept, Origin");
assert_eq!(append("", vec!["Accept", "Origin"]).unwrap(), "Accept, Origin");
```

`vary` reads a `Vary` header off a target, appends to it, and writes it back.
It works against `HeaderStore` or any type that implements `VaryTarget`.

```rust
use http_vary::{vary, HeaderStore};

let mut res = HeaderStore::with_vary("Accept");
vary(&mut res, "Origin").unwrap();
assert_eq!(res.vary(), Some("Accept, Origin"));
```

## Behavior

- Names are validated against the RFC 7230 `token` grammar. A name with a colon,
  space, control byte, or any byte at or above `0x80` is rejected.
- Comparison is case-insensitive. Output keeps the case of each name as written.
- A single string field is split on commas and spaces. A list entry is taken
  verbatim and never split again.
- A `*` on either side collapses the value to `"*"`. Validation runs first, so a
  bad name fails even when a `*` is present.
- An empty list is a no-op. Through `vary`, an empty result leaves the header
  unset.

## License

Licensed under the [MIT license](LICENSE).
