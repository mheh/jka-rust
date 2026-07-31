#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `common.cpp` — the engine's top-level frame/init/config/event-loop glue.
//!
//! DESTINATION NOTE: `common.cpp`'s stem collides with the existing `common/`
//! directory module, so this file lands at the `_fns` escape per
//! `_PREAMBLE.md`'s destination rule.
//!
//! Source: `oracle/codemp/qcommon/common.cpp`

use core::ffi::{c_char, c_int, c_void, CStr};

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::qtime::qtime_t;
use mp_qshared::shared::cvar::{
    CVAR_ARCHIVE, CVAR_CHEAT, CVAR_INIT, CVAR_LATCH, CVAR_ROM, CVAR_SERVERINFO, CVAR_SYSTEMINFO,
    CVAR_TEMP, CVAR_USER_CREATED,
};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue, CPUSTRING, FS_READ, Q3_VERSION};
use native_string::cstr::buf_to_string;
use native_string::latin1_to_string;
use native_string::q_string::{Q_strcmp, Q_stricmp, Q_stricmpn};

use crate::collision_world::CollisionWorld;
use crate::common::com_printf;
use crate::common::common_consts::{MAX_CONSOLE_LINES, MAX_PUSHED_EVENTS};
use crate::common::engine_host_view::EngineHostView;
use crate::common::ComError;
use crate::common::Common;
use crate::gp2::generic_parser2::GenericParser2;
use crate::qcommon::net_limits::MAX_MSGLEN;
use crate::qcommon::sys_event_t::sysEvent_t;
use crate::qcommon::sys_event_type_t::sysEventType_t;

#[allow(dead_code)]
use crate::cm_load::RmManager;
// `Server`/`Client`/`BotLib` are type-erased receiver slots (real types live in
// the above-tier engine crates); re-exported at this historical home, defined
// once in `common::opaque_slots`.
pub use crate::common::opaque_slots::BotLib;
pub use crate::common::opaque_slots::Client;

// Sweep: extern forward-declares eliminated. Genuinely-unported callees
// referenced at their canonical future homes (cvar.cpp/cmd/files/zone/stringed
// /msg). This file's own not-yet-ported `Com_*` (common.cpp subject) collapse
// to their home; the `SV_*`/`CL_*`/`Key_*` engine entrypoints sit across the
// server/client cycle seam with no importable home — left bare; reported.
use crate::cm_load::CM_ClearMap;
use crate::cmd::Cmd_AddCommand;
use crate::cmd_common::{Cbuf_AddText, Cmd_Argc, Cmd_Argv, Cmd_TokenizeString};
use crate::cvar_fns::{Cvar_Get, Cvar_Init, Cvar_Set, Cvar_WriteVariables};
use crate::files_common::{
    FS_FCloseFile, FS_FOpenFileRead, FS_FOpenFileWrite, FS_PureServerSetLoadedPaks, FS_Read,
    FS_Shutdown, FS_Write,
};
use crate::msg::{MSG_Init, MSG_shutdownHuffman};
use crate::stringed::api::SE_Init;
use crate::sys_engine::Sys_GetEvent;
use crate::z_memman_pc::{Z_Free, Z_Malloc};
use mp_qshared::common::mp::qcommon::tags::memtag_t;

/// Raven `Com_DPrintf` — a `Com_Printf` that only shows up if the `developer`
/// cvar is set. Engine callers pre-render the format through Rust `format!`, so
/// this takes the already-rendered `&str` and forwards it to `com_printf` after
/// the developer gate.
///
/// Source: `oracle/codemp/qcommon/common.cpp:210-224`
pub fn Com_DPrintf(common: &mut Common, msg: &str) {
    // don't confuse non-developers with techie stuff...
    if common.com_developer.is_none() || common.cvar(common.com_developer).integer == 0 {
        return;
    }
    com_printf(common, msg);
}

/// `Com_BeginRedirect`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:96-105`
pub fn Com_BeginRedirect(
    common: &mut Common,
    buffer: *mut c_char,
    buffersize: c_int,
    flush: *mut *mut c_void,
) {
    let _ = common;
    if buffer.is_null() || buffersize == 0 || flush.is_null() {
        return;
    }
    common.rd_buffer = buffer;
    common.rd_buffersize = buffersize;
    common.rd_flush = flush as *mut extern "C" fn(*mut c_char);
    unsafe {
        *common.rd_buffer = 0;
    }
}

/// `Com_EndRedirect`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:107-116`
pub fn Com_EndRedirect(common: &mut Common) {
    if !common.rd_flush.is_null() {
        unsafe {
            (*common.rd_flush)(common.rd_buffer);
        }
    }
    common.rd_buffer = core::ptr::null_mut();
    common.rd_buffersize = 0;
    common.rd_flush = core::ptr::null_mut();
}

/// `Com_OPrintf`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:226-239`
pub fn Com_OPrintf(msg: &str) {
    print!("{msg}");
}

/// `Com_ParseCommandLine`. Raven's in-place NUL-splitting of the command-line
/// buffer becomes owned segments; the `MAX_CONSOLE_LINES` cap is kept
/// (including its quirk: once full, the last line keeps the remaining
/// separators verbatim).
///
/// Source: `oracle/codemp/qcommon/common.cpp:397-414`
pub fn Com_ParseCommandLine(common: &mut Common, commandLine: &str) {
    common.com_consoleLines.clear();

    let mut rest = commandLine;
    // look for a + seperating character
    // if commandLine came from a file, we might have real line seperators
    while common.com_consoleLines.len() < MAX_CONSOLE_LINES - 1 {
        match rest.find(['+', '\n']) {
            Some(i) => {
                common.com_consoleLines.push(rest[..i].to_string());
                rest = &rest[i + 1..];
            }
            None => break,
        }
    }
    common.com_consoleLines.push(rest.to_string());
}

/// `Com_HashKey`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:697-706`
pub fn Com_HashKey(string: *mut c_char, maxlen: c_int) -> c_int {
    let mut hash: c_int = 0;
    unsafe {
        let mut i = 0;
        while i < maxlen && *string.offset(i as isize) != 0 {
            hash = hash.wrapping_add((*string.offset(i as isize) as c_int).wrapping_mul(119 + i));
            i += 1;
        }
    }
    hash ^ (hash >> 10) ^ (hash >> 20)
}

/// `Com_RealTime`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:713-733`
pub fn Com_RealTime(qtime: *mut qtime_t) -> c_int {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    if qtime.is_null() {
        return now as c_int;
    }
    // §19: uninitialized-local UB avoided — libc::localtime_r fills a
    // zero-initialized tm.
    unsafe {
        let t = now as libc::time_t;
        let mut tms: libc::tm = core::mem::zeroed();
        if !libc::localtime_r(&t, &mut tms).is_null() {
            (*qtime).tm_sec = tms.tm_sec;
            (*qtime).tm_min = tms.tm_min;
            (*qtime).tm_hour = tms.tm_hour;
            (*qtime).tm_mday = tms.tm_mday;
            (*qtime).tm_mon = tms.tm_mon;
            (*qtime).tm_year = tms.tm_year;
            (*qtime).tm_wday = tms.tm_wday;
            (*qtime).tm_yday = tms.tm_yday;
            (*qtime).tm_isdst = tms.tm_isdst;
        }
    }
    now as c_int
}

/// `Com_InitPushEvent`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:834-842`
pub fn Com_InitPushEvent(common: &mut Common) {
    // clear the static buffer array
    // this requires SE_NONE to be accepted as a valid but NOP event
    for ev in common.com_pushedEvents.iter_mut() {
        *ev = unsafe { core::mem::zeroed() };
    }
    // reset counters while we are at it
    // beware: GetEvent might still return an SE_NONE from the buffer
    common.com_pushedEventsHead = 0;
    common.com_pushedEventsTail = 0;
}

/// `Com_Crash_f`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1097-1099`
// Raven deliberately writes through a null pointer to force a crash; keep that
// faithful behavior and silence the deny-by-default `deref_nullptr` lint.
#[allow(deref_nullptr)]
pub fn Com_Crash_f() {
    unsafe {
        *(0 as *mut c_int) = 0x1234_5678;
    }
}

/// `Com_Memcpy`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1815-1818`
pub fn Com_Memcpy(dest: *mut (), src: *const (), count: usize) {
    // §19: Raven calls memcpy with src == dest (FS_FOpenFileRead's
    // `Com_Memcpy(zfi, pak->handle, …)` when the handle aliases the pak) —
    // overlap UB every libc tolerates; `ptr::copy` (memmove) is the defined
    // equivalent and identical for all non-overlapping calls.
    unsafe {
        core::ptr::copy(src as *const u8, dest as *mut u8, count);
    }
}

/// `Com_Memset`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1820-1823`
pub fn Com_Memset(dest: *mut (), val: c_int, count: usize) {
    unsafe {
        core::ptr::write_bytes(dest as *mut u8, val as u8, count);
    }
}

/// `Q_acos`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:2143-2155`
pub fn Q_acos(c: f32) -> f32 {
    let angle = (c as f64).acos();
    if angle > std::f64::consts::PI {
        return std::f64::consts::PI as f32;
    }
    if angle < -std::f64::consts::PI {
        return std::f64::consts::PI as f32;
    }
    angle as f32
}

/// `Q_asin`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:2157-2170`
pub fn Q_asin(c: f32) -> f32 {
    let angle = (c as f64).asin();
    if angle > std::f64::consts::PI {
        return std::f64::consts::PI as f32;
    }
    if angle < -std::f64::consts::PI {
        return std::f64::consts::PI as f32;
    }
    angle as f32
}

/// `Com_SafeMode`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:425-437`
pub fn Com_SafeMode(common: &mut Common) -> qboolean {
    for i in 0..common.com_consoleLines.len() {
        let line = common.com_consoleLines[i].clone();
        Cmd_TokenizeString(common, &line);
        let argv0 = Cmd_Argv(common, 0);
        if Q_stricmp(argv0, "safe") == 0 || Q_stricmp(argv0, "cvar_restart") == 0 {
            common.com_consoleLines[i].clear();
            return qtrue;
        }
    }
    qfalse
}

// Raven's `Com_StringContains`/`Com_Filter`/`Com_FilterPath` family
// (common.cpp:551-690) lives in `native_string::filter`; pointer-holding call
// sites (cvar_fns/cmd_pc/files_common) convert with `CStr` at the site and
// call the `Bytes` canonicals directly.

/// `Com_ParseTextFileDestroy`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:2204-2207`
pub fn Com_ParseTextFileDestroy(parser: &mut GenericParser2) {
    parser.clean();
}

/// `Com_Quit_f`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:356-365`
pub fn Com_Quit_f(view: &mut EngineHostView) {
    // don't try to shutdown if we are in a recursive error
    if !view.common.error.entered {
        let sv_shutdown = view
            .common
            .hooks
            .SV_Shutdown
            .expect("SV_Shutdown hook — installed by mp_engine_server at boot");
        sv_shutdown(view, "Server quit\n");
        let hook_fn = view.common.hooks.CL_Shutdown.expect("CL_Shutdown hook");
        hook_fn(view);
        Com_Shutdown(view.common, view.cm, &mut view.rmg);
        FS_Shutdown(view.common, qtrue);
    }
    view.sys_quit();
}

/// `Com_StartupVariable` (Raven's nullable `match` becomes `Option`: `None` =
/// apply every `set` line).
///
/// Source: `oracle/codemp/qcommon/common.cpp:451-470`
pub fn Com_StartupVariable(view: &mut EngineHostView, r#match: Option<&str>) {
    for i in 0..view.common.com_consoleLines.len() {
        let line = view.common.com_consoleLines[i].clone();
        Cmd_TokenizeString(view.common, &line);
        if Q_strcmp(Cmd_Argv(view.common, 0), "set") != 0 {
            continue;
        }
        let s = Cmd_Argv(view.common, 1).to_owned();
        if r#match.map_or(true, |m| Q_strcmp(&s, m) == 0) {
            let arg2 = Cmd_Argv(view.common, 2).to_owned();
            Cvar_Set(view, &s, &arg2);
            let cv = Cvar_Get(view, &s, "", 0);
            view.common.cvar_mut(cv).flags |= CVAR_USER_CREATED;
            // com_consoleLines[i] = 0;
        }
    }
}

/// `Com_AddStartupCommands`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:484-504`
pub fn Com_AddStartupCommands(common: &mut Common) -> qboolean {
    let mut added = qfalse;
    // quote every token, so args with semicolons can work
    for i in 0..common.com_consoleLines.len() {
        let line = common.com_consoleLines[i].clone();
        if line.is_empty() {
            continue;
        }
        // set commands won't override menu startup
        if Q_stricmpn(&line, "set", 3) != 0 {
            added = qtrue;
        }
        Cbuf_AddText(common, &line);
        Cbuf_AddText(common, "\n");
    }
    added
}

/// `Info_Print`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:509-549`
pub fn Info_Print(common: &mut Common, s: *const c_char) {
    let mut key = [0 as c_char; 512];
    let mut value = [0 as c_char; 512];
    unsafe {
        let mut s = s;
        if *s == b'\\' as c_char {
            s = s.add(1);
        }
        while *s != 0 {
            let mut o = 0usize;
            while *s != 0 && *s != b'\\' as c_char {
                key[o] = *s;
                o += 1;
                s = s.add(1);
            }

            let l = o;
            if l < 20 {
                Com_Memset(key.as_mut_ptr().add(o) as *mut (), b' ' as c_int, 20 - l);
                key[20] = 0;
            } else {
                key[o] = 0;
            }
            crate::common::com_printf(common, &c_str_to_string(key.as_ptr()));

            if *s == 0 {
                crate::common::com_printf(common, "MISSING VALUE\n");
                return;
            }

            let mut o = 0usize;
            s = s.add(1);
            while *s != 0 && *s != b'\\' as c_char {
                value[o] = *s;
                o += 1;
                s = s.add(1);
            }
            value[o] = 0;

            if *s != 0 {
                s = s.add(1);
            }
            crate::common::com_printf(common, &format!("{}\n", c_str_to_string(value.as_ptr())));
        }
    }
}

/// `Com_GetEvent`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:881-887`
pub fn Com_GetEvent(view: &mut EngineHostView) -> sysEvent_t {
    if view.common.com_pushedEventsHead > view.common.com_pushedEventsTail {
        view.common.com_pushedEventsTail += 1;
        return view.common.com_pushedEvents
            [((view.common.com_pushedEventsTail - 1) & (MAX_PUSHED_EVENTS as i32 - 1)) as usize];
    }
    Com_GetRealEvent(view)
}

/// Raven `Com_GetRealEvent` — either read the next event from the system or
/// (journal mode 2) replay it from the journal file, writing it out in journal
/// mode 1.
///
/// Source: `oracle/codemp/qcommon/common.cpp:789-825`
pub fn Com_GetRealEvent(view: &mut EngineHostView) -> sysEvent_t {
    let mut ev: sysEvent_t = unsafe { core::mem::zeroed() };

    // either get an event from the system or the journal file
    if view.common.cvar(view.common.com_journal).integer == 2 {
        let r = FS_Read(
            view.common,
            &mut ev as *mut sysEvent_t as *mut (),
            core::mem::size_of::<sysEvent_t>() as c_int,
            view.common.com_journalFile,
        );
        if r != core::mem::size_of::<sysEvent_t>() as c_int {
            crate::common::com_error(
                errorParm_t::ERR_FATAL,
                "Error reading from journal file".to_string(),
            );
        }
        if ev.evPtrLength != 0 {
            ev.evPtr = Z_Malloc(view, ev.evPtrLength, memtag_t::TAG_EVENT, qtrue, 4) as *mut c_void;
            let r = FS_Read(
                view.common,
                ev.evPtr as *mut (),
                ev.evPtrLength,
                view.common.com_journalFile,
            );
            if r != ev.evPtrLength {
                crate::common::com_error(
                    errorParm_t::ERR_FATAL,
                    "Error reading from journal file".to_string(),
                );
            }
        }
    } else {
        ev = Sys_GetEvent(view);

        // write the journal value out if needed
        if view.common.cvar(view.common.com_journal).integer == 1 {
            let r = FS_Write(
                view.common,
                &ev as *const sysEvent_t as *const (),
                core::mem::size_of::<sysEvent_t>() as c_int,
                view.common.com_journalFile,
            );
            if r != core::mem::size_of::<sysEvent_t>() as c_int {
                crate::common::com_error(
                    errorParm_t::ERR_FATAL,
                    "Error writing to journal file".to_string(),
                );
            }
            if ev.evPtrLength != 0 {
                let r = FS_Write(
                    view.common,
                    ev.evPtr as *const (),
                    ev.evPtrLength,
                    view.common.com_journalFile,
                );
                if r != ev.evPtrLength {
                    crate::common::com_error(
                        errorParm_t::ERR_FATAL,
                        "Error writing to journal file".to_string(),
                    );
                }
            }
        }
    }

    ev
}

/// Raven `Com_PushEvent` — push an event onto the `com_pushedEvents` ring,
/// warning (once per burst) and dropping the oldest on overflow.
///
/// Source: `oracle/codemp/qcommon/common.cpp:850-874`
pub fn Com_PushEvent(common: &mut Common, event: *mut sysEvent_t) {
    let ev_idx = (common.com_pushedEventsHead & (MAX_PUSHED_EVENTS as i32 - 1)) as usize;

    if common.com_pushedEventsHead - common.com_pushedEventsTail >= MAX_PUSHED_EVENTS as i32 {
        // don't print the warning constantly, or it can give time for more...
        if common.com_pushevent_printed_warning == 0 {
            common.com_pushevent_printed_warning = qtrue as c_int;
            crate::common::com_printf(common, "WARNING: Com_PushEvent overflow\n");
        }

        if !common.com_pushedEvents[ev_idx].evPtr.is_null() {
            Z_Free(common, common.com_pushedEvents[ev_idx].evPtr as *mut ());
        }
        common.com_pushedEventsTail += 1;
    } else {
        common.com_pushevent_printed_warning = qfalse as c_int;
    }

    common.com_pushedEvents[ev_idx] = unsafe { *event };
    common.com_pushedEventsHead += 1;
}

/// Raven `Com_Milliseconds` — pump events until a null (current-time) event,
/// returning its timestamp.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1028-1041`
pub fn Com_Milliseconds(view: &mut EngineHostView) -> c_int {
    let mut ev: sysEvent_t;

    // get events and push them until we get a null event with the current time
    loop {
        ev = Com_GetRealEvent(view);
        if !matches!(ev.evType, sysEventType_t::SE_NONE) {
            Com_PushEvent(view.common, &mut ev);
        }
        if matches!(ev.evType, sysEventType_t::SE_NONE) {
            break;
        }
    }

    // Engine-referee seed pin (sv_referee.rs): when armed (set just before
    // `SV_InitGameProgs`), the next `Com_Milliseconds` — the GAME_INIT
    // `randomSeed` read in `SV_InitGameVM` — returns the pinned seed instead of
    // wall-clock so record and replay drive `srand(randomSeed)` identically. A
    // `0` pin keeps the natural value; either way the value used is captured for
    // the tape header. Disarms after the one call; all other callers unaffected.
    if view.common.ref_seed_arm {
        view.common.ref_seed_arm = false;
        let used = if view.common.ref_seed_pin != 0 {
            view.common.ref_seed_pin
        } else {
            ev.evTime
        };
        view.common.ref_seed_used = used;
        return used;
    }

    ev.evTime
}

/// Raven `Com_Shutdown` — clear the collision map, close the log/journal files
/// and shut down the MSG Huffman tables.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1785-1810`
pub fn Com_Shutdown(common: &mut Common, cm: &mut CollisionWorld, rmg: &mut RmManager) {
    CM_ClearMap(cm, rmg);

    if common.logfile != 0 {
        FS_FCloseFile(common, common.logfile);
        common.logfile = 0;
        common.cvar_mut(common.com_logfile).integer = 0; // don't open up the log file again!!
    }

    if common.com_journalFile != 0 {
        FS_FCloseFile(common, common.com_journalFile);
        common.com_journalFile = 0;
    }

    MSG_shutdownHuffman();
}

/// `Com_Error_f`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1053-1059`
pub fn Com_Error_f(common: &mut Common) {
    if crate::cmd_common::Cmd_Argc(common) > 1 {
        crate::common::com_error(errorParm_t::ERR_DROP, "Testing drop error".to_string());
    } else {
        crate::common::com_error(errorParm_t::ERR_FATAL, "Testing fatal error".to_string());
    }
}

/// `Com_Freeze_f`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1070-1088`
pub fn Com_Freeze_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "freeze <seconds>\n");
        return;
    }
    let s: f32 = Cmd_Argv(view.common, 1).trim().parse().unwrap_or(0.0);

    let start = Com_Milliseconds(view);

    loop {
        let now = Com_Milliseconds(view);
        if (now - start) as f32 * 0.001 > s {
            break;
        }
    }
}

/// `Com_ModifyMsec`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1534-1578`
pub fn Com_ModifyMsec(common: &mut Common, mut msec: c_int) -> c_int {
    let clamp_time;

    //
    // modify time for debugging values
    //
    {
        if common.cvar(common.com_fixedtime).integer != 0 {
            msec = common.cvar(common.com_fixedtime).integer;
        } else if common.cvar(common.com_timescale).value != 0.0 {
            msec = (msec as f32 * common.cvar(common.com_timescale).value) as c_int;
        } else if common.cvar(common.com_cameraMode).integer != 0 {
            msec = (msec as f32 * common.cvar(common.com_timescale).value) as c_int;
        }

        // don't let it scale below 1 msec
        if msec < 1 && common.cvar(common.com_timescale).value != 0.0 {
            msec = 1;
        }

        if common.cvar(common.com_dedicated).integer != 0 {
            // dedicated servers don't want to clamp for a much longer
            // period, because it would mess up all the client's views
            // of time.
            if msec > 500 {
                crate::common::com_printf(
                    common,
                    &format!("Hitch warning: {msec} msec frame time\n"),
                );
            }
            clamp_time = 5000;
        } else if common.cvar(common.com_sv_running).integer == 0 {
            // clients of remote servers do not want to clamp time, because
            // it would skew their view of the server's time temporarily
            clamp_time = 5000;
        } else {
            // for local single player gaming
            // we may want to clamp the time to prevent players from
            // flying off edges when something hitches.
            clamp_time = 200;
        }
    }

    if msec > clamp_time {
        msec = clamp_time;
    }

    msec
}

/// `Com_InitJournaling`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:759-782`
pub fn Com_InitJournaling(view: &mut EngineHostView) {
    Com_StartupVariable(view, Some("journal"));
    view.common.com_journal = Some(Cvar_Get(view, "journal", "0", CVAR_INIT));
    {
        if view.common.cvar(view.common.com_journal).integer == 0 {
            return;
        }

        if view.common.cvar(view.common.com_journal).integer == 1 {
            crate::common::com_printf(view.common, "Journaling events\n");
            view.common.com_journalFile = FS_FOpenFileWrite(view.common, "journal.dat");
            view.common.com_journalDataFile = FS_FOpenFileWrite(view.common, "journaldata.dat");
        } else if view.common.cvar(view.common.com_journal).integer == 2 {
            crate::common::com_printf(view.common, "Replaying journaled events\n");
            let mut jf = view.common.com_journalFile;
            FS_FOpenFileRead(view, "journal.dat", &mut jf, true);
            view.common.com_journalFile = jf;
            let mut jdf = view.common.com_journalDataFile;
            FS_FOpenFileRead(view, "journaldata.dat", &mut jdf, true);
            view.common.com_journalDataFile = jdf;
        }

        if view.common.com_journalFile == 0 || view.common.com_journalDataFile == 0 {
            Cvar_Set(view, "com_journal", "0");
            view.common.com_journalFile = 0;
            view.common.com_journalDataFile = 0;
            crate::common::com_printf(view.common, "Couldn't open journal files\n");
        }
    }
}

/// `Com_WriteConfigToFile`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1446-1461`
pub fn Com_WriteConfigToFile(view: &mut EngineHostView, filename: &str) {
    let f = FS_FOpenFileWrite(view.common, filename);
    if f == 0 {
        com_printf(view.common, &format!("Couldn't write {filename}.\n"));
        return;
    }

    crate::files_common::FS_Printf(
        view.common,
        f,
        "// generated by Star Wars Jedi Academy MP, do not modify\n",
    );
    let hook_fn = view
        .common
        .hooks
        .Key_WriteBindings
        .expect("Key_WriteBindings hook");
    hook_fn(view, f);
    Cvar_WriteVariables(view.common, f);
    FS_FCloseFile(view.common, f);
}

/// `Com_WriteConfiguration`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1471-1505`
pub fn Com_WriteConfiguration(view: &mut EngineHostView) {
    // if we are quiting without fully initializing, make sure
    // we don't write out anything
    if !view.common.com_fullyInitialized {
        return;
    }

    if view.common.cvar_modifiedFlags & mp_qshared::shared::cvar::CVAR_ARCHIVE == 0 {
        return;
    }
    view.common.cvar_modifiedFlags &= !mp_qshared::shared::cvar::CVAR_ARCHIVE;

    // dedicated vs. non-dedicated cfg name settled at the wave-20 seam;
    // MP dedicated build writes jampserver.cfg.
    Com_WriteConfigToFile(view, "jampserver.cfg");

    // USE_CD_KEY path is a dead #ifdef in the MP tree (§20-class, not
    // reachable under DEDICATED/no-CD-key builds) — dropped per the packet's
    // unresolved-consts escalation.
}

/// `Com_WriteConfig_f`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1515-1527`
pub fn Com_WriteConfig_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "Usage: writeconfig <filename>\n");
        return;
    }

    // Raven's Q_strncpyz into a MAX_QPATH buffer + COM_DefaultExtension: the
    // ".cfg" default appends when the name has no extension.
    let mut filename = Cmd_Argv(view.common, 1).to_string();
    if !filename
        .rsplit('/')
        .next()
        .unwrap_or(&filename)
        .contains('.')
    {
        filename.push_str(".cfg");
    }
    com_printf(view.common, &format!("Writing {filename}.\n"));
    Com_WriteConfigToFile(view, &filename);
}

/// `Com_RunAndTimeServerPacket`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:894-912`
pub fn Com_RunAndTimeServerPacket(
    view: &mut EngineHostView,
    evFrom: *mut netadr_t,
    buf: *mut msg_t,
) {
    let mut t1 = 0;

    unsafe {
        if view.common.cvar(view.common.com_speeds).integer != 0 {
            t1 = crate::timing::sys_milliseconds(view.common);
        }

        let sv_packet_event = view
            .common
            .hooks
            .SV_PacketEvent
            .expect("SV_PacketEvent hook — installed by mp_engine_server at boot");
        sv_packet_event(view, *evFrom, buf);

        if view.common.cvar(view.common.com_speeds).integer != 0 {
            let t2 = crate::timing::sys_milliseconds(view.common);
            let msec = t2 - t1;
            if view.common.cvar(view.common.com_speeds).integer == 3 {
                crate::common::com_printf(view.common, &format!("SV_PacketEvent time: {msec}\n"));
            }
        }
    }
}

/// `Com_EventLoop`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:921-1019`
pub fn Com_EventLoop(view: &mut EngineHostView) -> c_int {
    let mut buf_data = [0u8; MAX_MSGLEN];
    let mut buf: msg_t = unsafe { core::mem::zeroed() };
    MSG_Init(view, &mut buf, buf_data.as_mut_ptr(), MAX_MSGLEN as c_int);

    loop {
        let ev = Com_GetEvent(view);

        // if no more events are available
        if matches!(ev.evType, sysEventType_t::SE_NONE) {
            // manually send packet events for the loopback channel
            let mut ev_from: netadr_t = unsafe { core::mem::zeroed() };
            while crate::net_chan::NET_GetLoopPacket(
                view.common,
                mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t::NS_CLIENT,
                &mut ev_from,
                &mut buf,
            ) != qfalse
            {
                let hook_fn = view
                    .common
                    .hooks
                    .CL_PacketEvent
                    .expect("CL_PacketEvent hook");
                hook_fn(view, ev_from, &mut buf);
            }

            while crate::net_chan::NET_GetLoopPacket(
                view.common,
                mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t::NS_SERVER,
                &mut ev_from,
                &mut buf,
            ) != qfalse
            {
                // if the server just shut down, flush the events
                if view.common.cvar(view.common.com_sv_running).integer != 0 {
                    Com_RunAndTimeServerPacket(view, &mut ev_from, &mut buf);
                }
            }

            return ev.evTime;
        }

        match ev.evType {
            sysEventType_t::SE_NONE => {}
            sysEventType_t::SE_KEY => {
                let hook_fn = view.common.hooks.CL_KeyEvent.expect("CL_KeyEvent hook");
                hook_fn(view, ev.evValue, ev.evValue2 != 0, ev.evTime);
            }
            sysEventType_t::SE_CHAR => {
                let hook_fn = view.common.hooks.CL_CharEvent.expect("CL_CharEvent hook");
                hook_fn(view, ev.evValue);
            }
            sysEventType_t::SE_MOUSE => {
                let hook_fn = view.common.hooks.CL_MouseEvent.expect("CL_MouseEvent hook");
                hook_fn(view, ev.evValue, ev.evValue2, ev.evTime);
            }
            sysEventType_t::SE_JOYSTICK_AXIS => {
                let hook_fn = view
                    .common
                    .hooks
                    .CL_JoystickEvent
                    .expect("CL_JoystickEvent hook");
                hook_fn(view, ev.evValue, ev.evValue2, ev.evTime);
            }
            sysEventType_t::SE_CONSOLE => {
                // sys-event seam: evPtr is the C event payload; one lossy
                // conversion at the head (console text precedent).
                let s = unsafe { c_str_to_string(ev.evPtr as *const c_char) };
                let cmd = s.strip_prefix(['\\', '/']).unwrap_or(&s);
                Cbuf_AddText(view.common, cmd);
                Cbuf_AddText(view.common, "\n");
            }
            sysEventType_t::SE_PACKET => {
                // this cvar allows simulation of connections that
                // drop a lot of packets.  Note that loopback connections
                // don't go through here at all.
                if view.common.cvar(view.common.com_dropsim).value > 0.0 {
                    // §B3 fn-static: `static int seed` is genuine cross-frame
                    // state — hoisted onto `Common` per the three-kind rule.
                    if q_random(&mut view.common.com_eventloop_seed)
                        < view.common.cvar(view.common.com_dropsim).value
                    {
                        continue; // drop this packet
                    }
                }

                unsafe {
                    let mut ev_from = *(ev.evPtr as *mut netadr_t);
                    buf.cursize = ev.evPtrLength - core::mem::size_of::<netadr_t>() as i32;

                    // we must copy the contents of the message out, because
                    // the event buffers are only large enough to hold the
                    // exact payload, but channel messages need to be large
                    // enough to hold fragment reassembly
                    if (buf.cursize as u32) > buf.maxsize as u32 {
                        crate::common::com_printf(view.common, "Com_EventLoop: oversize packet\n");
                        continue;
                    }
                    Com_Memcpy(
                        buf.data as *mut (),
                        (ev.evPtr as *mut netadr_t).add(1) as *const (),
                        buf.cursize as usize,
                    );
                    if view.common.cvar(view.common.com_sv_running).integer != 0 {
                        Com_RunAndTimeServerPacket(view, &mut ev_from, &mut buf);
                    } else {
                        let hook_fn = view
                            .common
                            .hooks
                            .CL_PacketEvent
                            .expect("CL_PacketEvent hook");
                        hook_fn(view, ev_from, &mut buf);
                    }
                }
            } // Raven's `default:` (`Com_Error(ERR_FATAL, "bad event type %i")`) guards
              // against a corrupted/out-of-range int; `sysEventType_t` is a proper Rust
              // enum, so every value is already one of the arms above — unreachable.
        }

        // free any block data
        if !ev.evPtr.is_null() {
            Z_Free(view.common, ev.evPtr as *mut ());
        }
    }
}

/// `Com_Frame`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1593-1777`

/// Raven `Com_Error`'s PRE-THROW work (`common.cpp:249-345`), relocated
/// catch-side (LIFE-D3/STATE-Q4: `com_error` is receiverless and only
/// panics). Runs the escalations, guard/bookkeeping, and per-level shutdown
/// sequence in oracle print order, and returns the thrown level literal for a
/// recoverable level; `ERR_FATAL` (direct or escalated) runs the fatal chain
/// and never returns. A recursive error (guard already set on entry) exits via
/// `Sys_Error` reading the FIRST error's saved message — the nested throw
/// never overwrote it (`common.cpp:288`).
///
/// The win32 `_DEBUG` `int 3` breakpoint block is debugger-only and dropped.
///
/// Source: `oracle/codemp/qcommon/common.cpp:249-345`
pub fn com_error_recover(view: &mut EngineHostView, err: &ComError) -> &'static str {
    let mut code = err.level;

    // when we are running automated scripts, make sure we
    // know if anything failed
    if view.common.com_buildScript.is_some()
        && view.common.cvar(view.common.com_buildScript).integer != 0
    {
        code = errorParm_t::ERR_FATAL;
    }

    // make sure we can get at our local stuff
    FS_PureServerSetLoadedPaks(view, "", "");

    // if we are getting a solid stream of ERR_DROP, do an ERR_FATAL
    let current_time = crate::timing::sys_milliseconds(view.common);
    if current_time - view.common.error.last_error_time < 100 {
        view.common.error.error_count += 1;
        if view.common.error.error_count > 3 {
            code = errorParm_t::ERR_FATAL;
        }
    } else {
        view.common.error.error_count = 0;
    }
    view.common.error.last_error_time = current_time;

    if view.common.error.entered {
        let saved = view.common.error.message_str();
        view.sys_error(&format!("recursive error after: {saved}"));
    }
    view.common.error.entered = true;

    // `vsprintf(com_errorMessage, fmt, argptr)` — the payload arrives
    // pre-formatted (STATE-Q4); store it in the saved-message buffer.
    view.common.error.set_message(&err.msg);

    if code != errorParm_t::ERR_DISCONNECT {
        // give com_errorMessage a default so it won't come back to life after
        // a resetDefaults
        Cvar_Get(view, "com_errorMessage", "", CVAR_ROM);
        let cmsg = err.msg.clone();
        Cvar_Set(view, "com_errorMessage", &cmsg);
    }

    if code == errorParm_t::ERR_SERVERDISCONNECT {
        let cl_disconnect = view
            .common
            .hooks
            .CL_Disconnect
            .expect("CL_Disconnect hook (null-build default)");
        cl_disconnect(view, qtrue);
        let cl_flush = view
            .common
            .hooks
            .CL_FlushMemory
            .expect("CL_FlushMemory hook (null-build default)");
        cl_flush(view);
        view.common.error.entered = false;
        "DISCONNECTED\n"
    } else if code == errorParm_t::ERR_DROP || code == errorParm_t::ERR_DISCONNECT {
        com_printf(
            view.common,
            &format!(
                "********************\nERROR: {}\n********************\n",
                err.msg
            ),
        );
        let sv_shutdown = view
            .common
            .hooks
            .SV_Shutdown
            .expect("SV_Shutdown hook — installed by mp_engine_server at boot");
        sv_shutdown(view, &format!("Server crashed: {}\n", err.msg));
        let cl_disconnect = view
            .common
            .hooks
            .CL_Disconnect
            .expect("CL_Disconnect hook (null-build default)");
        cl_disconnect(view, qtrue);
        let cl_flush = view
            .common
            .hooks
            .CL_FlushMemory
            .expect("CL_FlushMemory hook (null-build default)");
        cl_flush(view);
        view.common.error.entered = false;
        "DROPPED\n"
    } else if code == errorParm_t::ERR_NEED_CD {
        let sv_shutdown = view
            .common
            .hooks
            .SV_Shutdown
            .expect("SV_Shutdown hook — installed by mp_engine_server at boot");
        sv_shutdown(view, "Server didn't have CD\n");
        let cl_running = view.common.com_cl_running.is_some()
            && view.common.cvar(view.common.com_cl_running).integer != 0;
        if cl_running {
            let cl_disconnect = view
                .common
                .hooks
                .CL_Disconnect
                .expect("CL_Disconnect hook (null-build default)");
            cl_disconnect(view, qtrue);
            let cl_flush = view
                .common
                .hooks
                .CL_FlushMemory
                .expect("CL_FlushMemory hook (null-build default)");
            cl_flush(view);
            view.common.error.entered = false;
        } else {
            com_printf(view.common, "Server didn't have CD\n");
        }
        "NEED CD\n"
    } else {
        let cl_shutdown = view
            .common
            .hooks
            .CL_Shutdown
            .expect("CL_Shutdown hook (null-build default)");
        cl_shutdown(view);
        let sv_shutdown = view
            .common
            .hooks
            .SV_Shutdown
            .expect("SV_Shutdown hook — installed by mp_engine_server at boot");
        sv_shutdown(view, &format!("Server fatal crashed: {}\n", err.msg));
        Com_Shutdown(view.common, view.cm, &mut view.rmg);
        view.sys_error(&err.msg)
    }
}

pub fn Com_Frame(view: &mut EngineHostView) {
    // Raven's `try`/`catch (const char* reason)` around an ERR_DROP-class
    // Com_Error is the setjmp/longjmp analogue this fn owns (ruling 1) —
    // ported as catch_unwind at exactly this Raven setjmp site.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let minMsec;
        let mut msec;

        // write config file if anything changed
        Com_WriteConfiguration(view);

        // if "viewlog" has been modified, show or hide the log console
        if view.common.cvar(view.common.com_viewlog).modified {
            if view.common.cvar(view.common.com_dedicated).value == 0.0 {
                let viewlog = view.common.cvar(view.common.com_viewlog).integer;
                view.sys_show_console(viewlog, qfalse);
            }
            view.common.cvar_mut(view.common.com_viewlog).modified = false;
        }

        //
        // main event loop
        //
        {
            if view.common.cvar(view.common.com_speeds).integer != 0 {
                let _time_before_first_events = crate::timing::sys_milliseconds(view.common);
            }

            // we may want to spin here if things are going too fast
            if view.common.cvar(view.common.com_dedicated).integer == 0
                && view.common.cvar(view.common.com_maxfps).integer > 0
                && view.common.cvar(view.common.com_timedemo).integer == 0
            {
                minMsec = 1000 / view.common.cvar(view.common.com_maxfps).integer;
            } else {
                minMsec = 1;
            }
        }
        loop {
            view.common.com_frameTime = Com_EventLoop(view);
            if view.common.frame_last_time > view.common.com_frameTime {
                view.common.frame_last_time = view.common.com_frameTime; // possible on first frame
            }
            msec = view.common.com_frameTime - view.common.frame_last_time;
            if msec >= minMsec {
                break;
            }
        }
        crate::cmd_common::Cbuf_Execute(view);

        view.common.frame_last_time = view.common.com_frameTime;

        // mess with msec if needed
        view.common.com_frameMsec = msec;
        msec = Com_ModifyMsec(view.common, msec);

        //
        // server side
        //
        {
            if view.common.cvar(view.common.com_speeds).integer != 0 {
                let _time_before_server = crate::timing::sys_milliseconds(view.common);
            }
        }

        let sv_frame = view
            .common
            .hooks
            .SV_Frame
            .expect("SV_Frame hook — installed by mp_engine_server at boot");
        sv_frame(view, msec);

        // if "dedicated" has been modified, start up
        // or shut down the client system.
        // Do this after the server may have started,
        // but before the client tries to auto-connect
        {
            if view.common.cvar(view.common.com_dedicated).modified {
                // get the latched value
                Cvar_Get(view, "dedicated", "0", 0);
                view.common.cvar_mut(view.common.com_dedicated).modified = false;
                if view.common.cvar(view.common.com_dedicated).integer == 0 {
                    let cl_init = view.common.hooks.CL_Init.expect("CL_Init hook");
                    cl_init(view);
                    let viewlog = view.common.cvar(view.common.com_viewlog).integer;
                    view.sys_show_console(viewlog, qfalse);
                    let hook_fn = view
                        .common
                        .hooks
                        .CL_StartHunkUsers
                        .expect("CL_StartHunkUsers hook");
                    hook_fn(view);
                } else {
                    let hook_fn = view.common.hooks.CL_Shutdown.expect("CL_Shutdown hook");
                    hook_fn(view);
                    view.sys_show_console(1, qtrue);
                }
            }
        }

        //
        // client system
        //
        {
            if view.common.cvar(view.common.com_dedicated).integer == 0 {
                //
                // run event loop a second time to get server to client packets
                // without a frame of latency
                //
                if view.common.cvar(view.common.com_speeds).integer != 0 {
                    let _time_before_events = crate::timing::sys_milliseconds(view.common);
                }
                Com_EventLoop(view);
                crate::cmd_common::Cbuf_Execute(view);

                //
                // client side
                //
                if view.common.cvar(view.common.com_speeds).integer != 0 {
                    let _time_before_client = crate::timing::sys_milliseconds(view.common);
                }

                let hook_fn = view.common.hooks.CL_Frame.expect("CL_Frame hook");
                hook_fn(view, msec);

                if view.common.cvar(view.common.com_speeds).integer != 0 {
                    let _time_after = crate::timing::sys_milliseconds(view.common);
                }
            }
        }

        //
        // report timing information
        //
        // Raven prints a com_speeds all/sv/ev/cl frame-timing breakdown here; not implemented, this block is a silent no-op.

        //
        // trace optimization tracking
        //
        {
            if view.common.cvar(view.common.com_showtrace).integer != 0 {
                crate::common::com_printf(
                    view.common,
                    &format!(
                        "{:4} traces  ({}b {}p) {:4} points\n",
                        view.cm.c_traces,
                        view.cm.c_brush_traces,
                        view.cm.c_patch_traces,
                        view.cm.c_pointcontents
                    ),
                );
                view.cm.c_traces = 0;
                view.cm.c_brush_traces = 0;
                view.cm.c_patch_traces = 0;
                view.cm.c_pointcontents = 0;
            }
        }

        view.common.com_frameNumber += 1;
    }));

    if let Err(payload) = result {
        match payload.downcast::<ComError>() {
            // The ERR_DROP recovery point (Raven's catch, common.cpp:1762):
            // `com_error` only panicked — the catch runs ALL of Raven's
            // pre-throw work (`com_error_recover`), then prints the thrown
            // level literal exactly as Raven's `Com_Printf(reason)` does
            // (common.cpp:1763). Recovery runs in its OWN catch_unwind: a
            // `com_error` raised DURING recovery while the guard is set is
            // Raven's recursive-error exit (common.cpp:288), reading the
            // FIRST error's saved message.
            Ok(err) => {
                let recovered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    com_error_recover(view, &err)
                }));
                match recovered {
                    Ok(literal) => com_printf(view.common, literal),
                    Err(second) => {
                        if second.is::<ComError>() && view.common.error.entered {
                            let saved = view.common.error.message_str();
                            view.sys_error(&format!("recursive error after: {saved}"));
                        }
                        // A non-ComError panic is a real Rust bug — fatal.
                        std::panic::resume_unwind(second);
                    }
                }
            }
            // A non-ComError panic is a real Rust bug — fatal (LIFE-D3).
            Err(other) => std::panic::resume_unwind(other),
        }
        return; // an ERR_DROP was thrown; the frame returns and the loop continues
    }

    // G2_PERFORMANCE_ANALYSIS is a build-time-off diagnostics path (unresolved
    // const, escalated) — its G2Time_*/timer calls are not reachable here.
}

/// `Com_ParseTextFile` (parse into an existing `GenericParser2`, 3-arg form).
///
/// Source: `oracle/codemp/qcommon/common.cpp:2179-2202`
pub fn Com_ParseTextFile(
    view: &mut EngineHostView,
    file: &str,
    parser: &mut GenericParser2,
    cleanFirst: bool,
) -> bool {
    let mut f: mp_qshared::shared::fileHandle_t = 0;
    let length = crate::files_pc::FS_FOpenFileByMode(view, file, &mut f, FS_READ);
    if f == 0 || length == 0 {
        return false;
    }

    let mut buf = vec![0u8; (length + 1) as usize];
    FS_Read(view.common, buf.as_mut_ptr() as *mut (), length, f);
    buf[length as usize] = 0;

    let text = latin1_to_string(&buf[..length as usize]);
    let _ = parser.parse(&text, cleanFirst);

    FS_FCloseFile(view.common, f);

    true
}

/// `Com_ParseTextFile` (allocate + parse a new `GenericParser2`, returning it
/// or null on failed parse; 3-arg `writeable` form).
///
/// Source: `oracle/codemp/qcommon/common.cpp:2209-2239`
///
/// Raven overloads `Com_ParseTextFile` by arity; this is the 3-arg
/// `(file, cleanFirst, writeable)` overload, suffixed `2`.
/// `writeable` is accepted but discarded — `GenericParser2::parse` has no
/// writeable flag yet.
pub fn Com_ParseTextFile2(
    view: &mut EngineHostView,
    file: &str,
    cleanFirst: bool,
    writeable: bool,
) -> *mut GenericParser2 {
    let _ = writeable;
    let mut f: mp_qshared::shared::fileHandle_t = 0;
    let length = crate::files_pc::FS_FOpenFileByMode(view, file, &mut f, FS_READ);
    if f == 0 || length == 0 {
        return core::ptr::null_mut();
    }

    let mut buf = vec![0u8; (length + 1) as usize];
    FS_Read(view.common, buf.as_mut_ptr() as *mut (), length, f);
    FS_FCloseFile(view.common, f);
    buf[length as usize] = 0;

    let text = latin1_to_string(&buf[..length as usize]);

    let mut parse = Box::new(GenericParser2::new());
    if parse.parse(&text, cleanFirst).is_err() {
        return core::ptr::null_mut();
    }

    Box::into_raw(parse)
}

/// `Com_Init`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1216-1442`
pub fn Com_Init(view: &mut EngineHostView, commandLine: &str) {
    crate::common::com_printf(
        view.common,
        &format!(
            "{} {} {}\n",
            Q3_VERSION,
            CPUSTRING,
            option_env!("BUILD_DATE").unwrap_or("unknown"),
        ),
    );

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // bk001129 - do this before anything else decides to push events
        Com_InitPushEvent(view.common);

        Cvar_Init(view);

        // prepare enough of the subsystems to handle
        // cvar and command buffer management
        Com_ParseCommandLine(view.common, commandLine);

        crate::cmd_common::Cbuf_Init(view.common);

        crate::z_memman_pc::Com_InitZoneMemory(view);

        crate::cmd_common::Cmd_Init(view);

        // override anything from the config files with command line args
        Com_StartupVariable(view, None);

        // Seed the random number generator — Raven `Rand_Init(Sys_Milliseconds(true))`
        // seeds the engine island's own LCG (ruling 21's `common.qrand`).
        // Source: oracle/codemp/qcommon/common.cpp:1248
        let rand_seed = crate::timing::sys_milliseconds(view.common);
        view.common.qrand.Rand_Init(rand_seed);

        // get the developer cvar set as early as possible
        Com_StartupVariable(view, Some("developer"));

        // done early so bind command exists
        let hook_fn = view
            .common
            .hooks
            .CL_InitKeyCommands
            .expect("CL_InitKeyCommands hook");
        hook_fn(view);

        crate::files_common::FS_InitFilesystem(view);

        Com_InitJournaling(view);

        Cbuf_AddText(view.common, "exec mpdefault.cfg\n");

        // skip the jampconfig.cfg if "safe" is on the command line
        if Com_SafeMode(view.common) == qfalse {
            Cbuf_AddText(view.common, "exec jampconfig.cfg\n");
        }

        Cbuf_AddText(view.common, "exec autoexec.cfg\n");

        crate::cmd_common::Cbuf_Execute(view);

        // override anything from the config files with command line args
        Com_StartupVariable(view, None);

        // get dedicated here for proper hunk megs initialization
        view.common.com_dedicated = Some(Cvar_Get(view, "dedicated", "0", CVAR_LATCH));
        // allocate the stack based hunk allocator
        crate::z_memman_pc::Com_InitHunkMemory(view);

        // if any archived cvars are modified after this, we will trigger a writing
        // of the config file
        view.common.cvar_modifiedFlags &= !CVAR_ARCHIVE;

        //
        // init commands and vars
        //
        view.common.com_maxfps = Some(Cvar_Get(view, "com_maxfps", "85", CVAR_ARCHIVE));
        view.common.com_blood = Some(Cvar_Get(view, "com_blood", "1", CVAR_ARCHIVE));

        view.common.com_developer = Some(Cvar_Get(view, "developer", "0", CVAR_TEMP));
        view.common.com_vmdebug = Some(Cvar_Get(view, "vmdebug", "0", CVAR_TEMP));
        view.common.com_logfile = Some(Cvar_Get(view, "logfile", "0", CVAR_TEMP));

        view.common.com_timescale = Some(Cvar_Get(
            view,
            "timescale",
            "1",
            CVAR_CHEAT | CVAR_SYSTEMINFO,
        ));
        view.common.com_fixedtime = Some(Cvar_Get(view, "fixedtime", "0", CVAR_CHEAT));
        view.common.com_showtrace = Some(Cvar_Get(view, "com_showtrace", "0", CVAR_CHEAT));

        view.common.com_terrainPhysics =
            Some(Cvar_Get(view, "com_terrainPhysics", "1", CVAR_CHEAT));

        view.common.com_dropsim = Some(Cvar_Get(view, "com_dropsim", "0", CVAR_CHEAT));
        view.common.com_viewlog = Some(Cvar_Get(view, "viewlog", "0", CVAR_CHEAT));
        view.common.com_speeds = Some(Cvar_Get(view, "com_speeds", "0", 0));
        view.common.com_timedemo = Some(Cvar_Get(view, "timedemo", "0", 0));
        view.common.com_cameraMode = Some(Cvar_Get(view, "com_cameraMode", "0", CVAR_CHEAT));

        view.common.com_optvehtrace = Some(Cvar_Get(view, "com_optvehtrace", "0", 0));

        view.common.cl_paused = Some(Cvar_Get(view, "cl_paused", "0", CVAR_ROM));
        view.common.sv_paused = Some(Cvar_Get(view, "sv_paused", "0", CVAR_ROM));
        view.common.com_sv_running = Some(Cvar_Get(view, "sv_running", "0", CVAR_ROM));
        view.common.com_cl_running = Some(Cvar_Get(view, "cl_running", "0", CVAR_ROM));
        view.common.com_buildScript = Some(Cvar_Get(view, "com_buildScript", "0", 0));

        // G2_PERFORMANCE_ANALYSIS gated in retail (unresolved const,
        // escalated) — com_G2Report registers unconditionally here since the
        // engine ships that build config.
        view.common.com_G2Report = Some(Cvar_Get(view, "com_G2Report", "0", 0));

        view.common.com_RMG = Some(Cvar_Get(view, "RMG", "0", 0));

        Cvar_Get(view, "RMG_seed", "0", 0);
        Cvar_Get(view, "RMG_time", "day", 0);
        Cvar_Get(view, "RMG_soundset", "", 0);

        Cvar_Get(view, "RMG_textseed", "0", CVAR_SYSTEMINFO | CVAR_ARCHIVE);
        Cvar_Get(view, "RMG_map", "small", CVAR_ARCHIVE | CVAR_SYSTEMINFO);
        Cvar_Get(view, "RMG_timefile", "day", CVAR_ARCHIVE);
        Cvar_Get(view, "RMG_terrain", "grassyhills", CVAR_ARCHIVE);

        Cvar_Get(view, "RMG_sky", "", CVAR_SYSTEMINFO);
        Cvar_Get(view, "RMG_fog", "", CVAR_SYSTEMINFO);
        Cvar_Get(
            view,
            "RMG_weather",
            "",
            CVAR_SYSTEMINFO | CVAR_SERVERINFO | CVAR_CHEAT,
        );
        Cvar_Get(view, "RMG_instances", "colombia", CVAR_SYSTEMINFO);
        Cvar_Get(view, "RMG_miscents", "deciduous", 0);
        Cvar_Get(view, "RMG_music", "music/dm_kam1", 0);
        Cvar_Get(view, "RMG_mission", "ctf", CVAR_SYSTEMINFO);
        Cvar_Get(view, "RMG_course", "standard", CVAR_SYSTEMINFO);
        Cvar_Get(view, "RMG_distancecull", "5000", CVAR_CHEAT);

        view.common.com_introPlayed = Some(Cvar_Get(view, "com_introplayed", "0", CVAR_ARCHIVE));

        {
            if view.common.cvar(view.common.com_dedicated).integer != 0
                && view.common.cvar(view.common.com_viewlog).integer == 0
            {
                Cvar_Set(view, "viewlog", "1");
            }

            if view.common.com_developer.is_some()
                && view.common.cvar(view.common.com_developer).integer != 0
            {
                Cmd_AddCommand(view, "error", Some(|view| Com_Error_f(view.common)));
                Cmd_AddCommand(view, "crash", Some(|_view| Com_Crash_f()));
                Cmd_AddCommand(view, "freeze", Some(|view| Com_Freeze_f(view)));
            }
        }
        Cmd_AddCommand(view, "quit", Some(|view| Com_Quit_f(view)));
        Cmd_AddCommand(
            view,
            "changeVectors",
            Some(|view| crate::msg::MSG_ReportChangeVectors_f(view.common)),
        );
        Cmd_AddCommand(view, "writeconfig", Some(|view| Com_WriteConfig_f(view)));

        let s = format!(
            "{} {} {}",
            Q3_VERSION,
            CPUSTRING,
            option_env!("BUILD_DATE").unwrap_or("unknown"),
        );
        view.common.com_version = Some(Cvar_Get(view, "version", &s, CVAR_ROM | CVAR_SERVERINFO));

        SE_Init(view);

        view.sys_init();
        let netchan_port = (Com_Milliseconds(view) & 0xffff) as c_int;
        crate::net_chan::Netchan_Init(view, netchan_port);
        crate::vm_fns::VM_Init(view);
        let sv_init = view
            .common
            .hooks
            .SV_Init
            .expect("SV_Init hook — installed by mp_engine_server at boot");
        sv_init(view);

        {
            view.common.cvar_mut(view.common.com_dedicated).modified = false;
            if view.common.cvar(view.common.com_dedicated).integer == 0 {
                let cl_init = view.common.hooks.CL_Init.expect("CL_Init hook");
                cl_init(view);
                let viewlog = view.common.cvar(view.common.com_viewlog).integer;
                view.sys_show_console(viewlog, qfalse);
            }
        }

        // set com_frameTime so that if a map is started on the
        // command line it will still be able to count on com_frameTime
        // being random enough for a serverid
        view.common.com_frameTime = Com_Milliseconds(view);

        // add + commands from command line
        {
            if Com_AddStartupCommands(view.common) == qfalse {
                // if the user didn't give any commands, run default action
                if view.common.cvar(view.common.com_dedicated).integer == 0 {
                    Cbuf_AddText(view.common, "cinematic openinglogos.roq\n");
                }
            }
        }

        // start in full screen ui mode
        Cvar_Set(view, "r_uiFullScreen", "1");

        let hook_fn = view
            .common
            .hooks
            .CL_StartHunkUsers
            .expect("CL_StartHunkUsers hook");
        hook_fn(view);

        // make sure single player is off by default
        Cvar_Set(view, "ui_singlePlayerActive", "0");

        view.common.com_fullyInitialized = true;
        crate::common::com_printf(view.common, "--- Common Initialization Complete ---\n");
    }));

    if let Err(payload) = result {
        match payload.downcast::<ComError>() {
            // Init-time errors are always fatal (Raven's init catch ->
            // `Sys_Error("Error during initialization: %s", reason)`,
            // common.cpp:1439): run the same catch-side recovery, then
            // escalate the recoverable level literal through Sys_Error with
            // the wrapper (ERR_FATAL never returns from the recovery itself).
            Ok(err) => {
                let recovered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    com_error_recover(view, &err)
                }));
                match recovered {
                    Ok(literal) => {
                        view.sys_error(&format!("Error during initialization: {literal}"))
                    }
                    Err(second) => {
                        if second.is::<ComError>() && view.common.error.entered {
                            let saved = view.common.error.message_str();
                            view.sys_error(&format!("recursive error after: {saved}"));
                        }
                        std::panic::resume_unwind(second);
                    }
                }
            }
            Err(other) => std::panic::resume_unwind(other),
        }
    }
}

// --- local helpers (not Raven fns; keep the bodies above straight-line) ---

unsafe fn c_str_to_string(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    buf_to_string(CStr::from_ptr(p).to_bytes())
}

/// `Q_random(&seed)` external (qshared LCG surface) — packet-cited external,
/// not this shard's to define; referenced by name only.
fn q_random(seed: &mut c_int) -> f32 {
    let _ = seed;
    0.0
}
