//! `EngineHostView` — the live engine-island world bundle and its
//! [`EngineHost`] implementation (host-seam restructure, user ruling
//! 2026-07-11; amends engine-fork-discovery ruling 43).
//!
//! One `&mut` path to the whole world: a host-consuming function takes
//! `view: &mut EngineHostView` as its single world parameter — `view.common`
//! for state, `view` forwarded to callees, `view.print(…)` for host services.
//! This dissolves the `&mut Common`-beside-`&mut dyn EngineHost` aliasing the
//! previous receiver-list convention could not implement live (a raw-pointer
//! host would violate the `noalias` contract on `&mut` parameters).
//!
//! Methods that need `Server`/`RenderModels` state (which qcommon cannot name
//! — the crate graph puts it below both) route through the accessor fields on
//! `Common.hooks` (the 2026-07-12 hook-table ruling, extended): the owning
//! crates install casting adapters at boot, and the slot casts stay where the
//! real types are nameable.
//!
//! Source: `docs/plans/2026-07-11-host-seam-restructure.md`

use core::ffi::{c_char, c_int};

use mp_host_interface::engine_host::EngineHost;
use mp_host_interface::mdx::mdxa::{MdxaParsed, MdxaRef, MdxaView};
use mp_host_interface::mdx::mdxm::{MdxmParsed, MdxmRef, MdxmView};
use mp_host_interface::vm_slot::VmSlot;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{fileHandle_t, qboolean, qhandle_t, vec3_t, FS_WRITE};

use crate::collision_world::CollisionWorld;
use crate::common::common::{com_printf, Common};
use crate::common::error::com_error;
use crate::common::opaque_slots::{BotLib, Client, Ghoul2System, RenderModels, RmManager, Server};
use crate::cvar_fns::{
    Cvar_FindVar, Cvar_Get, Cvar_VariableIntegerValue, Cvar_VariableString, Cvar_VariableValue,
};
use crate::files_common::{FS_FCloseFile, FS_FreeFile, FS_ListFiles, FS_ReadFile, FS_Write};
use crate::files_pc::{FS_FOpenFileByMode, FS_FileIsInPAK};
use crate::sys_net::Sys_IsLANAddress;

/// The live engine world as qcommon sees it: the two real state structs this
/// crate owns plus the type-erased slots for the islands it cannot name
/// (opaque-slot ruling, user 2026-07-12). Constructed by `mp_engine_core`'s
/// split constructor from `&mut Engine` field borrows; threaded as the single
/// world parameter of every host-consuming function.
///
/// Slot discipline (unchanged, restated): a slot cast (`server_from_slot`
/// et al.) is scoped — cast, use, drop — and never held across a call that
/// takes the view.
#[allow(non_snake_case)]
pub struct EngineHostView<'a> {
    /// cvars, cmd, cbuf, fs, net, modules, hooks — `Engine.common`.
    pub common: &'a mut Common,
    /// The collision world — `Engine.cm`.
    pub cm: &'a mut CollisionWorld,
    /// Type-erased `Engine.sv` (`mp_engine_server::Server`).
    pub sv: Server,
    /// Type-erased `Engine.cl` (`mp_engine_client::Client`; null on dedicated).
    pub cl: Client,
    /// Type-erased `Engine.bot` (`mp_engine_botlib::BotLib`).
    pub bot: BotLib,
    /// Type-erased `Engine.render_models` (`mp_renderer` `RenderModels`).
    pub rm: RenderModels,
    /// Type-erased `Engine.rmg` (`mp_engine_rmg::RmManager`).
    pub rmg: RmManager,
    /// Type-erased `Engine.g2` (`mp_engine_ghoul2::Ghoul2System`).
    pub g2: Ghoul2System,
}

impl EngineHost for EngineHostView<'_> {
    /// Raven `SV_Trace` — server-installed accessor (qcommon cannot name
    /// `Server`); the adapter casts `view.sv` and calls the real
    /// `sv_world::SV_Trace`.
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
    ) {
        let f = self
            .common
            .hooks
            .SV_Trace
            .expect("SV_Trace hook — installed by mp_engine_server at boot");
        f(
            self,
            results,
            *start,
            *mins,
            *maxs,
            *end,
            pass_entity_num,
            contentmask,
            capsule as c_int,
            trace_flags,
            use_lod,
        );
    }

    /// Raven `FS_ReadFile` + the paired `FS_FreeFile` (the buffer is copied
    /// into an owned `Vec`, then Raven's buffer is freed here — read/free
    /// collapse at the seam, ruling 24).
    /// Source: `oracle/codemp/qcommon/files.cpp:1670,1798`
    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>> {
        let mut buf: *mut () = core::ptr::null_mut();
        let len = FS_ReadFile(self, qpath, &mut buf);
        if len < 0 || buf.is_null() {
            return None;
        }
        // SAFETY: FS_ReadFile returned a live `len`-byte buffer it allocated;
        // copied whole before FS_FreeFile releases it.
        let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, len as usize) }.to_vec();
        FS_FreeFile(self.common, buf);
        Some(bytes)
    }

    /// Raven `Com_Printf` (pre-formatted at the call site, §C).
    /// Source: `oracle/codemp/qcommon/common.cpp:128`
    fn print(&mut self, msg: &str) {
        com_printf(self.common, msg);
    }

    /// Raven `Com_Error` — the receiverless panic path (STATE-Q4/DEC-08).
    /// Source: `oracle/codemp/qcommon/common.cpp:249`
    fn error(&mut self, code: errorParm_t, msg: &str) -> ! {
        com_error(code, msg.to_string())
    }

    /// Raven `VM_Call( vm, callnum, … )` — server-installed accessor: the
    /// adapter resolves `VmSlot::Gvm` to `sv.gvm` (`Cgvm` is NULL under
    /// DEDICATED and takes Raven's own NULL-vm fatal path, ruling 33b).
    /// Source: `oracle/codemp/qcommon/vm.cpp:787`
    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize {
        let f = self
            .common
            .hooks
            .VM_CallSlot
            .expect("VM_CallSlot hook — installed by mp_engine_server at boot");
        f(self, vm, callnum, args)
    }

    /// Raven `sv.mSharedMemory` — server-installed accessor.
    /// Source: `oracle/codemp/server/server.h:87`
    fn shared_memory(&mut self) -> *mut c_char {
        let f = self
            .common
            .hooks
            .SV_SharedMemory
            .expect("SV_SharedMemory hook — installed by mp_engine_server at boot");
        f(self)
    }

    /// Raven `Q_flrand` off the engine island's own LCG (ruling 21).
    /// Source: `oracle/codemp/game/q_math.c:1451`
    fn flrand(&mut self, min: f32, max: f32) -> f32 {
        self.common.qrand.flrand(min, max)
    }

    /// Raven `Q_irand` off the same LCG.
    /// Source: `oracle/codemp/game/q_math.c:1471`
    fn irand(&mut self, min: i32, max: i32) -> i32 {
        self.common.qrand.irand(min, max)
    }

    /// Raven `SV_GentityNum` — server-installed accessor.
    /// Source: `oracle/codemp/server/sv_game.cpp:54`
    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t {
        let f = self
            .common
            .hooks
            .SV_GentityNum
            .expect("SV_GentityNum hook — installed by mp_engine_server at boot");
        f(self, ent_num)
    }

    /// Raven `Cvar_VariableIntegerValue` (unregistered name reads 0).
    /// Source: `oracle/codemp/qcommon/cvar.cpp:118-124`
    fn cvar_integer(&mut self, name: &str) -> i32 {
        Cvar_VariableIntegerValue(self.common, name)
    }

    /// Raven `Cvar_VariableValue` (unregistered name reads 0.0).
    /// Source: `oracle/codemp/qcommon/cvar.cpp:105-111`
    fn cvar_value(&mut self, name: &str) -> f32 {
        Cvar_VariableValue(self.common, name)
    }

    /// Raven `svs.time` — server-installed accessor.
    /// Source: `oracle/codemp/server/server.h:211`
    fn sv_time(&mut self) -> i32 {
        let f = self
            .common
            .hooks
            .SVS_Time
            .expect("SVS_Time hook — installed by mp_engine_server at boot");
        f(self)
    }

    /// Raven's `FS_FOpenFileByMode(FS_WRITE)` + `FS_Write` + `FS_FCloseFile`
    /// sequence collapsed (ruling 36); `false` mirrors the NULL-handle open
    /// failure.
    /// Source: `oracle/codemp/server/NPCNav/navigator.cpp:670-699`
    fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool {
        let mut f: fileHandle_t = 0;
        FS_FOpenFileByMode(self, qpath, &mut f, FS_WRITE);
        if f == 0 {
            return false;
        }
        FS_Write(
            self.common,
            data.as_ptr() as *const (),
            data.len() as c_int,
            f,
        );
        FS_FCloseFile(self.common, f);
        true
    }

    /// Raven `R_GetModelByHandle( model )->mdxm` — renderer-installed accessor
    /// (G2SV-D5 / DEC-35). `None` where the loader pointer is NULL. Composes the
    /// block `view` and the parse-once `parsed` sidecar the renderer built at
    /// ingest into one Copy [`MdxmRef`].
    /// Source: `oracle/codemp/renderer/tr_local.h:1128`
    fn model_mdxm(&mut self, model: qhandle_t) -> Option<MdxmRef<'static>> {
        let f = self
            .common
            .hooks
            .R_ModelMdxm
            .expect("R_ModelMdxm hook — installed by the renderer at boot");
        let (block, parsed) = f(self, model);
        if block.is_null() {
            return None;
        }
        // A non-null block with a null sidecar is a load-path bug (the renderer
        // builds `parsed` at ingest for every block); one defined behavior.
        debug_assert!(!parsed.is_null(), "model_mdxm: block without a parsed sidecar");
        if parsed.is_null() {
            return None;
        }
        // SAFETY: DEC-35 — the one sanctioned conjure site. `block` is the
        // loader's live parsed `.glm` block and `parsed` the registry entry's
        // `MdxmParsed`; both valid until model eviction and revalidated by
        // `G2_SetupModelPointers`.
        let view = unsafe { MdxmView::from_block(block) };
        let parsed = unsafe { &*(parsed as *const MdxmParsed) };
        Some(MdxmRef { parsed, view })
    }

    /// Raven `R_GetModelByHandle( model )->mdxa` — renderer-installed accessor
    /// (G2SV-D5 / DEC-35). `None` where the loader pointer is NULL. Composes the
    /// block `view` and the parse-once `parsed` sidecar into one [`MdxaRef`].
    /// Source: `oracle/codemp/renderer/tr_local.h:1129`
    fn model_mdxa(&mut self, model: qhandle_t) -> Option<MdxaRef<'static>> {
        let f = self
            .common
            .hooks
            .R_ModelMdxa
            .expect("R_ModelMdxa hook — installed by the renderer at boot");
        let (block, parsed) = f(self, model);
        if block.is_null() {
            return None;
        }
        debug_assert!(!parsed.is_null(), "model_mdxa: block without a parsed sidecar");
        if parsed.is_null() {
            return None;
        }
        // SAFETY: DEC-35 — the one sanctioned conjure site. `block` is the
        // loader's live parsed `.gla` block and `parsed` the registry entry's
        // `MdxaParsed`; both valid until model eviction and revalidated by
        // `G2_SetupModelPointers`.
        let view = unsafe { MdxaView::from_block(block) };
        let parsed = unsafe { &*(parsed as *const MdxaParsed) };
        Some(MdxaRef { parsed, view })
    }

    /// Raven `R_GetSkinByHandle`, flattened to the per-surface
    /// `(surface, shader)` rows (server skins name-pool ruling, 2026-07-12) —
    /// renderer-installed accessor.
    /// Source: `oracle/codemp/renderer/tr_image.cpp:3342-3347`
    fn skin_surfaces(&mut self, h_skin: qhandle_t) -> Vec<(String, String)> {
        let f = self
            .common
            .hooks
            .R_SkinSurfaces
            .expect("R_SkinSurfaces hook — installed by the renderer at boot");
        f(self, h_skin)
    }

    /// Raven `Cvar_Get`'s registration side (ruling 55) — the returned
    /// `cvar_t*` collapses away; reads go through the by-name services.
    /// Source: `oracle/codemp/qcommon/cvar.cpp:188`
    fn cvar_register(&mut self, name: &str, default: &str, flags: i32) {
        Cvar_Get(self, name, default, flags);
    }

    /// Raven `Cvar_VariableString` (missing name reads `""`).
    /// Source: `oracle/codemp/qcommon/cvar.cpp:133-140`
    fn cvar_string(&mut self, name: &str) -> String {
        Cvar_VariableString(self.common, name).to_string()
    }

    /// Read-and-clear of Raven's `cvar_t->modified` (ruling 55).
    /// Source: `oracle/codemp/qcommon/stringed_ingame.cpp:1252-1259`
    fn cvar_take_modified(&mut self, name: &str) -> bool {
        let Some(h) = Cvar_FindVar(self.common, name) else {
            return false;
        };
        let var = self.common.cvar_mut(h);
        let was = var.modified;
        var.modified = false;
        was
    }

    /// Raven `FS_ListFiles` + `FS_FreeFileList` collapsed (ruling 55).
    /// `want_subs = true` has zero live callers (the ruled surface's
    /// extension); loud fatal rather than a silent divergence.
    /// Source: `oracle/codemp/qcommon/files.cpp:2174`
    fn fs_list_files(&mut self, dir: &str, ext: &str, want_subs: bool) -> Vec<String> {
        if want_subs {
            //TODO: Port fs_list_files want_subs=true (no live caller)
            // Source: docs/subsystems/stringed.md (ruling 55 surface note)
            com_error(
                errorParm_t::ERR_FATAL,
                "fs_list_files: want_subs=true unported (no live caller)".to_string(),
            );
        }
        FS_ListFiles(self, dir, ext)
    }

    /// Raven `FS_FileIsInPAK` collapsed per §C7 (ruling 59a):
    /// `Some(pure_checksum)` = the `1` path, `None` = every `-1` path.
    /// Source: `oracle/codemp/qcommon/files.cpp:1602-1659`
    fn fs_file_is_in_pak(&mut self, qpath: &str) -> Option<i32> {
        FS_FileIsInPAK(self.common, qpath)
    }

    /// Raven `MSG_ReadDeltaEntity`'s `cl_shownet` probe — server-installed
    /// accessor (ruling 56c).
    /// Source: `oracle/codemp/qcommon/msg.cpp:1268-1270`
    fn sv_shownet_entity_classname(&mut self, number: i32) -> Option<String> {
        let f = self
            .common
            .hooks
            .SV_ShownetEntityClassname
            .expect("SV_ShownetEntityClassname hook — installed by mp_engine_server at boot");
        f(self, number)
    }

    /// Raven `RE_RegisterServerModel` — renderer-installed accessor (the
    /// ghoul2-server.md gap closed, user ruling 2026-07-12).
    /// Source: `oracle/codemp/renderer/tr_model.cpp:588`
    fn model_register(&mut self, name: &str) -> qhandle_t {
        let f = self
            .common
            .hooks
            .R_RegisterServerModel
            .expect("R_RegisterServerModel hook — installed by the renderer at boot");
        f(self, name)
    }

    /// Raven unix `Sys_Init` — arch/username cvars; the input-layer tail
    /// (`in_restart`, `IN_Init`) is client-shell slice work.
    /// Source: `oracle/codemp/unix/unix_main.c:160`
    fn sys_init(&mut self) {
        crate::sys_engine::Sys_Init(self);
    }

    /// Raven unix `Sys_Quit` — `CL_Shutdown()` (null no-op on dedicated),
    /// stdin restore, `Sys_Exit(0)`.
    /// Source: `oracle/codemp/unix/unix_main.c:154-158`
    fn sys_quit(&mut self) -> ! {
        let f = self
            .common
            .hooks
            .CL_Shutdown
            .expect("CL_Shutdown hook (null-build default)");
        f(self);
        native_platform::sys_main::Sys_Exit_restore_stdin(0)
    }

    /// Raven unix `Sys_Error` — stdin restore + stderr print + exit(1); the
    /// `CL_Shutdown` call is the null no-op on dedicated.
    /// Source: `oracle/codemp/unix/unix_main.c:208-224`
    fn sys_error(&mut self, msg: &str) -> ! {
        native_platform::platform::sys_fatal_print_exit(msg)
    }

    /// No console window on the unix tree — Raven's linux build has no
    /// `Sys_ShowConsole` body (the win32 syscon is unported per-OS twin).
    /// Source: `oracle/codemp/win32/win_syscon.cpp:396` (win32 twin)
    fn sys_show_console(&mut self, _level: i32, _quit_on_close: qboolean) {}

    /// Raven unix `Sys_IsLANAddress`.
    /// Source: `oracle/codemp/unix/unix_net.c:240`
    fn is_lan_address(&mut self, adr: &netadr_t) -> bool {
        Sys_IsLANAddress(adr)
    }
}
