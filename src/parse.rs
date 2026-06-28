//! Split a `Vary` header value into tokens.

/// Split `header` into tokens on commas, trimming surrounding ASCII spaces.
///
/// Only the comma `0x2c` and the space `0x20` are special. The comma ends a
/// token. A leading space is skipped. A trailing space is left out because the
/// token end does not advance past it. Every other byte extends the current
/// token, so tabs, newlines, and non-ASCII bytes stay inside the token and are
/// rejected later by field-name validation.
///
/// The split always yields at least one token. An empty input returns a single
/// empty string, which matches the source tokenizer and keeps the dedup and
/// accumulator logic in `append` correct for an empty header.
///
/// Tokens are returned as byte slices into `header`. Callers turn them into
/// `&str` where needed. Splitting on bytes is safe here because `0x20` and
/// `0x2c` never appear inside a UTF-8 multi-byte sequence.
pub(crate) fn parse(header: &str) -> Vec<&str> {
    let bytes = header.as_bytes();
    let mut list = Vec::new();
    let mut start = 0usize;
    let mut end = 0usize;

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            0x20 => {
                // Skip a leading space. A trailing space is dropped on its own
                // because end is not advanced for it.
                if start == end {
                    start = i + 1;
                    end = i + 1;
                }
            }
            0x2c => {
                list.push(&header[start..end]);
                start = i + 1;
                end = i + 1;
            }
            _ => {
                end = i + 1;
            }
        }
    }

    // The final token, always pushed, even for empty input.
    list.push(&header[start..end]);

    list
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn empty_input_yields_one_empty_token() {
        assert_eq!(parse(""), vec![""]);
    }

    #[test]
    fn trims_leading_and_trailing_spaces() {
        assert_eq!(
            parse("  Accept     ,     Origin    "),
            vec!["Accept", "Origin"]
        );
    }

    #[test]
    fn splits_without_surrounding_spaces() {
        assert_eq!(parse("Accept,Origin"), vec!["Accept", "Origin"]);
    }

    #[test]
    fn keeps_empty_tokens_from_stray_commas() {
        assert_eq!(parse("a,,b"), vec!["a", "", "b"]);
    }

    #[test]
    fn does_not_treat_tab_as_separator() {
        assert_eq!(parse("a\tb"), vec!["a\tb"]);
    }
}
