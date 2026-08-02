#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_uint};

use mp_qshared::shared::cinematic_status::e_status;
use mp_qshared::shared::limits::MAX_OSPATH;
use native_types::{byte, fileHandle_t, qboolean};

use super::vq_blitter::VqBlitter;

/// Raven `cin_cache` — one cinematic playback slot of `cinTable`. Internal to
/// the client, so it never crosses the ABI seam and carries no layout asserts.
///
/// The four `VQ*` blitter slots are Raven `void (*)(byte *status, void *qdata)`
/// pointers over a closed one-entry set, so they carry [`VqBlitter`] instead.
///
/// Type definition source: `oracle/codemp/client/cl_cin.cpp:78-118`
#[repr(C)]
pub struct cin_cache {
    pub fileName: [c_char; MAX_OSPATH],
    pub CIN_WIDTH: c_int,
    pub CIN_HEIGHT: c_int,
    pub xpos: c_int,
    pub ypos: c_int,
    pub width: c_int,
    pub height: c_int,
    pub looping: qboolean,
    pub holdAtEnd: qboolean,
    pub dirty: qboolean,
    pub alterGameState: qboolean,
    pub silent: qboolean,
    pub shader: qboolean,
    pub iFile: fileHandle_t,
    pub status: e_status,
    pub startTime: c_uint,
    pub lastTime: c_uint,
    pub tfps: c_long,
    pub RoQPlayed: c_long,
    pub ROQSize: c_long,
    pub RoQFrameSize: c_uint,
    pub onQuad: c_long,
    pub numQuads: c_long,
    pub samplesPerLine: c_long,
    pub roq_id: c_uint,
    pub screenDelta: c_long,

    pub VQ0: VqBlitter,
    pub VQ1: VqBlitter,
    pub VQNormal: VqBlitter,
    pub VQBuffer: VqBlitter,

    pub gray: *mut byte,
    pub xsize: c_uint,
    pub ysize: c_uint,
    pub maxsize: c_uint,
    pub minsize: c_uint,

    pub inMemory: qboolean,
    pub normalBuffer0: c_long,
    pub roq_flags: c_long,
    pub roqF0: c_long,
    pub roqF1: c_long,
    pub t: [c_long; 2],
    pub roqFPS: c_long,
    pub playonwalls: c_int,
    pub buf: *mut byte,
    pub drawX: c_long,
    pub drawY: c_long,
}

// Every field is a scalar, a null-valid pointer, or `VqBlitter` (whose zero
// discriminant is `None`), and Raven's `cinTable` is a zero-filled file static,
// so the all-zero image is a valid inhabitant.
unsafe impl native_platform::ZeroValid for cin_cache {}
