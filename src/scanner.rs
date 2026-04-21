/// Locates `bsn!` and `bsn_list!` macro invocations in .rs source text.
/// Returns byte spans of the macro content (inside delimiters) for replacement.

#[derive(Debug)]
pub struct MacroSpan {
    /// true = bsn!, false = bsn_list!
    pub is_bsn: bool,
    /// Byte offset of the opening delimiter (e.g. `{`)
    pub open: usize,
    /// Byte offset of the closing delimiter (e.g. `}`)
    pub close: usize,
    /// The delimiter character used (used in tests)
    #[cfg_attr(not(test), allow(dead_code))]
    pub delimiter: char,
}

impl MacroSpan {
    /// Returns the inner content range (excluding delimiters)
    pub fn inner_range(&self) -> std::ops::Range<usize> {
        (self.open + 1)..self.close
    }
}

/// Scan source text for bsn!/bsn_list! macro invocations.
/// Returns spans sorted by position.
pub fn scan_macros(source: &str) -> Vec<MacroSpan> {
    let mut spans = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Skip string literals
        if bytes[i] == b'"' {
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Skip raw string literals r#"..."#
        if bytes[i] == b'r'
            && i + 1 < len
            && (bytes[i + 1] == b'"' || bytes[i + 1] == b'#')
            && let Some(skip) = skip_raw_string(source, i)
        {
            i = skip;
            continue;
        }

        // Skip line comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Skip block comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            let mut depth = 1;
            while i + 1 < len && depth > 0 {
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // Check for bsn! or bsn_list!
        if let Some((is_bsn, after_bang)) = try_match_bsn_macro(source, i) {
            let after_bang = skip_whitespace(source, after_bang);
            if after_bang < len {
                let delim = bytes[after_bang];
                if delim == b'{' || delim == b'(' || delim == b'[' {
                    let close_char = match delim {
                        b'{' => b'}',
                        b'(' => b')',
                        b'[' => b']',
                        _ => unreachable!(),
                    };
                    if let Some(close_pos) = find_matching_delimiter(bytes, after_bang, close_char) {
                        spans.push(MacroSpan {
                            is_bsn,
                            open: after_bang,
                            close: close_pos,
                            delimiter: delim as char,
                        });
                        i = close_pos + 1;
                        continue;
                    }
                }
            }
        }

        i += 1;
    }

    spans
}

/// Try to match `bsn!` or `bsn_list!` at position `i`.
/// Returns (is_bsn, position_after_bang) if matched.
fn try_match_bsn_macro(source: &str, i: usize) -> Option<(bool, usize)> {
    let bytes = source.as_bytes();

    // Must not be preceded by an alphanumeric or underscore (word boundary)
    if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
        return None;
    }

    if source[i..].starts_with("bsn_list!") {
        Some((false, i + 9))
    } else if source[i..].starts_with("bsn!") {
        Some((true, i + 4))
    } else {
        None
    }
}

fn skip_whitespace(source: &str, mut i: usize) -> usize {
    let bytes = source.as_bytes();
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn find_matching_delimiter(bytes: &[u8], open_pos: usize, close_char: u8) -> Option<usize> {
    let open_char = bytes[open_pos];
    let mut depth = 1;
    let mut i = open_pos + 1;
    let len = bytes.len();

    while i < len && depth > 0 {
        // Skip string literals inside the macro
        if bytes[i] == b'"' {
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Skip char literals
        if bytes[i] == b'\'' {
            i += 1;
            if i < len && bytes[i] == b'\\' {
                i += 2;
            } else {
                i += 1;
            }
            if i < len && bytes[i] == b'\'' {
                i += 1;
            }
            continue;
        }

        // Skip line comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Skip block comments
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            let mut cdepth = 1;
            while i + 1 < len && cdepth > 0 {
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    cdepth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    cdepth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        if bytes[i] == open_char {
            depth += 1;
        } else if bytes[i] == close_char {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn skip_raw_string(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    if bytes[start] != b'r' {
        return None;
    }
    let mut i = start + 1;
    let mut hashes = 0;
    while i < len && bytes[i] == b'#' {
        hashes += 1;
        i += 1;
    }
    if i >= len || bytes[i] != b'"' {
        return None;
    }
    i += 1;
    // Find closing "###
    while i < len {
        if bytes[i] == b'"' {
            let mut end_hashes = 0;
            let mut j = i + 1;
            while j < len && bytes[j] == b'#' && end_hashes < hashes {
                end_hashes += 1;
                j += 1;
            }
            if end_hashes == hashes {
                return Some(j);
            }
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_basic_bsn_macro() {
        let src = r#"bsn! { Transform }"#;
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert!(spans[0].is_bsn);
        assert_eq!(spans[0].delimiter, '{');
        assert_eq!(&src[spans[0].inner_range()], " Transform ");
    }

    #[test]
    fn scan_bsn_list_macro() {
        let src = r#"bsn_list! { Transform, Visibility }"#;
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert!(!spans[0].is_bsn);
        assert_eq!(&src[spans[0].inner_range()], " Transform, Visibility ");
    }

    #[test]
    fn scan_skips_string_literal() {
        let src = r#"let s = "bsn! { fake }"; bsn! { Real }"#;
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(&src[spans[0].inner_range()], " Real ");
    }

    #[test]
    fn scan_skips_line_comment() {
        let src = "// bsn! { fake }\nbsn! { Real }";
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(&src[spans[0].inner_range()], " Real ");
    }

    #[test]
    fn scan_skips_block_comment() {
        let src = "/* bsn! { fake } */ bsn! { Real }";
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(&src[spans[0].inner_range()], " Real ");
    }

    #[test]
    fn scan_skips_raw_string() {
        let src = r###"let s = r#"bsn! { fake }"#; bsn! { Real }"###;
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(&src[spans[0].inner_range()], " Real ");
    }

    #[test]
    fn scan_nested_braces() {
        let src = r#"bsn! { Foo { bar: { x: 1 } } }"#;
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(&src[spans[0].inner_range()], " Foo { bar: { x: 1 } } ");
    }

    #[test]
    fn scan_multiple_macros() {
        let src = "bsn! { A } bsn_list! { B } bsn! { C }";
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 3);
        assert!(spans[0].is_bsn);
        assert!(!spans[1].is_bsn);
        assert!(spans[2].is_bsn);
        assert_eq!(&src[spans[0].inner_range()], " A ");
        assert_eq!(&src[spans[1].inner_range()], " B ");
        assert_eq!(&src[spans[2].inner_range()], " C ");
    }

    #[test]
    fn scan_no_macros() {
        let src = "fn main() { println!(\"hello\"); }";
        let spans = scan_macros(src);
        assert!(spans.is_empty());
    }

    #[test]
    fn scan_paren_delimiter() {
        let src = "bsn!( Transform )";
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].delimiter, '(');
        assert_eq!(&src[spans[0].inner_range()], " Transform ");
    }

    #[test]
    fn scan_bracket_delimiter() {
        let src = "bsn![ Transform ]";
        let spans = scan_macros(src);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].delimiter, '[');
    }
}
