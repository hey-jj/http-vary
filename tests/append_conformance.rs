//! Value cases for `append`. Each row asserts the exact returned string.

use http_vary::{append, Field};

/// A field argument, either a single string or a list.
#[derive(Debug, Clone)]
enum F {
    S(&'static str),
    A(&'static [&'static str]),
}

impl From<F> for Field {
    fn from(f: F) -> Self {
        match f {
            F::S(s) => Field::One(s.to_owned()),
            F::A(list) => Field::List(list.iter().map(|s| (*s).to_owned()).collect()),
        }
    }
}

struct Case {
    header: &'static str,
    field: F,
    want: &'static str,
}

const OK: &[Case] = &[
    // header empty
    Case {
        header: "",
        field: F::S("Origin"),
        want: "Origin",
    },
    Case {
        header: "",
        field: F::A(&["Origin", "User-Agent"]),
        want: "Origin, User-Agent",
    },
    Case {
        header: "",
        field: F::A(&["ORIGIN", "user-agent", "AccepT"]),
        want: "ORIGIN, user-agent, AccepT",
    },
    // header has values
    Case {
        header: "Accept",
        field: F::S("Origin"),
        want: "Accept, Origin",
    },
    Case {
        header: "Accept",
        field: F::A(&["Origin", "User-Agent"]),
        want: "Accept, Origin, User-Agent",
    },
    Case {
        header: "Accept",
        field: F::S("Accept"),
        want: "Accept",
    },
    Case {
        header: "Accept",
        field: F::S("accEPT"),
        want: "Accept",
    },
    Case {
        header: "Accept",
        field: F::S("AccepT"),
        want: "Accept",
    },
    // wildcard
    Case {
        header: "",
        field: F::S("*"),
        want: "*",
    },
    Case {
        header: "*",
        field: F::S("Origin"),
        want: "*",
    },
    Case {
        header: "Accept, Accept-Encoding",
        field: F::S("*"),
        want: "*",
    },
    Case {
        header: "Accept, Accept-Encoding, *",
        field: F::S("Origin"),
        want: "*",
    },
    // field is string
    Case {
        header: "",
        field: F::S("Accept"),
        want: "Accept",
    },
    Case {
        header: "",
        field: F::S("Accept, Accept-Encoding"),
        want: "Accept, Accept-Encoding",
    },
    Case {
        header: "",
        field: F::S("  Accept     ,     Origin    "),
        want: "Accept, Origin",
    },
    Case {
        header: "",
        field: F::S("Accept,*"),
        want: "*",
    },
    // field is array
    Case {
        header: "",
        field: F::A(&["Accept", "Accept-Language"]),
        want: "Accept, Accept-Language",
    },
    Case {
        header: "",
        field: F::A(&["Accept", "Accept"]),
        want: "Accept",
    },
    Case {
        header: "",
        field: F::A(&["Accept", "ACCEPT"]),
        want: "Accept",
    },
    Case {
        header: "",
        field: F::A(&["Origin", "User-Agent", "*", "Accept"]),
        want: "*",
    },
    Case {
        header: "Accept, Accept-Encoding",
        field: F::A(&["origin", "accept", "accept-charset"]),
        want: "Accept, Accept-Encoding, origin, accept-charset",
    },
];

#[test]
fn append_value_cases() {
    for c in OK {
        let got = append(c.header, c.field.clone()).unwrap();
        assert_eq!(got, c.want, "append({:?}, {:?})", c.header, c.field);
    }
}

// Extra value rows drawn from the benchmark inputs. These reuse the same merge
// paths as the table above and pin a few more shapes.

#[test]
fn field_to_existing_wildcard() {
    assert_eq!(append("*", "Accept-Encoding").unwrap(), "*");
}

#[test]
fn wildcard_field_to_existing_value() {
    assert_eq!(append("Accept-Encoding", "*").unwrap(), "*");
}

#[test]
fn single_field_to_empty() {
    assert_eq!(append("", "Accept-Encoding").unwrap(), "Accept-Encoding");
}

#[test]
fn array_to_empty() {
    assert_eq!(
        append("", vec!["Accept", "Accept-Encoding", "Accept-Language"]).unwrap(),
        "Accept, Accept-Encoding, Accept-Language"
    );
}

#[test]
fn list_string_to_empty() {
    assert_eq!(
        append("", "Accept, Accept-Encoding, Accept-Language").unwrap(),
        "Accept, Accept-Encoding, Accept-Language"
    );
}

#[test]
fn field_to_existing_list() {
    assert_eq!(
        append("Accept, Accept-Encoding, Accept-Language", "X-Foo").unwrap(),
        "Accept, Accept-Encoding, Accept-Language, X-Foo"
    );
}

// Added rows to close gaps the value table does not reach.

#[test]
fn empty_list_is_a_noop_on_empty_header() {
    let empty: Vec<&str> = Vec::new();
    assert_eq!(append("", empty).unwrap(), "");
}

#[test]
fn empty_list_leaves_existing_header() {
    let empty: Vec<&str> = Vec::new();
    assert_eq!(append("Accept", empty).unwrap(), "Accept");
}

#[test]
fn accepts_full_tchar_set() {
    let name = "!#$%&'+-.^_`|~0Az";
    assert_eq!(append("", name).unwrap(), name);
}

#[test]
fn wildcard_dedup_is_stable() {
    assert_eq!(append("Accept, *", "*").unwrap(), "*");
}

// The accumulator is the raw header string, not a value rebuilt from parsed
// tokens. Existing spacing and stray separators carry through untouched, and
// only each newly added field gets a ", " prefix. These rows pin that. A merge
// that canonicalized the existing value would diverge on every one of them.

#[test]
fn keeps_header_spacing_verbatim() {
    // header is two spaces; the new field appends after a literal ", ".
    assert_eq!(append("  ", "Origin").unwrap(), "  , Origin");
}

#[test]
fn keeps_bare_comma_header_verbatim() {
    assert_eq!(append(",", "Origin").unwrap(), ",, Origin");
}

#[test]
fn keeps_tight_comma_in_existing_header() {
    // No space after the existing comma stays missing; the new field still
    // joins with ", ".
    assert_eq!(append("Accept,Origin", "X").unwrap(), "Accept,Origin, X");
}

#[test]
fn keeps_space_before_existing_comma() {
    assert_eq!(append("Accept ,Origin", "X").unwrap(), "Accept ,Origin, X");
}

#[test]
fn keeps_leading_space_in_existing_header() {
    assert_eq!(append(" Accept", "X").unwrap(), " Accept, X");
}

#[test]
fn dedup_trims_for_compare_but_keeps_header_trailing_space() {
    // The trailing space is trimmed only for the dedup compare. "Accept" dups
    // the existing token, so nothing is added and the trailing space remains.
    assert_eq!(append("Accept ", "Accept").unwrap(), "Accept ");
}

// The header == "*" short-circuit returns "*" only after field validation
// passes. With a valid field and an array, the existing wildcard wins.

#[test]
fn existing_wildcard_with_valid_field() {
    assert_eq!(append("*", "Accept-Encoding").unwrap(), "*");
}

#[test]
fn existing_wildcard_with_empty_list() {
    let empty: Vec<&str> = Vec::new();
    assert_eq!(append("*", empty).unwrap(), "*");
}

#[test]
fn existing_wildcard_embedded_in_multivalue_header() {
    // A wildcard anywhere in the existing list collapses the result.
    assert_eq!(append("Accept, *, Accept-Encoding", "Origin").unwrap(), "*");
}

#[test]
fn intra_array_dedup_across_three_entries() {
    assert_eq!(append("", vec!["a", "A", "a"]).unwrap(), "a");
}
