//! `UiSaberState` — `ui_saber.c`'s file-scope globals as one `UiWorld`
//! sub-struct.

#![allow(non_snake_case)]

use mp_qshared::shared::qhandle_t;

/// Raven `#define MAX_SABER_DATA_SIZE 0x80000`.
///
/// Source: `oracle/codemp/ui/ui_saber.c:17`
pub const MAX_SABER_DATA_SIZE: usize = 0x80000;

/// The saber-hilt parse buffer and the blade glow/core shader cache
/// (`ui_saber.c` file-scope statics folded onto `UiWorld`, §B3).
///
/// Source: `oracle/codemp/ui/ui_saber.c:22-36`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiSaberState {
    /// Raven `static char SaberParms[MAX_SABER_DATA_SIZE]` — the concatenated
    /// `.sab` text every hilt lookup re-parses.
    /// Source: `oracle/codemp/ui/ui_saber.c:22`
    pub SaberParms: String,
    /// Raven `qboolean ui_saber_parms_parsed`.
    /// Source: `oracle/codemp/ui/ui_saber.c:23`
    pub ui_saber_parms_parsed: bool,

    /// Raven `static qhandle_t redSaberGlowShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:25`
    pub redSaberGlowShader: qhandle_t,
    /// Raven `static qhandle_t redSaberCoreShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:26`
    pub redSaberCoreShader: qhandle_t,
    /// Raven `static qhandle_t orangeSaberGlowShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:27`
    pub orangeSaberGlowShader: qhandle_t,
    /// Raven `static qhandle_t orangeSaberCoreShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:28`
    pub orangeSaberCoreShader: qhandle_t,
    /// Raven `static qhandle_t yellowSaberGlowShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:29`
    pub yellowSaberGlowShader: qhandle_t,
    /// Raven `static qhandle_t yellowSaberCoreShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:30`
    pub yellowSaberCoreShader: qhandle_t,
    /// Raven `static qhandle_t greenSaberGlowShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:31`
    pub greenSaberGlowShader: qhandle_t,
    /// Raven `static qhandle_t greenSaberCoreShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:32`
    pub greenSaberCoreShader: qhandle_t,
    /// Raven `static qhandle_t blueSaberGlowShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:33`
    pub blueSaberGlowShader: qhandle_t,
    /// Raven `static qhandle_t blueSaberCoreShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:34`
    pub blueSaberCoreShader: qhandle_t,
    /// Raven `static qhandle_t purpleSaberGlowShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:35`
    pub purpleSaberGlowShader: qhandle_t,
    /// Raven `static qhandle_t purpleSaberCoreShader`.
    /// Source: `oracle/codemp/ui/ui_saber.c:36`
    pub purpleSaberCoreShader: qhandle_t,
}
