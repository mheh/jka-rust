//! Raven's console/file glob-matcher family (`Com_Filter` and friends),
//! `&str`-fronted with `Bytes` siblings for seam callers that hold raw C
//! data. One canonical for the pointer twins in `native_platform`
//! `sys_shared.rs` and qcommon `common_fns.rs`.

use crate::ctype::toupper_byte;

/// Raven `MAX_QPATH` — `Com_FilterPath`'s normalize buffers are this wide.
/// Source: `oracle/codemp/game/q_shared.h`
const MAX_QPATH: usize = 64;

/// Byte at `i`, or 0 at/past the end — C reads the NUL terminator at
/// `len`; reads past it are UB in Raven (a `?` in the filter can step the
/// name cursor beyond its NUL) and the one defined behavior picked here is
/// "keep reading NUL" (porting-rules §19).
fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// Raven `Com_StringContains` — first (case-toggled) occurrence of `str2` in
/// `str1`, as the byte offset of the match (Raven returns the pointer into
/// `str1`; an empty `str2` matches at 0).
///
/// Source: `oracle/codemp/qcommon/common.cpp:551-578`
pub fn Com_StringContains(str1: &str, str2: &str, casesensitive: bool) -> Option<usize> {
    Com_StringContainsBytes(str1.as_bytes(), str2.as_bytes(), casesensitive)
}

/// [`Com_StringContains`] over raw bytes.
pub fn Com_StringContainsBytes(str1: &[u8], str2: &[u8], casesensitive: bool) -> Option<usize> {
    if str2.len() > str1.len() {
        return None;
    }
    for i in 0..=(str1.len() - str2.len()) {
        let mut j = 0usize;
        while j < str2.len() {
            let matches = if casesensitive {
                str1[i + j] == str2[j]
            } else {
                toupper_byte(str1[i + j]) == toupper_byte(str2[j])
            };
            if !matches {
                break;
            }
            j += 1;
        }
        if j == str2.len() {
            return Some(i);
        }
    }
    None
}

/// Raven `Com_Filter` — glob match of `name` against `filter`: `*` matches
/// any run (via [`Com_StringContains`] on the following literal run), `?`
/// matches exactly one char, `[a-z]`/`[abc]` match a class (leading `[[`
/// collapses to one `[`), everything else matches literally.
///
/// Source: `oracle/codemp/qcommon/common.cpp:585-658`
pub fn Com_Filter(filter: &str, name: &str, casesensitive: bool) -> bool {
    Com_FilterBytes(filter.as_bytes(), name.as_bytes(), casesensitive)
}

/// [`Com_Filter`] over raw bytes.
pub fn Com_FilterBytes(filter: &[u8], name: &[u8], casesensitive: bool) -> bool {
    let mut f = 0usize;
    let mut n = 0usize;
    while at(filter, f) != 0 {
        let c = at(filter, f);
        if c == b'*' {
            f += 1;
            let run_start = f;
            while at(filter, f) != 0 && at(filter, f) != b'*' && at(filter, f) != b'?' {
                f += 1;
            }
            // Raven copies the literal run into a `char buf[MAX_TOKEN_CHARS]`
            // (overrunning it at >= 1024 bytes, UB); the slice carries the
            // full run instead (porting-rules §19).
            let run = &filter[run_start..f.min(filter.len())];
            if !run.is_empty() {
                let searched = n.min(name.len());
                let Some(pos) = Com_StringContainsBytes(&name[searched..], run, casesensitive)
                else {
                    return false;
                };
                n = searched + pos + run.len();
            }
        } else if c == b'?' {
            f += 1;
            n += 1;
        } else if c == b'[' && at(filter, f + 1) == b'[' {
            f += 1;
        } else if c == b'[' {
            f += 1;
            let mut found = false;
            while at(filter, f) != 0 && !found {
                if at(filter, f) == b']' && at(filter, f + 1) != b']' {
                    break;
                }
                if at(filter, f + 1) == b'-'
                    && at(filter, f + 2) != 0
                    && (at(filter, f + 2) != b']' || at(filter, f + 3) == b']')
                {
                    let (lo, hi, nc) = (at(filter, f), at(filter, f + 2), at(name, n));
                    found = if casesensitive {
                        nc >= lo && nc <= hi
                    } else {
                        toupper_byte(nc) >= toupper_byte(lo) && toupper_byte(nc) <= toupper_byte(hi)
                    };
                    f += 3;
                } else {
                    let (fc, nc) = (at(filter, f), at(name, n));
                    found = if casesensitive {
                        fc == nc
                    } else {
                        toupper_byte(fc) == toupper_byte(nc)
                    };
                    f += 1;
                }
            }
            if !found {
                return false;
            }
            while at(filter, f) != 0 {
                if at(filter, f) == b']' && at(filter, f + 1) != b']' {
                    break;
                }
                f += 1;
            }
            f += 1;
            n += 1;
        } else {
            let nc = at(name, n);
            let matches = if casesensitive {
                c == nc
            } else {
                toupper_byte(c) == toupper_byte(nc)
            };
            if !matches {
                return false;
            }
            f += 1;
            n += 1;
        }
    }
    true
}

/// Raven `Com_FilterPath` — [`Com_Filter`] after normalizing `\` and `:` to
/// `/` in both strings (each truncated to `MAX_QPATH - 1` like Raven's
/// stack buffers).
///
/// Source: `oracle/codemp/qcommon/common.cpp:665-690`
pub fn Com_FilterPath(filter: &str, name: &str, casesensitive: bool) -> bool {
    Com_FilterPathBytes(filter.as_bytes(), name.as_bytes(), casesensitive)
}

/// [`Com_FilterPath`] over raw bytes.
pub fn Com_FilterPathBytes(filter: &[u8], name: &[u8], casesensitive: bool) -> bool {
    fn normalize(src: &[u8]) -> Vec<u8> {
        src.iter()
            .take(MAX_QPATH - 1)
            .take_while(|&&b| b != 0)
            .map(|&b| if b == b'\\' || b == b':' { b'/' } else { b })
            .collect()
    }
    Com_FilterBytes(&normalize(filter), &normalize(name), casesensitive)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_and_case() {
        assert!(Com_Filter("abc", "abc", true));
        assert!(!Com_Filter("abc", "aBc", true));
        assert!(Com_Filter("abc", "aBc", false));
    }

    #[test]
    fn star_matches_any_run() {
        assert!(Com_Filter("*map*", "mp/duel1_map.bsp", true));
        assert!(!Com_Filter("*zzz*", "mp/duel1_map.bsp", true));
        // Trailing `*` matches the rest, including nothing.
        assert!(Com_Filter("duel*", "duel", true));
    }

    #[test]
    fn question_matches_exactly_one() {
        assert!(Com_Filter("du?l1", "duel1", true));
        // Name exhausted: `?` steps past the NUL, the following literal
        // compares against NUL and fails (matches Raven up to its UB edge).
        assert!(!Com_Filter("duel1?x", "duel1", true));
    }

    #[test]
    fn char_class_and_range() {
        assert!(Com_Filter("[abc]x", "bx", true));
        assert!(!Com_Filter("[abc]x", "dx", true));
        assert!(Com_Filter("[a-c]x", "bx", true));
        assert!(!Com_Filter("[a-c]x", "dx", true));
        assert!(Com_Filter("[A-C]x", "bx", false));
    }

    #[test]
    fn string_contains_offsets() {
        assert_eq!(Com_StringContains("abcdef", "cde", true), Some(2));
        assert_eq!(Com_StringContains("abcdef", "CDE", false), Some(2));
        assert_eq!(Com_StringContains("abcdef", "zzz", true), None);
        assert_eq!(Com_StringContains("abc", "", true), Some(0));
        assert_eq!(Com_StringContains("ab", "abc", true), None);
    }

    #[test]
    fn filter_path_normalizes_separators() {
        assert!(Com_FilterPath("maps/mp/*", "maps\\mp\\duel1.bsp", false));
        assert!(Com_FilterPath("c/foo", "c:foo", false));
    }
}
