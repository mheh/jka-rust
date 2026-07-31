//! Server-side `EngineHooks` installation: the boot-time fill of the `SV_*`
//! upcall fields plus the accessor hooks backing qcommon's live `EngineHost`
//! implementation (`engine_host_view.rs`) — the casting adapters live here
//! because only this crate can name the real `Server` (opaque-slot ruling,
//! user 2026-07-12; host-seam restructure, user 2026-07-11).
//!
//! Slot-cast rule (per-slot): a cast reborrows the raw pointer out of the
//! view's type-erased slot (`as_raw()`), so the view itself stays usable —
//! sound as long as no call made while the cast borrow is live casts the SAME
//! slot again. The accessor bodies below either drop the cast before touching
//! the view or make no view call at all.

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::EngineHooks;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_host_interface::VmSlot;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;
use native_string::latin1_to_string;

use crate::server::server_state_t::serverState_t;
use crate::server_host::Server;
use crate::sv_game::SV_GentityNum;
use crate::sv_world::SV_Trace;

/// Cast the view's type-erased `sv` slot back to the live `Server`. The raw
/// pointer is copied out first (`as_raw`), so the returned borrow is NOT tied
/// to the view — the per-slot rule above governs its use.
///
/// SAFETY (caller): the slot was built by `mp_engine_core`'s view constructor
/// from the live, unique `&mut Engine.sv`; the engine is single-threaded and
/// no other cast of this slot is live for the returned borrow's duration.
pub(crate) unsafe fn sv_from_view<'a>(view: &mut EngineHostView) -> &'a mut Server {
    &mut *(view.sv.as_raw() as *mut Server)
}

/// Install the server tier's hook fields: the lifecycle upcalls (including the
/// `SV_Frame`/`SV_PacketEvent` frame path) plus every server-backed
/// `EngineHost` accessor.
pub fn install_engine_hooks(hooks: &mut EngineHooks) {
    hooks.SV_Init = Some(crate::sv_init::SV_Init);
    hooks.SV_Shutdown = Some(crate::sv_init::SV_Shutdown);
    hooks.SV_Frame = Some(crate::sv_main::SV_Frame);
    hooks.SV_PacketEvent = Some(crate::sv_main::SV_PacketEvent);
    hooks.SV_GameCommand = Some(crate::sv_game::SV_GameCommand);
    hooks.SV_ShutdownGameProgs = Some(crate::sv_game::SV_ShutdownGameProgs);
    hooks.SV_Trace = Some(sv_trace_hook);
    hooks.SV_GentityNum = Some(sv_gentity_num_hook);
    hooks.SV_SharedMemory = Some(sv_shared_memory_hook);
    hooks.SVS_Time = Some(svs_time_hook);
    hooks.SV_ShownetEntityClassname = Some(sv_shownet_entity_classname_hook);
    hooks.VM_CallSlot = Some(vm_call_slot_hook);
}

/// `EngineHost::trace` backing — Raven `SV_Trace`.
/// Source: `oracle/codemp/server/sv_world.cpp:803`
#[allow(clippy::too_many_arguments)]
fn sv_trace_hook(
    view: &mut EngineHostView,
    results: &mut trace_t,
    start: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    end: vec3_t,
    pass_entity_num: c_int,
    contentmask: c_int,
    capsule: c_int,
    trace_flags: c_int,
    use_lod: c_int,
) {
    SV_Trace(
        view,
        results as *mut trace_t,
        start,
        mins,
        maxs,
        end,
        pass_entity_num,
        contentmask,
        capsule,
        trace_flags,
        use_lod,
    );
}

/// `EngineHost::gentity` backing — Raven `SV_GentityNum`.
/// Source: `oracle/codemp/server/sv_game.cpp:54`
fn sv_gentity_num_hook(view: &mut EngineHostView, ent_num: c_int) -> *mut sharedEntity_t {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let sv = unsafe { sv_from_view(view) };
    SV_GentityNum(sv, ent_num)
}

/// `EngineHost::shared_memory` backing — Raven `sv.mSharedMemory`.
/// Source: `oracle/codemp/server/server.h:87`
fn sv_shared_memory_hook(view: &mut EngineHostView) -> *mut c_char {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let sv = unsafe { sv_from_view(view) };
    sv.sv.mSharedMemory
}

/// `EngineHost::sv_time` backing — Raven `svs.time`.
/// Source: `oracle/codemp/server/server.h:211`
fn svs_time_hook(view: &mut EngineHostView) -> c_int {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let sv = unsafe { sv_from_view(view) };
    sv.svs.time
}

/// `EngineHost::sv_shownet_entity_classname` backing (ruling 56c) — Raven's
/// `if (sv.state) … SV_GentityNum(number)->classname` probe.
/// Source: `oracle/codemp/qcommon/msg.cpp:1268-1270`
fn sv_shownet_entity_classname_hook(view: &mut EngineHostView, number: c_int) -> Option<String> {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let sv = unsafe { sv_from_view(view) };
    if sv.sv.state == serverState_t::SS_DEAD {
        return None;
    }
    let ent = SV_GentityNum(sv, number);
    // SAFETY: SV_GentityNum indexes the live gentities block; `classname` is
    // the game module's static string or NULL.
    unsafe {
        if ent.is_null() || (*ent).classname.is_null() {
            return None;
        }
        Some(latin1_to_string(
            CStr::from_ptr((*ent).classname).to_bytes(),
        ))
    }
}

/// `EngineHost::vm_call` backing — Raven `VM_Call( vm, … )` with the slot
/// resolution: `Gvm` -> `sv.gvm`; `Cgvm` is NULL under DEDICATED and takes
/// Raven's own NULL-vm fatal path inside `VM_Call` (ruling 33b). The words
/// stay `isize` end-to-end (plan §5.4 widening — pointer-carrying args and
/// returns must survive LP64).
/// Source: `oracle/codemp/qcommon/vm.cpp:787`
fn vm_call_slot_hook(
    view: &mut EngineHostView,
    vm: VmSlot,
    callnum: c_int,
    args: &[isize],
) -> isize {
    let gvm = match vm {
        VmSlot::Gvm => {
            // SAFETY: view-constructor slot, single-threaded; the cast borrow
            // ends before VM_Call takes the view.
            let sv = unsafe { sv_from_view(view) };
            sv.gvm
        }
        // NULL cgvm (DEDICATED): VM_Call's own guard raises Raven's fatal.
        VmSlot::Cgvm => core::ptr::null_mut(),
    };
    VM_Call(view.common, gvm, callnum, args)
}
