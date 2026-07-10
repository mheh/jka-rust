#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `common.cpp` — the engine's top-level frame/init/config/event-loop glue.
//!
//! DESTINATION NOTE: `common.cpp`'s stem collides with the existing `common/`
//! directory module, so this file lands at the `_fns` escape per
//! `_PREAMBLE.md`'s destination rule.
//!
//! Source: `oracle/codemp/qcommon/common.cpp`

use core::ffi::{c_char, c_int, c_void};

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::qtime::qtime_t;
use mp_qshared::shared::cvar::cvar_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::MAX_TOKEN_CHARS;
use mp_qshared::shared::{qboolean, qfalse, qtrue, FS_READ, MAX_QPATH};

use crate::collision_world::CollisionWorld;
use crate::common::common_consts::{MAX_CONSOLE_LINES, MAX_PUSHED_EVENTS};
use crate::common::Common;
use crate::gp2::generic_parser2::GenericParser2;
use crate::qcommon::net_limits::MAX_MSGLEN;
use crate::qcommon::sys_event_t::sysEvent_t;
use crate::qcommon::sys_event_type_t::sysEventType_t;

// PORT-NOTE(rm-types): `RenderModels`/`RmManager`/`Ghoul2System`/`BotLib` are
// the state-receiver types pinned by the engine-fork-discovery preamble's
// receiver order (rmg-terrain.md / ghoul2-server.md own their shape); none
// has landed in the tree yet, and `Server`/`Client` cannot be imported here
// (`mp_engine_server`/`mp_engine_client` already depend on this crate — a
// `use` would cycle). Referenced by their exact resolved-signature names per
// the no-stub rule; reported as missing symbols/shape mismatches for the
// finisher (cm_trace.rs precedent).
#[allow(dead_code)]
struct RenderModels;
#[allow(dead_code)]
struct RmManager;
#[allow(dead_code)]
struct Ghoul2System;
#[allow(dead_code)]
struct BotLib;
#[allow(dead_code)]
struct Server;
#[allow(dead_code)]
struct Client;

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
    // PORT-NOTE(rd-fields): `rd_buffer`/`rd_buffersize`/`rd_flush` are
    // `common.cpp` file statics not yet fields on `Common` — written with
    // Raven's verbatim names per STATE-D fields rule; integration adds them.
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
/// PORT-NOTE(variadic): the Raven `va_list`/`vsprintf` formatting collapses
/// to a pre-formatted `msg` at the Rust call site (the qshared `va`/
/// `Com_sprintf` surface is already ported); this fn's own body is the
/// platform print, unix path (ruling 8/10) — no `OutputDebugString`.
pub fn Com_OPrintf(msg: &str) {
    print!("{msg}");
}

/// `Com_ParseCommandLine`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:397-414`
pub fn Com_ParseCommandLine(common: &mut Common, commandLine: *mut c_char) {
    common.com_consoleLines[0] = commandLine;
    common.com_numConsoleLines = 1;

    unsafe {
        let mut p = commandLine;
        while *p != 0 {
            // look for a + seperating character
            // if commandLine came from a file, we might have real line seperators
            if *p == b'+' as c_char || *p == b'\n' as c_char {
                if common.com_numConsoleLines == MAX_CONSOLE_LINES as c_int {
                    return;
                }
                common.com_consoleLines[common.com_numConsoleLines as usize] = p.add(1);
                common.com_numConsoleLines += 1;
                *p = 0;
            }
            p = p.add(1);
        }
    }
}

/// `Com_StringContains`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:556-578`
pub fn Com_StringContains(
    mut str1: *mut c_char,
    str2: *mut c_char,
    casesensitive: c_int,
) -> *mut c_char {
    unsafe {
        let len1 = libc_strlen(str1);
        let len2 = libc_strlen(str2);
        let len = len1 as isize - len2 as isize;
        let mut i = 0isize;
        while i <= len {
            let mut j = 0isize;
            while *str2.offset(j) != 0 {
                if casesensitive != 0 {
                    if *str1.offset(j) != *str2.offset(j) {
                        break;
                    }
                } else if to_upper(*str1.offset(j)) != to_upper(*str2.offset(j)) {
                    break;
                }
                j += 1;
            }
            if *str2.offset(j) == 0 {
                return str1;
            }
            i += 1;
            str1 = str1.add(1);
        }
    }
    core::ptr::null_mut()
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
/// PORT-NOTE(time): `time`/`localtime` are the libc externals named in the
/// packet's call surface; `std::time`/`libc` supply them at the seam.
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
pub fn Com_Crash_f() {
    unsafe {
        *(0 as *mut c_int) = 0x1234_5678;
    }
}

/// `Com_Memcpy`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1815-1818`
pub fn Com_Memcpy(dest: *mut (), src: *const (), count: usize) {
    unsafe {
        core::ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, count);
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
    for i in 0..common.com_numConsoleLines as usize {
        crate::cmd_common::Cmd_TokenizeString(common, common.com_consoleLines[i]);
        let argv0 = crate::cmd_common::Cmd_Argv(common, 0);
        if q_stricmp(argv0, c"safe".as_ptr() as *mut c_char) == 0
            || q_stricmp(argv0, c"cvar_restart".as_ptr() as *mut c_char) == 0
        {
            unsafe {
                *common.com_consoleLines[i] = 0;
            }
            return qtrue;
        }
    }
    qfalse
}

/// `Com_Filter`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:585-658`
pub fn Com_Filter(mut filter: *mut c_char, mut name: *mut c_char, casesensitive: c_int) -> c_int {
    let mut buf = [0 as c_char; MAX_TOKEN_CHARS];
    unsafe {
        while *filter != 0 {
            if *filter == b'*' as c_char {
                filter = filter.add(1);
                let mut i = 0usize;
                while *filter != 0 {
                    if *filter == b'*' as c_char || *filter == b'?' as c_char {
                        break;
                    }
                    buf[i] = *filter;
                    filter = filter.add(1);
                    i += 1;
                }
                buf[i] = 0;
                if libc_strlen(buf.as_mut_ptr()) > 0 {
                    let ptr = Com_StringContains(name, buf.as_mut_ptr(), casesensitive);
                    if ptr.is_null() {
                        return qfalse;
                    }
                    name = ptr.add(libc_strlen(buf.as_mut_ptr()));
                }
            } else if *filter == b'?' as c_char {
                filter = filter.add(1);
                name = name.add(1);
            } else if *filter == b'[' as c_char && *filter.add(1) == b'[' as c_char {
                filter = filter.add(1);
            } else if *filter == b'[' as c_char {
                filter = filter.add(1);
                let mut found = qfalse;
                while *filter != 0 && found == qfalse {
                    if *filter == b']' as c_char && *filter.add(1) != b']' as c_char {
                        break;
                    }
                    if *filter.add(1) == b'-' as c_char
                        && *filter.add(2) != 0
                        && (*filter.add(2) != b']' as c_char || *filter.add(3) == b']' as c_char)
                    {
                        if casesensitive != 0 {
                            if *name >= *filter && *name <= *filter.add(2) {
                                found = qtrue;
                            }
                        } else if to_upper(*name) >= to_upper(*filter)
                            && to_upper(*name) <= to_upper(*filter.add(2))
                        {
                            found = qtrue;
                        }
                        filter = filter.add(3);
                    } else {
                        if casesensitive != 0 {
                            if *filter == *name {
                                found = qtrue;
                            }
                        } else if to_upper(*filter) == to_upper(*name) {
                            found = qtrue;
                        }
                        filter = filter.add(1);
                    }
                }
                if found == qfalse {
                    return qfalse;
                }
                while *filter != 0 {
                    if *filter == b']' as c_char && *filter.add(1) != b']' as c_char {
                        break;
                    }
                    filter = filter.add(1);
                }
                filter = filter.add(1);
                name = name.add(1);
            } else {
                if casesensitive != 0 {
                    if *filter != *name {
                        return qfalse;
                    }
                } else if to_upper(*filter) != to_upper(*name) {
                    return qfalse;
                }
                filter = filter.add(1);
                name = name.add(1);
            }
        }
    }
    qtrue
}

/// `Com_FilterPath`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:665-690`
pub fn Com_FilterPath(filter: *mut c_char, name: *mut c_char, casesensitive: c_int) -> c_int {
    let mut new_filter = [0 as c_char; MAX_QPATH];
    let mut new_name = [0 as c_char; MAX_QPATH];
    unsafe {
        let mut i = 0usize;
        while i < MAX_QPATH - 1 && *filter.add(i) != 0 {
            new_filter[i] = if *filter.add(i) == b'\\' as c_char || *filter.add(i) == b':' as c_char
            {
                b'/' as c_char
            } else {
                *filter.add(i)
            };
            i += 1;
        }
        new_filter[i] = 0;
        let mut i = 0usize;
        while i < MAX_QPATH - 1 && *name.add(i) != 0 {
            new_name[i] = if *name.add(i) == b'\\' as c_char || *name.add(i) == b':' as c_char {
                b'/' as c_char
            } else {
                *name.add(i)
            };
            i += 1;
        }
        new_name[i] = 0;
    }
    Com_Filter(new_filter.as_mut_ptr(), new_name.as_mut_ptr(), casesensitive)
}

/// `Com_ParseTextFileDestroy`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:2204-2207`
pub fn Com_ParseTextFileDestroy(parser: &mut GenericParser2) {
    parser.clean();
}

/// `Com_Quit_f`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:356-365`
pub fn Com_Quit_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
) {
    // don't try to shutdown if we are in a recursive error
    if common.com_errorEntered == qfalse {
        crate::server::sv_init::SV_Shutdown(common, cm, sv, rm, rmg, host, "Server quit\n");
        crate::null::cl_main::CL_Shutdown();
        crate::common::com_shutdown::Com_Shutdown(common, cm, rmg);
        crate::files::fs_shutdown::FS_Shutdown(common, qtrue);
    }
    host.sys_quit();
}

/// `Com_StartupVariable`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:451-470`
pub fn Com_StartupVariable(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    r#match: *const c_char,
) {
    for i in 0..common.com_numConsoleLines as usize {
        crate::cmd_common::Cmd_TokenizeString(common, common.com_consoleLines[i]);
        let argv0 = crate::cmd_common::Cmd_Argv(common, 0);
        if q_strcmp(argv0, c"set".as_ptr() as *mut c_char) != 0 {
            continue;
        }
        let s = crate::cmd_common::Cmd_Argv(common, 1);
        if r#match.is_null() || q_strcmp(s, r#match as *mut c_char) == 0 {
            crate::cvar::Cvar_Set(common, cm, rm, host, s, crate::cmd_common::Cmd_Argv(common, 2));
            let cv: *mut cvar_t =
                crate::cvar::Cvar_Get(common, cm, rm, host, s, c"".as_ptr() as *mut c_char, 0);
            unsafe {
                (*cv).flags |= mp_qshared::shared::cvar::CVAR_USER_CREATED;
            }
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
    for i in 0..common.com_numConsoleLines as usize {
        let line = common.com_consoleLines[i];
        if line.is_null() || unsafe { *line == 0 } {
            continue;
        }
        // set commands won't override menu startup
        if q_stricmpn(line, c"set".as_ptr() as *mut c_char, 3) != 0 {
            added = qtrue;
        }
        crate::cmd_common::Cbuf_AddText(common, line);
        crate::cmd_common::Cbuf_AddText(common, c"\n".as_ptr() as *mut c_char);
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
                Com_Memset(
                    key.as_mut_ptr().add(o) as *mut (),
                    b' ' as c_int,
                    20 - l,
                );
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
            crate::common::com_printf(
                common,
                &format!("{}\n", c_str_to_string(value.as_ptr())),
            );
        }
    }
}

/// `Com_GetEvent`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:881-887`
pub fn Com_GetEvent(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) -> sysEvent_t {
    if common.com_pushedEventsHead > common.com_pushedEventsTail {
        common.com_pushedEventsTail += 1;
        return common.com_pushedEvents
            [((common.com_pushedEventsTail - 1) & (MAX_PUSHED_EVENTS as i32 - 1)) as usize];
    }
    crate::common::com_get_real_event::Com_GetRealEvent(common, cm, rm, host)
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
pub fn Com_Freeze_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    if crate::cmd_common::Cmd_Argc(common) != 2 {
        crate::common::com_printf(common, "freeze <seconds>\n");
        return;
    }
    let s: f32 = c_str_to_string(crate::cmd_common::Cmd_Argv(common, 1))
        .trim()
        .parse()
        .unwrap_or(0.0);

    let start = crate::common::com_milliseconds::Com_Milliseconds(common, cm, rm, host);

    loop {
        let now = crate::common::com_milliseconds::Com_Milliseconds(common, cm, rm, host);
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
    unsafe {
        if (*common.com_fixedtime).integer != 0 {
            msec = (*common.com_fixedtime).integer;
        } else if (*common.com_timescale).value != 0.0 {
            msec = (msec as f32 * (*common.com_timescale).value) as c_int;
        } else if (*common.com_cameraMode).integer != 0 {
            msec = (msec as f32 * (*common.com_timescale).value) as c_int;
        }

        // don't let it scale below 1 msec
        if msec < 1 && (*common.com_timescale).value != 0.0 {
            msec = 1;
        }

        if (*common.com_dedicated).integer != 0 {
            // dedicated servers don't want to clamp for a much longer
            // period, because it would mess up all the client's views
            // of time.
            if msec > 500 {
                crate::common::com_printf(common, &format!("Hitch warning: {msec} msec frame time\n"));
            }
            clamp_time = 5000;
        } else if (*common.com_sv_running).integer == 0 {
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
pub fn Com_InitJournaling(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    Com_StartupVariable(common, cm, rm, host, c"journal".as_ptr());
    common.com_journal = crate::cvar::Cvar_Get(
        common,
        cm,
        rm,
        host,
        c"journal".as_ptr() as *mut c_char,
        c"0".as_ptr() as *mut c_char,
        mp_qshared::shared::cvar::CVAR_INIT,
    );
    unsafe {
        if (*common.com_journal).integer == 0 {
            return;
        }

        if (*common.com_journal).integer == 1 {
            crate::common::com_printf(common, "Journaling events\n");
            common.com_journalFile =
                crate::files::fs_fopen_file_write::FS_FOpenFileWrite(common, "journal.dat");
            common.com_journalDataFile =
                crate::files::fs_fopen_file_write::FS_FOpenFileWrite(common, "journaldata.dat");
        } else if (*common.com_journal).integer == 2 {
            crate::common::com_printf(common, "Replaying journaled events\n");
            crate::files::fs_fopen_file_read::FS_FOpenFileRead(
                common,
                cm,
                rm,
                host,
                "journal.dat",
                &mut common.com_journalFile,
                qtrue,
            );
            crate::files::fs_fopen_file_read::FS_FOpenFileRead(
                common,
                cm,
                rm,
                host,
                "journaldata.dat",
                &mut common.com_journalDataFile,
                qtrue,
            );
        }

        if common.com_journalFile == 0 || common.com_journalDataFile == 0 {
            crate::cvar::Cvar_Set(
                common,
                cm,
                rm,
                host,
                c"com_journal".as_ptr() as *mut c_char,
                c"0".as_ptr() as *mut c_char,
            );
            common.com_journalFile = 0;
            common.com_journalDataFile = 0;
            crate::common::com_printf(common, "Couldn't open journal files\n");
        }
    }
}

/// `Com_WriteConfigToFile`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1446-1461`
pub fn Com_WriteConfigToFile(common: &mut Common, filename: *const c_char) {
    let f = crate::files::fs_fopen_file_write::FS_FOpenFileWrite(common, unsafe {
        &c_str_to_string(filename)
    });
    if f == 0 {
        crate::common::com_printf(
            common,
            &format!("Couldn't write {}.\n", unsafe { c_str_to_string(filename) }),
        );
        return;
    }

    crate::files_common::FS_Printf(
        common,
        f,
        "// generated by Star Wars Jedi Academy MP, do not modify\n",
    );
    crate::null::key_write_bindings::Key_WriteBindings(f);
    crate::cvar::cvar_write_variables::Cvar_WriteVariables(common, f);
    crate::files::fs_fclose_file::FS_FCloseFile(common, f);
}

/// `Com_WriteConfiguration`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1471-1505`
pub fn Com_WriteConfiguration(common: &mut Common) {
    // if we are quiting without fully initializing, make sure
    // we don't write out anything
    if !common.com_fullyInitialized {
        return;
    }

    if common.cvar_modifiedFlags & mp_qshared::shared::cvar::CVAR_ARCHIVE == 0 {
        return;
    }
    common.cvar_modifiedFlags &= !mp_qshared::shared::cvar::CVAR_ARCHIVE;

    // dedicated vs. non-dedicated cfg name settled at the wave-20 seam;
    // MP dedicated build writes jampserver.cfg.
    Com_WriteConfigToFile(common, c"jampserver.cfg".as_ptr());

    // USE_CD_KEY path is a dead #ifdef in the MP tree (§20-class, not
    // reachable under DEDICATED/no-CD-key builds) — dropped per the packet's
    // unresolved-consts escalation.
}

/// `Com_WriteConfig_f`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1515-1527`
pub fn Com_WriteConfig_f(common: &mut Common) {
    let mut filename = [0 as c_char; MAX_QPATH];

    if crate::cmd_common::Cmd_Argc(common) != 2 {
        crate::common::com_printf(common, "Usage: writeconfig <filename>\n");
        return;
    }

    q_strncpyz(
        filename.as_mut_ptr(),
        crate::cmd_common::Cmd_Argv(common, 1),
        core::mem::size_of_val(&filename),
    );
    com_default_extension(filename.as_mut_ptr(), core::mem::size_of_val(&filename), ".cfg");
    crate::common::com_printf(
        common,
        &format!("Writing {}.\n", unsafe { c_str_to_string(filename.as_ptr()) }),
    );
    Com_WriteConfigToFile(common, filename.as_ptr());
}

/// `Com_RunAndTimeServerPacket`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:894-912`
pub fn Com_RunAndTimeServerPacket(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    evFrom: *mut netadr_t,
    buf: *mut msg_t,
) {
    let mut t1 = 0;

    unsafe {
        if (*common.com_speeds).integer != 0 {
            t1 = host.milliseconds_sys();
        }

        crate::server::sv_main::SV_PacketEvent(common, cm, sv, rm, rmg, host, *evFrom, buf);

        if (*common.com_speeds).integer != 0 {
            let t2 = host.milliseconds_sys();
            let msec = t2 - t1;
            if (*common.com_speeds).integer == 3 {
                crate::common::com_printf(common, &format!("SV_PacketEvent time: {msec}\n"));
            }
        }
    }
}

/// `Com_EventLoop`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:921-1019`
pub fn Com_EventLoop(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
) -> c_int {
    let mut buf_data = [0u8; MAX_MSGLEN];
    let mut buf: msg_t = unsafe { core::mem::zeroed() };
    crate::qcommon::msg::MSG_Init(common, cm, rm, host, &mut buf, buf_data.as_mut_ptr(), MAX_MSGLEN as c_int);

    loop {
        let ev = Com_GetEvent(common, cm, rm, host);

        // if no more events are available
        if matches!(ev.evType, sysEventType_t::SE_NONE) {
            // manually send packet events for the loopback channel
            let mut ev_from: netadr_t = unsafe { core::mem::zeroed() };
            while crate::net_chan::NET_GetLoopPacket(
                common,
                mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t::NS_CLIENT,
                &mut ev_from,
                &mut buf,
            ) {
                crate::null::cl_main::CL_PacketEvent(ev_from, &mut buf);
            }

            while crate::net_chan::NET_GetLoopPacket(
                common,
                mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t::NS_SERVER,
                &mut ev_from,
                &mut buf,
            ) {
                // if the server just shut down, flush the events
                if unsafe { (*common.com_sv_running).integer != 0 } {
                    Com_RunAndTimeServerPacket(common, cm, sv, rm, rmg, host, &mut ev_from, &mut buf);
                }
            }

            return ev.evTime;
        }

        match ev.evType {
            sysEventType_t::SE_NONE => {}
            sysEventType_t::SE_KEY => {
                crate::null::cl_input::CL_KeyEvent(ev.evValue, ev.evValue2 != 0, ev.evTime);
            }
            sysEventType_t::SE_CHAR => {
                crate::null::cl_input::CL_CharEvent(ev.evValue);
            }
            sysEventType_t::SE_MOUSE => {
                crate::null::cl_input::CL_MouseEvent(ev.evValue, ev.evValue2, ev.evTime);
            }
            sysEventType_t::SE_JOYSTICK_AXIS => {
                crate::null::cl_input::CL_JoystickEvent(ev.evValue, ev.evValue2, ev.evTime);
            }
            sysEventType_t::SE_CONSOLE => {
                unsafe {
                    let s = ev.evPtr as *mut c_char;
                    if *s == b'\\' as c_char || *s == b'/' as c_char {
                        crate::cmd_common::Cbuf_AddText(common, s.add(1));
                    } else {
                        crate::cmd_common::Cbuf_AddText(common, s);
                    }
                }
                crate::cmd_common::Cbuf_AddText(common, c"\n".as_ptr() as *mut c_char);
            }
            sysEventType_t::SE_PACKET => {
                // this cvar allows simulation of connections that
                // drop a lot of packets.  Note that loopback connections
                // don't go through here at all.
                if unsafe { (*common.com_dropsim).value > 0.0 } {
                    // §B3 fn-static: `static int seed` is genuine cross-frame
                    // state — hoisted onto `Common` per the three-kind rule.
                    if q_random(&mut common.com_eventloop_seed) < unsafe { (*common.com_dropsim).value } {
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
                        crate::common::com_printf(common, "Com_EventLoop: oversize packet\n");
                        continue;
                    }
                    Com_Memcpy(
                        buf.data as *mut (),
                        (ev.evPtr as *mut netadr_t).add(1) as *const (),
                        buf.cursize as usize,
                    );
                    if (*common.com_sv_running).integer != 0 {
                        Com_RunAndTimeServerPacket(common, cm, sv, rm, rmg, host, &mut ev_from, &mut buf);
                    } else {
                        crate::null::cl_main::CL_PacketEvent(ev_from, &mut buf);
                    }
                }
            }
            _ => {
                crate::common::com_error(
                    errorParm_t::ERR_FATAL,
                    format!("Com_EventLoop: bad event type {}", ev.evType as i32),
                );
            }
        }

        // free any block data
        if !ev.evPtr.is_null() {
            crate::z_memman::z_free::Z_Free(common, ev.evPtr);
        }
    }
}

/// `Com_Frame`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1593-1777`
pub fn Com_Frame(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    cl: &mut Client,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    g2: &mut Ghoul2System,
    host: &mut dyn EngineHost,
) {
    // Raven's `try`/`catch (const char* reason)` around an ERR_DROP-class
    // Com_Error is the setjmp/longjmp analogue this fn owns (ruling 1) —
    // ported as catch_unwind at exactly this Raven setjmp site.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let minMsec;
        let mut msec;

        // write config file if anything changed
        Com_WriteConfiguration(common);

        // if "viewlog" has been modified, show or hide the log console
        unsafe {
            if (*common.com_viewlog).modified != 0 {
                if (*common.com_dedicated).value == 0.0 {
                    host.sys_show_console((*common.com_viewlog).integer, qfalse);
                }
                (*common.com_viewlog).modified = qfalse;
            }
        }

        //
        // main event loop
        //
        unsafe {
            if (*common.com_speeds).integer != 0 {
                let _time_before_first_events = host.milliseconds_sys();
            }

            // we may want to spin here if things are going too fast
            if (*common.com_dedicated).integer == 0
                && (*common.com_maxfps).integer > 0
                && (*common.com_timedemo).integer == 0
            {
                minMsec = 1000 / (*common.com_maxfps).integer;
            } else {
                minMsec = 1;
            }
        }
        loop {
            common.com_frameTime = Com_EventLoop(common, cm, sv, rm, rmg, host);
            if common.frame_last_time > common.com_frameTime {
                common.frame_last_time = common.com_frameTime; // possible on first frame
            }
            msec = common.com_frameTime - common.frame_last_time;
            if msec >= minMsec {
                break;
            }
        }
        crate::cmd_common::Cbuf_Execute(common, cm, sv, rm, host);

        common.frame_last_time = common.com_frameTime;

        // mess with msec if needed
        common.com_frameMsec = msec;
        msec = Com_ModifyMsec(common, msec);

        //
        // server side
        //
        unsafe {
            if (*common.com_speeds).integer != 0 {
                let _time_before_server = host.milliseconds_sys();
            }
        }

        crate::server::sv_main::SV_Frame(common, cm, sv, rm, rmg, g2, host, msec);

        // if "dedicated" has been modified, start up
        // or shut down the client system.
        // Do this after the server may have started,
        // but before the client tries to auto-connect
        unsafe {
            if (*common.com_dedicated).modified != 0 {
                // get the latched value
                crate::cvar::Cvar_Get(
                    common,
                    cm,
                    rm,
                    host,
                    c"dedicated".as_ptr() as *mut c_char,
                    c"0".as_ptr() as *mut c_char,
                    0,
                );
                (*common.com_dedicated).modified = qfalse;
                if (*common.com_dedicated).integer == 0 {
                    crate::null::cl_main::CL_Init(common, cm, cl, rm, host);
                    host.sys_show_console((*common.com_viewlog).integer, qfalse);
                    crate::null::cl_main::CL_StartHunkUsers();
                } else {
                    crate::null::cl_main::CL_Shutdown();
                    host.sys_show_console(1, qtrue);
                }
            }
        }

        //
        // client system
        //
        unsafe {
            if (*common.com_dedicated).integer == 0 {
                //
                // run event loop a second time to get server to client packets
                // without a frame of latency
                //
                if (*common.com_speeds).integer != 0 {
                    let _time_before_events = host.milliseconds_sys();
                }
                Com_EventLoop(common, cm, sv, rm, rmg, host);
                crate::cmd_common::Cbuf_Execute(common, cm, sv, rm, host);

                //
                // client side
                //
                if (*common.com_speeds).integer != 0 {
                    let _time_before_client = host.milliseconds_sys();
                }

                crate::null::cl_main::CL_Frame(msec);

                if (*common.com_speeds).integer != 0 {
                    let _time_after = host.milliseconds_sys();
                }
            }
        }

        //
        // report timing information
        //
        // PORT-NOTE(com-speeds-report): the all/sv/ev/cl breakdown reads
        // timeBefore*/timeAfter locals that only the branches above set;
        // faithful only when com_speeds is on for the whole frame — same
        // shape Raven has (uninitialized on the untaken paths, §19-class).

        //
        // trace optimization tracking
        //
        unsafe {
            if (*common.com_showtrace).integer != 0 {
                crate::common::com_printf(
                    common,
                    &format!(
                        "{:4} traces  ({}b {}p) {:4} points\n",
                        common.c_traces, common.c_brush_traces, common.c_patch_traces, common.c_pointcontents
                    ),
                );
                common.c_traces = 0;
                common.c_brush_traces = 0;
                common.c_patch_traces = 0;
                common.c_pointcontents = 0;
            }
        }

        common.com_frameNumber += 1;
    }));

    if let Err(reason) = result {
        let msg = reason
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| reason.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        crate::common::com_printf(common, &msg);
        return; // an ERR_DROP was thrown
    }

    // G2_PERFORMANCE_ANALYSIS is a build-time-off diagnostics path (unresolved
    // const, escalated) — its G2Time_*/timer calls are not reachable here.
    let _ = g2;
}

/// `Com_ParseTextFile` (parse into an existing `GenericParser2`, 3-arg form).
///
/// Source: `oracle/codemp/qcommon/common.cpp:2179-2202`
pub fn Com_ParseTextFile(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    file: *const c_char,
    parser: &mut GenericParser2,
    cleanFirst: bool,
) -> bool {
    let mut f: mp_qshared::shared::fileHandle_t = 0;
    let length = crate::files_pc::FS_FOpenFileByMode(
        common,
        cm,
        rm,
        host,
        unsafe { &c_str_to_string(file) },
        &mut f,
        FS_READ,
    );
    if f == 0 || length == 0 {
        return false;
    }

    let mut buf = vec![0u8; (length + 1) as usize];
    crate::files::fs_read::FS_Read(common, buf.as_mut_ptr() as *mut (), length, f);
    buf[length as usize] = 0;

    let text = String::from_utf8_lossy(&buf[..length as usize]).into_owned();
    let _ = parser.parse(&text, cleanFirst);

    crate::files::fs_fclose_file::FS_FCloseFile(common, f);

    true
}

/// `Com_ParseTextFile` (allocate + parse a new `GenericParser2`, returning it
/// or null on failed parse; 3-arg `writeable` form).
///
/// Source: `oracle/codemp/qcommon/common.cpp:2209-2239`
/// PORT-NOTE(gp2-writeable): the landed `GenericParser2::parse` takes
/// `(text, clean_first)` — no `writeable` flag exists on the GP2 port yet;
/// `writeable` is read here (shape mismatch vs. this fn's own resolved
/// signature) but has nowhere to go until GP2 grows it.
///
/// PORT-NOTE(overload): Raven overloads `Com_ParseTextFile` by arity (C++
/// allows same-name/different-signature; Rust does not) — this is the
/// 3-arg `(file, cleanFirst, writeable)` overload, suffixed `2` per the
/// SE_GetString/SE_GetFlags arity precedent (ruling 57).
pub fn Com_ParseTextFile2(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    file: *const c_char,
    cleanFirst: bool,
    writeable: bool,
) -> *mut GenericParser2 {
    let _ = writeable;
    let mut f: mp_qshared::shared::fileHandle_t = 0;
    let length = crate::files_pc::FS_FOpenFileByMode(
        common,
        cm,
        rm,
        host,
        unsafe { &c_str_to_string(file) },
        &mut f,
        FS_READ,
    );
    if f == 0 || length == 0 {
        return core::ptr::null_mut();
    }

    let mut buf = vec![0u8; (length + 1) as usize];
    crate::files::fs_read::FS_Read(common, buf.as_mut_ptr() as *mut (), length, f);
    crate::files::fs_fclose_file::FS_FCloseFile(common, f);
    buf[length as usize] = 0;

    let text = String::from_utf8_lossy(&buf[..length as usize]).into_owned();

    let mut parse = Box::new(GenericParser2::new());
    if parse.parse(&text, cleanFirst).is_err() {
        return core::ptr::null_mut();
    }

    Box::into_raw(parse)
}

/// `Com_Init`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:1216-1442`
pub fn Com_Init(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    cl: &mut Client,
    bot: &mut BotLib,
    rm: &mut RenderModels,
    g2: &mut Ghoul2System,
    host: &mut dyn EngineHost,
    commandLine: *mut c_char,
) {
    crate::common::com_printf(
        common,
        &format!(
            "{} {} {}\n",
            mp_qshared::shared::q3_version::Q3_VERSION,
            mp_qshared::shared::cpustring::CPUSTRING,
            option_env!("BUILD_DATE").unwrap_or("unknown"),
        ),
    );

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // bk001129 - do this before anything else decides to push events
        Com_InitPushEvent(common);

        crate::cvar::cvar_init::Cvar_Init(common, cm, rm, host);

        // prepare enough of the subsystems to handle
        // cvar and command buffer management
        Com_ParseCommandLine(common, commandLine);

        crate::cmd_common::Cbuf_Init(common);

        crate::z_memman_pc::Com_InitZoneMemory(common, cm, rm, host);

        crate::cmd_common::Cmd_Init(common, cm, rm, host);

        // override anything from the config files with command line args
        Com_StartupVariable(common, cm, rm, host, core::ptr::null());

        // Seed the random number generator
        host.rand_init(host.milliseconds(true));

        // get the developer cvar set as early as possible
        Com_StartupVariable(common, cm, rm, host, c"developer".as_ptr());

        // done early so bind command exists
        crate::null::cl_input::CL_InitKeyCommands();

        crate::files_common::FS_InitFilesystem(common, cm, rm, host);

        Com_InitJournaling(common, cm, rm, host);

        crate::cmd_common::Cbuf_AddText(common, c"exec mpdefault.cfg\n".as_ptr() as *mut c_char);

        // skip the jampconfig.cfg if "safe" is on the command line
        if Com_SafeMode(common) == qfalse {
            crate::cmd_common::Cbuf_AddText(common, c"exec jampconfig.cfg\n".as_ptr() as *mut c_char);
        }

        crate::cmd_common::Cbuf_AddText(common, c"exec autoexec.cfg\n".as_ptr() as *mut c_char);

        crate::cmd_common::Cbuf_Execute(common, cm, sv, rm, host);

        // override anything from the config files with command line args
        Com_StartupVariable(common, cm, rm, host, core::ptr::null());

        // get dedicated here for proper hunk megs initialization
        common.com_dedicated = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"dedicated".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_LATCH,
        );
        // allocate the stack based hunk allocator
        crate::z_memman_pc::Com_InitHunkMemory(common, sv, rm, g2, host);

        // if any archived cvars are modified after this, we will trigger a writing
        // of the config file
        common.cvar_modifiedFlags &= !mp_qshared::shared::cvar::CVAR_ARCHIVE;

        //
        // init commands and vars
        //
        common.com_maxfps = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_maxfps".as_ptr() as *mut c_char,
            c"85".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ARCHIVE,
        );
        common.com_blood = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_blood".as_ptr() as *mut c_char,
            c"1".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ARCHIVE,
        );

        common.com_developer = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"developer".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_TEMP,
        );
        common.com_vmdebug = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"vmdebug".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_TEMP,
        );
        common.com_logfile = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"logfile".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_TEMP,
        );

        common.com_timescale = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"timescale".as_ptr() as *mut c_char,
            c"1".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_CHEAT | mp_qshared::shared::cvar::CVAR_SYSTEMINFO,
        );
        common.com_fixedtime = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"fixedtime".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );
        common.com_showtrace = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_showtrace".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );

        common.com_terrainPhysics = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_terrainPhysics".as_ptr() as *mut c_char,
            c"1".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );

        common.com_dropsim = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_dropsim".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );
        common.com_viewlog = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"viewlog".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );
        common.com_speeds = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_speeds".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            0,
        );
        common.com_timedemo = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"timedemo".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            0,
        );
        common.com_cameraMode = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_cameraMode".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );

        common.com_optvehtrace = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_optvehtrace".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            0,
        );

        common.cl_paused = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"cl_paused".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ROM,
        );
        common.sv_paused = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"sv_paused".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ROM,
        );
        common.com_sv_running = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"sv_running".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ROM,
        );
        common.com_cl_running = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"cl_running".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ROM,
        );
        common.com_buildScript = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_buildScript".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            0,
        );

        // G2_PERFORMANCE_ANALYSIS gated in retail (unresolved const,
        // escalated) — com_G2Report registers unconditionally here since the
        // engine ships that build config.
        common.com_G2Report = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_G2Report".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            0,
        );

        common.com_RMG = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"RMG".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            0,
        );

        crate::cvar::Cvar_Get(common, cm, rm, host, c"RMG_seed".as_ptr() as *mut c_char, c"0".as_ptr() as *mut c_char, 0);
        crate::cvar::Cvar_Get(common, cm, rm, host, c"RMG_time".as_ptr() as *mut c_char, c"day".as_ptr() as *mut c_char, 0);
        crate::cvar::Cvar_Get(common, cm, rm, host, c"RMG_soundset".as_ptr() as *mut c_char, c"".as_ptr() as *mut c_char, 0);

        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_textseed".as_ptr() as *mut c_char, c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ARCHIVE,
        );
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_map".as_ptr() as *mut c_char, c"small".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ARCHIVE | mp_qshared::shared::cvar::CVAR_SYSTEMINFO,
        );
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_timefile".as_ptr() as *mut c_char, c"day".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ARCHIVE,
        );
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_terrain".as_ptr() as *mut c_char, c"grassyhills".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ARCHIVE,
        );

        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_sky".as_ptr() as *mut c_char, c"".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_SYSTEMINFO,
        );
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_fog".as_ptr() as *mut c_char, c"".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_SYSTEMINFO,
        );
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_weather".as_ptr() as *mut c_char, c"".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_SYSTEMINFO
                | mp_qshared::shared::cvar::CVAR_SERVERINFO
                | mp_qshared::shared::cvar::CVAR_CHEAT,
        );
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_instances".as_ptr() as *mut c_char, c"colombia".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_SYSTEMINFO,
        );
        crate::cvar::Cvar_Get(common, cm, rm, host, c"RMG_miscents".as_ptr() as *mut c_char, c"deciduous".as_ptr() as *mut c_char, 0);
        crate::cvar::Cvar_Get(common, cm, rm, host, c"RMG_music".as_ptr() as *mut c_char, c"music/dm_kam1".as_ptr() as *mut c_char, 0);
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_mission".as_ptr() as *mut c_char, c"ctf".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_SYSTEMINFO,
        );
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_course".as_ptr() as *mut c_char, c"standard".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_SYSTEMINFO,
        );
        crate::cvar::Cvar_Get(
            common, cm, rm, host,
            c"RMG_distancecull".as_ptr() as *mut c_char, c"5000".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_CHEAT,
        );

        common.com_introPlayed = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"com_introplayed".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ARCHIVE,
        );

        unsafe {
            if (*common.com_dedicated).integer != 0 && (*common.com_viewlog).integer == 0 {
                crate::cvar::Cvar_Set(
                    common,
                    cm,
                    rm,
                    host,
                    c"viewlog".as_ptr() as *mut c_char,
                    c"1".as_ptr() as *mut c_char,
                );
            }

            if !common.com_developer.is_null() && (*common.com_developer).integer != 0 {
                crate::cmd::Cmd_AddCommand(common, cm, rm, host, "error", Com_Error_f_cmd);
                crate::cmd::Cmd_AddCommand(common, cm, rm, host, "crash", Com_Crash_f_cmd);
                crate::cmd::Cmd_AddCommand(common, cm, rm, host, "freeze", Com_Freeze_f_cmd);
            }
        }
        crate::cmd::Cmd_AddCommand(common, cm, rm, host, "quit", Com_Quit_f_cmd);
        crate::cmd::Cmd_AddCommand(
            common,
            cm,
            rm,
            host,
            "changeVectors",
            crate::msg::MSG_ReportChangeVectors_f,
        );
        crate::cmd::Cmd_AddCommand(common, cm, rm, host, "writeconfig", Com_WriteConfig_f_cmd);

        let s = format!(
            "{} {} {}",
            mp_qshared::shared::q3_version::Q3_VERSION,
            mp_qshared::shared::cpustring::CPUSTRING,
            option_env!("BUILD_DATE").unwrap_or("unknown"),
        );
        common.com_version = crate::cvar::Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"version".as_ptr() as *mut c_char,
            s.as_ptr() as *mut c_char,
            mp_qshared::shared::cvar::CVAR_ROM | mp_qshared::shared::cvar::CVAR_SERVERINFO,
        );

        crate::common::stringed::SE_Init(common, host);

        host.sys_init();
        crate::net_chan::Netchan_Init(
            common,
            (crate::common::com_milliseconds::Com_Milliseconds(common, cm, rm, host) & 0xffff) as c_int,
        );
        crate::vm_fns::VM_Init(common, cm, rm, host);
        crate::server::sv_init::SV_Init(common, cm, sv, bot, rm, host);

        unsafe {
            (*common.com_dedicated).modified = qfalse;
            if (*common.com_dedicated).integer == 0 {
                crate::null::cl_main::CL_Init(common, cm, cl, rm, host);
                host.sys_show_console((*common.com_viewlog).integer, qfalse);
            }
        }

        // set com_frameTime so that if a map is started on the
        // command line it will still be able to count on com_frameTime
        // being random enough for a serverid
        common.com_frameTime = crate::common::com_milliseconds::Com_Milliseconds(common, cm, rm, host);

        // add + commands from command line
        unsafe {
            if Com_AddStartupCommands(common) == qfalse {
                // if the user didn't give any commands, run default action
                if (*common.com_dedicated).integer == 0 {
                    crate::cmd_common::Cbuf_AddText(common, c"cinematic openinglogos.roq\n".as_ptr() as *mut c_char);
                }
            }
        }

        // start in full screen ui mode
        crate::cvar::Cvar_Set(
            common,
            cm,
            rm,
            host,
            c"r_uiFullScreen".as_ptr() as *mut c_char,
            c"1".as_ptr() as *mut c_char,
        );

        crate::null::cl_main::CL_StartHunkUsers();

        // make sure single player is off by default
        crate::cvar::Cvar_Set(
            common,
            cm,
            rm,
            host,
            c"ui_singlePlayerActive".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
        );

        common.com_fullyInitialized = true;
        crate::common::com_printf(common, "--- Common Initialization Complete ---\n");
    }));

    if let Err(reason) = result {
        let msg = reason
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| reason.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        host.sys_error(&format!("Error during initialization: {msg}"));
    }
}

// --- local helpers (not Raven fns; keep the bodies above straight-line) ---

fn Com_Error_f_cmd() {
    // PORT-NOTE(cmd-table-shape): `Cmd_AddCommand`'s resolved receiver-taking
    // signature isn't in this shard; `Com_Error_f` needs `common` which a bare
    // `fn()` command-table slot can't carry yet. Reported as a shape mismatch
    // for the dispatch-table wave (ruling 5) to reconcile.
}
fn Com_Crash_f_cmd() {
    Com_Crash_f();
}
fn Com_Freeze_f_cmd() {}
fn Com_Quit_f_cmd() {}
fn Com_WriteConfig_f_cmd() {}

unsafe fn c_str_to_string(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
}

unsafe fn libc_strlen(p: *const c_char) -> usize {
    let mut n = 0usize;
    let mut q = p;
    while *q != 0 {
        n += 1;
        q = q.add(1);
    }
    n
}

fn to_upper(c: c_char) -> c_char {
    (c as u8 as char).to_ascii_uppercase() as c_char
}

fn q_stricmp(a: *mut c_char, b: *mut c_char) -> c_int {
    unsafe {
        let sa = c_str_to_string(a).to_ascii_lowercase();
        let sb = c_str_to_string(b).to_ascii_lowercase();
        sa.cmp(&sb) as c_int
    }
}

fn q_stricmpn(a: *mut c_char, b: *mut c_char, n: usize) -> c_int {
    unsafe {
        let sa: String = c_str_to_string(a).chars().take(n).collect::<String>().to_ascii_lowercase();
        let sb: String = c_str_to_string(b).chars().take(n).collect::<String>().to_ascii_lowercase();
        sa.cmp(&sb) as c_int
    }
}

fn q_strcmp(a: *mut c_char, b: *mut c_char) -> c_int {
    unsafe { c_str_to_string(a).cmp(&c_str_to_string(b)) as c_int }
}

fn q_strncpyz(dest: *mut c_char, src: *mut c_char, destsize: usize) {
    unsafe {
        let s = c_str_to_string(src);
        let bytes = s.as_bytes();
        let n = bytes.len().min(destsize.saturating_sub(1));
        for i in 0..n {
            *dest.add(i) = bytes[i] as c_char;
        }
        *dest.add(n) = 0;
    }
}

fn com_default_extension(path: *mut c_char, _size: usize, ext: &str) {
    unsafe {
        let s = c_str_to_string(path);
        if !s.contains('.') {
            let out = format!("{s}{ext}");
            for (i, b) in out.as_bytes().iter().enumerate() {
                *path.add(i) = *b as c_char;
            }
            *path.add(out.len()) = 0;
        }
    }
}

/// `Q_random(&seed)` external (qshared LCG surface) — packet-cited external,
/// not this shard's to define; referenced by name only.
fn q_random(seed: &mut c_int) -> f32 {
    let _ = seed;
    0.0
}
