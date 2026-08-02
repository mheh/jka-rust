//! Boot installation of the client tier's `EngineHooks` entries.
//!
//! Raven picks the sound tier at link time: the dedicated build links
//! `null_snddma.cpp` and the client build links `snd_dma.cpp`. One binary ships
//! both here, so the installed hook reads the view's `snd` slot and takes the
//! null-build answer when `Engine.snd` is `None`.
//!
//! Source: `oracle/codemp/null/null_snddma.cpp:41-49`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::EngineHooks;
use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::client_host::snd_from_view;
use crate::snd_dma::{SND_FreeOldestSound, SND_RegisterAudio_LevelLoadEnd};

/// Install the client tier's hook fields over the null-build defaults.
/// Runs once in `main()` beside the server and renderer installers.
pub fn install_engine_hooks(hooks: &mut EngineHooks) {
    hooks.SND_FreeOldestSound = Some(SND_FreeOldestSound_hook);
    hooks.SND_RegisterAudio_LevelLoadEnd = Some(SND_RegisterAudio_LevelLoadEnd_hook);
}

/// Raven `SND_FreeOldestSound(void)`, which the zone allocator calls to recover
/// from a failed `Z_Malloc`.
/// Source: `oracle/codemp/client/snd_dma.cpp:5216-5219`
fn SND_FreeOldestSound_hook(view: &mut EngineHostView) -> c_int {
    if view.snd.as_raw().is_null() {
        return 0;
    }
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    SND_FreeOldestSound(view, snd, None)
}

/// Raven `SND_RegisterAudio_LevelLoadEnd`, which the renderer and the zone
/// allocator call to bring the audio pool back under its cap.
/// Source: `oracle/codemp/client/snd_dma.cpp:5228`
fn SND_RegisterAudio_LevelLoadEnd_hook(
    view: &mut EngineHostView,
    bDeleteEverythingNotUsedThisLevel: qboolean,
) -> qboolean {
    if view.snd.as_raw().is_null() {
        return qfalse;
    }
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    let dropped = SND_RegisterAudio_LevelLoadEnd(view, snd, bDeleteEverythingNotUsedThisLevel != 0);
    if dropped {
        qtrue
    } else {
        qfalse
    }
}
