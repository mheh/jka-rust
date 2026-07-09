//! `EngineHost` — the in-engine service surface for the §F subsystems.
//!
//! Each method transcribes one host function the icarus / RMG / ghoul2 /
//! NPCNav / ROFF subsystems call in the oracle; the `Source:` cite names that
//! function. Per `engine-fork-discovery.md` ruling 11 the trait is ONE services
//! trait in this Stage-0 crate, and per ruling 24 its consumers store
//! `&mut dyn EngineHost` — so it stays dyn-compatible: no generic methods, no
//! by-value `Self` returns.

use core::ffi::c_char;

use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::vec3_t;

use crate::vm_slot::VmSlot;

/// Raven's host service surface for the server-side game subsystems.
///
/// The method set is fixed by ruling 24 (trace, FS read/free, print/error,
/// `VM_Call`, shared-memory window, `flrand`/`irand`, gentity).
pub trait EngineHost {
    /// Raven `SV_Trace` — sweep a box through the collision world, writing the
    /// result into `results` (kept as an out-param to transcribe the NPCNav
    /// call sites `SV_Trace( &trace, ... )` 1:1; `capsule` is Raven's
    /// `qboolean`, idiomatic `bool` per porting-rules §C7).
    /// Source: `oracle/codemp/server/sv_world.cpp:803`
    #[allow(clippy::too_many_arguments)]
    fn trace(
        &mut self,
        results: &mut trace_t,
        start: &vec3_t,
        mins: &vec3_t,
        maxs: &vec3_t,
        end: &vec3_t,
        pass_entity_num: i32,
        contentmask: i32,
        capsule: bool,
        trace_flags: i32,
        use_lod: i32,
    );

    /// Raven `FS_ReadFile` — read a file whole; `None` mirrors Raven's `-1`
    /// length / `NULL` buffer (missing file). The returned `Vec` is the file
    /// bytes (its `len()` is Raven's returned length); FS_ReadFile's trailing
    /// NUL is an FS-impl detail, not part of the contract.
    /// Source: `oracle/codemp/qcommon/files.cpp:1670`
    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>>;

    /// Raven `FS_FreeFile` — release a buffer from [`fs_read_file`]. Consuming
    /// the `Vec` keeps the read/free pairing at the call site; ownership makes
    /// the free itself a drop (default no-op).
    /// Source: `oracle/codemp/qcommon/files.cpp:1798`
    ///
    /// [`fs_read_file`]: EngineHost::fs_read_file
    fn fs_free_file(&mut self, _buffer: Vec<u8>) {}

    /// Raven `Com_Printf` — print pre-formatted text. Raven's varargs collapse
    /// to a formatted `&str` at the call site (porting-rules §C).
    /// Source: `oracle/codemp/qcommon/common.cpp:128`
    fn print(&mut self, msg: &str);

    /// Raven `Com_Error` — diverts through the panic + `catch_unwind` model
    /// (ruling fork-1): the payload carries `code` + `msg`, so this never
    /// returns. `code` is `errorParm_t` (enum fidelity over Raven's `int`).
    /// Source: `oracle/codemp/qcommon/common.cpp:249`
    fn error(&mut self, code: errorParm_t, msg: &str) -> !;

    /// Raven `VM_Call( vm, callnum, ... )` — invoke a loaded module. `vm`
    /// mirrors Raven's first parameter ([`VmSlot::Gvm`]/[`VmSlot::Cgvm`],
    /// ruling 33b); args are `intptr_t`-width slots (ruling 6); the return is
    /// `intptr_t` too, since ROFF casts it straight to a pointer
    /// (`RoffSystem.cpp:837`). The icarus arms pass no args (their request
    /// travels through [`shared_memory`]); NPCNav's `gameCallbacks` pass up
    /// to seven.
    /// Source: `oracle/codemp/qcommon/vm.cpp:787`
    ///
    /// [`shared_memory`]: EngineHost::shared_memory
    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize;

    /// Raven `sv.mSharedMemory` — the `char *` window the game handed over via
    /// `G_SET_SHARED_BUFFER`. A subsystem writes its `T_G_ICARUS_*` request
    /// struct here, then [`vm_call`]s the matching game export.
    /// Source: `oracle/codemp/server/server.h:87` (`sv_game.cpp:940` arms it)
    ///
    /// [`vm_call`]: EngineHost::vm_call
    fn shared_memory(&mut self) -> *mut c_char;

    /// Raven `Q_flrand` — a float `min <= x < max` off the engine's own
    /// `q_math.c` `holdrand` LCG instance (ruling 21: a qshared `QRand`-type
    /// field on `Engine.common`, reached through this method).
    /// Source: `oracle/codemp/game/q_math.c:1451`
    fn flrand(&mut self, min: f32, max: f32) -> f32;

    /// Raven `Q_irand` — an integer `min <= x <= max` off the same LCG.
    /// Source: `oracle/codemp/game/q_math.c:1471`
    fn irand(&mut self, min: i32, max: i32) -> i32;

    /// Raven `SV_GentityNum` — the game entity at slot `ent_num`. Returns the
    /// raw `*mut sharedEntity_t` exactly as the trap marshals it (rulings
    /// 19/23/30, transcription-first): the entity-taking icarus/NPCNav arms
    /// already carry the pointer, this serves genuinely index-based access.
    /// Source: `oracle/codemp/server/sv_game.cpp:54`
    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t;
}
