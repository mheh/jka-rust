#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use sp_qshared::common::sp::renderer::glconfig_t::glconfig_t;
use sp_qshared::shared::{gameState_t, qhandle_t, sfxHandle_t, vec3_t, MAX_CLIENTS, MAX_QPATH};

use super::cg_effects_t::cgEffects_t;
use super::cg_media_t::cgMedia_t;

/// Raven `STRIPED_LEVELNAME_VARIATIONS` — to cope with levels that use text
/// from more than one SP file (plus 1 for common).
///
/// Source: `oracle/oracle/code/cgame/cg_media.h:369`
pub const STRIPED_LEVELNAME_VARIATIONS: usize = 3;

/// Raven `MAX_MODELS` (SP).
///
/// Source: `oracle/oracle/code/game/q_shared.h:1461`
pub const MAX_MODELS: usize = 256;

/// Raven `MAX_SOUNDS` (SP).
///
/// Source: `oracle/oracle/code/game/q_shared.h:1462`
pub const MAX_SOUNDS: usize = 380;

/// Raven `MAX_FORCES`.
///
/// Source: `oracle/oracle/code/game/q_shared.h:1467`
pub const MAX_FORCES: usize = 96;

/// Raven `MAX_CHARSKINS` — character skins.
///
/// Source: `oracle/oracle/code/game/q_shared.h:1477`
pub const MAX_CHARSKINS: usize = 64;

/// Raven `MAX_SUBMODELS` — nine bits.
///
/// Source: `oracle/oracle/code/game/q_shared.h:1469`
pub const MAX_SUBMODELS: usize = 512;

/// Raven `clientInfo_t` — per-client rendering info shared between game and
/// cgame; already ported at `sp_game::shared::client_info_t::clientInfo_t`,
/// but `sp_cgame` has no dependency on `sp_game`, so it stays opaque here.
/// `[u64; 62]` preserves the real type's exact size (496 B) and its 8-byte
/// pointer alignment.
//TODO: Port clientInfo_t (cross-crate, sp_game -> sp_cgame not wired)
// Source: oracle/oracle/code/game/g_shared.h:76-103
type OpaqueClientInfo_t = [u64; 62];

/// Raven `cgs_t` — the client game static structure, holding everything
/// loaded or calculated from the gamestate.
///
/// Raven: The client game static (cgs) structure hold everything loaded or
/// calculated from the gamestate. It will NOT be cleared when a tournement
/// restart is done, allowing all clients to begin playing instantly.
/// Type definition source: `oracle/oracle/code/cgame/cg_media.h:370-409`
#[repr(C)]
pub struct cgs_t {
    /// gamestate from server
    pub gameState: gameState_t,
    /// rendering configuration
    pub glconfig: glconfig_t,

    /// reliable command stream counter
    pub serverCommandSequence: c_int,

    // parsed from serverinfo
    pub dmflags: c_int,
    pub teamflags: c_int,
    pub timelimit: c_int,
    pub maxclients: c_int,
    pub mapname: [c_char; MAX_QPATH],
    pub stripLevelName: [[c_char; MAX_QPATH]; STRIPED_LEVELNAME_VARIATIONS],

    //
    // locally derived information from gamestate
    //
    pub model_draw: [qhandle_t; MAX_MODELS],
    pub sound_precache: [sfxHandle_t; MAX_SOUNDS],

    // Raven: `#ifdef _IMMERSION` — force-feedback registration; layout
    // reflects the `_IMMERSION`-enabled build the packet's offsets were
    // captured against.
    //TODO: Port ffHandle_t
    // Source: oracle/oracle/code/ff/ff_public.h:8
    pub force_precache: [c_int; MAX_FORCES],

    // Ghoul2 start
    pub skins: [qhandle_t; MAX_CHARSKINS],
    // Ghoul2 end
    pub numInlineModels: c_int,
    pub inlineDrawModel: [qhandle_t; MAX_SUBMODELS],
    pub inlineModelMidpoints: [vec3_t; MAX_SUBMODELS],

    pub clientinfo: [OpaqueClientInfo_t; MAX_CLIENTS],

    /// media
    pub media: cgMedia_t,

    /// effects
    pub effects: cgEffects_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<cgs_t>() == 35232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, gameState) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, glconfig) == 21208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, serverCommandSequence) == 21304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, dmflags) == 21308);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, teamflags) == 21312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, timelimit) == 21316);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, maxclients) == 21320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, mapname) == 21324);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, stripLevelName) == 21388);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, model_draw) == 21580);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, sound_precache) == 22604);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, force_precache) == 24124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, skins) == 24508);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, numInlineModels) == 24764);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, inlineDrawModel) == 24768);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, inlineModelMidpoints) == 26816);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, clientinfo) == 32960);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, media) == 33456);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, effects) == 35096);
