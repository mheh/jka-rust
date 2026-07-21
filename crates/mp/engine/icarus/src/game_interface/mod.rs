//! MP ICARUS `GameInterface.cpp` — the inbound `G_ICARUS_*` seam callees plus
//! the buffer/ent-list bookkeeping (§F idiomatic reimplementation).
//!
//! The five entity-field fns (`icarus_run_script`/`icarus_valid_ent`/
//! `icarus_init_ent`/`icarus_free_ent`/`icarus_associate_ent`) carry the
//! `*mut sharedEntity_t` the arm passes — which **points to `ConvertedEntity`'s
//! by-value copy** (ICARUS-D3 / ruling 37), so reads are faithful and writes are
//! dropped exactly as retail (e.g. `icarus_init_ent`'s `memset(&ent.taskID,-1)`,
//! `GameInterface.cpp:664`). The three presence-check fns carry an `ent_num: i32`.
//! Out-of-range entnum on the five unchecked `gSequencers`/`gTaskManagers` paths
//! guards-and-returns per §19 (ICARUS-D3 / ruling 15). `host.gentity` is reached
//! from `icarus_valid_ent` (behaviorSet write-back to the TRUE entity),
//! `icarus_shutdown` (per-slot teardown sweep), and `ICARUS_LinkEntity`;
//! `icarus_associate_ent` reads `ent.s.number` off the copy and needs no gentity.
//!
//! **§20-dropped:** `Svcmd_ICARUS_f` (`GameInterface.h:32`,
//! `GameInterface.cpp:700-730`) — commented-out body, zero callers/registrations,
//! no `G_ICARUS_*` arm (ICARUS-D3 / ruling 17). Its `ICARUS_entFilter` writer is
//! dropped with it, so `ent_filter` stays `-1` for the process lifetime.

#![allow(non_snake_case)]

use std::ffi::CStr;

use core::ffi::c_char;

use mp_host_interface::{EngineHost, VmSlot};
use mp_qshared::common::mp::gentity::NUM_TIDS;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::qcommon::t_g_icarus_getsetidforstring::T_G_ICARUS_GETSETIDFORSTRING;
use mp_qshared::common::mp::qcommon::t_g_icarus_soundindex::T_G_ICARUS_SOUNDINDEX;
use mp_qshared::shared::limits::MAX_GENTITIES;
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::wl_e::WL_e;

use crate::blockstream::cblock::bytes_to_c_string;
use crate::instance::icarus_instance::IcarusInstance;
use crate::q3_interface::set_type_t::setType_t;
use crate::taskmanager::ctask_manager::TaskManager;
use crate::Icarus;

pub mod pscript_s;

// ===========================================================================
// Local constants for out-of-scope/out-of-crate values this file still needs.
// ===========================================================================

/// Raven `IBI_EXT` — precompiled ICARUS script extension.
/// Source: `oracle/codemp/icarus/blockstream.h:18`
const IBI_EXT: &str = ".IBI";

/// Raven `Q3_SCRIPT_DIR` — the script search root.
/// Source: `oracle/codemp/game/q_shared.h:10`
const Q3_SCRIPT_DIR: &str = "scripts";

/// Raven `interpreter.h`/`tokenizer.h` block-type IDs `ICARUS_InterrogateScript`
/// matches on. Those headers are out-of-scope for this port (§ Out of scope —
/// `Interpreter.cpp`/`Tokenizer.cpp` are not in the link set), so the resolved
/// values are pinned here as local constants (same treatment as
/// `blockstream/cblock_member.rs`'s `ID_RANDOM`), derived from the enum chain
/// `TK_USERDEF`(8)..`NUM_USER_TOKENS`(19)..`NUM_IDS`(51):
/// `TK_STRING` = 4 (`tokenizer.h:64-73`); `ID_SOUND` = `NUM_USER_TOKENS`+1 = 20,
/// `ID_SET` = +7 = 26, `ID_RUN` = +13 = 32, `ID_CAMERA` = +16 = 35,
/// `ID_PLAY` = +29 = 48 (`interpreter.h:33-64`); `TYPE_PATH` = `NUM_IDS`+10 = 61
/// (`interpreter.h:68-91`).
const TK_STRING: i32 = 4;
const ID_SOUND: i32 = 20;
const ID_SET: i32 = 26;
const ID_RUN: i32 = 32;
const ID_CAMERA: i32 = 35;
const ID_PLAY: i32 = 48;
const TYPE_PATH: i32 = 61;

/// Raven `GAME_ICARUS_SOUNDINDEX`/`GAME_ICARUS_GETSETIDFORSTRING` vmcall ids.
/// `mp_abi` (home of the Rust `MpGameExport` mirror) is not a dependency of
/// this engine-tier crate, so the ordinals are pinned directly off Raven's
/// 0-based, no-explicit-discriminant `gameExport_t`.
/// Source: `oracle/codemp/game/g_public.h:734-787`
const GAME_ICARUS_SOUNDINDEX: i32 = 28;
const GAME_ICARUS_GETSETIDFORSTRING: i32 = 29;

/// Raven `BSTable[]` — `bState_t` name/id lookup used only by
/// `ICARUS_PrecacheEnt` to tell a behaviorSet keyword apart from a script
/// path. A read-only parse table (§ State ownership: "const tables stay
/// `const`"); duplicated locally rather than reached from `mp_game` (engine
/// tier cannot depend on the game crate — workspace-architecture tiers).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:592-611`, values from
/// `oracle/codemp/game/g_public.h:585-595` (`bState_e`, explicit `BS_DEFAULT = 0`).
const BS_TABLE: &[(&str, i32)] = &[
    ("BS_DEFAULT", 0),
    ("BS_ADVANCE_FIGHT", 1),
    ("BS_SLEEP", 2),
    ("BS_FOLLOW_LEADER", 3),
    ("BS_JUMP", 4),
    ("BS_SEARCH", 5),
    ("BS_WANDER", 6),
    ("BS_NOCLIP", 7),
    ("BS_REMOVE", 8),
    ("BS_CINEMATIC", 9),
];

// ===========================================================================
// Small local helpers (raw C-string / shared-memory plumbing).
// ===========================================================================

/// Raven `VALIDSTRING(a)` — `(a != 0) && (a[0] != 0)`.
/// Source: `oracle/codemp/game/q_shared.h:30`
fn valid_c_str(ptr: *const c_char) -> bool {
    !ptr.is_null() && unsafe { *ptr != 0 }
}

/// NUL-terminated C string → owned `String` (lossy); NULL → empty. The seam
/// hands back raw `char *` entity fields, which this crate only ever reads.
fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: caller-supplied pointer is a live entity field the seam owns
    // for the duration of this call (§D11 — dereferencing the seam pointer
    // is the confined ABI unsafe).
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

/// Raven `strncpy( temp, str, 1023 ); temp[1023] = 0;` — the 1023-byte
/// truncation `ICARUS_FreeEnt`/`ICARUS_AssociateEnt` apply before their
/// `ICARUS_EntList` key lookup/insert.
fn truncate_1023(mut s: String) -> String {
    if s.len() > 1023 {
        s.truncate(1023);
    }
    s
}

/// Bounded copy into a fixed `[c_char; N]` shared-memory field. Raven's raw
/// `strcpy` here is unbounded; truncating to fit is the defined choice for an
/// over-long name (§19, same treatment as the OOB-entnum guards elsewhere in
/// this file).
fn write_shared_c_str(dst: &mut [c_char], src: &str) {
    let bytes = src.as_bytes();
    let cap = dst.len().saturating_sub(1);
    let n = bytes.len().min(cap);
    for (slot, &b) in dst.iter_mut().zip(bytes[..n].iter()) {
        *slot = b as c_char;
    }
    dst[n] = 0;
}

/// Raven `GetIDForString` — case-insensitive linear scan, `-1` if absent.
/// Source: `oracle/codemp/game/q_shared.c:13-27`
fn get_id_for_string(table: &[(&str, i32)], name: &str) -> i32 {
    table
        .iter()
        .find(|(entry_name, _)| entry_name.eq_ignore_ascii_case(name))
        .map(|(_, id)| *id)
        .unwrap_or(-1)
}

// ===========================================================================
// Init / shutdown.
// ===========================================================================

/// Raven `ICARUS_Init` — `Interface_Init(&interface_export)` then
/// `iICARUS = ICARUS_Instance::Create(...)`; NULL result → `host.error(ERR_DROP)`.
///
/// `IcarusInstance::create` returns an owned value, not a fallible pointer, so
/// there is no reachable NULL case here to route through `host.error` — the
/// owned-construction model (porting-rules §C9) has no allocation-failure path.
/// Source: `oracle/codemp/icarus/GameInterface.cpp:143-156`
pub fn icarus_init(icarus: &mut Icarus, host: &mut dyn EngineHost) {
    crate::q3_interface::Interface_Init(&mut icarus.interface_export);
    icarus.instance = Some(IcarusInstance::create());
    let _ = host;
}

/// Raven `ICARUS_Shutdown` — walks `host.gentity(i)` over all `MAX_GENTITIES`
/// and feeds the real pointer into `icarus_free_ent` (`:184`).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:166-186`
pub fn icarus_shutdown(icarus: &mut Icarus, host: &mut dyn EngineHost) {
    for i in 0..MAX_GENTITIES {
        if icarus.sequencers[i].is_some() {
            // Raven's `ent->s.number` cross-check (`:174-178`) is a
            // debug-only `assert(0)` guard; dropped (never fires under
            // correct entity/index bookkeeping).
            let ent = host.gentity(i as i32);
            icarus_free_ent(icarus, host, ent);
        }
    }

    // `ICARUS_Free`-per-blob + `delete` teardown collapses to the map's
    // owned `Vec<u8>`/`Pscript` drops (ICARUS-D3/ruling 20 — no arena).
    icarus.buffer_list.clear();
    icarus.ent_list.clear();

    if let Some(mut instance) = icarus.instance.take() {
        instance.delete();
    }
}

// ===========================================================================
// Entity-field fns (ConvertedEntity-copy pointer; ruling 37).
// ===========================================================================

/// Raven `ICARUS_RunScript` — arm `sv_game.cpp:740` (`ent` is the ConvertedEntity
/// copy). Indexes `gSequencers[ent.s.number]` (guard-and-return, §19).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:70-140`
pub fn icarus_run_script(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent: *mut sharedEntity_t,
    name: &str,
) -> bool {
    // SAFETY: `ent` is the seam's ConvertedEntity-copy pointer (ruling 37);
    // this fn only reads its fields.
    let (ent_num, classname, targetname) = unsafe {
        let e = &*ent;
        (e.s.number, e.classname, e.targetname)
    };

    // Unchecked in Raven (`gSequencers[ent->s.number]`, `:75`); guard-and-return, §19.
    if ent_num < 0
        || ent_num as usize >= MAX_GENTITIES
        || icarus.sequencers[ent_num as usize].is_none()
    {
        return false;
    }

    if !icarus_get_script(icarus, host, name) {
        return false;
    }
    let buf = match icarus.buffer_list.get(name) {
        Some(script) if !script.buffer.is_empty() => script.buffer.clone(),
        _ => return false,
    };

    // `S_FAILED(a)` == `a != SEQ_OK` (`SEQ_OK == 0`, `sequencer.h:36,52`).
    if crate::sequencer::csequencer::run(icarus, host, ent_num, &buf) != 0 {
        return false;
    }

    if icarus.ent_filter == -1 || icarus.ent_filter == ent_num {
        let msg = format!(
            "{} Script {} executed by {} {}\n",
            host.sv_time(),
            name,
            cstr_to_string(classname),
            cstr_to_string(targetname),
        );
        crate::q3_interface::Q3_DebugPrint(icarus, host, WL_e::WL_VERBOSE as i32, &msg);
    }

    true
}

/// Raven `ICARUS_ValidEnt` — arm `sv_game.cpp:750`. For a behaviorSet-carrying
/// entity with no `script_targetname`, writes back through
/// `host.gentity(ent.s.number)` to the TRUE entity (`:288`/`:291`).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:268-297`
pub fn icarus_valid_ent(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent: *mut sharedEntity_t,
) -> bool {
    let _ = icarus;
    // SAFETY: `ent` is the seam's ConvertedEntity-copy pointer (ruling 37);
    // reads only (the write-back below targets the TRUE entity via `host.gentity`).
    let (ent_num, script_targetname, behavior_set) = unsafe {
        let e = &*ent;
        (e.s.number, e.script_targetname, e.behaviorSet)
    };

    if valid_c_str(script_targetname) {
        return true;
    }

    for bset in behavior_set.iter() {
        if valid_c_str(*bset) {
            let true_entity = host.gentity(ent_num);
            // SAFETY: `host.gentity` returns the real, live entity at this slot.
            unsafe {
                (*true_entity).script_targetname = (*true_entity).targetname;
            }
            return true;
        }
    }

    false
}

/// Raven `ICARUS_InitEnt` — arm `sv_game.cpp:789`. The `memset(&ent.taskID,-1)`
/// writes the copy and is dropped, faithfully (`:664`, ruling 37).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:646-677`
pub fn icarus_init_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t) {
    // SAFETY: `ent` is the seam's ConvertedEntity-copy pointer (ruling 37).
    let ent_num = unsafe { (*ent).s.number };

    // Unchecked in Raven (`:660`); guard-and-return, §19.
    if ent_num < 0 || ent_num as usize >= MAX_GENTITIES {
        return;
    }
    let idx = ent_num as usize;

    // Raven's `assert(gTaskManagers[n]==NULL); assert(gSequencers[n]==NULL);`
    // "fresh ent" precondition is dropped (debug-only); the two guards below
    // reproduce its actual runtime behavior.
    if icarus.sequencers[idx].is_some() {
        return;
    }
    if icarus.task_managers[idx].is_some() {
        return;
    }

    // Raven: `gSequencers[n]=iICARUS->GetSequencer(n);
    // gTaskManagers[n]=gSequencers[n]->GetTaskManager();`. Under ICARUS-D3
    // (ruling 27) the created task manager isn't reachable off the Rust
    // `Sequencer` (no back-ref), so it is constructed directly into
    // `Icarus.task_managers` instead of fetched through the sequencer.
    let sequencer_id = icarus
        .instance
        .as_mut()
        .expect("ICARUS_InitEnt: iICARUS must be initialized (Raven `assert(iICARUS)`, :649)")
        .get_sequencer(ent_num);
    icarus.sequencers[idx] = Some(sequencer_id);
    icarus.task_managers[idx] = Some(TaskManager::create());

    // `memset(&ent->taskID,-1,sizeof(ent->taskID))` — writes ConvertedEntity's
    // by-value copy and is dropped, faithfully (ruling 37, `:664`).
    unsafe {
        (*ent).taskID = [-1i32; NUM_TIDS];
    }

    icarus_associate_ent(icarus, host, ent);
    icarus_precache_ent(icarus, host, ent);
}

/// Raven `ICARUS_FreeEnt` — arm `sv_game.cpp:793`. Guards
/// `s.number < 0 || >= MAX_GENTITIES` (`:224-229`).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:220-256`
pub fn icarus_free_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t) {
    let _ = host;
    // SAFETY: `ent` is the ConvertedEntity-copy pointer for the arm call, or
    // the real entity for `icarus_shutdown`'s teardown sweep — reads only.
    let (ent_num, script_targetname) = unsafe {
        let e = &*ent;
        (e.s.number, e.script_targetname)
    };

    if ent_num < 0 || ent_num as usize >= MAX_GENTITIES {
        return;
    }
    let idx = ent_num as usize;

    if icarus.sequencers[idx].is_none() {
        return;
    }

    if valid_c_str(script_targetname) {
        let key = truncate_1023(cstr_to_string(script_targetname)).to_uppercase();
        icarus.ent_list.remove(&key);
    }

    if let Some(sequencer_id) = icarus.sequencers[idx] {
        if let Some(instance) = icarus.instance.as_mut() {
            instance.delete_sequencer(sequencer_id);
        }
    }

    icarus.sequencers[idx] = None;
    icarus.task_managers[idx] = None;
}

/// Raven `ICARUS_AssociateEnt` — arm `sv_game.cpp:797`. Only reads `ent.s.number`
/// off the copy into `ent_list` (`:317`) — no `host.gentity`.
/// Source: `oracle/codemp/icarus/GameInterface.cpp:307-318`
pub fn icarus_associate_ent(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent: *mut sharedEntity_t,
) {
    let _ = host;
    // SAFETY: `ent` is the seam's ConvertedEntity-copy pointer (ruling 37);
    // reads only, present-by-value on the copy — no `host.gentity` needed.
    let (ent_num, script_targetname) = unsafe {
        let e = &*ent;
        (e.s.number, e.script_targetname)
    };

    if !valid_c_str(script_targetname) {
        return;
    }

    let key = truncate_1023(cstr_to_string(script_targetname)).to_uppercase();
    icarus.ent_list.insert(key, ent_num);
}

// ===========================================================================
// Presence-check fns (int entnum arms).
// ===========================================================================

/// Raven `G_ICARUS_ISINITIALIZED` — arm `sv_game.cpp:752` (`int entID`);
/// `gSequencers[entID]` presence (guard-and-return, §19).
/// Source: `oracle/codemp/server/sv_game.cpp:752-762`
pub fn icarus_is_initialized(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_num: i32) -> bool {
    let _ = host;
    // Unchecked in Raven (`entID = args[1]`); guard-and-return, §19.
    if ent_num < 0 || ent_num as usize >= MAX_GENTITIES {
        return false;
    }
    let idx = ent_num as usize;
    icarus.sequencers[idx].is_some() && icarus.task_managers[idx].is_some()
}

/// Raven `G_ICARUS_MAINTAINTASKMANAGER` — arm `sv_game.cpp:763` (`int entID`);
/// per-frame `CTaskManager::Update` (guard-and-return, §19).
/// Source: `oracle/codemp/server/sv_game.cpp:763-773`
pub fn icarus_maintain_task_manager(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    ent_num: i32,
) -> bool {
    if ent_num < 0 || ent_num as usize >= MAX_GENTITIES {
        return false;
    }
    let idx = ent_num as usize;
    if icarus.task_managers[idx].is_none() {
        return false;
    }
    crate::taskmanager::ctask_manager::update(icarus, host, ent_num);
    true
}

/// Raven `G_ICARUS_ISRUNNING` — arm `sv_game.cpp:774` (`int entID`);
/// `gTaskManagers[entID]` presence (guard-and-return, §19).
/// Source: `oracle/codemp/server/sv_game.cpp:774-782`
pub fn icarus_is_running(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_num: i32) -> bool {
    let _ = host;
    if ent_num < 0 || ent_num as usize >= MAX_GENTITIES {
        return false;
    }
    match &icarus.task_managers[ent_num as usize] {
        Some(tm) => tm.is_running(),
        None => false,
    }
}

// ===========================================================================
// Outbound I_LinkEntity target.
// ===========================================================================

/// Raven `ICARUS_LinkEntity` — the outbound `I_LinkEntity` target; resolves
/// `entID` via `host.gentity` (`:681`), wires the per-entity tables, then calls
/// `icarus_associate_ent` on the real pointer.
///
/// PORTER NOTE (reported, not improvised): Raven's actual signature is
/// `(int entID, CSequencer *sequencer, CTaskManager *taskManager)`
/// (`interface.h:68`); this frozen skeleton only carries `ent_id`, so the
/// `gSequencers[n]=sequencer; gTaskManagers[n]=taskManager;` assignments
/// (`:686-687`) have no params to transcribe from here. Separately, this fn's
/// only oracle call site (`Sequencer.cpp:2413`) sits inside an `#if 0` block
/// in `CSequencer::Load`, itself dead (`Save`/`Load` unconditionally
/// `return false;`) — so in the compiled oracle it is never actually invoked.
/// Source: `oracle/codemp/icarus/GameInterface.cpp:679-692`
pub fn ICARUS_LinkEntity(icarus: &mut Icarus, host: &mut dyn EngineHost, ent_id: i32) -> i32 {
    let ent = host.gentity(ent_id);
    if ent.is_null() {
        return false as i32;
    }
    icarus_associate_ent(icarus, host, ent);
    true as i32
}

// ===========================================================================
// Script cache / precache helpers.
// ===========================================================================

/// Raven `ICARUS_GetScript` — ensure the named script is cached (registering
/// it from disk if needed); the actual bytes are then read by the caller
/// directly off `Icarus.buffer_list` (the out-param `char **buf` collapses
/// into that shared, already-owned storage rather than a returned pointer).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:32-58`
pub fn icarus_get_script(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> bool {
    if icarus.buffer_list.contains_key(name) {
        return true;
    }
    icarus_register_script(icarus, host, name, false)
}

/// Raven `ICARUS_RegisterScript` — arm `sv_game.cpp:743` (no ent). Reads/frees a
/// precompiled `.IBI` blob via `host.fs_read_file`/`fs_free_file`.
/// Source: `oracle/codemp/icarus/GameInterface.cpp:346-395`
pub fn icarus_register_script(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    name: &str,
    called_during_interrogate: bool,
) -> bool {
    if icarus.buffer_list.contains_key(name) {
        // Raven's special interrogate-recursion guard: already cached during
        // interrogate MUST return false (stops recursion); a normal caller
        // gets true (`:369-373`).
        return !called_during_interrogate;
    }

    let newname = format!("{}{}", name, IBI_EXT);

    // `length <= 0` covers both Raven's `-1` (missing file, our `None`) and a
    // present-but-empty file (`Some` with an empty `Vec`).
    let buffer = host.fs_read_file(&newname).filter(|b| !b.is_empty());
    let Some(buffer) = buffer else {
        if !called_during_interrogate {
            host.print(&format!("^1Could not open file '{}'\n", newname));
        }
        return false;
    };

    // Raven's `pscript->buffer = ICARUS_Malloc(length); memcpy(...);
    // FS_FreeFile(buffer);` copies the read buffer into a second, longer-lived
    // allocation and frees the first. The owned `Vec<u8>` `fs_read_file`
    // returns already IS that longer-lived allocation (ICARUS-D3/ruling 20),
    // so it moves directly into the `Pscript`; no second copy or explicit
    // `fs_free_file` call is needed.
    let length = buffer.len() as i64;
    icarus.buffer_list.insert(
        name.to_string(),
        crate::game_interface::pscript_s::Pscript { buffer, length },
    );

    true
}

/// Raven `ICARUS_SoundPrecache` — the `GAME_ICARUS_SOUNDINDEX` outbound vmcall.
/// Source: `oracle/codemp/icarus/GameInterface.cpp:400-406`
pub fn icarus_sound_precache(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) {
    let _ = icarus;
    let mem = host.shared_memory() as *mut T_G_ICARUS_SOUNDINDEX;
    // SAFETY: `shared_memory` is the engine's fixed `gSharedBuffer` window,
    // large enough for every `T_G_ICARUS_*` payload (ABI contract).
    unsafe {
        write_shared_c_str(&mut (*mem).filename, name);
    }
    host.vm_call(VmSlot::Gvm, GAME_ICARUS_SOUNDINDEX, &[]);
}

/// Raven `ICARUS_GetIDForString` — the `GAME_ICARUS_GETSETIDFORSTRING`
/// round-trip vmcall (distinct from the local `GetIDForString`/`BS_TABLE`
/// linear scan `icarus_precache_ent` uses — this one asks the game side to
/// resolve a `setType_t` name).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:408-415`
fn icarus_get_id_for_string(icarus: &mut Icarus, host: &mut dyn EngineHost, string: &str) -> i32 {
    let _ = icarus;
    let mem = host.shared_memory() as *mut T_G_ICARUS_GETSETIDFORSTRING;
    // SAFETY: see `icarus_sound_precache`.
    unsafe {
        write_shared_c_str(&mut (*mem).string, string);
    }
    host.vm_call(VmSlot::Gvm, GAME_ICARUS_GETSETIDFORSTRING, &[]) as i32
}

/// Raven `ICARUS_PrecacheEnt` — precache all scripts referenced by the
/// entity's `behaviorSet[]` entries that aren't recognized `bState_t`
/// keywords (those are handled elsewhere; only script paths are interrogated).
/// Source: `oracle/codemp/icarus/GameInterface.cpp:614-638`
fn icarus_precache_ent(icarus: &mut Icarus, host: &mut dyn EngineHost, ent: *mut sharedEntity_t) {
    // SAFETY: see `icarus_valid_ent` — reads only.
    let behavior_set = unsafe { (*ent).behaviorSet };
    for bs in behavior_set.iter() {
        if bs.is_null() {
            continue;
        }
        let name = cstr_to_string(*bs);
        if get_id_for_string(BS_TABLE, &name) == -1 {
            let newname = format!("{}/{}", Q3_SCRIPT_DIR, name);
            icarus_interrogate_script(icarus, host, &newname);
        }
    }
}

/// Raven `ICARUS_InterrogateScript` — parse a script's blocks via
/// `BlockStream::Open` to harvest sound/entity references (`:465-476`).
///
/// The `ID_CAMERA`/`ID_PLAY` `theROFFSystem.Cache(...)` precache calls
/// (`:479-490`, `:494-503`) are elided: the ROFF engine subsystem crate does
/// not exist in this workspace yet (no `mp_engine_roff`), and this crate has
/// no dependency path to it. Skipping the precache only removes a load-time
/// cache warm — actual ROFF playback still loads lazily — and it is outside
/// this doc's § Verification golden scope (BlockStream/Q3_Registers/Sequencer
/// goldens don't touch ROFF), so parse correctness is unaffected. Reported as
/// a problem, not silently guessed.
/// Source: `oracle/codemp/icarus/GameInterface.cpp:424-593`
pub fn icarus_interrogate_script(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) {
    if name.eq_ignore_ascii_case("NULL") || name.eq_ignore_ascii_case("default") {
        return;
    }

    // Ensure the "scripts" (Q3_SCRIPT_DIR) prefix, which is missing if this
    // was called recursively (`Q_stricmpn`/`va`, `:434-443`).
    let s_filename = if name.len() >= Q3_SCRIPT_DIR.len()
        && name[..Q3_SCRIPT_DIR.len()].eq_ignore_ascii_case(Q3_SCRIPT_DIR)
    {
        name.to_string()
    } else {
        format!("{}/{}", Q3_SCRIPT_DIR, name)
    };

    if !icarus_register_script(icarus, host, &s_filename, true) {
        return;
    }

    let buf = match icarus.buffer_list.get(&s_filename) {
        Some(script) if !script.buffer.is_empty() => script.buffer.clone(),
        _ => return,
    };

    let mut stream = crate::blockstream::cblock_stream::BlockStream::default();
    if stream.open(&buf) == 0 {
        return;
    }

    let mut block = crate::blockstream::cblock::Block {
        m_members: Vec::new(),
        m_id: 0,
        m_flags: 0,
    };

    while stream.block_available() != 0 {
        if stream.read_block(&mut block) == 0 {
            return;
        }

        match block.get_block_id() {
            ID_CAMERA => {
                if let Some(data) = block.get_member_data(0) {
                    if data.len() >= 4 {
                        // Raven `*(float *) block.GetMemberData(0)` — a raw
                        // pointer reinterpret, native-endian (matches
                        // `Block::write_float`'s `to_ne_bytes` encoding).
                        let f = f32::from_ne_bytes(data[..4].try_into().unwrap());
                        if f == TYPE_PATH as f32 {
                            // theROFFSystem.Cache(...) elided — see fn doc.
                        }
                    }
                }
            }
            ID_PLAY => {
                if let Some(data0) = block.get_member_data(0) {
                    if bytes_to_c_string(data0).eq_ignore_ascii_case("PLAY_ROFF") {
                        // theROFFSystem.Cache(...) elided — see fn doc.
                    }
                }
            }
            ID_RUN => {
                if let Some(data0) = block.get_member_data(0) {
                    let run_name = COM_StripExtension(&bytes_to_c_string(data0));
                    icarus_interrogate_script(icarus, host, &run_name);
                }
            }
            ID_SOUND => {
                if let Some(data1) = block.get_member_data(1) {
                    let sound_name = bytes_to_c_string(data1);
                    icarus_sound_precache(icarus, host, &sound_name);
                }
            }
            ID_SET => {
                let member0_is_string = block.get_member(0).map(|m| m.m_id) == Some(TK_STRING);
                if member0_is_string {
                    let s_val1 = block
                        .get_member_data(0)
                        .map(bytes_to_c_string)
                        .unwrap_or_default();
                    let s_val2 = block
                        .get_member_data(1)
                        .map(bytes_to_c_string)
                        .unwrap_or_default();
                    let set_id = icarus_get_id_for_string(icarus, host, &s_val1);

                    if set_id == setType_t::SET_SPAWNSCRIPT as i32
                        || set_id == setType_t::SET_USESCRIPT as i32
                        || set_id == setType_t::SET_AWAKESCRIPT as i32
                        || set_id == setType_t::SET_ANGERSCRIPT as i32
                        || set_id == setType_t::SET_ATTACKSCRIPT as i32
                        || set_id == setType_t::SET_VICTORYSCRIPT as i32
                        || set_id == setType_t::SET_LOSTENEMYSCRIPT as i32
                        || set_id == setType_t::SET_PAINSCRIPT as i32
                        || set_id == setType_t::SET_FLEESCRIPT as i32
                        || set_id == setType_t::SET_DEATHSCRIPT as i32
                        || set_id == setType_t::SET_DELAYEDSCRIPT as i32
                        || set_id == setType_t::SET_BLOCKEDSCRIPT as i32
                        || set_id == setType_t::SET_FFIRESCRIPT as i32
                        || set_id == setType_t::SET_FFDEATHSCRIPT as i32
                        || set_id == setType_t::SET_MINDTRICKSCRIPT as i32
                        || set_id == setType_t::SET_CINEMATIC_SKIPSCRIPT as i32
                    {
                        icarus_interrogate_script(icarus, host, &s_val2);
                    } else if set_id == setType_t::SET_LOOPSOUND as i32 {
                        icarus_sound_precache(icarus, host, &s_val2);
                    }
                    // SET_VIDEO_PLAY / SET_ADDRHANDBOLT_MODEL /
                    // SET_ADDLHANDBOLT_MODEL: no-op in MP (Raven: "do nothing
                    // for MP", `:590-597`).
                }
            }
            _ => {}
        }

        block.free();
    }

    stream.free();
}
