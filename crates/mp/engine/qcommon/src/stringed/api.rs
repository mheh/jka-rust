//! StringEd load / language-selection API + file-static parse helpers.
//!
//! Idiomatic reimplementation of the `SE_*` public C API and the file-static
//! helpers (`Leetify`, `CopeWithDumbStringData`, `SE_Load_Actual`,
//! `SE_GetFoundFile`) from `oracle/codemp/qcommon/stringed_ingame.cpp` (design
//! frozen in `docs/subsystems/stringed.md`). These are internal Rust→Rust
//! functions with idiomatic snake_case names (SE-D2/SE-D7, RULING 40) — not
//! link/syscall targets — each threading `&mut StringEdPackage` +
//! `&mut impl EngineHost` (SE-D3).
//!
//! The arity-overloaded lookup getters (`SE_GetString`/`SE_GetFlags` pairs and
//! the flag getters) are NOT here — they are [`StringEdPackage`] seam methods
//! (SE-D7/RULING 57).

use std::collections::BTreeSet;

use mp_host_interface::EngineHost;
use mp_qshared::shared::error_parm::errorParm_t;

use super::interface::{se_build_file_list, se_free_file_data_after_load, se_load_file_data};
use super::package::StringEdPackage;
use super::{
    SE_EXPORT_FILE_EXTENSION, SE_INGAME_FILE_EXTENSION, SE_KEYWORD_ENDMARKER, SE_STRINGS_DIR,
};

// Raven cvar registration flags (`oracle/codemp/game/q_shared.h:1782,1795,1799`);
// not yet ported to a shared qshared const table, so inlined at their sole use.
const CVAR_ARCHIVE: i32 = 0x0000_0001;
const CVAR_ROM: i32 = 0x0000_0040;
const CVAR_NORESTART: i32 = 0x0000_0400;

/// Raven `Leetify` — leet-substitute a string when `sp_leet->integer == 42`
/// (`sp_leet` read via `cvar_integer`, SE-D3). Raven's process-wide `static
/// string` becomes an owned return (SE-D1(3), RULING 3).
///
/// Called by `StringEdPackage::set_string` on the english/debug-english store
/// path. Byte-level substitution; every replacement is ASCII, so it preserves
/// UTF-8 validity.
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:752-772`
pub fn leetify(string: &[u8], host: &mut impl EngineHost) -> Vec<u8> {
    let mut out = string.to_vec();
    if host.cvar_integer("sp_leet") == 42 {
        // Laziness because of `strchr()` — the upper-case run is duplicated.
        const REPLACE: [(u8, u8); 16] = [
            (b'o', b'0'),
            (b'l', b'1'),
            (b'e', b'3'),
            (b'a', b'4'),
            (b's', b'5'),
            (b't', b'7'),
            (b'i', b'!'),
            (b'h', b'#'),
            (b'O', b'0'),
            (b'L', b'1'),
            (b'E', b'3'),
            (b'A', b'4'),
            (b'S', b'5'),
            (b'T', b'7'),
            (b'I', b'!'),
            (b'H', b'#'),
        ];
        for (from, to) in REPLACE {
            for byte in out.iter_mut() {
                if *byte == from {
                    *byte = to;
                }
            }
        }
    }
    out
}

/// Raven `CopeWithDumbStringData` — clean up Windows-1252 "rich text" bytes
/// (smart quotes, ellipsis, dashes) for the western SBCS languages only
/// (ENGLISH/FRENCH/GERMAN/ITALIAN/SPANISH/POLISH/RUSSIAN); Asian/MBCS text is
/// left untouched. Raven `Z_Malloc(strlen*3)`s a scratch buffer (the `*3`
/// covers the 0x85 ellipsis 1→3 expansion) and the caller `Z_Free`s it; that
/// becomes an owned `Vec<u8>` (SE-V5, §C9). The `*3` sizing is a C
/// buffer-safety detail, not observable output.
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:483-574`
pub fn cope_with_dumb_string_data(sentence: &[u8], this_language: &str) -> Vec<u8> {
    let western = ["ENGLISH", "FRENCH", "GERMAN", "ITALIAN", "SPANISH", "POLISH", "RUSSIAN"]
        .iter()
        .any(|lang| lang.eq_ignore_ascii_case(this_language));
    if !western {
        return sentence.to_vec();
    }

    let mut out = Vec::with_capacity(sentence.len());
    for &byte in sentence {
        match byte {
            // 0x92 "rich" apostrophe / 0x91 -> '\''
            0x92 | 0x91 => out.push(0x27),
            // 0x93 / 0x94 smart quotes -> '"'
            0x93 | 0x94 => out.push(b'"'),
            // 0x0B -> full stop
            0x0B => out.push(b'.'),
            // 0x85 ellipsis char -> 3-char "..."
            0x85 => {
                out.push(b'.');
                out.push(b'.');
                out.push(b'.');
            }
            // 0x96 / 0x97 -> '-'
            0x96 | 0x97 => out.push(0x2D),
            _ => out.push(byte),
        }
    }

    // Bug fix for picky grammatical errors: replace "?." with "? ".
    let mut i = 0;
    while i + 1 < out.len() {
        if out[i] == b'?' && out[i + 1] == b'.' {
            out[i + 1] = b' ';
        }
        i += 1;
    }

    // StripEd and our print code don't support tabs.
    for byte in out.iter_mut() {
        if *byte == 0x09 {
            *byte = b' ';
        }
    }

    out
}

/// Raven `SE_GetFoundFile` — pop one `';'`-delimited entry from the front of a
/// build-file-list result string (erasing it in place) and return the extracted
/// name. Empty results → Raven's `NULL` loop-terminator (`:880`) → `None`.
/// Raven's `static char sTemp[1024]` scratch becomes the owned return (SE-D1(3)).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:875-902`
pub fn se_get_found_file(results: &mut String) -> Option<String> {
    if results.is_empty() {
        return None;
    }

    match results.find(';') {
        Some(pos) => {
            let name = results[..pos].to_owned();
            // Erase through the semicolon.
            *results = results[pos + 1..].to_owned();
            Some(name)
        }
        None => {
            // No semicolon found (probably last entry) — consume the remainder.
            let name = std::mem::take(results);
            Some(name)
        }
    }
}

/// Raven `SE_Load_Actual` — load one file, parse it line-by-line into the store,
/// then require the ENDMARKER. Returns `None` for ok, else a `va()`-formatted
/// error message (SE-D2 owned `String`). A `bSpeculativeLoad` miss is not an
/// error (the `.ste` override path).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:828-873`
pub fn se_load_actual(
    pkg: &mut StringEdPackage,
    file_name: &str,
    load_debug: bool,
    speculative_load: bool,
    host: &mut impl EngineHost,
) -> Option<String> {
    let mut error_message: Option<String> = None;

    match se_load_file_data(file_name, host) {
        Some(loaded_data) => {
            let mut parse_pos: &[u8] = &loaded_data;
            pkg.setup_new_file_parse(file_name, load_debug);

            while error_message.is_none() {
                match pkg.read_line(&mut parse_pos) {
                    Some(line) => {
                        if !line.is_empty() {
                            error_message = pkg.parse_line(&line, host);
                        }
                    }
                    None => break,
                }
            }

            se_free_file_data_after_load(loaded_data, host);

            if error_message.is_none() && !pkg.end_marker_found_during_parse() {
                error_message = Some(format!(
                    "Truncated file, failed to find \"{SE_KEYWORD_ENDMARKER}\" at file end!"
                ));
            }
        }
        None => {
            if !speculative_load {
                error_message = Some(format!("Unable to load \"{file_name}\"!"));
            }
        }
    }

    error_message
}

/// Raven `SE_Load` — load a language file (prepending `strings/<language>/`
/// when the name is path-less), then speculatively load the matching `.ste`
/// override. Returns `None` for ok, else the error message; a critical failure
/// raises `Com_Error(ERR_DROP)` (→ `EngineHost::error`, diverges), a non-critical
/// one is a developer print (`Com_DPrintf`).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:910-966`
pub fn se_load(
    pkg: &mut StringEdPackage,
    file_name: &str,
    load_debug: bool,
    fail_is_critical: bool,
    host: &mut impl EngineHost,
) -> Option<String> {
    // In-game callers pass names without paths, but a language load expects them.
    let mut path = String::new();
    if !file_name.contains('/') {
        path.push_str(SE_STRINGS_DIR);
        path.push('/');
        // `se_language` always exists once `SE_Init` has registered it.
        path.push_str(&host.cvar_string("se_language"));
        path.push('/');
    }
    path.push_str(file_name);
    default_extension(&mut path, SE_INGAME_FILE_EXTENSION);
    let file_name = path;

    let mut error_message = se_load_actual(pkg, &file_name, load_debug, false, host);

    // Check for a corresponding / overriding .STE file and load it afterwards.
    if error_message.is_none() {
        if let Some(dot) = file_name.rfind('.') {
            // Only when the current extension is the same length as ".ste".
            if file_name.len() - dot == SE_EXPORT_FILE_EXTENSION.len() {
                let mut ste_name = file_name[..dot].to_owned();
                ste_name.push_str(SE_EXPORT_FILE_EXTENSION);
                error_message = se_load_actual(pkg, &ste_name, load_debug, true, host);
            }
        }
    }

    if let Some(ref message) = error_message {
        if fail_is_critical {
            host.error(
                errorParm_t::ERR_DROP,
                &format!(
                    "SE_Load(): Couldn't load \"{file_name}\"!\n\nError: \"{message}\"\n"
                ),
            );
        } else {
            // Com_DPrintf (developer-gated print, SE-D3).
            host.print(&format!("SE_Load(): Couldn't load \"{file_name}\"!\n"));
        }
    }

    error_message
}

/// Raven `SE_NewLanguage` — `Clear(SE_TRUE)`, keeping the flag tables defined
/// across a language change (Clear's `!bChangingLanguages` semantics).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1144-1147`
pub fn se_new_language(pkg: &mut StringEdPackage) {
    pkg.clear(true);
}

/// Raven `SE_LoadLanguage` — reload the whole language: `SE_NewLanguage`, then
/// build the strings file list and `SE_Load` each file whose extracted language
/// matches. Threads the first `SE_Load` error up (→ `None` on ok). Raven's
/// `bLoadDebug` defaults to `SE_TRUE` at the call sites.
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1208-1243`
pub fn se_load_language(
    pkg: &mut StringEdPackage,
    language: &str,
    load_debug: bool,
    host: &mut impl EngineHost,
) -> Option<String> {
    let mut error_message: Option<String> = None;

    if !language.is_empty() {
        se_new_language(pkg);

        let (mut results, _files_found) = se_build_file_list(SE_STRINGS_DIR, host);

        while error_message.is_none() {
            match se_get_found_file(&mut results) {
                Some(found) => {
                    let this_lang = pkg.extract_language_from_path(&found);
                    if language.eq_ignore_ascii_case(&this_lang) {
                        error_message = se_load(pkg, &found, load_debug, true, host);
                    }
                }
                None => break,
            }
        }
    }
    // else: Raven `__ASSERT(0 && "Bad language name!")` — release no-op (SE-V3).

    error_message
}

/// Raven `SE_CheckForLanguageUpdates` — called every `Com_Frame`; if the
/// `se_language` cvar was modified, reload the language. Raven's
/// read-then-clear-`modified` idiom collapses into `cvar_take_modified` (SE-D3,
/// so no host round-trip observes the in-between state).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1250-1261`
pub fn se_check_for_language_updates(pkg: &mut StringEdPackage, host: &mut impl EngineHost) {
    if host.cvar_take_modified("se_language") {
        let language = host.cvar_string("se_language");
        if let Some(message) = se_load_language(pkg, &language, true, host) {
            host.error(errorParm_t::ERR_DROP, &message);
        }
    }
}

/// Raven `SE_GetNumLanguages` — lazily populate the language cache (scan the
/// `strings/` tree, dedup via a set, english first) and return its size. Slow
/// (a full directory scan); the result is cached (`languages_available`).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1072-1116`
pub fn se_get_num_languages(pkg: &mut StringEdPackage, host: &mut impl EngineHost) -> i32 {
    if pkg.languages_available.is_empty() {
        let (mut results, _files_found) = se_build_file_list(SE_STRINGS_DIR, host);

        let mut unique: BTreeSet<String> = BTreeSet::new();
        while let Some(found) = se_get_found_file(&mut results) {
            let language = pkg.extract_language_from_path(&found);
            if !unique.contains(&language) {
                unique.insert(language.clone());
                // If english is available it should always be first.
                if language.eq_ignore_ascii_case("english") {
                    pkg.languages_available.insert(0, language);
                } else {
                    pkg.languages_available.push(language);
                }
            }
        }
    }

    pkg.languages_available.len() as i32
}

/// Raven `SE_GetLanguageName` — the cached language name at `lang_index`, eg
/// "german" (`SE_GetNumLanguages` must have run first). Borrows the stored cache
/// entry (SE-D1(2)). Out-of-range / negative → "" (SE-V3/V4: the release
/// `assert` is a no-op, the faithful observable is the fall-through empty string;
/// the negative guard reproduces Raven's signed/unsigned wrap).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1120-1129`
pub fn se_get_language_name(pkg: &StringEdPackage, lang_index: i32) -> &str {
    if lang_index < 0 || lang_index as usize >= pkg.languages_available.len() {
        return "";
    }
    &pkg.languages_available[lang_index as usize]
}

/// Raven `SE_GetLanguageDir` — `va("strings/<name>")`, eg "strings/german". The
/// `va()` scratch becomes an owned `String` (SE-D1(3)). Out-of-range / negative
/// → "" (SE-V3/V4).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1133-1142`
pub fn se_get_language_dir(pkg: &StringEdPackage, lang_index: i32) -> String {
    if lang_index < 0 || lang_index as usize >= pkg.languages_available.len() {
        return String::new();
    }
    format!("{}/{}", SE_STRINGS_DIR, pkg.languages_available[lang_index as usize])
}

/// Raven `SE_Init` — register the three cvars and load the current language.
/// If `com_buildScript == 2`, load every language first (build-script path).
/// A load failure raises `Com_Error(ERR_DROP)` (→ `EngineHost::error`).
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1156-1196`
pub fn se_init(pkg: &mut StringEdPackage, host: &mut impl EngineHost) {
    pkg.clear(false);

    host.cvar_register("se_language", "english", CVAR_ARCHIVE | CVAR_NORESTART);
    host.cvar_register("se_debug", "0", 0);
    host.cvar_register("sp_leet", "0", CVAR_ROM);

    // If doing a buildscript, load all languages.
    if host.cvar_integer("com_buildScript") == 2 {
        let languages = se_get_num_languages(pkg, host);
        for lang in 0..languages {
            let language = se_get_language_name(pkg, lang).to_owned();
            host.print(&format!(
                "com_buildScript(2): Loading language \"{language}\"...\n"
            ));
            se_load_language(pkg, &language, true, host);
        }
    }

    let language = host.cvar_string("se_language");
    if let Some(message) = se_load_language(pkg, &language, true, host) {
        host.error(
            errorParm_t::ERR_DROP,
            &format!("SE_Init() Unable to load language: \"{language}\"!\nError: \"{message}\"\n"),
        );
    }
}

/// Raven `SE_ShutDown` — `Clear(SE_FALSE)`.
///
/// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1198-1201`
pub fn se_shut_down(pkg: &mut StringEdPackage) {
    pkg.clear(false);
}

/// Raven `COM_DefaultExtension` — append `extension` unless `path` already has
/// one (a `.` after the last `/`). Ported locally since the shared qshared
/// helper is not yet available; used only by [`se_load`].
///
/// Source: `oracle/codemp/game/q_shared.c` (`COM_DefaultExtension`)
fn default_extension(path: &mut String, extension: &str) {
    let has_extension = match path.rfind('.') {
        Some(dot) => match path.rfind('/') {
            Some(slash) => dot > slash,
            None => true,
        },
        None => false,
    };
    if !has_extension {
        path.push_str(extension);
    }
}
