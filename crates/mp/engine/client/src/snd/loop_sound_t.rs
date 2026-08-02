//! Raven `loopSound_t` — one entry of the per-frame looping-sound list.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

/// Raven `MAX_LOOP_SOUNDS` — the looping-sound list length.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:184`
pub const MAX_LOOP_SOUNDS: usize = 32;

/// Raven `loopSound_t` — a looping sound the game re-adds every frame.
///
/// `S_Respatialize` merges the list into channels and `S_ClearLoopingSounds`
/// empties it, so an entry never outlives one frame.
/// The OpenAL `bProcessed`/`bRelative` pair is dropped with the OpenAL arm (DEC-57.4).
/// Type definition source: `oracle/codemp/client/snd_dma.cpp:170-182`
#[derive(Default, Clone, Copy)]
pub struct loopSound_t {
    pub volume: u8,
    pub origin: vec3_t,
    pub velocity: vec3_t,
    /// The `s_knownSfx` slot. `None` is Raven's NULL `sfx`.
    pub sfx: Option<usize>,
    pub mergeFrame: c_int,
    pub entnum: c_int,
}
