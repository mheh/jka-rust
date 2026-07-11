//! `EngineHost` — the in-engine service surface for the §F subsystems.
//!
//! Each method transcribes one host function the icarus / RMG / ghoul2 /
//! NPCNav / ROFF subsystems call in the oracle; the `Source:` cite names that
//! function. Per `engine-fork-discovery.md` ruling 11 the trait is ONE services
//! trait in this Stage-0 crate, and per ruling 24 its consumers store
//! `&mut dyn EngineHost` — so it stays dyn-compatible: no generic methods, no
//! by-value `Self` returns.

use core::ffi::{c_char, c_void};

use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qboolean, qhandle_t, vec3_t};

use crate::vm_slot::VmSlot;

/// Raven's host service surface for the server-side game subsystems.
///
/// The method set is fixed by ruling 24 (trace, FS read/free, print/error,
/// `VM_Call`, shared-memory window, `flrand`/`irand`, gentity), extended by
/// ruling 36 (cvar read, `svs.time`, FS write, loader model memory),
/// ruling 55 (cvar register/string/take-modified, VFS file listing), and
/// ruling 59a (pak membership).
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

    /// Per-call integer cvar read (ruling 36) — collapses Raven's cached
    /// `cvar_t->integer` pattern (a `Cvar_Get`-seeded file-static read at each
    /// gate): `com_developer` (`Q3_Interface.cpp:638-643`),
    /// `cg_g2MarksAllModels` (`G2_misc.cpp:40`, read `:1524`), nav's
    /// `d_altRoutes`/`d_patched` (`navigator.cpp:480,1403,1933`). An
    /// unregistered name reads 0, as `Cvar_VariableIntegerValue` does.
    /// Source: `oracle/codemp/qcommon/cvar.cpp:118-124`
    fn cvar_integer(&mut self, name: &str) -> i32;

    /// Raven `svs.time` — the `serverStatic_t` frame clock ("strictly
    /// increasing across level changes"), consumed by nav's failed-node/edge
    /// recheck timers (`navigator.cpp:1733,1763,1778,1797,1987,2010,2065,
    /// 2137`). NOT the same clock as `PlatformHost::milliseconds`
    /// (`Sys_Milliseconds`, the wall/profiling clock): `svs.time` advances in
    /// fixed frame steps and only while the server runs frames.
    /// Source: `oracle/codemp/server/server.h:211` (`extern svs`: `:232`)
    fn sv_time(&mut self) -> i32;

    /// Whole-file write (ruling 36) — Raven's
    /// `FS_FOpenFileByMode(qpath, &f, FS_WRITE)` + `FS_Write` calls +
    /// `FS_FCloseFile` sequence collapsed; `false` mirrors the NULL-handle
    /// open failure (`CNavigator::Save` returns false there, the live
    /// `G_NAV_SAVE` arm).
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:670-699`
    /// (`FS_FOpenFileByMode`: `files.cpp:3547`, `FS_Write`: `files.cpp:1477`)
    fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool;

    /// Loader model memory, mesh half (ruling 36 / G2SV-D5) — Raven
    /// `R_GetModelByHandle( model )->mdxm`: the raw pointer to the parsed
    /// `.glm` block (`mdxmHeader_t` at offset 0). `c_void` because the mdx
    /// header types are `mp_renderer`-owned and never named at this seam
    /// (G2SV-D5); NULL exactly where Raven's pointer is NULL (not a GL2M
    /// model). No re-parsing — this is the loader's live block.
    /// Source: `oracle/codemp/renderer/tr_local.h:1128` (`model_t.mdxm`);
    /// chain: `oracle/codemp/ghoul2/G2_API.cpp:2716-2721`
    /// (`R_GetModelByHandle`: `tr_model.cpp:593`)
    fn model_mdxm(&mut self, model: qhandle_t) -> *mut c_void;

    /// Loader model memory, animation half (ruling 36 / G2SV-D5) — Raven
    /// `R_GetModelByHandle( model )->mdxa`: the raw pointer to the parsed
    /// `.gla` block (`mdxaHeader_t` at offset 0; `CBoneCache` parent seeding,
    /// skeleton build, and ragdoll basepose resolve do byte arithmetic off
    /// it, `tr_ghoul2.cpp:416-421,614-615`). Callers reach the anim handle
    /// via the mesh header's `animIndex`, as `G2_SetupModelPointers` does.
    /// Source: `oracle/codemp/renderer/tr_local.h:1129` (`model_t.mdxa`);
    /// chain: `oracle/codemp/ghoul2/G2_API.cpp:2735-2739`
    fn model_mdxa(&mut self, model: qhandle_t) -> *mut c_void;

    /// Raven `Cvar_Get`'s registration side (ruling 55) — establish the cvar
    /// with `default` exactly once (creation sets string=default, integer=
    /// atoi, `modified = qtrue`, cvar.cpp:261-273); an already-existing cvar
    /// keeps its value and only ORs `flags` in (cvar.cpp:209-232). The
    /// returned `cvar_t*` collapses away — reads go through the by-name
    /// services. StringEd registers `se_language`/`se_debug`/`sp_leet` this
    /// way in SE_Init.
    /// Source: `oracle/codemp/qcommon/cvar.cpp:188` (SE_Init sites:
    /// `oracle/codemp/qcommon/stringed_ingame.cpp:1169-1171`)
    fn cvar_register(&mut self, name: &str, default: &str, flags: i32);

    /// Per-call string cvar read (ruling 55) — collapses Raven's cached
    /// `cvar_t->string` reads (SE_Load's `se_language->string` path build,
    /// `stringed_ingame.cpp:921-925`). A missing name reads `""`, as
    /// `Cvar_VariableString` returns.
    /// Source: `oracle/codemp/qcommon/cvar.cpp:133-140`
    fn cvar_string(&mut self, name: &str) -> String;

    /// Read-and-clear of Raven's `cvar_t->modified` flag (ruling 55): returns
    /// the flag and clears it in the same call — Raven's two-step update-check
    /// idiom (`if (se_language->modified) { ...; se_language->modified =
    /// SE_FALSE; }`, SE_CheckForLanguageUpdates) collapsed so no host
    /// round-trip can observe the in-between state. A missing name reads
    /// `false`.
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1252-1259`
    fn cvar_take_modified(&mut self, name: &str) -> bool;

    /// Raven `FS_ListFiles` + `FS_FreeFileList` collapsed (ruling 55) — the
    /// VFS/pk3-aware listing over the FS search paths, DISTINCT from
    /// `PlatformHost::list_files` (`Sys_ListFiles`, a raw OS directory scan):
    /// this one sees pak contents. Subdirectories are requested with
    /// `ext = "/"` (`SE_R_ListFiles`, `stringed_interface.cpp:139`), files by
    /// extension (`:158`); the free collapses into the `Vec` drop
    /// (`:182-183`). `want_subs` extends the match into subdirectories (the
    /// ruled surface; Raven's own `FS_ListFiles` is 3-param, `files.cpp:2174`
    /// — today's call sites pass `false`).
    /// Source: `oracle/codemp/qcommon/files.cpp:2174`
    fn fs_list_files(&mut self, dir: &str, ext: &str, want_subs: bool) -> Vec<String>;

    /// Raven `FS_FileIsInPAK( filename, &checksum )` collapsed per §C7
    /// (ruling 59a). Raven's convention: returns `1` and writes
    /// `*pChecksum = pak->pure_checksum` when the file is found in a
    /// pure-allowed pak; returns `-1` (never `0`) in every other case — a
    /// file found only on disk, not found at all, an illegal `..`/`::` path,
    /// or a hit in a non-pure pak (skipped by `FS_PakIsPure`). So
    /// `Some(pure_checksum)` = the `1` path, `None` = every `-1` path. Live
    /// consumers: the `iPAKFileCheckSum` stamp in
    /// `RE_RegisterServerModels_Malloc` (`tr_model.cpp:212`) and the purity
    /// re-check in `RE_RegisterModels_DumpNonPure` (`tr_model.cpp:434-436`).
    /// Source: `oracle/codemp/qcommon/files.cpp:1602-1659` (decl:
    /// `qcommon.h:551`)
    fn fs_file_is_in_pak(&mut self, qpath: &str) -> Option<i32>;

    /// Raven `MSG_ReadDeltaEntity`'s `cl_shownet` debug probe (msg.cpp:1268-1270):
    /// returns `None` when `sv.state == SS_DEAD` (Raven's `if (sv.state)`),
    /// else the classname of `SV_GentityNum(number)`. Collapses the `sv`/
    /// `SV_GentityNum` reach `msg.cpp` cannot make from `mp_engine_qcommon`
    /// (server depends on qcommon — the sanctioned host edge, ruling 56c).
    /// The real body lands with the server-spine wave.
    /// Source: `oracle/codemp/qcommon/msg.cpp:1268-1270`
    /// (`SV_GentityNum`: `oracle/codemp/server/sv_game.cpp:58`)
    fn sv_shownet_entity_classname(&mut self, number: i32) -> Option<String>;

    /// Raven `Sys_Init` — one-time platform-layer init (`Com_Init` calls it late,
    /// `common.cpp:1287`). No-op in a test mock.
    /// Source: `oracle/codemp/win32/win_main.cpp:834`
    fn sys_init(&mut self);

    /// Raven `Sys_Quit` — orderly process exit; never returns.
    /// Source: `oracle/codemp/win32/win_main.cpp:333`
    fn sys_quit(&mut self) -> !;

    /// Raven `Sys_Error` — fatal platform error print + exit; never returns.
    /// Source: `oracle/codemp/win32/win_main.cpp:350`
    fn sys_error(&mut self, msg: &str) -> !;

    /// Raven `Sys_ShowConsole( visLevel, quitOnClose )` — show/hide the dedicated
    /// console window (`quit_on_close` kept as Raven's `qboolean` to transcribe
    /// the `Com_Init`/`Com_Frame` call sites 1:1).
    /// Source: `oracle/codemp/win32/win_syscon.cpp:396`
    fn sys_show_console(&mut self, level: i32, quit_on_close: qboolean);
}

/// Forwarding impl so a `&mut dyn EngineHost` (the consumer-stored form, ruling
/// 24) satisfies the `&mut impl EngineHost` bounds the §F/stringed subsystems
/// take: the dyn seam bridges into the generic API without erasing it.
impl<T: EngineHost + ?Sized> EngineHost for &mut T {
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
    ) {
        (**self).trace(
            results,
            start,
            mins,
            maxs,
            end,
            pass_entity_num,
            contentmask,
            capsule,
            trace_flags,
            use_lod,
        )
    }

    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>> {
        (**self).fs_read_file(qpath)
    }

    fn fs_free_file(&mut self, buffer: Vec<u8>) {
        (**self).fs_free_file(buffer)
    }

    fn print(&mut self, msg: &str) {
        (**self).print(msg)
    }

    fn error(&mut self, code: errorParm_t, msg: &str) -> ! {
        (**self).error(code, msg)
    }

    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize {
        (**self).vm_call(vm, callnum, args)
    }

    fn shared_memory(&mut self) -> *mut c_char {
        (**self).shared_memory()
    }

    fn flrand(&mut self, min: f32, max: f32) -> f32 {
        (**self).flrand(min, max)
    }

    fn irand(&mut self, min: i32, max: i32) -> i32 {
        (**self).irand(min, max)
    }

    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t {
        (**self).gentity(ent_num)
    }

    fn cvar_integer(&mut self, name: &str) -> i32 {
        (**self).cvar_integer(name)
    }

    fn sv_time(&mut self) -> i32 {
        (**self).sv_time()
    }

    fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool {
        (**self).fs_write_file(qpath, data)
    }

    fn model_mdxm(&mut self, model: qhandle_t) -> *mut c_void {
        (**self).model_mdxm(model)
    }

    fn model_mdxa(&mut self, model: qhandle_t) -> *mut c_void {
        (**self).model_mdxa(model)
    }

    fn cvar_register(&mut self, name: &str, default: &str, flags: i32) {
        (**self).cvar_register(name, default, flags)
    }

    fn cvar_string(&mut self, name: &str) -> String {
        (**self).cvar_string(name)
    }

    fn cvar_take_modified(&mut self, name: &str) -> bool {
        (**self).cvar_take_modified(name)
    }

    fn fs_list_files(&mut self, dir: &str, ext: &str, want_subs: bool) -> Vec<String> {
        (**self).fs_list_files(dir, ext, want_subs)
    }

    fn fs_file_is_in_pak(&mut self, qpath: &str) -> Option<i32> {
        (**self).fs_file_is_in_pak(qpath)
    }

    fn sv_shownet_entity_classname(&mut self, number: i32) -> Option<String> {
        (**self).sv_shownet_entity_classname(number)
    }

    fn sys_init(&mut self) {
        (**self).sys_init()
    }

    fn sys_quit(&mut self) -> ! {
        (**self).sys_quit()
    }

    fn sys_error(&mut self, msg: &str) -> ! {
        (**self).sys_error(msg)
    }

    fn sys_show_console(&mut self, level: i32, quit_on_close: qboolean) {
        (**self).sys_show_console(level, quit_on_close)
    }
}
