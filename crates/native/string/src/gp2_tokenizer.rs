//! GP2's internal tokenizer (Raven's file-local `GetToken`) — the one GP2
//! component with no per-mode behavior, byte-identical MP/SP (DEC-32 twin
//! hoist); both sides' `gp2` modules re-export from here.

/// Raven `MAX_TOKEN_SIZE` — a token that fills the buffer is discarded as empty.
/// Source: `oracle/codemp/qcommon/GenericParser2.cpp:12`
pub const MAX_TOKEN_SIZE: usize = 1024;

/// Raven `GetToken` — GP2's private whitespace/comment-aware tokenizer.
///
/// Raven scans a `char **` cursor into a `static char token[]` buffer; this is
/// the same state machine over a byte slice returning owned tokens (no global
/// buffer). Raven compares signed `char`s, so bytes >= 0x80 sort below `' '`
/// and act as whitespace/delimiters outside quoted strings — the `as i8` casts
/// preserve that.
/// Source: `oracle/codemp/qcommon/GenericParser2.cpp:15-161`
pub struct Tokenizer<'a> {
    data: &'a [u8],
    /// Byte cursor. `None` mirrors Raven nulling the caller's pointer on end of
    /// data (every subsequent read returns an empty token).
    pos: Option<usize>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(text: &'a str) -> Self {
        Tokenizer {
            data: text.as_bytes(),
            pos: Some(0),
        }
    }

    /// Byte at `i`, or 0 at/past end of data (C reads the NUL terminator).
    fn at(&self, i: usize) -> u8 {
        self.data.get(i).copied().unwrap_or(0)
    }

    pub fn get_token(&mut self, allow_line_breaks: bool, read_until_eol: bool) -> String {
        let Some(mut p) = self.pos else {
            return String::new();
        };

        // Skip whitespace and comments down to the start of a token.
        let mut c;
        loop {
            let mut found_line_break = false;
            loop {
                c = self.at(p);
                if (c as i8) > b' ' as i8 {
                    break;
                }
                if c == 0 {
                    self.pos = None;
                    return String::new();
                }
                if c == b'\n' {
                    found_line_break = true;
                }
                p += 1;
            }
            if found_line_break && !allow_line_breaks {
                self.pos = Some(p);
                return String::new();
            }

            if c == b'/' && self.at(p + 1) == b'/' {
                // skip single line comment
                p += 2;
                while self.at(p) != 0 && self.at(p) != b'\n' {
                    p += 1;
                }
            } else if c == b'/' && self.at(p + 1) == b'*' {
                // skip multi line comments
                p += 2;
                while self.at(p) != 0 && !(self.at(p) == b'*' && self.at(p + 1) == b'/') {
                    p += 1;
                }
                if self.at(p) != 0 {
                    p += 2;
                }
            } else {
                // found the start of a token
                break;
            }
        }

        let mut token: Vec<u8> = Vec::new();
        let mut length = 0usize;

        if c == b'"' {
            // handle a string
            p += 1;
            loop {
                c = self.at(p);
                p += 1;
                if c == b'"' {
                    break;
                } else if c == 0 {
                    // Raven's cursor ends up one past the NUL here and later
                    // calls read beyond the buffer (UB); we clamp to the end.
                    p -= 1;
                    break;
                } else if length < MAX_TOKEN_SIZE {
                    token.push(c);
                    length += 1;
                }
            }
        } else if read_until_eol {
            // absorb all characters until EOL
            while c != b'\n' && c != b'\r' {
                if c == b'/' && (self.at(p + 1) == b'/' || self.at(p + 1) == b'*') {
                    break;
                }
                if c == 0 {
                    // Raven has no end-of-data check in this loop and scans past
                    // the buffer (UB); we stop at the end.
                    break;
                }
                if length < MAX_TOKEN_SIZE {
                    token.push(c);
                    length += 1;
                }
                p += 1;
                c = self.at(p);
            }
            // Raven: "remove trailing white space" — the comparison is signed
            // `< ' '`, so control bytes and >= 0x80 are trimmed but spaces stay.
            while length > 0 && (token[length - 1] as i8) < b' ' as i8 {
                length -= 1;
            }
            token.truncate(length);
        } else {
            while (c as i8) > b' ' as i8 {
                if length < MAX_TOKEN_SIZE {
                    token.push(c);
                    length += 1;
                }
                p += 1;
                c = self.at(p);
            }
        }

        // Raven's post-hoc quote-stripping (GenericParser2.cpp:142-151) is
        // unreachable — the string branch never stores the opening quote, and
        // the other branches cannot begin with one — and is omitted.

        if length >= MAX_TOKEN_SIZE {
            token.clear();
        }
        self.pos = Some(p);

        String::from_utf8_lossy(&token).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(text: &str) -> String {
        Tokenizer::new(text).get_token(true, false)
    }

    fn eol(text: &str) -> String {
        Tokenizer::new(text).get_token(true, true)
    }

    #[test]
    fn bare_tokens_split_on_whitespace() {
        let mut t = Tokenizer::new("  foo\tbar\n");
        assert_eq!(t.get_token(true, false), "foo");
        assert_eq!(t.get_token(true, false), "bar");
        assert_eq!(t.get_token(true, false), "");
    }

    #[test]
    fn comments_are_skipped() {
        assert_eq!(one("// c\n tok"), "tok");
        assert_eq!(one("/* c \n c */ tok"), "tok");
    }

    #[test]
    fn quoted_strings_keep_spaces_and_drop_quotes() {
        assert_eq!(one("\"a b\""), "a b");
        assert_eq!(one("\"\" x"), "");
    }

    #[test]
    fn eol_token_absorbs_line_and_keeps_trailing_spaces() {
        // Trailing comment breaks the scan; the space before it is kept
        // (Raven trims only bytes < ' ').
        assert_eq!(eol("value // comment\n"), "value ");
        assert_eq!(eol("a b c\r\n"), "a b c");
        assert_eq!(eol("v\t\n"), "v");
    }

    #[test]
    fn eol_quoted_value_stops_at_close_quote() {
        // The quote branch wins over read-until-EOL.
        assert_eq!(eol("\"a b\" rest\n"), "a b");
    }

    #[test]
    fn oversize_token_becomes_empty() {
        let long = "x".repeat(MAX_TOKEN_SIZE);
        assert_eq!(one(&long), "");
        let ok = "x".repeat(MAX_TOKEN_SIZE - 1);
        assert_eq!(one(&ok), ok);
    }

    #[test]
    fn high_bit_bytes_delimit_bare_tokens() {
        // 0xC3 0xA9 = "é": signed-char compare makes both bytes whitespace-like.
        assert_eq!(one("caf\u{e9} x"), "caf");
        let mut t = Tokenizer::new("caf\u{e9} x");
        t.get_token(true, false);
        assert_eq!(t.get_token(true, false), "x");
    }

    #[test]
    fn eof_kills_the_cursor() {
        let mut t = Tokenizer::new("last");
        assert_eq!(t.get_token(true, false), "last");
        assert_eq!(t.get_token(true, false), "");
        assert_eq!(t.get_token(true, false), "");
    }
}
