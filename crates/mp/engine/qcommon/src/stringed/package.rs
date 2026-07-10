//! `CStringEdPackage` — the StringEd localization store.
//!
//! Design frozen in `docs/subsystems/stringed.md` (roster row: `stringed/package.rs`).
//! Renamed `StringEdPackage` per SE-D2/RULING 40 (bare `C`-prefix drop). One
//! instance lives as `Common.stringed` (SE-D1(1)/SE-D6, RULING 2/50) — no
//! singleton, no `static mut`; `Engine::new()`'s zeroed-alloc write-list
//! constructs it (SE-D6).
//!
//! Class definition source: `oracle/codemp/qcommon/stringed_ingame.cpp:64-122`
//!
//! **§20 dead surface (module-doc notes, not ported — SE-V2):**
//! `GetNumStrings` (`:109`), `SetReference(int,LPCSTR)` (`:111`), and
//! `GetCurrentFileName()` (`:113`) are declared in the class body but never
//! defined and never called anywhere in the TU. Zero-caller drops.

use std::collections::BTreeMap;

use mp_host_interface::EngineHost;

use super::entry::SeEntry;
use super::{
    api, SE_DEBUGSTR_PREFIX, SE_DEBUGSTR_SUFFIX, SE_EXPORT_SAME, SE_KEYWORD_CONFIG,
    SE_KEYWORD_ENDMARKER, SE_KEYWORD_FILENOTES, SE_KEYWORD_FLAGS, SE_KEYWORD_LANG,
    SE_KEYWORD_NOTES, SE_KEYWORD_REFERENCE, SE_KEYWORD_VERSION, SE_VERSION,
};

/// Minimal `atoi`-semantics parse for the `VERSION` line's number field: skip
/// leading whitespace, an optional sign, then digits; non-numeric input (or
/// none) is `0` — matching C `atoi` (`stringed_ingame.cpp:583`), not
/// `str::parse` (which rejects trailing garbage instead of stopping at it).
fn c_atoi(bytes: &[u8]) -> i32 {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let negative = i < bytes.len() && bytes[i] == b'-';
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
        i += 1;
    }
    let mut value: i64 = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        value = value * 10 + i64::from(bytes[i] - b'0');
        i += 1;
    }
    if negative {
        (-value) as i32
    } else {
        value as i32
    }
}

/// Raven `CStringEdPackage` — the localized-string store: the parse-only
/// scratch fields, the entry store, the flag tables, and the language cache.
/// Members are owned Rust collections (§C9); this struct is the `Common`
/// sub-struct field `engine.common.stringed` (SE-D1(1), no singleton).
///
/// Class definition source: `oracle/codemp/qcommon/stringed_ingame.cpp:64-122`
#[derive(Debug, Clone)]
pub struct StringEdPackage {
    // --- parse-only scratch (`_ParseOnly` fields, `:68-73`) ---
    /// Raven `m_bEndMarkerFound_ParseOnly` — set by `ParseLine` on `ENDMARKER`;
    /// read by `EndMarkerFoundDuringParse`.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:68`
    pub end_marker_found_parse_only: bool,

    /// Raven `m_strCurrentEntryRef_ParseOnly` — the reference currently being
    /// parsed (set by `AddEntry`, read by `GetCurrentReference_ParseOnly`).
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:69`
    pub current_entry_ref_parse_only: String,

    /// Raven `m_strCurrentEntryEnglish_ParseOnly` — the cached english text of
    /// the current entry, for later `"#same"` resolving in a foreign sentence.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:70`
    pub current_entry_english_parse_only: String,

    /// Raven `m_strCurrentFileRef_ParseOnly` — the uppercased file reference
    /// (eg `"OBJECTIVES"`), set by `SetupNewFileParse`.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:71`
    pub current_file_ref_parse_only: String,

    /// Raven `m_strLoadingLanguage_ParseOnly` — eg `"german"`, set by
    /// `SetupNewFileParse` via `ExtractLanguageFromPath`.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:72`
    pub loading_language_parse_only: String,

    /// Raven `m_bLoadingEnglish_ParseOnly`.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:73`
    pub loading_english_parse_only: bool,

    // --- the entry store (`:62,87`) ---
    /// Raven `m_StringEntries` (`mapStringEntries_t`, `map<string,SE_Entry_t>`)
    /// — `BTreeMap` keeps Raven's sorted iteration order (SE-D1(4)).
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:62,87`
    pub string_entries: BTreeMap<String, SeEntry>,

    /// Raven `m_bLoadDebug` — whether `SetString` also populates the debug text.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:88`
    pub load_debug: bool,

    // --- flag tables (`:92-93`) ---
    /// Raven `m_vstrFlagNames` — flag names in first-seen encounter order; the
    /// bit position (`AddFlagReference`) is driven by this `Vec`'s push order,
    /// not by `flag_masks`' (sorted) iteration.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:92`
    pub flag_names: Vec<String>,

    /// Raven `m_mapFlagMasks` (`map<string,int>`) — `BTreeMap` per SE-D1(4).
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:93`
    pub flag_masks: BTreeMap<String, i32>,

    // --- language cache ---
    /// Raven file-scope global `gvLanguagesAvailable` (`vector<string>`) —
    /// folded into this struct per RULING 3 (cross-run state → host-struct
    /// field, not a Rust global). Lazily populated by `se_get_num_languages`.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1071`
    pub languages_available: Vec<String>,
}

impl StringEdPackage {
    /// Raven `CStringEdPackage()` ctor — `Clear(SE_FALSE)`. `SE-D6`:
    /// `StringEdPackage::default()` is this ctor, written explicitly into
    /// `Engine::new()`'s zeroed-alloc write-list (the fields are not
    /// all-zero-valid).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:77-80`
    pub fn new() -> Self {
        let mut pkg = Self {
            end_marker_found_parse_only: false,
            current_entry_ref_parse_only: String::new(),
            current_entry_english_parse_only: String::new(),
            current_file_ref_parse_only: String::new(),
            loading_language_parse_only: String::new(),
            loading_english_parse_only: false,
            string_entries: BTreeMap::new(),
            load_debug: false,
            flag_names: Vec::new(),
            flag_masks: BTreeMap::new(),
            languages_available: Vec::new(),
        };
        pkg.clear(false);
        pkg
    }

    /// Raven `CStringEdPackage::Clear` — clears `m_StringEntries`; clears the
    /// flag tables too **unless** `changing_languages` (flags stay defined once
    /// seen, so cached game-side flag masks survive a language reload; only
    /// the dtor kills them, `:133-141`). Resets the end-marker + current-ref
    /// parse-only scratch.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:127-152`
    pub fn clear(&mut self, changing_languages: bool) {
        self.string_entries.clear();

        if !changing_languages {
            // If we're changing languages, leave these alone. This is to do with
            // any (potentially) cached flag bitmasks on the game side: flags
            // stay defined once they're defined, and only the destructor (at
            // app-end) kills them.
            self.flag_names.clear();
            self.flag_masks.clear();
        }

        self.end_marker_found_parse_only = false;
        self.current_entry_ref_parse_only.clear();
        self.current_entry_english_parse_only.clear();
        // the other vars are cleared in setup_new_file_parse(), and are ok to
        // not do here.
    }

    /// Raven `CStringEdPackage::SetupNewFileParse` — file ref =
    /// `Filename_WithoutPath(Filename_WithoutExt(file_name))` uppercased (eg
    /// `"OBJECTIVES"`); loading-language = `ExtractLanguageFromPath(file_name)`;
    /// `loading_english_parse_only` = (language == "english"); `load_debug` set.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:236-247`
    pub fn setup_new_file_parse(&mut self, file_name: &str, load_debug: bool) {
        let file_ref = self.filename_without_path(&self.filename_without_ext(file_name));
        self.current_file_ref_parse_only = file_ref.to_ascii_uppercase(); // eg "OBJECTIVES"
        self.loading_language_parse_only = self.extract_language_from_path(file_name);
        self.loading_english_parse_only = self
            .loading_language_parse_only
            .eq_ignore_ascii_case("english");
        self.load_debug = load_debug;
    }

    /// Raven `CStringEdPackage::ReadLine` — splits `parse_pos` on `\n`, copies
    /// one line out, skips `\r\n` runs, right-trims whitespace, then
    /// [`Self::rem_kill`]s the comment tail. `parse_pos` advances past the
    /// consumed line (Raven's `LPCSTR &psParsePos` out-param). Raven's
    /// `SE_BOOL` "more lines available" return collapses into `Option` (`None`
    /// at end of data, matching `psParsePos[0] == 0`); the fixed caller-owned
    /// `psDest` buffer becomes an owned `Vec<u8>` return (RULING 3). Bytes, not
    /// `str`: the raw file content is not guaranteed valid UTF-8 (CP1252
    /// hi-chars on `LANG_` sentence lines, cleaned up later by
    /// `CopeWithDumbStringData`).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:344-384`
    pub fn read_line(&mut self, parse_pos: &mut &[u8]) -> Option<Vec<u8>> {
        if parse_pos.is_empty() {
            return None;
        }

        let mut dest: Vec<u8>;
        match parse_pos.iter().position(|&b| b == b'\n') {
            Some(newline_pos) => {
                dest = parse_pos[..newline_pos].to_vec();
                *parse_pos = &parse_pos[newline_pos..];
                // skip over CR or CR/LF pairs
                while !parse_pos.is_empty() && (parse_pos[0] == b'\r' || parse_pos[0] == b'\n') {
                    *parse_pos = &parse_pos[1..];
                }
            }
            None => {
                // last line...
                dest = parse_pos.to_vec();
                *parse_pos = &parse_pos[parse_pos.len()..];
            }
        }

        // clean up the line...
        if !dest.is_empty() {
            while let Some(&last) = dest.last() {
                if last.is_ascii_whitespace() {
                    dest.pop();
                } else {
                    break;
                }
            }

            self.rem_kill(&mut dest);
        }

        Some(dest)
    }

    /// Raven `CStringEdPackage::ParseLine` — keyword-matches the line, in
    /// order: `VERSION` (must equal `SE_VERSION` else error), `CONFIG`/
    /// `FILENOTES`/`NOTES` (absorbed, ignored), `REFERENCE` → `add_entry`,
    /// `FLAGS` → tokenize on `" \t"`, uppercase each, `add_flag_reference`,
    /// `ENDMARKER` → set the end-marker flag, `LANG_` (prefix) → parse the
    /// language word + quoted sentence, run `ConvertCRLiterals_Read` +
    /// `cope_with_dumb_string_data` (SE-D3, needs `sp_leet`'s host indirectly
    /// via `set_string`'s `Leetify` call), then `set_string`; unknown keyword →
    /// error string. Returns `None` for ok, else a `va()`-formatted error
    /// message (SE-D2 owned `String`).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:578-727`
    pub fn parse_line(&mut self, line: &[u8], host: &mut impl EngineHost) -> Option<String> {
        let mut rest: &[u8] = line;

        if self.check_line_for_keyword(SE_KEYWORD_VERSION, &mut rest) {
            // VERSION "1"
            let version_bytes = self.inside_quotes(rest);
            let version_number = c_atoi(&version_bytes);

            if version_number != SE_VERSION {
                Some(format!(
                    "Unexpected version number {version_number}, expecting {SE_VERSION}!\n"
                ))
            } else {
                None
            }
        } else if self.check_line_for_keyword(SE_KEYWORD_CONFIG, &mut rest)
            || self.check_line_for_keyword(SE_KEYWORD_FILENOTES, &mut rest)
            || self.check_line_for_keyword(SE_KEYWORD_NOTES, &mut rest)
        {
            // not used ingame, but need to absorb the token
            None
        } else if self.check_line_for_keyword(SE_KEYWORD_REFERENCE, &mut rest) {
            // REFERENCE	GUARD_GOOD_TO_SEE_YOU
            let local_reference = self.inside_quotes(rest);
            let local_reference = String::from_utf8_lossy(&local_reference).into_owned();
            self.add_entry(&local_reference);
            None
        } else if self.check_line_for_keyword(SE_KEYWORD_FLAGS, &mut rest) {
            // FLAGS 	FLAG_CAPTION FLAG_TYPEMATIC
            let reference = self.get_current_reference_parse_only().to_string();
            if !reference.is_empty() {
                for token in rest
                    .split(|&b| b == b' ' || b == b'\t')
                    .filter(|t| !t.is_empty())
                {
                    // psToken = flag name (in caps)
                    let flag_name = String::from_utf8_lossy(token).to_ascii_uppercase(); // jic
                    self.add_flag_reference(&reference, &flag_name);
                }
                None
            } else {
                Some(format!(
                    "Error parsing file: Unexpected \"{SE_KEYWORD_FLAGS}\"\n"
                ))
            }
        } else if self.check_line_for_keyword(SE_KEYWORD_ENDMARKER, &mut rest) {
            // ENDMARKER — the only major error checking I bother to do (for file truncation)
            self.end_marker_found_parse_only = true;
            None
        } else if rest.len() >= SE_KEYWORD_LANG.len()
            && rest[..SE_KEYWORD_LANG.len()].eq_ignore_ascii_case(SE_KEYWORD_LANG.as_bytes())
        {
            // LANG_ENGLISH 	"GUARD:  Good to see you, sir.  ..."
            let reference = self.get_current_reference_parse_only().to_string();
            if !reference.is_empty() {
                let lang_bytes = &rest[SE_KEYWORD_LANG.len()..];

                // what language is this?...
                let word_end = lang_bytes
                    .iter()
                    .position(|&b| b == b' ' || b == b'\t')
                    .unwrap_or(lang_bytes.len());
                let this_language = String::from_utf8_lossy(&lang_bytes[..word_end]).into_owned();
                let after_lang = &lang_bytes[word_end..];

                let quoted = self.inside_quotes(after_lang);
                let converted = self.convert_cr_literals_read(&quoted);
                // Dammit, I hate having to do crap like this just because other
                // people mess up and put stupid data in their text, so I have to
                // cope with it.
                let coped = api::cope_with_dumb_string_data(&converted, &this_language);

                if self.loading_english_parse_only {
                    // if loading just "english", then go ahead and store it...
                    self.set_string(&reference, &coped, false, host);
                    None
                } else {
                    // if loading a foreign language...
                    let sentence_is_english = this_language.eq_ignore_ascii_case("english");

                    // this check can be omitted, I'm just being extra careful here...
                    if !sentence_is_english
                        && !self
                            .loading_language_parse_only
                            .eq_ignore_ascii_case(&this_language)
                    {
                        // basically this is just checking that an .STE file
                        // override is the same language as the .STR...
                        Some(format!(
                            "Language \"{this_language}\" found when expecting \"{}\"!\n",
                            self.loading_language_parse_only
                        ))
                    } else {
                        self.set_string(&reference, &coped, sentence_is_english, host);
                        None
                    }
                }
            } else {
                Some(format!(
                    "Error parsing file: Unexpected \"{SE_KEYWORD_LANG}\"\n"
                ))
            }
        } else {
            Some(format!(
                "Unknown keyword at linestart: \"{}\"\n",
                String::from_utf8_lossy(rest)
            ))
        }
    }

    /// Raven `CStringEdPackage::GetFlagMask` / the `SE_GetFlagMask` free-function
    /// wrapper that collapses into it (SE-D7/RULING 57 seam method) — map find
    /// → mask, else 0.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:441-451`
    /// (free-function wrapper: `:1057-1060`)
    pub fn get_flag_mask(&self, flag_name: &str) -> i32 {
        self.flag_masks.get(flag_name).copied().unwrap_or(0)
    }

    /// Raven `CStringEdPackage::ExtractLanguageFromPath` —
    /// `Filename_WithoutPath(Filename_PathOnly(file_name))`. Chains two owned-
    /// return `Filename_*` helpers (SE-D1(3)); no caller ever holds two results
    /// of the same helper at once, so the owned-`String` chain is faithful.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:230-233`
    pub fn extract_language_from_path(&self, file_name: &str) -> String {
        self.filename_without_path(&self.filename_path_only(file_name))
    }

    /// Raven `CStringEdPackage::EndMarkerFoundDuringParse` — inline getter for
    /// `end_marker_found_parse_only`, checked by `se_load_actual` to require
    /// the `ENDMARKER` line else raise a "Truncated file" error.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:101-104`
    pub fn end_marker_found_during_parse(&self) -> bool {
        self.end_marker_found_parse_only
    }

    // --- private: parse-mutation helpers (Raven `private:` section, `:106-121`) ---

    /// Raven `CStringEdPackage::AddEntry` — key = `"<file_ref>_<local_reference>"`;
    /// insert an empty entry if absent (never overwrite — `.STE` override files
    /// carry no flags and must not wipe parsed flags, `:740-742`), then set
    /// `current_entry_ref_parse_only`.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:738-750`
    fn add_entry(&mut self, local_reference: &str) {
        // the reason I don't just assign it anyway is because the optional
        // .STE override files don't contain flags, and therefore would wipe
        // out the parsed flags of the .STR file...
        let key = format!("{}_{}", self.current_file_ref_parse_only, local_reference);
        self.string_entries.entry(key).or_insert_with(SeEntry::new);
        self.current_entry_ref_parse_only = local_reference.to_string();
    }

    /// Raven `CStringEdPackage::SetString` — find the current file-ref-keyed
    /// entry; if english/debug-english, store `leetify(new_string)` (SE-D3:
    /// threads `host` for the `sp_leet` cvar read) into `m_strString`, and if
    /// `load_debug` build `m_strDebug` = `[` + text + `]`, cache the english for
    /// later `"#same"`; else (foreign) if text == `"#same"`
    /// (`SE_EXPORT_SAME`) copy the cached english, else store the foreign text.
    /// Miss → `__ASSERT(0)` (SE-V3: release no-op, faithful behavior is a no-op
    /// fall-through, not a panic). `new_string` is bytes (post-
    /// `cope_with_dumb_string_data`, pre-UTF-8 validation) — the stored
    /// `SeEntry::m_str_string`/`m_str_debug` are `String`, so the conversion
    /// happens at the point of storage (lossy, matching that fixtures are
    /// hand-authored ASCII/UTF-8-safe test data per SE-D4).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:775-820`
    fn set_string(
        &mut self,
        local_reference: &str,
        new_string: &[u8],
        english_debug: bool,
        host: &mut impl EngineHost,
    ) {
        let key = format!("{}_{}", self.current_file_ref_parse_only, local_reference);

        if !self.string_entries.contains_key(&key) {
            // should never happen — SE-V3: `__ASSERT(0)` is a release no-op,
            // so the faithful fall-through is doing nothing.
            return;
        }

        let load_debug = self.load_debug;

        if english_debug || self.loading_english_parse_only {
            // then this is the leading english text of a foreign sentence pair
            // (so it's the debug-key text), or it's the only text when it's
            // english being loaded...
            let leet = api::leetify(new_string, host);
            let leet_string = String::from_utf8_lossy(&leet).into_owned();
            let raw_string = String::from_utf8_lossy(new_string).into_owned();

            if let Some(entry) = self.string_entries.get_mut(&key) {
                entry.m_str_string = leet_string;
                if load_debug {
                    entry.m_str_debug =
                        format!("{SE_DEBUGSTR_PREFIX}{raw_string}{SE_DEBUGSTR_SUFFIX}");
                }
            }
            // for possible "#same" resolving in foreign later
            self.current_entry_english_parse_only = raw_string;
        } else {
            // then this is foreign text (so check for "#same" resolving)...
            if new_string.eq_ignore_ascii_case(SE_EXPORT_SAME.as_bytes()) {
                let cached_english = self.current_entry_english_parse_only.clone();
                if let Some(entry) = self.string_entries.get_mut(&key) {
                    // foreign "#same" is now english
                    entry.m_str_string = cached_english;
                    if load_debug {
                        // english (debug) is now "#same"
                        entry.m_str_debug =
                            format!("{SE_DEBUGSTR_PREFIX}{SE_EXPORT_SAME}{SE_DEBUGSTR_SUFFIX}");
                    }
                }
            } else {
                // foreign is just foreign
                let raw_string = String::from_utf8_lossy(new_string).into_owned();
                if let Some(entry) = self.string_entries.get_mut(&key) {
                    entry.m_str_string = raw_string;
                }
            }
        }
    }

    /// Raven `CStringEdPackage::AddFlagReference` — if the flag name is new,
    /// push to `flag_names` and set `flag_masks[name] = 1 << (len - 1)` (the
    /// bit is the first-seen encounter index, parity-critical, driven by the
    /// `Vec` push order); then OR the mask into the current entry's flags.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:454-474`
    fn add_flag_reference(&mut self, local_reference: &str, flag_name: &str) {
        // add the flag to the list of known ones...
        let mut mask = self.get_flag_mask(flag_name);
        if mask == 0 {
            self.flag_names.push(flag_name.to_string());
            mask = 1 << (self.flag_names.len() - 1);
            self.flag_masks.insert(flag_name.to_string(), mask);
        }

        // then add the reference to this flag to the currently-parsed reference...
        let key = format!("{}_{}", self.current_file_ref_parse_only, local_reference);
        if let Some(entry) = self.string_entries.get_mut(&key) {
            entry.m_i_flags |= mask;
        }
    }

    /// Raven `CStringEdPackage::GetCurrentReference_ParseOnly` — returns
    /// `current_entry_ref_parse_only`, else `""` for none. Borrows the stored
    /// `String` (SE-D1(2)).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:731-734`
    fn get_current_reference_parse_only(&self) -> &str {
        &self.current_entry_ref_parse_only
    }

    /// Raven `CStringEdPackage::CheckLineForKeyword` — case-insensitive prefix
    /// match (`Q_stricmpn`) of `keyword` against `line`; on match advances
    /// `line` past the keyword + any `\t`/` ` whitespace and returns `true`,
    /// else leaves `line` untouched and returns `false`.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:254-270`
    fn check_line_for_keyword(&self, keyword: &str, line: &mut &[u8]) -> bool {
        let kw = keyword.as_bytes();
        if line.len() >= kw.len() && line[..kw.len()].eq_ignore_ascii_case(kw) {
            *line = &line[kw.len()..];

            // skip whitespace to arrive at next item...
            while !line.is_empty() && (line[0] == b'\t' || line[0] == b' ') {
                *line = &line[1..];
            }
            true
        } else {
            false
        }
    }

    /// Raven `CStringEdPackage::InsideQuotes` — strip leading whitespace +
    /// opening quote, strip trailing whitespace + closing quote. Raven's
    /// process-wide `static string str` becomes an owned `Vec<u8>` return
    /// (SE-D1(3), RULING 3); bytes, not `str` — this also strips the raw
    /// `LANG_` sentence text, which may carry CP1252 hi-chars pre-cleanup.
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:388-436`
    fn inside_quotes(&self, line: &[u8]) -> Vec<u8> {
        let mut i = 0;
        // skip any leading whitespace...
        while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
            i += 1;
        }
        // skip any leading quote...
        if i < line.len() && line[i] == b'"' {
            i += 1;
        }

        let mut result = line[i..].to_vec();

        if !result.is_empty() {
            // lose any trailing whitespace... (bounds-checked unlike Raven's
            // `str.c_str()[strlen-1]`, which underflows if trimming empties the
            // string; the defined behavior here is simply an empty result)
            while let Some(&last) = result.last() {
                if last == b' ' || last == b'\t' {
                    result.pop();
                } else {
                    break;
                }
            }
            // lose any trailing quote...
            if result.last() == Some(&b'"') {
                result.pop();
            }
        }

        result
    }

    /// Raven `CStringEdPackage::ConvertCRLiterals_Read` — rewrite the 2-byte
    /// `\n` literal (`"\\n"`) to a 1-byte newline. Raven's `static string str`
    /// becomes an owned `Vec<u8>` return (SE-D1(3)).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:275-287`
    fn convert_cr_literals_read(&self, string: &[u8]) -> Vec<u8> {
        let mut out = string.to_vec();
        while let Some(pos) = out.windows(2).position(|w| w == b"\\n") {
            out[pos] = b'\n';
            out.remove(pos + 1);
        }
        out
    }

    /// Raven `CStringEdPackage::REMKill` — kill off any `"//"`-onwards comment
    /// tail in `buffer`, but NOT if it's inside a quoted string (parity of
    /// double-quote count before the match); right-trims whitespace after the
    /// cut. Mutates in place (Raven's `char *psBuffer` in/out param).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:292-340`
    fn rem_kill(&self, buffer: &mut Vec<u8>) {
        let mut scan_pos = 0usize;
        let mut double_quotes_so_far = 0i32;

        // scan forwards in case there are more than one (and the first is
        // inside quotes)...
        loop {
            let found = buffer[scan_pos..].windows(2).position(|w| w == b"//");
            let Some(rel_pos) = found else {
                return;
            };
            let p = scan_pos + rel_pos;

            // count the number of double quotes before this point, if odd
            // number, then we're inside quotes...
            let mut double_quote_count = double_quotes_so_far;
            for &byte in &buffer[scan_pos..p] {
                if byte == b'"' {
                    double_quote_count += 1;
                }
            }

            if double_quote_count % 2 == 0 {
                // not inside quotes, so kill line here...
                buffer.truncate(p);

                // and remove any trailing whitespace...
                while let Some(&last) = buffer.last() {
                    if last.is_ascii_whitespace() {
                        buffer.pop();
                    } else {
                        break;
                    }
                }
                return;
            } else {
                // inside quotes (blast), oh well, skip past and keep scanning...
                scan_pos = p + 1;
                double_quotes_so_far = double_quote_count;
            }
        }
    }

    /// Raven `CStringEdPackage::Filename_PathOnly` — loses anything after the
    /// path (if any), eg `"dir/name.bmp"` → `"dir"` (copes with either slash
    /// scheme). Raven's own `static char sString[iSE_MAX_FILENAME_LENGTH]`
    /// becomes an owned `String` return (SE-D1(3), RULING 3).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:162-175`
    fn filename_path_only(&self, filename: &str) -> String {
        let bytes = filename.as_bytes();
        match bytes.iter().rposition(|&b| b == b'\\' || b == b'/') {
            Some(pos) => filename[..pos].to_string(),
            None => filename.to_string(),
        }
    }

    /// Raven `CStringEdPackage::Filename_WithoutExt` — returns eg `"dir/name"`
    /// for `"dir/name.bmp"`; truncates at the last `.` only if it is past the
    /// last slash (guards a path with no extension). Owned `String` return
    /// (SE-D1(3)).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:184-203`
    fn filename_without_ext(&self, filename: &str) -> String {
        let bytes = filename.as_bytes();
        let last_dot = bytes.iter().rposition(|&b| b == b'.');
        let last_backslash = bytes.iter().rposition(|&b| b == b'\\');
        let last_slash = bytes.iter().rposition(|&b| b == b'/');

        // special check, make sure the first suffix we found from the end
        // wasn't just a directory suffix (eg on a path'd filename with no
        // extension anyway)
        if let Some(dot) = last_dot {
            let past_backslash = last_backslash.is_none_or(|p| dot > p);
            let past_slash = last_slash.is_none_or(|p| dot > p);
            if past_backslash && past_slash {
                return filename[..dot].to_string();
            }
        }

        filename.to_string()
    }

    /// Raven `CStringEdPackage::Filename_WithoutPath` — scans for the last
    /// slash, returns the tail (the actual filename only, no path). Owned
    /// `String` return (SE-D1(3)).
    ///
    /// Method source: `oracle/codemp/qcommon/stringed_ingame.cpp:211-227`
    fn filename_without_path(&self, filename: &str) -> String {
        let bytes = filename.as_bytes();
        let mut copy_pos = 0;
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'/' || b == b'\\' {
                copy_pos = i + 1;
            }
        }
        filename[copy_pos..].to_string()
    }

    // --- the frozen lookup seam (Seam definition (b), SE-D7/RULING 57) ---
    // Raven's `SE_GetString`/`SE_GetFlags` overload pairs + `SE_GetNumFlags`/
    // `SE_GetFlagName` are free functions in Raven, not class members; they
    // become `StringEdPackage` seam methods here (SE-D7) — the sole home of
    // the arity-overloaded lookup API, not re-exported `SE_*` free functions.

    /// Raven `SE_GetString(psPackageAndStringReference)` — copies + uppercases
    /// the key internally (owned scratch), finds it, and returns the debug
    /// text when `se_debug->integer && load_debug` else the resolved text;
    /// miss → `""` (no active assert, SE-V3). Borrows from the entry's owned
    /// `String` (SE-D1(2)).
    ///
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:981-1007`
    pub fn get_string(&self, reference: &str, host: &mut impl EngineHost) -> &str {
        let key = reference.to_ascii_uppercase();

        match self.string_entries.get(&key) {
            Some(entry) => {
                if host.cvar_integer("se_debug") != 0 && self.load_debug {
                    &entry.m_str_debug
                } else {
                    &entry.m_str_string
                }
            }
            // should never get here, but fall back anyway... (except we DO use
            // this to see if there's a debug-friendly key bind, which may not
            // exist)
            None => "",
        }
    }

    /// Raven `SE_GetString(psPackageReference, psStringReference)` — builds
    /// `"PKG_REF"` (owned), uppercases, delegates to [`Self::get_string`].
    ///
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:971-978`
    pub fn get_string2(&self, package: &str, string_ref: &str, host: &mut impl EngineHost) -> &str {
        let reference = format!("{package}_{string_ref}");
        self.get_string(&reference, host)
    }

    /// Raven `SE_GetFlags(psPackageAndStringReference)` — find; hit → flags,
    /// miss → `__ASSERT(0)` then `0` (SE-V3: release no-op, faithful fall-
    /// through is `0`, not a panic).
    ///
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1021-1036`
    pub fn get_flags(&self, reference: &str) -> i32 {
        match self.string_entries.get(reference) {
            Some(entry) => entry.m_i_flags,
            // should never get here, but fall back anyway... (SE-V3:
            // `__ASSERT(0)` is a release no-op)
            None => 0,
        }
    }

    /// Raven `SE_GetFlags(psPackageReference, psStringReference)` — builds
    /// `"PKG_REF"`, delegates to [`Self::get_flags`].
    ///
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1012-1018`
    pub fn get_flags2(&self, package: &str, string_ref: &str) -> i32 {
        let reference = format!("{package}_{string_ref}");
        self.get_flags(&reference)
    }

    /// Raven `SE_GetNumFlags` — `flag_names.len()`.
    ///
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1039-1042`
    pub fn get_num_flags(&self) -> i32 {
        self.flag_names.len() as i32
    }

    /// Raven `SE_GetFlagName` — `flag_names[flag_index]`; out-of-range or
    /// negative → `""` (SE-V3/V4: the release `assert` is a no-op and the
    /// signed/unsigned compare would otherwise wrap a negative index to a huge
    /// value, so an explicit `idx < 0 || idx >= len` guard reproduces the
    /// faithful fall-through). Borrows from the stored `Vec` (SE-D1(2)).
    ///
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1044-1053`
    pub fn get_flag_name(&self, flag_index: i32) -> &str {
        // SE-V4: a signed/unsigned compare in the oracle would wrap a negative
        // index into a huge value and fail the bound; guard explicitly instead.
        if flag_index < 0 || flag_index as usize >= self.flag_names.len() {
            return "";
        }
        &self.flag_names[flag_index as usize]
    }
}

impl Default for StringEdPackage {
    /// SE-D6: `StringEdPackage::default()` = Raven ctor's `Clear(SE_FALSE)`
    /// (`:79`) — the write `Engine::new()`'s zeroed-alloc write-list performs
    /// in place of a nonexistent `Common::default()`.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use mp_host_interface::mock::MockHost;

    use super::*;

    /// `REMKill` (`:292-340`) must NOT kill a `//` that's inside a quoted
    /// string — the whole point of the double-quote-parity scan.
    #[test]
    fn rem_kill_respects_quotes() {
        let pkg = StringEdPackage::new();
        let mut buf = br#""http://example.com" // trailing comment"#.to_vec();
        pkg.rem_kill(&mut buf);
        assert_eq!(buf, br#""http://example.com""#);
    }

    #[test]
    fn rem_kill_no_comment_is_untouched() {
        let pkg = StringEdPackage::new();
        let mut buf = b"REFERENCE FOO".to_vec();
        pkg.rem_kill(&mut buf);
        assert_eq!(buf, b"REFERENCE FOO");
    }

    #[test]
    fn inside_quotes_strips_quotes_and_whitespace() {
        let pkg = StringEdPackage::new();
        assert_eq!(pkg.inside_quotes(b"  \"hello world\"  "), b"hello world");
        assert_eq!(
            pkg.inside_quotes(b"\"no trailing quote"),
            b"no trailing quote"
        );
    }

    /// SE-D1(4): the flag bit is the first-seen encounter index driven by
    /// `flag_names`' push order, not by (sorted) `flag_masks` iteration.
    #[test]
    fn add_flag_reference_bit_is_encounter_order() {
        let mut pkg = StringEdPackage::new();
        pkg.current_file_ref_parse_only = "OBJ".to_string();
        pkg.string_entries
            .insert("OBJ_A".to_string(), SeEntry::new());

        pkg.add_flag_reference("A", "ZEBRA"); // first-seen -> bit 0
        pkg.add_flag_reference("A", "APPLE"); // second-seen -> bit 1, despite sorting before ZEBRA

        assert_eq!(pkg.get_flag_mask("ZEBRA"), 1);
        assert_eq!(pkg.get_flag_mask("APPLE"), 2);
        assert_eq!(pkg.string_entries["OBJ_A"].m_i_flags, 1 | 2);
        // re-referencing an existing flag must not shift the assigned bit
        pkg.add_flag_reference("A", "ZEBRA");
        assert_eq!(pkg.get_flag_mask("ZEBRA"), 1);
    }

    /// `SetString`'s foreign `"#same"` path (`:796-804`) copies the cached
    /// english text/debug rather than storing the literal token.
    #[test]
    fn set_string_same_resolves_to_cached_english() {
        let mut host = MockHost::new();
        let mut pkg = StringEdPackage::new();
        pkg.current_file_ref_parse_only = "OBJ".to_string();
        pkg.loading_english_parse_only = false;
        pkg.load_debug = true;
        pkg.string_entries
            .insert("OBJ_A".to_string(), SeEntry::new());

        // Prime the english cache as the english pass of SetString would.
        pkg.set_string("A", b"Hello there", true, &mut host);
        assert_eq!(pkg.string_entries["OBJ_A"].m_str_string, "Hello there");

        // Foreign "#same" should now reuse the cached english text.
        pkg.set_string("A", b"#same", false, &mut host);
        assert_eq!(pkg.string_entries["OBJ_A"].m_str_string, "Hello there");
        assert_eq!(pkg.string_entries["OBJ_A"].m_str_debug, "[#same]");
    }

    /// SE-V3/V4: `GetFlagName` on an out-of-range or negative index is a
    /// no-op fall-through to `""`, not a panic.
    #[test]
    fn get_flag_name_out_of_range_and_negative() {
        let mut pkg = StringEdPackage::new();
        pkg.flag_names.push("ONE".to_string());
        assert_eq!(pkg.get_flag_name(0), "ONE");
        assert_eq!(pkg.get_flag_name(1), "");
        assert_eq!(pkg.get_flag_name(-1), "");
    }

    /// `ParseLine`'s `VERSION` keyword (`:588-598`) rejects any version other
    /// than `SE_VERSION`.
    #[test]
    fn parse_line_version_mismatch_errors() {
        let mut host = MockHost::new();
        let mut pkg = StringEdPackage::new();
        pkg.current_file_ref_parse_only = "OBJ".to_string();

        assert!(pkg.parse_line(b"VERSION \"1\"", &mut host).is_none());
        assert!(pkg.parse_line(b"VERSION \"2\"", &mut host).is_some());
    }

    /// End-to-end: REFERENCE + FLAGS + LANG_ENGLISH + ENDMARKER round-trips
    /// through `parse_line` into the entry store and end-marker flag,
    /// matching `SE_Load_Actual`'s per-line drive loop.
    #[test]
    fn parse_line_full_reference_round_trip() {
        let mut host = MockHost::new();
        let mut pkg = StringEdPackage::new();
        pkg.setup_new_file_parse("strings/english/obj.str", false);

        assert!(pkg
            .parse_line(b"REFERENCE GUARD_HELLO", &mut host)
            .is_none());
        assert!(pkg.parse_line(b"FLAGS FLAG_CAPTION", &mut host).is_none());
        assert!(pkg
            .parse_line(b"LANG_ENGLISH \"Good to see you.\"", &mut host)
            .is_none());
        assert!(pkg.parse_line(b"ENDMARKER", &mut host).is_none());

        assert!(pkg.end_marker_found_during_parse());
        let key = "OBJ_GUARD_HELLO";
        assert_eq!(pkg.string_entries[key].m_str_string, "Good to see you.");
        assert_eq!(
            pkg.string_entries[key].m_i_flags,
            pkg.get_flag_mask("FLAG_CAPTION")
        );
        assert_eq!(pkg.get_string(key, &mut host), "Good to see you.");
    }

    /// `CheckLineForKeyword` (`:254-270`) is case-insensitive and advances
    /// past the keyword plus any following whitespace, leaving a non-matching
    /// line untouched.
    #[test]
    fn check_line_for_keyword_case_insensitive_and_advances() {
        let pkg = StringEdPackage::new();
        let mut line: &[u8] = b"version \t\"1\"";
        assert!(pkg.check_line_for_keyword("VERSION", &mut line));
        assert_eq!(line, b"\"1\"");

        let mut untouched: &[u8] = b"NOTAKEYWORD foo";
        assert!(!pkg.check_line_for_keyword("VERSION", &mut untouched));
        assert_eq!(untouched, b"NOTAKEYWORD foo");
    }

    /// `Filename_WithoutExt` (`:184-203`) only truncates at a `.` that comes
    /// after the last path separator, so a dotted directory with an
    /// extension-less filename is left alone.
    #[test]
    fn filename_without_ext_guards_dotted_directory() {
        let pkg = StringEdPackage::new();
        assert_eq!(pkg.filename_without_ext("dir/name.bmp"), "dir/name");
        assert_eq!(pkg.filename_without_ext("dir.v2/name"), "dir.v2/name");
    }
}
