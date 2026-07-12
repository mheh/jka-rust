//! MP ICARUS `Q3_Interface.cpp` — the outbound `I_*` implementations, the
//! `Interface_Init` wiring, and the `G_ICARUS_*` task-id/set-var seam callees
//! (§F idiomatic reimplementation).
//!
//! Each `Q3_*`/`CGCam_*` interface target writes a `T_G_ICARUS_*` struct into
//! `host.shared_memory()` then `host.vm_call(VmSlot::Gvm, GAME_ICARUS_*, &[])`
//! (both via `EngineHost`, ICARUS-D3 / ruling 24). `Q3_DebugPrint` gates all
//! output on `host.cvar_integer("developer") != 0` (ruling 36). The task-id
//! helpers take the **bare real-entity** `ent` the seam carries, so their
//! `ent->taskID[]` writes persist (ruling 37). `tagsTable[]`
//! (`Q3_Interface.cpp:22`) is commented-out dead surface and ports nothing
//! (Divergences).

#![allow(non_snake_case)]

use core::ffi::{c_char, CStr};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use mp_host_interface::vm_slot::VmSlot;
use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::game_export_t::gameExport_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::qcommon::t_g_icarus_getfloat::T_G_ICARUS_GETFLOAT;
use mp_qshared::common::mp::qcommon::t_g_icarus_getstring::T_G_ICARUS_GETSTRING;
use mp_qshared::common::mp::qcommon::t_g_icarus_gettag::T_G_ICARUS_GETTAG;
use mp_qshared::common::mp::qcommon::t_g_icarus_getvector::T_G_ICARUS_GETVECTOR;
use mp_qshared::common::mp::qcommon::t_g_icarus_kill::T_G_ICARUS_KILL;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_angles::T_G_ICARUS_LERP2ANGLES;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_end::T_G_ICARUS_LERP2END;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_origin::T_G_ICARUS_LERP2ORIGIN;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_pos::T_G_ICARUS_LERP2POS;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_start::T_G_ICARUS_LERP2START;
use mp_qshared::common::mp::qcommon::t_g_icarus_play::T_G_ICARUS_PLAY;
use mp_qshared::common::mp::qcommon::t_g_icarus_playsound::T_G_ICARUS_PLAYSOUND;
use mp_qshared::common::mp::qcommon::t_g_icarus_remove::T_G_ICARUS_REMOVE;
use mp_qshared::common::mp::qcommon::t_g_icarus_set::T_G_ICARUS_SET;
use mp_qshared::common::mp::qcommon::t_g_icarus_use::T_G_ICARUS_USE;
use mp_qshared::common::mp::qcommon::task_id_t::taskID_t;
use mp_qshared::shared::limits::MAX_GENTITIES;
use mp_qshared::shared::vec3_t;
use mp_qshared::shared::wl_e::WL_e;

use crate::game_interface::{icarus_get_script, ICARUS_LinkEntity};
use crate::interface::interface_export_s::InterfaceExport;
use crate::q3_registers::{
    q3_declare_variable, q3_free_variable, q3_get_float_variable, q3_variable_declared,
    Q3_SetFloatVariable, Q3_SetStringVariable, Q3_SetVectorVariable, VTYPE_FLOAT, VTYPE_NONE,
    VTYPE_STRING, VTYPE_VECTOR,
};
use crate::taskmanager::ctask_manager::TaskManager;
use crate::Icarus;

pub mod play_type_t;
pub mod set_type_t;

// ---------------------------------------------------------------------------
// Local constants the oracle pulls from headers not (yet) ported into this
// crate. Cited to their Raven definition; internal-only, so no ABI concern.
// ---------------------------------------------------------------------------

/// Raven `#define Q3_SCRIPT_DIR "scripts"` — the script directory prefix.
/// Source: `oracle/codemp/game/q_shared.h:10`
const Q3_SCRIPT_DIR: &str = "scripts";

/// Raven console color escapes. Source: `oracle/codemp/game/q_shared.h:1161-1164`
const S_COLOR_RED: &str = "^1";
const S_COLOR_GREEN: &str = "^2";
const S_COLOR_YELLOW: &str = "^3";
const S_COLOR_BLUE: &str = "^4";

/// Raven `WL_e` print levels as `i32` (mirrors `mp_qshared` `WL_e`); the many
/// `Q3_DebugPrint(level, …)` call sites take an `int level`, and the switch
/// matches these.
/// Source: `oracle/codemp/game/q_shared.h:428-433`
const WL_ERROR: i32 = WL_e::WL_ERROR as i32;
const WL_WARNING: i32 = WL_e::WL_WARNING as i32;
const WL_VERBOSE: i32 = WL_e::WL_VERBOSE as i32;
const WL_DEBUG: i32 = WL_e::WL_DEBUG as i32;

// Token-type ids `Q3_Evaluate` compares against (from the out-of-set tokenizer/
// interpreter — untouched skeletons per Scope, so the needed ids are declared
// here, the sole in-scope consumer).
// Source: `oracle/codemp/icarus/tokenizer.h:63-75`,
// `oracle/codemp/icarus/interpreter.h:14-30`
const TK_STRING: i32 = 4;
const TK_INT: i32 = 5;
const TK_FLOAT: i32 = 6;
const TK_IDENTIFIER: i32 = 7;
const TK_VECTOR: i32 = 14;
const TK_GREATER_THAN: i32 = 15;
const TK_LESS_THAN: i32 = 16;
const TK_EQUALS: i32 = 17;
const TK_NOT: i32 = 18;

// ---------------------------------------------------------------------------
// Small C-runtime shims (transcription helpers), all pure / no host.
// ---------------------------------------------------------------------------

/// `strcpy` into a fixed C-char field, but **bounded** to `cap` (NUL-reserved)
/// to avoid the buffer-overrun UB Raven's unbounded `strcpy` risks (§19); the
/// script strings are far shorter than the 2048-byte shared-memory windows.
unsafe fn strcpy_bounded(dst: *mut u8, cap: usize, src: &str) {
    let bytes = src.as_bytes();
    let n = bytes.len().min(cap.saturating_sub(1));
    core::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, n);
    *dst.add(n) = 0;
}

/// Read a NUL-terminated C string out of a fixed byte field.
unsafe fn read_c_field(ptr: *const u8, cap: usize) -> String {
    let mut len = 0;
    while len < cap && *ptr.add(len) != 0 {
        len += 1;
    }
    let slice = core::slice::from_raw_parts(ptr, len);
    String::from_utf8_lossy(slice).into_owned()
}

/// C `atof` — parse the leading floating-point prefix, `0.0` on no match.
fn c_atof(s: &str) -> f32 {
    let b = s.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n && b[i].is_ascii_whitespace() {
        i += 1;
    }
    let start = i;
    if i < n && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let mut has_digits = false;
    while i < n && b[i].is_ascii_digit() {
        i += 1;
        has_digits = true;
    }
    if i < n && b[i] == b'.' {
        i += 1;
        while i < n && b[i].is_ascii_digit() {
            i += 1;
            has_digits = true;
        }
    }
    if has_digits && i < n && (b[i] == b'e' || b[i] == b'E') {
        let save = i;
        i += 1;
        if i < n && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let mut exp_digits = false;
        while i < n && b[i].is_ascii_digit() {
            i += 1;
            exp_digits = true;
        }
        if !exp_digits {
            i = save;
        }
    }
    if !has_digits {
        return 0.0;
    }
    s[start..i].parse::<f32>().unwrap_or(0.0)
}

/// C `atoi`/`sscanf("%d")` — parse the leading integer prefix, `0` on no match.
fn c_atoi(s: &str) -> i32 {
    let b = s.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n && b[i].is_ascii_whitespace() {
        i += 1;
    }
    let start = i;
    if i < n && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let mut has_digits = false;
    while i < n && b[i].is_ascii_digit() {
        i += 1;
        has_digits = true;
    }
    if !has_digits {
        return 0;
    }
    s[start..i].parse::<i32>().unwrap_or(0)
}

/// C `stricmp` — case-insensitive byte compare returning sign of the first
/// differing (lowercased) byte, `0` when equal (missing bytes read as NUL).
fn stricmp(a: &str, b: &str) -> i32 {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let n = ab.len().max(bb.len());
    for i in 0..n {
        let x = ab.get(i).map(|c| c.to_ascii_lowercase()).unwrap_or(0);
        let y = bb.get(i).map(|c| c.to_ascii_lowercase()).unwrap_or(0);
        if x != y {
            return x as i32 - y as i32;
        }
    }
    0
}

/// Raven `VectorCompare` — exact (bit-for-bit) equality of all three axes.
/// Source: `oracle/codemp/game/q_shared.h`
fn vector_compare(a: &vec3_t, b: &vec3_t) -> bool {
    a[0] == b[0] && a[1] == b[1] && a[2] == b[2]
}

/// `sscanf(s, "%f %f %f", …)` — whitespace-split, leading-float per token,
/// unmatched axes stay `0.0`.
fn sscanf_vec(s: &str) -> vec3_t {
    let mut v = [0.0f32; 3];
    for (i, tok) in s.split_whitespace().take(3).enumerate() {
        v[i] = c_atof(tok);
    }
    v
}

/// Inlines Raven `CTaskManager::Completed(int id)` (`TaskManager.cpp:912-925`):
/// mark the task complete in the first task group that owns it (a pure
/// `m_taskGroups` walk + `MarkTaskComplete`, no host dispatch). Inlined here —
/// over a `tm.completed(id)` call — because the `TaskManager::completed` method
/// is **absent from the current ctask_manager skeleton** (reported); the two
/// pub building blocks it needs (`m_task_groups`, `TaskGroup::mark_task_complete`)
/// do exist, and the behavior is identical.
fn task_manager_completed(tm: &mut TaskManager, id: i32) {
    // Mark the task as completed in the first group that owns it.
    for tg in tm.m_task_groups.iter_mut() {
        if tg.mark_task_complete(id) {
            break;
        }
    }
}

/// Read a `taskID_t`'s `#[repr(i32)]` discriminant **without moving** it (the
/// enum is not `Copy`) so a helper can both index `ent->taskID[]` and forward
/// the value to a sibling task-id call, as Raven reuses `taskType`.
fn task_index(task_type: &taskID_t) -> i32 {
    // `taskID_t` is `#[repr(i32)]`; its discriminant is the leading `i32`.
    unsafe { *(task_type as *const taskID_t as *const i32) }
}

// ===========================================================================
// Interface_Init — populate the outbound I_* table (Q3_Interface.cpp:956-1008).
// ===========================================================================

/// Raven `Interface_Init` — the live table wiring (`Q3_Interface.cpp:956`;
/// `Interface.cpp`'s copy is commented-out dead surface, Divergences).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:956`
pub fn Interface_Init(pe: &mut InterfaceExport) {
    // General
    pe.i_load_file = Q3_ReadScript;
    pe.i_center_print = Q3_CenterPrint;
    pe.i_dprintf = Q3_DebugPrint;
    pe.i_get_entity_by_name = Q3_GetEntityByName;
    pe.i_get_time = Q3_GetTime;
    pe.i_get_time_scale = Q3_GetTimeScale;
    pe.i_play_sound = Q3_PlaySound;
    pe.i_lerp2_pos = Q3_Lerp2Pos;
    pe.i_lerp2_origin = Q3_Lerp2Origin;
    pe.i_lerp2_angles = Q3_Lerp2Angles;
    pe.i_get_tag = Q3_GetTag;
    pe.i_lerp2_start = Q3_Lerp2Start;
    pe.i_lerp2_end = Q3_Lerp2End;
    pe.i_use = Q3_Use;
    pe.i_kill = Q3_Kill;
    pe.i_remove = Q3_Remove;
    pe.i_set = Q3_Set;
    pe.i_random = Q_flrand;
    pe.i_play = Q3_Play;

    // Camera functions
    pe.i_camera_enable = CGCam_Enable;
    pe.i_camera_disable = CGCam_Disable;
    pe.i_camera_zoom = CGCam_Zoom;
    pe.i_camera_move = CGCam_Move;
    pe.i_camera_pan = CGCam_Pan;
    pe.i_camera_roll = CGCam_Roll;
    pe.i_camera_track = CGCam_Track;
    pe.i_camera_follow = CGCam_Follow;
    pe.i_camera_distance = CGCam_Distance;
    pe.i_camera_shake = CGCam_Shake;
    pe.i_camera_fade = Q3_CameraFade;
    pe.i_camera_path = Q3_CameraPath;

    // Variable information
    pe.i_get_float = Q3_GetFloat;
    pe.i_get_vector = Q3_GetVector;
    pe.i_get_string = Q3_GetString;

    pe.i_evaluate = Q3_Evaluate;

    pe.i_declare_variable = q3_declare_variable;
    pe.i_free_variable = q3_free_variable;

    // Save / Load functions
    pe.i_write_save_data = AppendToSaveGame;
    pe.i_read_save_data = ReadFromSaveGame;
    pe.i_link_entity = ICARUS_LinkEntity;
}

// ===========================================================================
// Outbound I_* implementations (interface_export targets).
// ===========================================================================

/// Raven `Q3_ReadScript` — the `I_LoadFile` target. Reads a (hopefully cached)
/// script under `Q3_SCRIPT_DIR`. Raven's `void **buf` out-param folds to the
/// return: the sibling `icarus_get_script` returns success/failure (its bool
/// signature drops the buffer pointer), so on success the bytes are re-fetched
/// from the buffer list.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:45-48`
pub fn Q3_ReadScript(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    name: &str,
) -> Option<Vec<u8>> {
    let path = format!("{}/{}", Q3_SCRIPT_DIR, name);
    if icarus_get_script(icarus, host, &path) {
        icarus.buffer_list.get(&path).map(|p| p.buffer.clone())
    } else {
        None
    }
}

/// Raven `Q3_CenterPrint` — the `I_CenterPrint` target. Raven's `'@'`/`'!'`
/// key branch issues `SV_SendServerCommand( NULL, "cp \"%s\"", … )`, which has
/// **no** `EngineHost` binding at this seam (reported), so the `cp` dispatch is
/// dropped here; the developer note still prints.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:59-86`
pub fn Q3_CenterPrint(icarus: &mut Icarus, host: &mut dyn EngineHost, msg: &str) {
    // §seam-gap: SV_SendServerCommand is not on EngineHost — the '@'/'!' key
    // dispatch cannot be issued here; only the WL_VERBOSE developer note runs.
    Q3_DebugPrint(icarus, host, WL_VERBOSE, &format!("{}\n", msg));
}

/// Raven `Q3_DebugPrint` — the `I_DPrintf` target; gates on
/// `host.cvar_integer("developer") != 0` (ruling 36) and, in the `WL_DEBUG`
/// branch, reaches `host.gentity(entNum)` for the log line (`:679`).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:638-687`
pub fn Q3_DebugPrint(icarus: &mut Icarus, host: &mut dyn EngineHost, level: i32, msg: &str) {
    // Don't print messages they don't want to see (ruling 36).
    if host.cvar_integer("developer") == 0 {
        return;
    }

    // Raven's varargs are already formatted into `msg` at the call site (§C).
    match level {
        WL_ERROR => host.print(&format!("{}ERROR: {}", S_COLOR_RED, msg)),
        WL_WARNING => host.print(&format!("{}WARNING: {}", S_COLOR_YELLOW, msg)),
        WL_DEBUG => {
            // §19: Raven leaves `entNum` uninitialized on a failed `sscanf`;
            // seed 0 (the range check below folds most garbage to 0 anyway).
            let mut ent_num = c_atoi(msg);

            if icarus.ent_filter >= 0 && icarus.ent_filter != ent_num {
                return;
            }

            // buffer += 5 — §19: guard strings shorter than the skipped prefix.
            let buffer = msg.get(5..).unwrap_or("");

            if ent_num < 0 || ent_num > MAX_GENTITIES as i32 {
                ent_num = 0;
            }

            let ent = host.gentity(ent_num);
            // §19: a NULL entity / NULL script_targetname prints empty rather
            // than dereferencing (Raven's `%s` of NULL is platform UB).
            let stn = if ent.is_null() {
                String::new()
            } else {
                let p = unsafe { (*ent).script_targetname };
                if p.is_null() {
                    String::new()
                } else {
                    unsafe { CStr::from_ptr(p as *const c_char) }
                        .to_string_lossy()
                        .into_owned()
                }
            };

            host.print(&format!(
                "{}DEBUG: {}({}): {}\n",
                S_COLOR_BLUE, stn, ent_num, buffer
            ));
        }
        // default / WL_VERBOSE
        _ => host.print(&format!("{}INFO: {}", S_COLOR_GREEN, msg)),
    }
}

/// Raven `Q3_GetEntityByName` — the `I_GetEntityByName` target; resolves a
/// script name to a real entity via `host.gentity` (`:238`, ruling 37).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:221-241`
pub fn Q3_GetEntityByName(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    name: &str,
) -> *mut sharedEntity_t {
    if name.is_empty() {
        return null_mut();
    }

    // Q_strupr — the ent list is keyed by the upper-cased name (ASCII).
    let key = name.to_ascii_uppercase();

    match icarus.ent_list.get(&key) {
        Some(&entnum) => host.gentity(entnum),
        None => null_mut(),
    }
}

/// Raven `Q3_GetTime` — the `I_GetTime` target (`svs.time`).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:254-257`
pub fn Q3_GetTime(_icarus: &mut Icarus, host: &mut dyn EngineHost) -> u32 {
    host.sv_time() as u32
}

/// Raven `Q3_GetTimeScale` — the `I_GetTimeScale` target
/// (`(DWORD)com_timescale->value`, read as the `timescale` cvar).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:762-765`
pub fn Q3_GetTimeScale(_icarus: &mut Icarus, host: &mut dyn EngineHost) -> u32 {
    host.cvar_integer("timescale") as u32
}

/// Raven `Q3_PlaySound` — the `I_PlaySound` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:313-323`
pub fn Q3_PlaySound(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    task_id: i32,
    ent_id: i32,
    name: &str,
    channel: &str,
) -> i32 {
    let sm = host.shared_memory() as *mut T_G_ICARUS_PLAYSOUND;
    unsafe {
        (*sm).taskID = task_id;
        (*sm).entID = ent_id;
        strcpy_bounded(addr_of_mut!((*sm).name) as *mut u8, 2048, name);
        strcpy_bounded(addr_of_mut!((*sm).channel) as *mut u8, 2048, channel);
    }
    host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_PLAYSOUND as i32, &[]) as i32
}

/// Raven `Q3_Lerp2Pos` — the `I_Lerp2Pos` target. Raven copies the
/// (possibly game-modified) `origin`/`angles` back out; the frozen signature
/// passes them by value, so that copy-back is a no-op here (Divergences).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:767-795`
pub fn Q3_Lerp2Pos(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    task_id: i32,
    ent_id: i32,
    origin: vec3_t,
    angles: vec3_t,
    duration: f32,
) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_LERP2POS;
    unsafe {
        (*sm).taskID = task_id;
        (*sm).entID = ent_id;
        (*sm).origin = origin;
        // Raven distinguishes a NULL `angles` pointer; the frozen signature is
        // always-present, so `nullAngles` is always false here.
        (*sm).angles = angles;
        (*sm).nullAngles = 0; // qfalse
        (*sm).duration = duration;
    }
    host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_LERP2POS as i32, &[]);
    // Copy-back to `origin`/`angles` dropped — by-value params (Divergences).
}

/// Raven `Q3_Lerp2Origin` — the `I_Lerp2Origin` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:797-808`
pub fn Q3_Lerp2Origin(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    task_id: i32,
    ent_id: i32,
    origin: vec3_t,
    duration: f32,
) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_LERP2ORIGIN;
    unsafe {
        (*sm).taskID = task_id;
        (*sm).entID = ent_id;
        (*sm).origin = origin;
        (*sm).duration = duration;
    }
    host.vm_call(
        VmSlot::Gvm,
        gameExport_t::GAME_ICARUS_LERP2ORIGIN as i32,
        &[],
    );
    // Copy-back to `origin` dropped — by-value param (Divergences).
}

/// Raven `Q3_Lerp2Angles` — the `I_Lerp2Angles` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:810-821`
pub fn Q3_Lerp2Angles(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    task_id: i32,
    ent_id: i32,
    angles: vec3_t,
    duration: f32,
) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_LERP2ANGLES;
    unsafe {
        (*sm).taskID = task_id;
        (*sm).entID = ent_id;
        (*sm).angles = angles;
        (*sm).duration = duration;
    }
    host.vm_call(
        VmSlot::Gvm,
        gameExport_t::GAME_ICARUS_LERP2ANGLES as i32,
        &[],
    );
    // Copy-back to `angles` dropped — by-value param (Divergences).
}

/// Raven `Q3_GetTag` — the `I_GetTag` target (out `info` folded to `&mut`).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:823-836`
pub fn Q3_GetTag(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent_id: i32,
    name: &str,
    lookup: i32,
    info: &mut vec3_t,
) -> i32 {
    let sm = host.shared_memory() as *mut T_G_ICARUS_GETTAG;
    unsafe {
        (*sm).entID = ent_id;
        strcpy_bounded(addr_of_mut!((*sm).name) as *mut u8, 2048, name);
        (*sm).lookup = lookup;
        (*sm).info = *info;
    }
    let r = host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_GETTAG as i32, &[]) as i32;
    unsafe {
        *info = (*sm).info;
    }
    r
}

/// Raven `Q3_Lerp2Start` — the `I_Lerp2Start` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:838-847`
pub fn Q3_Lerp2Start(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    task_id: i32,
    ent_id: i32,
    duration: f32,
) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_LERP2START;
    unsafe {
        (*sm).taskID = task_id;
        (*sm).entID = ent_id;
        (*sm).duration = duration;
    }
    host.vm_call(
        VmSlot::Gvm,
        gameExport_t::GAME_ICARUS_LERP2START as i32,
        &[],
    );
}

/// Raven `Q3_Lerp2End` — the `I_Lerp2End` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:849-858`
pub fn Q3_Lerp2End(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    task_id: i32,
    ent_id: i32,
    duration: f32,
) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_LERP2END;
    unsafe {
        (*sm).taskID = task_id;
        (*sm).entID = ent_id;
        (*sm).duration = duration;
    }
    host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_LERP2END as i32, &[]);
}

/// Raven `Q3_Use` — the `I_Use` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:860-868`
pub fn Q3_Use(_icarus: &mut Icarus, host: &mut dyn EngineHost, ent_id: i32, name: &str) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_USE;
    unsafe {
        (*sm).entID = ent_id;
        strcpy_bounded(addr_of_mut!((*sm).target) as *mut u8, 2048, name);
    }
    host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_USE as i32, &[]);
}

/// Raven `Q3_Kill` — the `I_Kill` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:870-878`
pub fn Q3_Kill(_icarus: &mut Icarus, host: &mut dyn EngineHost, ent_id: i32, name: &str) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_KILL;
    unsafe {
        (*sm).entID = ent_id;
        strcpy_bounded(addr_of_mut!((*sm).name) as *mut u8, 2048, name);
    }
    host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_KILL as i32, &[]);
}

/// Raven `Q3_Remove` — the `I_Remove` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:880-888`
pub fn Q3_Remove(_icarus: &mut Icarus, host: &mut dyn EngineHost, ent_id: i32, name: &str) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_REMOVE;
    unsafe {
        (*sm).entID = ent_id;
        strcpy_bounded(addr_of_mut!((*sm).name) as *mut u8, 2048, name);
    }
    host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_REMOVE as i32, &[]);
}

/// Raven `Q3_Set` — the `I_Set` target. On a successful game-side set it
/// completes the task on the owning task manager.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:388-401`
pub fn Q3_Set(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    task_id: i32,
    ent_id: i32,
    type_name: &str,
    data: &str,
) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_SET;
    unsafe {
        (*sm).taskID = task_id;
        (*sm).entID = ent_id;
        strcpy_bounded(addr_of_mut!((*sm).type_name) as *mut u8, 2048, type_name);
        strcpy_bounded(addr_of_mut!((*sm).data) as *mut u8, 2048, data);
    }

    if host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_SET as i32, &[]) != 0 {
        // §19: Raven indexes gTaskManagers[entID] unchecked; guard OOB/absent.
        if ent_id >= 0 && (ent_id as usize) < MAX_GENTITIES {
            if let Some(tm) = icarus.task_managers[ent_id as usize].as_mut() {
                task_manager_completed(tm, task_id);
            }
        }
    }
}

/// Raven `Q_flrand` — the `I_Random` target (wraps `host.flrand`).
/// Source: `oracle/codemp/game/q_math.c:1451` (wired `Q3_Interface.cpp:978`)
pub fn Q_flrand(_icarus: &mut Icarus, host: &mut dyn EngineHost, min: f32, max: f32) -> f32 {
    host.flrand(min, max)
}

/// Raven `Q3_Play` — the `I_Play` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:890-900`
pub fn Q3_Play(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    task_id: i32,
    ent_id: i32,
    type_: &str,
    name: &str,
) {
    let sm = host.shared_memory() as *mut T_G_ICARUS_PLAY;
    unsafe {
        (*sm).taskID = task_id;
        (*sm).entID = ent_id;
        strcpy_bounded(addr_of_mut!((*sm).r#type) as *mut u8, 2048, type_);
        strcpy_bounded(addr_of_mut!((*sm).name) as *mut u8, 2048, name);
    }
    host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_PLAY as i32, &[]);
}

// --- Camera targets — all unsupported in MP (CGCam_Anything). ---------------

/// Raven `CGCam_Anything` — the shared "NOT SUPPORTED IN MP" warning.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:689-692`
fn CGCam_Anything(icarus: &mut Icarus, host: &mut dyn EngineHost) {
    Q3_DebugPrint(
        icarus,
        host,
        WL_WARNING,
        "Camera functions NOT SUPPORTED IN MP\n",
    );
}

/// Raven `CGCam_Enable` — the `I_CameraEnable` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:706-709`
pub fn CGCam_Enable(icarus: &mut Icarus, host: &mut dyn EngineHost) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Disable` — the `I_CameraDisable` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:711-714`
pub fn CGCam_Disable(icarus: &mut Icarus, host: &mut dyn EngineHost) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Zoom` — the `I_CameraZoom` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:716-719`
pub fn CGCam_Zoom(icarus: &mut Icarus, host: &mut dyn EngineHost, fov: f32, duration: f32) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Move` — the `I_CameraMove` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:726-729`
pub fn CGCam_Move(icarus: &mut Icarus, host: &mut dyn EngineHost, origin: vec3_t, duration: f32) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Pan` — the `I_CameraPan` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:721-724`
pub fn CGCam_Pan(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    angles: vec3_t,
    dir: vec3_t,
    duration: f32,
) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Roll` — the `I_CameraRoll` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:755-758`
pub fn CGCam_Roll(icarus: &mut Icarus, host: &mut dyn EngineHost, angle: f32, duration: f32) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Track` — the `I_CameraTrack` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:745-748`
pub fn CGCam_Track(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    name: &str,
    speed: f32,
    init_lerp: f32,
) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Follow` — the `I_CameraFollow` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:740-743`
pub fn CGCam_Follow(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    name: &str,
    speed: f32,
    init_lerp: f32,
) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Distance` — the `I_CameraDistance` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:750-753`
pub fn CGCam_Distance(icarus: &mut Icarus, host: &mut dyn EngineHost, dist: f32, init_lerp: f32) {
    CGCam_Anything(icarus, host);
}

/// Raven `CGCam_Shake` — the `I_CameraShake` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:734-737`
pub fn CGCam_Shake(icarus: &mut Icarus, host: &mut dyn EngineHost, intensity: f32, duration: i32) {
    CGCam_Anything(icarus, host);
}

/// Raven `Q3_CameraFade` — the `I_CameraFade` target (unsupported in MP).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:618-621`
#[allow(clippy::too_many_arguments)]
pub fn Q3_CameraFade(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    sr: f32,
    sg: f32,
    sb: f32,
    sa: f32,
    dr: f32,
    dg: f32,
    db: f32,
    da: f32,
    duration: f32,
) {
    Q3_DebugPrint(
        icarus,
        host,
        WL_WARNING,
        "Q3_CameraFade: NOT SUPPORTED IN MP\n",
    );
}

/// Raven `Q3_CameraPath` — the `I_CameraPath` target (unsupported in MP).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:628-631`
pub fn Q3_CameraPath(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) {
    Q3_DebugPrint(
        icarus,
        host,
        WL_WARNING,
        "Q3_CameraPath: NOT SUPPORTED IN MP\n",
    );
}

/// Raven `Q3_GetFloat` — the `I_GetFloat` target (out `value` folded to `&mut`).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:902-915`
pub fn Q3_GetFloat(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent_id: i32,
    var_type: i32,
    name: &str,
    value: &mut f32,
) -> i32 {
    let sm = host.shared_memory() as *mut T_G_ICARUS_GETFLOAT;
    unsafe {
        (*sm).entID = ent_id;
        (*sm).r#type = var_type;
        strcpy_bounded(addr_of_mut!((*sm).name) as *mut u8, 2048, name);
        (*sm).value = 0.0;
    }
    let r = host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_GETFLOAT as i32, &[]) as i32;
    unsafe {
        *value = (*sm).value;
    }
    r
}

/// Raven `Q3_GetVector` — the `I_GetVector` target.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:917-930`
pub fn Q3_GetVector(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent_id: i32,
    var_type: i32,
    name: &str,
    value: &mut vec3_t,
) -> i32 {
    let sm = host.shared_memory() as *mut T_G_ICARUS_GETVECTOR;
    unsafe {
        (*sm).entID = ent_id;
        (*sm).r#type = var_type;
        strcpy_bounded(addr_of_mut!((*sm).name) as *mut u8, 2048, name);
        (*sm).value = *value;
    }
    let r = host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_GETVECTOR as i32, &[]) as i32;
    unsafe {
        *value = (*sm).value;
    }
    r
}

/// Raven `Q3_GetString` — the `I_GetString` target (`char **value` → `Option`;
/// the string is valid iff the game-side call succeeds).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:932-945`
pub fn Q3_GetString(
    _icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent_id: i32,
    var_type: i32,
    name: &str,
) -> Option<String> {
    let sm = host.shared_memory() as *mut T_G_ICARUS_GETSTRING;
    unsafe {
        (*sm).entID = ent_id;
        (*sm).r#type = var_type;
        strcpy_bounded(addr_of_mut!((*sm).name) as *mut u8, 2048, name);
    }
    let r = host.vm_call(VmSlot::Gvm, gameExport_t::GAME_ICARUS_GETSTRING as i32, &[]);
    if r != 0 {
        Some(unsafe { read_c_field(addr_of!((*sm).value) as *const u8, 2048) })
    } else {
        None
    }
}

/// Raven `Q3_Evaluate` — the `I_Evaluate` target. Compares two typed operands.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:416-611`
pub fn Q3_Evaluate(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    p1_type: i32,
    p1: &str,
    p2_type: i32,
    p2: &str,
    operator_type: i32,
) -> i32 {
    let mut f1 = 0.0f32;
    let mut f2 = 0.0f32;
    let mut v1 = [0.0f32; 3];
    let mut v2 = [0.0f32; 3];
    let mut i1 = 0i32;
    let mut i2 = 0i32;
    let mut p1_type = p1_type;
    let mut p2_type = p2_type;

    // Always demote to int on float↔int comparisons.
    if (p1_type == TK_FLOAT && p2_type == TK_INT) || (p1_type == TK_INT && p2_type == TK_FLOAT) {
        p1_type = TK_INT;
        p2_type = TK_INT;
    }

    // Cannot compare two dissimilar types.
    if p1_type != p2_type {
        Q3_DebugPrint(
            icarus,
            host,
            WL_ERROR,
            "Q3_Evaluate comparing two disimilar types!\n",
        );
        return 0;
    }

    // Format the parameters.
    match p1_type {
        TK_FLOAT => {
            f1 = c_atof(p1);
            f2 = c_atof(p2);
        }
        TK_INT => {
            i1 = c_atoi(p1);
            i2 = c_atoi(p2);
        }
        TK_VECTOR => {
            v1 = sscanf_vec(p1);
            v2 = sscanf_vec(p2);
        }
        TK_STRING | TK_IDENTIFIER => {
            // c1/c2 are p1/p2 directly.
        }
        _ => {
            Q3_DebugPrint(icarus, host, WL_WARNING, "Q3_Evaluate unknown type used!\n");
            return 0;
        }
    }

    // Compare them and return the result.
    match operator_type {
        // EQUAL TO
        TK_EQUALS => match p1_type {
            TK_FLOAT => (f1 == f2) as i32,
            TK_INT => (i1 == i2) as i32,
            TK_VECTOR => vector_compare(&v1, &v2) as i32,
            // `!stricmp` — equal strings compare true.
            TK_STRING | TK_IDENTIFIER => (stricmp(p1, p2) == 0) as i32,
            _ => {
                Q3_DebugPrint(icarus, host, WL_ERROR, "Q3_Evaluate unknown type used!\n");
                0
            }
        },
        // GREATER THAN
        TK_GREATER_THAN => match p1_type {
            TK_FLOAT => (f1 > f2) as i32,
            TK_INT => (i1 > i2) as i32,
            TK_VECTOR => {
                Q3_DebugPrint(
                    icarus,
                    host,
                    WL_ERROR,
                    "Q3_Evaluate vector comparisons of type GREATER THAN cannot be performed!",
                );
                0
            }
            TK_STRING | TK_IDENTIFIER => {
                Q3_DebugPrint(
                    icarus,
                    host,
                    WL_ERROR,
                    "Q3_Evaluate string comparisons of type GREATER THAN cannot be performed!",
                );
                0
            }
            _ => {
                Q3_DebugPrint(icarus, host, WL_ERROR, "Q3_Evaluate unknown type used!\n");
                0
            }
        },
        // LESS THAN
        TK_LESS_THAN => match p1_type {
            TK_FLOAT => (f1 < f2) as i32,
            TK_INT => (i1 < i2) as i32,
            TK_VECTOR => {
                Q3_DebugPrint(
                    icarus,
                    host,
                    WL_ERROR,
                    "Q3_Evaluate vector comparisons of type LESS THAN cannot be performed!",
                );
                0
            }
            TK_STRING | TK_IDENTIFIER => {
                Q3_DebugPrint(
                    icarus,
                    host,
                    WL_ERROR,
                    "Q3_Evaluate string comparisons of type LESS THAN cannot be performed!",
                );
                0
            }
            _ => {
                Q3_DebugPrint(icarus, host, WL_ERROR, "Q3_Evaluate unknown type used!\n");
                0
            }
        },
        // NOT (implied "NOT EQUAL TO")
        TK_NOT => match p1_type {
            TK_FLOAT => (f1 != f2) as i32,
            TK_INT => (i1 != i2) as i32,
            TK_VECTOR => (!vector_compare(&v1, &v2)) as i32,
            // Raven returns the raw `stricmp` result (nonzero when different).
            TK_STRING | TK_IDENTIFIER => stricmp(p1, p2),
            _ => {
                Q3_DebugPrint(icarus, host, WL_ERROR, "Q3_Evaluate unknown type used!\n");
                0
            }
        },
        _ => {
            Q3_DebugPrint(
                icarus,
                host,
                WL_ERROR,
                "Q3_Evaluate unknown operator used!\n",
            );
            0
        }
    }
}

/// Raven `AppendToSaveGame` — the `I_WriteSaveData` target; inert (`return 1`)
/// in MP dedicated (Divergences).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:695-698`
pub fn AppendToSaveGame(
    _icarus: &mut Icarus,
    _host: &mut dyn EngineHost,
    _chid: u32,
    _data: &[u8],
) -> i32 {
    1
}

/// Raven `ReadFromSaveGame` — the `I_ReadSaveData` target; inert (`return 1`)
/// in MP dedicated (Divergences).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:701-704`
pub fn ReadFromSaveGame(
    _icarus: &mut Icarus,
    _host: &mut dyn EngineHost,
    _chid: u32,
    _length: i32,
) -> i32 {
    1
}

// ===========================================================================
// Inbound G_ICARUS_* seam callees homed here (Q3_Interface.cpp source).
// ===========================================================================

/// Raven `Q3_CheckStringCounterIncrement` — a leading `+`/`-` makes a `Set`
/// value an increment/decrement of the current value.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:189-212`
fn Q3_CheckStringCounterIncrement(string: &str) -> f32 {
    let b = string.as_bytes();
    let mut val = 0.0f32;
    if b.first() == Some(&b'+') {
        if b.len() > 1 {
            val = c_atof(&string[1..]);
        }
    } else if b.first() == Some(&b'-') && b.len() > 1 {
        val = c_atof(&string[1..]) * -1.0;
    }
    val
}

/// Raven `Q3_SetVar` — the `G_ICARUS_SETVAR` arm callee (`sv_game.cpp:817`).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:337-375`
pub fn q3_set_var(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    _task_id: i32,
    _ent_num: i32,
    type_name: &str,
    data: &str,
) {
    let vret = q3_variable_declared(icarus, host, type_name);

    if vret != VTYPE_NONE {
        match vret {
            VTYPE_FLOAT => {
                // Check to see if increment command.
                let val = Q3_CheckStringCounterIncrement(data);
                let float_data = if val != 0.0 {
                    q3_get_float_variable(icarus, host, type_name).unwrap_or(0.0) + val
                } else {
                    c_atof(data)
                };
                Q3_SetFloatVariable(icarus, type_name, float_data);
            }
            VTYPE_STRING => {
                Q3_SetStringVariable(icarus, type_name, data);
            }
            VTYPE_VECTOR => {
                Q3_SetVectorVariable(icarus, type_name, data);
            }
            _ => {}
        }
        return;
    }

    Q3_DebugPrint(
        icarus,
        host,
        WL_ERROR,
        &format!("{} variable or field not found!\n", type_name),
    );
}

/// Raven `Q3_TaskIDSet` — `G_ICARUS_TASKIDSET`; writes `ent->taskID[]` on the
/// **bare real entity** (writes persist, ruling 37). `task_type` is already
/// in-range (the §19 int→enum check lives at the server-dispatch boundary).
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:167-178`
pub fn q3_task_id_set(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent: *mut sharedEntity_t,
    task_type: taskID_t,
    task_id: i32,
) {
    let tt = task_index(&task_type);
    // Raven's own guard; `task_type` is in-range by construction, so it never
    // fires here — transcribed faithfully (Divergences).
    if tt < taskID_t::TID_CHAN_VOICE as i32 || tt >= taskID_t::NUM_TIDS as i32 {
        return;
    }

    // Might be stomping an old task, so complete and clear the previous one.
    q3_task_id_complete(icarus, host, ent, task_type);

    unsafe {
        (*ent).taskID[tt as usize] = task_id;
    }
}

/// Raven `Q3_TaskIDComplete` — `G_ICARUS_TASKIDCOMPLETE`.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:134-159`
pub fn q3_task_id_complete(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent: *mut sharedEntity_t,
    task_type: taskID_t,
) {
    let tt = task_index(&task_type);
    if tt < taskID_t::TID_CHAN_VOICE as i32 || tt >= taskID_t::NUM_TIDS as i32 {
        return;
    }

    let number = unsafe { (*ent).s.number };
    // §19: Raven indexes gTaskManagers[ent->s.number] unchecked; guard OOB.
    let in_range = number >= 0 && (number as usize) < MAX_GENTITIES;
    let has_tm = in_range && icarus.task_managers[number as usize].is_some();

    if has_tm && q3_task_id_pending(icarus, host, ent, task_type) {
        // Complete it.
        let clear_task = unsafe { (*ent).taskID[tt as usize] };
        if let Some(tm) = icarus.task_managers[number as usize].as_mut() {
            task_manager_completed(tm, clear_task);
        }

        // See if any other tasks have the same number and clear them so we
        // don't complete more than once. (`Q3_TaskIDClear` sets the slot -1.)
        for tid in 0..(taskID_t::NUM_TIDS as usize) {
            unsafe {
                if (*ent).taskID[tid] == clear_task {
                    (*ent).taskID[tid] = -1;
                }
            }
        }
    }
    // otherwise, wasn't waiting for a task to complete anyway
}

/// Raven `Q3_TaskIDPending` — `G_ICARUS_TASKIDPENDING`.
/// Source: `oracle/codemp/icarus/Q3_Interface.cpp:109-127`
pub fn q3_task_id_pending(
    icarus: &mut Icarus,
    _host: &mut dyn EngineHost,
    ent: *mut sharedEntity_t,
    task_type: taskID_t,
) -> bool {
    let number = unsafe { (*ent).s.number };
    // §19: Raven indexes gSequencers/gTaskManagers[ent->s.number] unchecked;
    // guard OOB and treat an absent slot as Raven's NULL → qfalse.
    if number < 0 || (number as usize) >= MAX_GENTITIES {
        return false;
    }
    let n = number as usize;
    if icarus.sequencers[n].is_none() || icarus.task_managers[n].is_none() {
        return false;
    }

    let tt = task_index(&task_type);
    if tt < taskID_t::TID_CHAN_VOICE as i32 || tt >= taskID_t::NUM_TIDS as i32 {
        return false;
    }

    // -1 is none.
    unsafe { (*ent).taskID[tt as usize] >= 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_atof_parses_leading_float() {
        assert_eq!(c_atof("5"), 5.0);
        assert_eq!(c_atof("5.5abc"), 5.5);
        assert_eq!(c_atof("  -2.25"), -2.25);
        assert_eq!(c_atof("1e3"), 1000.0);
        assert_eq!(c_atof("nope"), 0.0);
        assert_eq!(c_atof(""), 0.0);
    }

    #[test]
    fn c_atoi_parses_leading_int() {
        assert_eq!(c_atoi("42"), 42);
        assert_eq!(c_atoi("-7x"), -7);
        assert_eq!(c_atoi("  3 4"), 3);
        assert_eq!(c_atoi("x"), 0);
    }

    #[test]
    fn stricmp_is_case_insensitive() {
        assert_eq!(stricmp("Hello", "hello"), 0);
        assert!(stricmp("abc", "abd") < 0);
        assert!(stricmp("abd", "abc") > 0);
        assert!(stricmp("ab", "abc") < 0);
    }

    #[test]
    fn check_string_counter_increment() {
        assert_eq!(Q3_CheckStringCounterIncrement("+3"), 3.0);
        assert_eq!(Q3_CheckStringCounterIncrement("-2.5"), -2.5);
        assert_eq!(Q3_CheckStringCounterIncrement("5"), 0.0);
        assert_eq!(Q3_CheckStringCounterIncrement("+"), 0.0);
    }

    #[test]
    fn sscanf_vec_fills_present_axes() {
        assert_eq!(sscanf_vec("1 2 3"), [1.0, 2.0, 3.0]);
        assert_eq!(sscanf_vec("1.5 -2"), [1.5, -2.0, 0.0]);
        assert_eq!(sscanf_vec(""), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn vector_compare_is_exact() {
        assert!(vector_compare(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]));
        assert!(!vector_compare(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.1]));
    }
}
