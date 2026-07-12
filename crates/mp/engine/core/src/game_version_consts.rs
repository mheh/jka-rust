//! `game_version.h` version-banner constant.
//!
//! Source: `oracle/codemp/qcommon/game_version.h:6-11`

/// Raven `Q3_VERSION` — version banner printed by `Com_Init`
/// (`common.cpp:1219`, see [`crate::lifecycle::com_init`]'s inline TODO).
/// Picks the `FINAL_BUILD` branch: retail binaries define `FINAL_BUILD`, not
/// `_DEBUG`. `VERSION_STRING_DOTTED` is `"1.0.1.0"`
/// (`oracle/codemp/win32/AutoVersion.h:10`).
///
/// Source: `oracle/codemp/qcommon/game_version.h:6-9`
pub const Q3_VERSION: &str = "JAmp: v1.0.1.0";
