#![allow(non_camel_case_types, non_snake_case)]

/// Raven `directory_t` — a search path directory (base path + game subdirectory).
///
/// Engine-internal only (reached through `searchpath_t.dir`, never crosses
/// the module ABI), so the C layout + asserts are dropped (string-data
/// migration, DEC-32). Raven's `MAX_OSPATH` field size survives as the
/// truncation bound at the `Q_strncpyz` write sites (`cap_ospath` in
/// `files_common`), matching the silent 1023-byte cut.
///
/// Raven: none.
/// Type definition source: `oracle/codemp/qcommon/files.h:58-61`
pub struct directory_t {
    /// c:\jk2
    pub path: String,
    /// base
    pub gamedir: String,
}
