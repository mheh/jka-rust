#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `l_log.cpp` (bot library log file open/close/
//! write/flush/shutdown).
//!
//! Ported per the engine C-track packets (`botlib__0583`..`botlib__1560`).
//! Source: `oracle/codemp/botlib/l_log.cpp`.
//!
// The `bot: &mut BotLib` receiver named in every signature below is the
// campaign's threaded-state aggregate (ruling 2); `BotLib` does not exist in
// this worktree slice yet (`_PREAMBLE.md`'s "botlib waves" note,
// `be_aas_main.rs`/`be_aas_debug_fns.rs` precedent). Every reference to
// `logfile`/`botimport`/`botlibglobals` below is the exact Raven global name
// per house rule, reached as a field on `bot` — resolved when the aggregate
// lands.

use core::ffi::c_char;

use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_MESSAGE};

use crate::l_log::consts::MAX_LOGFILENAMESIZE;
use crate::BotLib;

use crate::l_libvar_fns::LibVarValue;

/// Raven `Log_Close`.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:69-79`
pub fn Log_Close(bot: &mut BotLib) {
    unsafe {
        if bot.logfile.fp.is_null() {
            return;
        }
        if libc::fclose(bot.logfile.fp) != 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"can't close log file %s\n".as_ptr() as *mut c_char,
                bot.logfile.filename.as_ptr(),
            );
            return;
        }
        bot.logfile.fp = core::ptr::null_mut();
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"Closed log %s\n".as_ptr() as *mut c_char,
            bot.logfile.filename.as_ptr(),
        );
    }
}

/// Raven `Log_Write`.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:96-106`
///
/// Stable Rust cannot define a C-variadic fn; per the `Com_OPrintf` precedent
/// (`crates/mp/engine/qcommon/src/common_fns.rs`), the `va_list`/`vfprintf`
/// formatting collapses to a pre-formatted `msg` written at the call site.
pub fn Log_Write(bot: &mut BotLib, msg: *mut c_char) {
    unsafe {
        if bot.logfile.fp.is_null() {
            return;
        }
        libc::fprintf(bot.logfile.fp, c"%s".as_ptr(), msg);
        //fprintf(logfile.fp, "\r\n");
        libc::fflush(bot.logfile.fp);
    }
}

/// Raven `Log_WriteTimeStamped`.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:113-131`
pub fn Log_WriteTimeStamped(bot: &mut BotLib, msg: *mut c_char) {
    unsafe {
        if bot.logfile.fp.is_null() {
            return;
        }
        libc::fprintf(
            bot.logfile.fp,
            c"%d   %02d:%02d:%02d:%02d   ".as_ptr(),
            bot.logfile.numwrites,
            (bot.botlibglobals.time / 60.0 / 60.0) as core::ffi::c_int,
            (bot.botlibglobals.time / 60.0) as core::ffi::c_int,
            (bot.botlibglobals.time) as core::ffi::c_int,
            ((bot.botlibglobals.time * 100.0) as core::ffi::c_int)
                - (bot.botlibglobals.time as core::ffi::c_int) * 100,
        );
        libc::fprintf(bot.logfile.fp, c"%s".as_ptr(), msg);
        libc::fprintf(bot.logfile.fp, c"\r\n".as_ptr());
        bot.logfile.numwrites += 1;
        libc::fflush(bot.logfile.fp);
    }
}

/// Raven `Log_FilePointer`.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:138-141`
pub fn Log_FilePointer(bot: &mut BotLib) -> *mut libc::FILE {
    bot.logfile.fp
}

/// Raven `Log_Flush`.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:148-151`
pub fn Log_Flush(bot: &mut BotLib) {
    unsafe {
        if !bot.logfile.fp.is_null() {
            libc::fflush(bot.logfile.fp);
        }
    }
}

/// Raven `Log_Shutdown`.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:86-89`
pub fn Log_Shutdown(bot: &mut BotLib) {
    if !bot.logfile.fp.is_null() {
        Log_Close(bot);
    }
}

/// Raven `Log_Open`.
///
/// Source: `oracle/codemp/botlib/l_log.cpp:41-62`
pub fn Log_Open(bot: &mut BotLib, filename: *mut c_char) {
    unsafe {
        if LibVarValue(bot, "log", "0") == 0.0
        {
            return;
        }
        if filename.is_null() || libc::strlen(filename) == 0 {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"openlog <filename>\n".as_ptr() as *mut c_char,
            );
            return;
        }
        if !bot.logfile.fp.is_null() {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"log file %s is already opened\n".as_ptr() as *mut c_char,
                bot.logfile.filename.as_ptr(),
            );
            return;
        }
        bot.logfile.fp = libc::fopen(filename, c"wb".as_ptr());
        if bot.logfile.fp.is_null() {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"can't open the log file %s\n".as_ptr() as *mut c_char,
                filename,
            );
            return;
        }
        libc::strncpy(
            bot.logfile.filename.as_mut_ptr(),
            filename,
            MAX_LOGFILENAMESIZE as usize,
        );
        bot.botimport.Print.unwrap()(
            PRT_MESSAGE,
            c"Opened log %s\n".as_ptr() as *mut c_char,
            bot.logfile.filename.as_ptr(),
        );
    }
}
