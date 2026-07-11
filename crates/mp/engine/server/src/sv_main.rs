//! `sv_main.cpp` — server main loop helpers: newline expansion, pending
//! server-command replacement, pause check, and master-server resolve throttle.
//!
//! Source: `oracle/codemp/server/sv_main.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue, MAX_STRING_CHARS};

use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server_host::NEW_RESOLVE_DURATION;
use crate::Server;
use mp_qshared::shared::q_string::{Q_strncmp, Q_strncpyz};

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn sscanf(s: *const c_char, fmt: *const c_char, ...) -> c_int;
}

/// Raven `SV_ExpandNewlines` — expands `\n` in a string to a slash and 'n'
/// (so it can be printed on a single line).
///
/// Source: `oracle/codemp/server/sv_main.cpp:59-76`
pub fn SV_ExpandNewlines(sv: &mut Server, r#in: *mut c_char) -> *mut c_char {
    let string = &mut sv.sv_expand_newlines_string;
    let mut l: usize = 0;

    unsafe {
        let mut p = r#in;
        while *p != 0 && l < string.len() - 3 {
            if *p == b'\n' as c_char {
                string[l] = b'\\' as c_char;
                l += 1;
                string[l] = b'n' as c_char;
                l += 1;
            } else {
                string[l] = *p;
                l += 1;
            }
            p = p.offset(1);
        }
        string[l] = 0;
    }

    string.as_mut_ptr()
}

/// Raven `SV_ReplacePendingServerCommands`.
///
/// Source: `oracle/codemp/server/sv_main.cpp:85-106`
pub fn SV_ReplacePendingServerCommands(client: *mut client_t, cmd: *const c_char) -> c_int {
    unsafe {
        let mut i = (*client).reliableSent + 1;
        while i <= (*client).reliableSequence {
            let index = (i & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize;
            //
            if Q_strncmp(
                cmd,
                (*client).reliableCommands[index].as_ptr(),
                strlen(c"cs".as_ptr()) as c_int,
            ) == 0
            {
                let mut csnum1: c_int = 0;
                let mut csnum2: c_int = 0;
                sscanf(cmd, c"cs %i".as_ptr(), &mut csnum1 as *mut c_int);
                sscanf(
                    (*client).reliableCommands[index].as_ptr(),
                    c"cs %i".as_ptr(),
                    &mut csnum2 as *mut c_int,
                );
                if csnum1 == csnum2 {
                    Q_strncpyz(
                        (*client).reliableCommands[index].as_mut_ptr(),
                        cmd,
                        MAX_STRING_CHARS as c_int,
                    );
                    /*
                    if ( client->netchan.remoteAddress.type != NA_BOT ) {
                        Com_Printf( "WARNING: client %i removed double pending config string %i: %s\n", client-svs.clients, csnum1, cmd );
                    }
                    */
                    return qtrue;
                }
            }
            i += 1;
        }
        qfalse
    }
}

/// Raven `SV_CheckPaused` — only pause if there is just a single client
/// connected.
///
/// Source: `oracle/codemp/server/sv_main.cpp:759-784`
pub fn SV_CheckPaused(common: &mut Common, sv: &mut Server) -> qboolean {
    unsafe {
        if (*common.cl_paused).integer == 0 {
            return qfalse;
        }

        // only pause if there is just a single client connected
        let mut count = 0;
        for i in 0..(*common.sv_maxclients).integer {
            let cl = sv.svs.clients.offset(i as isize);
            if (*cl).state as c_int >= clientState_t::CS_CONNECTED as c_int
                && (*cl).netchan.remoteAddress.r#type != netadrtype_t::NA_BOT
            {
                count += 1;
            }
        }

        if count > 1 {
            // don't pause
            (*common.sv_paused).integer = 0;
            return qfalse;
        }

        (*common.sv_paused).integer = 1;
        qtrue
    }
}

/// Raven `SV_MasterNeedsResolving` — refresh every so often regardless of if the
/// actual address was modified. -rww
///
/// Source: `oracle/codemp/server/sv_main.cpp:194-207`
pub fn SV_MasterNeedsResolving(sv: &mut Server, server: c_int, time: c_int) -> bool {
    if sv.master_heartbeat[server as usize] > time {
        // time flowed backwards?
        return true;
    }

    if (time - sv.master_heartbeat[server as usize]) > NEW_RESOLVE_DURATION {
        // it's time again
        return true;
    }

    false
}
