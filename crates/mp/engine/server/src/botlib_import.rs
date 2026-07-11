//! The botlib import slot — the state-capture dual of the game `GAME_SLOT`
//! (SEAM-D11, `qcommon/vm/trampoline.rs`).
//!
//! Raven's `botlib_import_t` is a C-shaped table of bare `extern "C"` function
//! pointers (`botlib.h:157-193`); the bot library (`mp_engine_botlib`) calls
//! them with no engine context, exactly as it calls Raven's file-static
//! `BotImport_*` functions. Those bodies reach threaded engine state
//! (`Com_Printf`/`SV_PointContents`/`Z_Malloc`/the debug-polygon pool/…) that a
//! bare `extern "C" fn` cannot carry. So — mirroring how the game slot injects
//! an opaque `ctx` the raw trampoline reads before dispatch — this module owns a
//! single armed cell (`BOTLIB_SLOT`) holding raw pointers to the `Engine`
//! islands the callbacks need, and a table of C-ABI thunks that recover those
//! islands and enter the receiver-threaded real bodies (`sv_bot.rs`,
//! `sv_world.rs`, `sv_game.rs`, `z_memman_pc.rs`, `files_*.rs`).
//!
//! `Print` is C-variadic (`fn(int, char*, ...)`) — stable Rust cannot define
//! that fn, so its slot is a C shim (`botlib_import_trampoline.c`, built by
//! `build.rs`) that formats the varargs and forwards to
//! `bot_import_print_forward` below, the botlib-import twin of
//! `game_syscall_trampoline.c`.
//!
//! Source: `oracle/codemp/server/sv_bot.cpp:264-722`

use core::cell::UnsafeCell;
use core::ffi::{c_char, c_int, c_long, c_void, CStr};

use mp_engine_qcommon::cm_load::{CM_EntityString, RenderModels};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_Write};
use mp_engine_qcommon::files_pc::{FS_FOpenFileByMode, FS_Read2, FS_Seek};
use mp_engine_qcommon::z_memman_pc::{Hunk_Alloc, Hunk_CheckMark, Z_Free, Z_Malloc, Z_MemSize};
use mp_host_interface::engine_host::EngineHost;

use mp_qshared::common::mp::botlib::botlib_import_s::botlib_import_t;
use mp_qshared::common::mp::botlib::print_type::{
    PRT_ERROR, PRT_EXIT, PRT_FATAL, PRT_MESSAGE, PRT_WARNING,
};
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::ha_pref;
use mp_qshared::shared::{fileHandle_t, fsMode_t, qtrue, vec3_t};

use crate::sv_bot::{
    BotImport_DebugLineCreate, BotImport_DebugLineDelete, BotImport_DebugLineShow,
    BotImport_DebugPolygonCreate, BotImport_DebugPolygonDelete,
};
use crate::sv_game::SV_inPVS;
use crate::sv_world::SV_PointContents;
use crate::Server;

extern "C" {
    /// The C-variadic `Print` shim (`botlib_import_trampoline.c`) assigned to the
    /// `botlib_import_t.Print` slot; it formats the varargs and calls
    /// `bot_import_print_forward`. Declared here so the table can take its
    /// address — the botlib-import twin of `game_syscall_trampoline`.
    fn bot_import_print_trampoline(r#type: c_int, fmt: *mut c_char, ...);
}

/// The `Engine` islands the `BotImport_*` callbacks need, captured as raw
/// pointers when `SV_BotInitBotLib` arms the slot — the botlib-import analogue
/// of the game slot's injected `ctx` (`engine_slot.rs`). Each callback recovers
/// the islands it needs and enters the receiver-threaded real body.
struct BotImportCtx {
    common: *mut Common,
    cm: *mut CollisionWorld,
    sv: *mut Server,
    rm: *mut RenderModels,
    host: *mut dyn EngineHost,
}

struct BotImportSlotCell(UnsafeCell<Option<BotImportCtx>>);

// SAFETY (Sync only): armed once at server init and read single-threaded per
// Raven's contract — the same argument as the game slot's `GameSlotCell`.
unsafe impl Sync for BotImportSlotCell {}

/// The armed botlib-import cell — the §D11 engine-side static exemption twin of
/// the game slot's `GAME_SLOT`, one for the (single) botlib.
static BOTLIB_SLOT: BotImportSlotCell = BotImportSlotCell(UnsafeCell::new(None));

/// Read the armed slot; every `BotImport_*` thunk enters through here.
fn ctx() -> &'static BotImportCtx {
    // SAFETY: a callback fires only after `SV_BotInitBotLib` armed the slot;
    // module dispatch is single-threaded, so this shared read cannot race an
    // arm or another callback.
    unsafe {
        (*BOTLIB_SLOT.0.get())
            .as_ref()
            .expect("botlib import slot armed before any botlib callback")
    }
}

/// Arm the botlib-import slot with the `Engine` islands `SV_BotInitBotLib` holds
/// — the botlib-import twin of `arm_game_slot`. Captures raw pointers to
/// long-lived `Engine` fields; the `&mut` borrows end when this returns.
pub fn arm_botlib_slot(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    // SAFETY: single-threaded server init; the captured pointers alias
    // long-lived `Engine` island fields that outlive every botlib callback, and
    // no callback can race this arm. The `host` trait-object pointer's borrow
    // lifetime is erased to `'static` for slot storage (the game slot's `ctx`
    // precedent: injected engine state read back single-threaded at the seam).
    let host: *mut dyn EngineHost = unsafe {
        core::mem::transmute::<*mut dyn EngineHost, *mut (dyn EngineHost + 'static)>(
            host as *mut dyn EngineHost,
        )
    };
    let ctx = BotImportCtx {
        common: common as *mut Common,
        cm: cm as *mut CollisionWorld,
        sv: sv as *mut Server,
        rm: rm as *mut RenderModels,
        host,
    };
    // SAFETY: single-threaded arm; see the cell's `Sync` note.
    unsafe {
        *BOTLIB_SLOT.0.get() = Some(ctx);
    }
}

/// The `Engine` islands' arming-time `iMaxBOTLIBMem` (`sv_bot.cpp:678`) — the
/// simulated zone ceiling `bot_Z_AvailableMemory` reports against.
const I_MAX_BOTLIB_MEM: c_int = 8 * 1024 * 1024;

/// Raven `BotImport_Print` receiver — the C shim (`botlib_import_trampoline.c`)
/// has already `vsnprintf`'d the varargs; this half owns the `PRT_*` switch and
/// the `Com_Printf`/`Com_Error` calls (`sv_bot.cpp:275-303`). `extern "C-unwind"`
/// so a `PRT_EXIT` `com_error` panic unwinds back through the shim's C frame.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:267-304`
#[no_mangle]
pub extern "C-unwind" fn bot_import_print_forward(r#type: c_int, str: *const c_char) {
    let common = unsafe { &mut *ctx().common };
    // SAFETY: the shim passes its NUL-terminated `char str[2048]`.
    let text = unsafe { CStr::from_ptr(str) }.to_string_lossy();
    match r#type {
        PRT_MESSAGE => com_printf(common, text.as_ref()),
        PRT_WARNING => com_printf(common, &format!("^3Warning: {text}")),
        PRT_ERROR => com_printf(common, &format!("^1Error: {text}")),
        PRT_FATAL => com_printf(common, &format!("^1Fatal: {text}")),
        PRT_EXIT => com_error(errorParm_t::ERR_DROP, format!("^1Exit: {text}")),
        _ => com_printf(common, "unknown print type\n"),
    }
}

/// Raven `BotImport_PointContents` (`sv_bot.cpp:360-362`).
extern "C" fn bot_import_point_contents(point: *mut vec3_t) -> c_int {
    let ctx = ctx();
    SV_PointContents(
        unsafe { &mut *ctx.common },
        unsafe { &mut *ctx.cm },
        unsafe { &mut *ctx.sv },
        unsafe { *point },
        -1,
    )
}

/// Raven `BotImport_inPVS` (`sv_bot.cpp:369-371`).
extern "C" fn bot_import_in_pvs(p1: *mut vec3_t, p2: *mut vec3_t) -> c_int {
    let ctx = ctx();
    SV_inPVS(unsafe { &mut *ctx.cm }, unsafe { *p1 }, unsafe { *p2 }) as c_int
}

/// Raven `BotImport_BSPEntityData` (`sv_bot.cpp:378-380`).
extern "C" fn bot_import_bsp_entity_data() -> *mut c_char {
    CM_EntityString(unsafe { &mut *ctx().cm })
}

/// Raven `BotImport_GetMemory` — `Z_Malloc(size, TAG_BOTLIB, qtrue)`
/// (`sv_bot.cpp:438-443`).
extern "C" fn bot_import_get_memory(size: c_int) -> *mut c_void {
    let ctx = ctx();
    Z_Malloc(
        unsafe { &mut *ctx.common },
        unsafe { &mut *ctx.cm },
        unsafe { &mut *ctx.rm },
        unsafe { &mut *ctx.host },
        size,
        memtag_t::TAG_BOTLIB,
        qtrue,
        0,
    ) as *mut c_void
}

/// Raven `BotImport_FreeMemory` — `Z_Free(ptr)` (`sv_bot.cpp:450-452`).
extern "C" fn bot_import_free_memory(ptr: *mut c_void) {
    Z_Free(unsafe { &mut *ctx().common }, ptr as *mut ());
}

/// Raven `bot_Z_AvailableMemory` — simulated zone ceiling minus the live
/// `TAG_BOTLIB` usage (`sv_bot.cpp:672-677`).
extern "C" fn bot_import_available_memory() -> c_int {
    I_MAX_BOTLIB_MEM - Z_MemSize(unsafe { &mut *ctx().common }, memtag_t::TAG_BOTLIB)
}

/// Raven `BotImport_HunkAlloc` — guard against a set hunk mark, then
/// `Hunk_Alloc(size, h_high)` (`sv_bot.cpp:459-464`).
extern "C" fn bot_import_hunk_alloc(size: c_int) -> *mut c_void {
    let ctx = ctx();
    let common = unsafe { &mut *ctx.common };
    if Hunk_CheckMark(common) == qtrue {
        com_error(
            errorParm_t::ERR_DROP,
            "SV_Bot_HunkAlloc: Alloc with marks already set\n".to_string(),
        );
    }
    Hunk_Alloc(
        common,
        unsafe { &mut *ctx.cm },
        unsafe { &mut *ctx.rm },
        unsafe { &mut *ctx.host },
        size,
        ha_pref::h_high,
    ) as *mut c_void
}

/// Raven `FS_FOpenFileByMode` (installed as `FS_FOpenFile`, `sv_bot.cpp:707`).
extern "C" fn bot_import_fs_fopen_file(
    qpath: *const c_char,
    file: *mut fileHandle_t,
    mode: fsMode_t,
) -> c_int {
    let ctx = ctx();
    FS_FOpenFileByMode(
        unsafe { &mut *ctx.common },
        unsafe { &mut *ctx.cm },
        unsafe { &mut *ctx.rm },
        unsafe { &mut *ctx.host },
        qpath,
        file,
        mode,
    )
}

/// Raven `FS_Read2` (installed as `FS_Read`, `sv_bot.cpp:708`).
extern "C" fn bot_import_fs_read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int {
    FS_Read2(unsafe { &mut *ctx().common }, buffer as *mut (), len, f)
}

/// Raven `FS_Write` (installed as `FS_Write`, `sv_bot.cpp:709`).
extern "C" fn bot_import_fs_write(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int {
    FS_Write(unsafe { &mut *ctx().common }, buffer as *const (), len, f)
}

/// Raven `FS_FCloseFile` (`sv_bot.cpp:710`).
extern "C" fn bot_import_fs_fclose_file(f: fileHandle_t) {
    FS_FCloseFile(unsafe { &mut *ctx().common }, f);
}

/// Raven `FS_Seek` (`sv_bot.cpp:711`).
extern "C" fn bot_import_fs_seek(f: fileHandle_t, offset: c_long, origin: c_int) -> c_int {
    let ctx = ctx();
    FS_Seek(
        unsafe { &mut *ctx.common },
        unsafe { &mut *ctx.cm },
        unsafe { &mut *ctx.rm },
        unsafe { &mut *ctx.host },
        f,
        offset,
        origin,
    )
}

/// Raven `BotImport_DebugLineCreate` (`sv_bot.cpp:525-528`).
extern "C" fn bot_import_debug_line_create() -> c_int {
    BotImport_DebugLineCreate(unsafe { &mut *ctx().sv })
}

/// Raven `BotImport_DebugLineDelete` (`sv_bot.cpp:535-537`).
extern "C" fn bot_import_debug_line_delete(line: c_int) {
    BotImport_DebugLineDelete(unsafe { &mut *ctx().sv }, line);
}

/// Raven `BotImport_DebugLineShow` (`sv_bot.cpp:544-570`).
extern "C" fn bot_import_debug_line_show(
    line: c_int,
    start: *mut vec3_t,
    end: *mut vec3_t,
    color: c_int,
) {
    BotImport_DebugLineShow(
        unsafe { &mut *ctx().sv },
        line,
        unsafe { *start },
        unsafe { *end },
        color,
    );
}

/// Raven `BotImport_DebugPolygonCreate` (`sv_bot.cpp:471-491`).
extern "C" fn bot_import_debug_polygon_create(
    color: c_int,
    numPoints: c_int,
    points: *mut vec3_t,
) -> c_int {
    BotImport_DebugPolygonCreate(unsafe { &mut *ctx().sv }, color, numPoints, points as *const vec3_t)
}

/// Raven `BotImport_DebugPolygonDelete` (`sv_bot.cpp:514-518`).
extern "C" fn bot_import_debug_polygon_delete(id: c_int) {
    BotImport_DebugPolygonDelete(unsafe { &mut *ctx().sv }, id);
}

/// Build the `botlib_import_t` table Raven fills in `SV_BotInitBotLib`
/// (`sv_bot.cpp:691-720`). The slot must already be armed (`arm_botlib_slot`).
///
/// Four fields stay `None`, deferred to a later slice, because their real bodies
/// reach engine state not threaded to `SV_BotInitBotLib`'s call site:
/// - `Trace`/`EntityTrace` — `SV_Trace`/`SV_ClipToEntity` need the `RmManager`
///   and `Ghoul2System` islands, which `SV_Init`/`SV_BotInitBotLib` do not
///   receive.
/// - `BotClientCommand` — `SV_ExecuteClientCommand` needs the `RmManager`
///   island, likewise not threaded here.
/// - `BSPModelMinsMaxsOrigin` — `CM_ModelBounds` is ported with the by-value
///   out-param convention and does not propagate the computed bounds back, so a
///   faithful body cannot be assembled without touching that seam.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:691-720`
pub fn botlib_import_table() -> botlib_import_t {
    botlib_import_t {
        Print: Some(bot_import_print_trampoline),
        Trace: None,
        EntityTrace: None,
        PointContents: Some(bot_import_point_contents),
        inPVS: Some(bot_import_in_pvs),
        BSPEntityData: Some(bot_import_bsp_entity_data),
        BSPModelMinsMaxsOrigin: None,
        BotClientCommand: None,
        GetMemory: Some(bot_import_get_memory),
        FreeMemory: Some(bot_import_free_memory),
        AvailableMemory: Some(bot_import_available_memory),
        HunkAlloc: Some(bot_import_hunk_alloc),
        FS_FOpenFile: Some(bot_import_fs_fopen_file),
        FS_Read: Some(bot_import_fs_read),
        FS_Write: Some(bot_import_fs_write),
        FS_FCloseFile: Some(bot_import_fs_fclose_file),
        FS_Seek: Some(bot_import_fs_seek),
        DebugLineCreate: Some(bot_import_debug_line_create),
        DebugLineDelete: Some(bot_import_debug_line_delete),
        DebugLineShow: Some(bot_import_debug_line_show),
        DebugPolygonCreate: Some(bot_import_debug_polygon_create),
        DebugPolygonDelete: Some(bot_import_debug_polygon_delete),
    }
}
