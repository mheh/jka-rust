//! Raven's `null` sound-DMA stubs — the DEDICATED/no-sound build's
//! `SNDDMA_*`/`S_*`/`SND_*` entry points, every body an intentional no-op.
//!
//! Source: `oracle/codemp/null/null_snddma.cpp`

use std::os::raw::{c_char, c_int};

use mp_qshared::shared::{qboolean, qfalse, sfxHandle_t};

/// Raven `SNDDMA_Init`.
///
/// Raven: `return qfalse;`
/// Source: `oracle/codemp/null/null_snddma.cpp:9-12`
pub fn SNDDMA_Init() -> qboolean {
    qfalse
}

/// Raven `SNDDMA_GetDMAPos`.
///
/// Raven: `return 0;`
/// Source: `oracle/codemp/null/null_snddma.cpp:14-17`
pub fn SNDDMA_GetDMAPos() -> c_int {
    0
}

/// Raven `SNDDMA_Shutdown`.
///
/// Source: `oracle/codemp/null/null_snddma.cpp:19-21`
pub fn SNDDMA_Shutdown() {}

/// Raven `SNDDMA_BeginPainting`.
///
/// Source: `oracle/codemp/null/null_snddma.cpp:23-25`
pub fn SNDDMA_BeginPainting() {}

/// Raven `SNDDMA_Submit`.
///
/// Source: `oracle/codemp/null/null_snddma.cpp:27-29`
pub fn SNDDMA_Submit() {}

/// Raven `S_RegisterSound`.
///
/// Raven: `return 0;`
/// Source: `oracle/codemp/null/null_snddma.cpp:31-33`
pub fn S_RegisterSound(name: *const c_char) -> sfxHandle_t {
    let _ = name;
    0
}

/// Raven `S_StartLocalSound`.
///
/// Source: `oracle/codemp/null/null_snddma.cpp:35-36`
pub fn S_StartLocalSound(sfxHandle: sfxHandle_t, channelNum: c_int) {
    let _ = (sfxHandle, channelNum);
}

/// Raven `S_ClearSoundBuffer`.
///
/// Source: `oracle/codemp/null/null_snddma.cpp:38-39`
pub fn S_ClearSoundBuffer() {}

/// Raven `SND_RegisterAudio_LevelLoadEnd`.
///
/// Raven: `return qfalse;`
/// Source: `oracle/codemp/null/null_snddma.cpp:41-44`
pub fn SND_RegisterAudio_LevelLoadEnd(something: qboolean) -> qboolean {
    let _ = something;
    qfalse
}

/// Raven `SND_FreeOldestSound`.
///
/// Raven: `return 0;`
/// Source: `oracle/codemp/null/null_snddma.cpp:46-49`
pub fn SND_FreeOldestSound() -> c_int {
    0
}
