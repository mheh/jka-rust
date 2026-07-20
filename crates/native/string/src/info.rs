//! Raven `Info_*` — the `\key\value` info-string family, transcribed
//! byte-faithfully over `&str`/`String`. Effectful consumers (`Com_Printf`
//! on a rejected set) map [`InfoSetResult`] to their own output; the value
//! logic lives only here (DEC-32).
//!
//! Source: `oracle/codemp/game/q_shared.c:1057-1366`

use crate::q_string::Q_stricmpBytes;

/// Raven `MAX_INFO_STRING`.
/// Source: `oracle/codemp/game/q_shared.h:384`
pub const MAX_INFO_STRING: usize = 1024;

/// Raven `BIG_INFO_STRING`.
/// Source: `oracle/codemp/game/q_shared.h:388`
pub const BIG_INFO_STRING: usize = 8192;

/// Why an [`Info_SetValueForKey`] call was rejected — Raven prints a
/// message and returns without setting; callers reproduce the print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoSetResult {
    /// Key set (or removed, for an empty value).
    Set,
    /// Raven: "Can't use keys or values with a \\".
    ContainsBackslash,
    /// Raven: "Can't use keys or values with a semicolon".
    ContainsSemicolon,
    /// Raven: "Can't use keys or values with a \"".
    ContainsQuote,
    /// Raven: "Info string length exceeded".
    LengthExceeded,
}

/// Raven `Info_ValueForKey` — case-insensitive key lookup; returns the empty
/// string when the key is absent.
///
/// Source: `oracle/codemp/game/q_shared.c:1051-1100`
pub fn Info_ValueForKey(s: &str, key: &str) -> String {
    if s.len() >= BIG_INFO_STRING {
        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
        panic!("Info_ValueForKey: oversize infostring");
    }

    let b = s.as_bytes();
    let mut p = 0usize;
    if p < b.len() && b[p] == b'\\' {
        p += 1;
    }

    loop {
        let kstart = p;
        while p < b.len() && b[p] != b'\\' {
            p += 1;
        }
        if p >= b.len() {
            // NUL hit while scanning a key.
            return String::new();
        }
        let pkey = &b[kstart..p];
        p += 1;

        let vstart = p;
        while p < b.len() && b[p] != b'\\' {
            p += 1;
        }
        let value = &b[vstart..p];

        if Q_stricmpBytes(key.as_bytes(), pkey) == 0 {
            return String::from_utf8_lossy(value).into_owned();
        }

        if p >= b.len() {
            return String::new();
        }
        p += 1;
    }
}

/// Shared walk for `Info_RemoveKey`/`Info_RemoveKey_Big` (identical bodies in
/// Raven apart from the length guard): key match is case-SENSITIVE (`strcmp`,
/// unlike `Info_ValueForKey`'s `Q_stricmp`).
fn remove_key_walk(s: &mut String, key: &str) {
    if key.contains('\\') {
        return;
    }

    let mut start = 0usize;
    loop {
        let b = s.as_bytes();
        let mut p = start;
        if p < b.len() && b[p] == b'\\' {
            p += 1;
        }

        let kstart = p;
        while p < b.len() && b[p] != b'\\' {
            p += 1;
        }
        if p >= b.len() {
            return;
        }
        let key_matches = &b[kstart..p] == key.as_bytes();
        p += 1;

        while p < b.len() && b[p] != b'\\' {
            p += 1;
        }

        if key_matches {
            s.replace_range(start..p, "");
            return;
        }

        if p >= b.len() {
            return;
        }
        start = p;
    }
}

/// Raven `Info_RemoveKey`.
///
/// Source: `oracle/codemp/game/q_shared.c:1147-1195`
pub fn Info_RemoveKey(s: &mut String, key: &str) {
    if s.len() >= MAX_INFO_STRING {
        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
        panic!("Info_RemoveKey: oversize infostring");
    }
    remove_key_walk(s, key);
}

/// Raven `Info_RemoveKey_Big`.
///
/// Source: `oracle/codemp/game/q_shared.c:1202-1250`
pub fn Info_RemoveKey_Big(s: &mut String, key: &str) {
    if s.len() >= BIG_INFO_STRING {
        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
        panic!("Info_RemoveKey_Big: oversize infostring");
    }
    remove_key_walk(s, key);
}

/// Raven `Info_Validate` — `qfalse` for info strings carrying `"` or `;`.
///
/// Source: `oracle/codemp/game/q_shared.c:1263-1271`
pub fn Info_Validate(s: &str) -> bool {
    !s.contains('"') && !s.contains(';')
}

/// Raven `Info_SetValueForKey` — removes the key, then PREPENDS
/// `\key\value` (`strcat(newi, s); strcpy(s, newi)`).
///
/// Source: `oracle/codemp/game/q_shared.c:1280-1319`
pub fn Info_SetValueForKey(s: &mut String, key: &str, value: &str) -> InfoSetResult {
    if s.len() >= MAX_INFO_STRING {
        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
        panic!("Info_SetValueForKey: oversize infostring");
    }

    if let Some(bad) = bad_chars(key, value) {
        return bad;
    }

    Info_RemoveKey(s, key);
    if value.is_empty() {
        return InfoSetResult::Set;
    }

    let newi = format!("\\{key}\\{value}");
    if newi.len() + s.len() > MAX_INFO_STRING {
        return InfoSetResult::LengthExceeded;
    }

    s.insert_str(0, &newi);
    InfoSetResult::Set
}

/// Raven `Info_SetValueForKey_Big` — removes the key, then APPENDS
/// `\key\value` (`strcat(s, newi)`; reversed order vs the non-Big form).
///
/// Source: `oracle/codemp/game/q_shared.c:1328-1366`
pub fn Info_SetValueForKey_Big(s: &mut String, key: &str, value: &str) -> InfoSetResult {
    if s.len() >= BIG_INFO_STRING {
        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
        panic!("Info_SetValueForKey: oversize infostring");
    }

    if let Some(bad) = bad_chars(key, value) {
        return bad;
    }

    Info_RemoveKey_Big(s, key);
    if value.is_empty() {
        return InfoSetResult::Set;
    }

    let newi = format!("\\{key}\\{value}");
    if newi.len() + s.len() > BIG_INFO_STRING {
        return InfoSetResult::LengthExceeded;
    }

    s.push_str(&newi);
    InfoSetResult::Set
}

/// The three reject checks shared verbatim by both `Info_SetValueForKey`
/// forms, in Raven's order: backslash, semicolon, quote.
fn bad_chars(key: &str, value: &str) -> Option<InfoSetResult> {
    if key.contains('\\') || value.contains('\\') {
        return Some(InfoSetResult::ContainsBackslash);
    }
    if key.contains(';') || value.contains(';') {
        return Some(InfoSetResult::ContainsSemicolon);
    }
    if key.contains('"') || value.contains('"') {
        return Some(InfoSetResult::ContainsQuote);
    }
    None
}

#[cfg(test)]
mod info_tests {
    use super::*;

    #[test]
    fn set_prepends_and_get_round_trips() {
        let mut s = String::new();
        assert_eq!(Info_SetValueForKey(&mut s, "name", "Kyle"), InfoSetResult::Set);
        assert_eq!(Info_SetValueForKey(&mut s, "model", "jedi"), InfoSetResult::Set);
        assert_eq!(s, "\\model\\jedi\\name\\Kyle");
        assert_eq!(Info_ValueForKey(&s, "name"), "Kyle");
        assert_eq!(Info_ValueForKey(&s, "model"), "jedi");
        assert_eq!(Info_ValueForKey(&s, "absent"), "");
    }

    #[test]
    fn get_is_case_insensitive_remove_is_not() {
        let mut s = String::new();
        Info_SetValueForKey(&mut s, "Name", "Kyle");
        assert_eq!(Info_ValueForKey(&s, "name"), "Kyle");
        Info_RemoveKey(&mut s, "name");
        assert_eq!(s, "\\Name\\Kyle");
        Info_RemoveKey(&mut s, "Name");
        assert_eq!(s, "");
    }

    #[test]
    fn set_replaces_existing_key() {
        let mut s = String::new();
        Info_SetValueForKey(&mut s, "team", "red");
        Info_SetValueForKey(&mut s, "skill", "4");
        Info_SetValueForKey(&mut s, "team", "blue");
        assert_eq!(Info_ValueForKey(&s, "team"), "blue");
        assert_eq!(Info_ValueForKey(&s, "skill"), "4");
    }

    #[test]
    fn big_appends() {
        let mut s = String::new();
        Info_SetValueForKey_Big(&mut s, "a", "1");
        Info_SetValueForKey_Big(&mut s, "b", "2");
        assert_eq!(s, "\\a\\1\\b\\2");
    }

    #[test]
    fn rejects_bad_chars_and_overflow() {
        let mut s = String::new();
        assert_eq!(
            Info_SetValueForKey(&mut s, "k\\ey", "v"),
            InfoSetResult::ContainsBackslash
        );
        assert_eq!(
            Info_SetValueForKey(&mut s, "key", "v;1"),
            InfoSetResult::ContainsSemicolon
        );
        assert_eq!(
            Info_SetValueForKey(&mut s, "k\"ey", "v"),
            InfoSetResult::ContainsQuote
        );
        let long = "x".repeat(1020);
        assert_eq!(
            Info_SetValueForKey(&mut s, "key", &long),
            InfoSetResult::LengthExceeded
        );
        assert_eq!(s, "");
    }

    #[test]
    fn empty_value_removes() {
        let mut s = String::new();
        Info_SetValueForKey(&mut s, "team", "red");
        Info_SetValueForKey(&mut s, "team", "");
        assert_eq!(s, "");
    }

    #[test]
    fn validate() {
        assert!(Info_Validate("\\name\\Kyle"));
        assert!(!Info_Validate("\\name\\Ky;le"));
        assert!(!Info_Validate("\\name\\Ky\"le"));
    }
}
