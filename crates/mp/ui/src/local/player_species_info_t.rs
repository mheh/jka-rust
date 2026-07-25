//! `PlayerSpeciesInfo` — Raven `playerSpeciesInfo_t`.

/// Raven `#define MAX_PLAYERMODELS 32`.
///
/// Source: `oracle/codemp/ui/ui_local.h:594`
pub const MAX_PLAYERMODELS: usize = 32;

/// Raven `playerSpeciesInfo_t` — one player species and the head/torso/leg
/// skins and color shaders the player-model menu offers for it.
///
/// PORT-NOTE: Raven's five `[MAX_PLAYERMODELS][N]` char matrices plus their
/// `SkinHeadCount`/`SkinTorsoCount`/`SkinLegCount`/`ColorCount` become owned
/// `Vec<String>`s; each count is the matching `len()` (`ColorCount` covers the
/// parallel `ColorShader`/`ColorActionText` pair).
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:716-727`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(alias = "playerSpeciesInfo_t")]
#[allow(non_snake_case)]
pub struct PlayerSpeciesInfo {
    pub Name: String,
    pub SkinHeadNames: Vec<String>,
    pub SkinTorsoNames: Vec<String>,
    pub SkinLegNames: Vec<String>,
    pub ColorShader: Vec<String>,
    pub ColorActionText: Vec<String>,
}
