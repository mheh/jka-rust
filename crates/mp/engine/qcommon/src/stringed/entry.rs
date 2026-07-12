//! `SE_Entry_s` — one localized-string record.
//!
//! Design frozen in `docs/subsystems/stringed.md` (roster row: `stringed/entry.rs`).

/// Raven `SE_Entry_s` / `SE_Entry_t` — one localized-string record, a value of
/// `CStringEdPackage::m_StringEntries` (`map<string, SE_Entry_t>`).
///
/// Renamed `SeEntry` per SE-D2: its members are `std::string`, so it has no
/// fixed ABI layout — internal naming (RULING 40) applies, not a link-name
/// freeze. Fields are owned `String`/`i32` (§C9 manual-storage → ownership);
/// shape and field names are pinned by this doc's frontmatter roster row.
///
/// Class definition source: `oracle/codemp/qcommon/stringed_ingame.cpp:48-59`
#[derive(Debug, Clone)]
pub struct SeEntry {
    /// Raven `m_strString` — the resolved localized text (english text run
    /// through `Leetify`, or the foreign-language text, or a `#same` copy of
    /// the cached english).
    pub m_str_string: String,

    /// Raven `m_strDebug` — english (or `"#same"`) debug text, prefixed/
    /// suffixed `[`…`]`; populated only when `m_bLoadDebug` is set. Used only
    /// for debugging, never shipped.
    pub m_str_debug: String,

    /// Raven `m_iFlags` — the `AddFlagReference` bitmask OR'd onto this entry.
    pub m_i_flags: i32,
}

impl SeEntry {
    /// Raven `SE_Entry_s()` — the default ctor: zeroes `m_iFlags`.
    /// `m_strString`/`m_strDebug` are `std::string` members, which
    /// default-construct empty with no explicit ctor-body statement; `String`
    /// mirrors that with an empty `String`.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:54-57`
    pub fn new() -> Self {
        Self {
            m_str_string: String::new(),
            m_str_debug: String::new(),
            m_i_flags: 0,
        }
    }
}

impl Default for SeEntry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Raven's ctor (`:54-57`) only zeroes `m_iFlags`; the two `string`
    /// members default-construct empty with no explicit statement.
    #[test]
    fn new_zeroes_flags_and_empties_strings() {
        let entry = SeEntry::new();
        assert_eq!(entry.m_i_flags, 0);
        assert_eq!(entry.m_str_string, "");
        assert_eq!(entry.m_str_debug, "");
    }

    #[test]
    fn default_matches_new() {
        let a = SeEntry::default();
        let b = SeEntry::new();
        assert_eq!(a.m_i_flags, b.m_i_flags);
        assert_eq!(a.m_str_string, b.m_str_string);
        assert_eq!(a.m_str_debug, b.m_str_debug);
    }
}
