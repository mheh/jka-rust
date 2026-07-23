//! `sv_ccmds.cpp` — server console commands: player lookup helpers, kick/status/
//! say/force-toggle/map/killserver/map_restart commands, and the operator
//! command-table registration.
//!
//! Source: `oracle/codemp/server/sv_ccmds.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};
use std::ffi::CString;

use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::cmd::cmd_function_t::CmdFunction;
use mp_engine_qcommon::cmd::Cmd_AddCommand;
use mp_engine_qcommon::cmd_common::{Cmd_Argc, Cmd_Args, Cmd_Argv};
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common_fns::Info_Print;
use mp_engine_qcommon::cvar_fns::{
    Cvar_Get, Cvar_InfoString, Cvar_Set, Cvar_SetLatched, Cvar_SetValue, Cvar_VariableString,
    Cvar_VariableValue,
};
use mp_engine_qcommon::files_common::FS_ReadFile;
use mp_engine_qcommon::net_chan::NET_AdrToString;
use mp_engine_qcommon::stringed::SE_GetString;
use mp_engine_qcommon::vm::VM_Call;
use mp_engine_qcommon::vm_fns::VM_ExplicitArgPtr;
use mp_qshared::common::mp::playerstate::PERS_SCORE;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::shared::cvar::{CVAR_LATCH, CVAR_SERVERINFO, CVAR_SYSTEMINFO};
use mp_qshared::shared::force_powers::NUM_FORCE_POWERS;
use mp_qshared::shared::force_reload::ForceReload_e;
use mp_qshared::shared::q_string::Q_CleanStr;
use mp_qshared::shared::{qfalse, qtrue, SNAPFLAG_SERVERCOUNT};
use native_string::atoi::atoi;
use native_string::q_string::{Q_stricmp, Q_stricmpBytes, Q_stricmpn};
use native_string::q_strncpyz::Q_strncpyzBytes;

use crate::hook_install::sv_from_view;
use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server::server_state_t::serverState_t;
use crate::sv_client::SV_ClientEnterWorld;
use crate::sv_game::{SV_GameClientNum, SV_RestartGameProgs};
use crate::sv_init::{SV_SetConfigstring, SV_Shutdown, SV_SpawnServer};
use crate::sv_world::SV_SectorList_f;
use crate::{SV_AddServerCommand, SV_DropClient, SV_SendServerCommand, Server};

/// Raven `SV_GetStringEdString`.
///
/// Raven: Well, it would've been lovely doing it the above way, but it would
/// mean mixing languages for the client depending on what the server is. So
/// we'll mark this as a stringed reference with @@@ and send the refname to
/// the client, and when it goes to print it will get scanned for the
/// stringed reference indication and dealt with properly.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:16-32`
pub fn SV_GetStringEdString(sv: &mut Server, _refSection: &str, refName: &str) -> String {
    let _ = sv;
    // Function-scope static `text` (fork-3 kind 2: rotating scratch/return
    // buffer) becomes an owned return value instead of a hidden cell.
    format!("@@@{refName}")
}

/// Raven `SV_GetPlayerByFedName`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:354-387`
pub fn SV_GetPlayerByFedName(common: &mut Common, sv: &mut Server, name: &str) -> *mut client_t {
    // make sure server is running
    if common.cvar(common.com_sv_running).integer == 0 {
        return core::ptr::null_mut();
    }

    // check for a name match
    let n = sv.svs.clients.as_mut_ptr();
    for i in 0..common.cvar(common.sv_maxclients).integer {
        let cl = unsafe { n.offset(i as isize) };
        if unsafe { (*cl).state as i32 } == 0 {
            continue;
        }
        // client_t.name is a String now; byte-exact compares at the site (Q_CleanStr
        // works on the local C-copy).
        unsafe {
            if Q_stricmpBytes(
                (*cl).name.as_bytes(),
                name.as_bytes(),
            ) == 0
            {
                return cl;
            }

            let mut cleanName = [0 as c_char; 64];
            let clean_len = cleanName.len();
            Q_strncpyzBytes(
                &mut cleanName,
                (*cl).name.as_bytes(),
                clean_len,
            );
            Q_CleanStr(cleanName.as_mut_ptr());
            if Q_stricmpBytes(
                core::ffi::CStr::from_ptr(cleanName.as_ptr()).to_bytes(),
                name.as_bytes(),
            ) == 0
            {
                return cl;
            }
        }
    }

    core::ptr::null_mut()
}

/// Raven `SV_Heartbeat_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:876-878`
pub fn SV_Heartbeat_f(sv: &mut Server) {
    sv.svs.nextHeartbeatTime = -9999999;
}

/// Raven `SV_RemoveOperatorCommands`.
///
/// Raven: `#if 0`-guarded — removing these won't let the server start again.
/// Faithful transcription of the dead `#if 0` body: no-op.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:1003-1018`
pub fn SV_RemoveOperatorCommands() {
    // §C10: the entire body is `#if 0`; faithful transcription is a no-op.
}

/// Raven `SV_GetPlayerByName`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:43-80`
pub fn SV_GetPlayerByName(common: &mut Common, sv: &mut Server) -> *mut client_t {
    // make sure server is running
    if common.cvar(common.com_sv_running).integer == 0 {
        return core::ptr::null_mut();
    }

    if Cmd_Argc(common) < 2 {
        com_printf(common, "No player specified.\n");
        return core::ptr::null_mut();
    }

    let s = Cmd_Argv(common, 1).to_owned();

    // check for a name match
    let n = sv.svs.clients.as_mut_ptr();
    for i in 0..common.cvar(common.sv_maxclients).integer {
        let cl = unsafe { n.offset(i as isize) };
        if unsafe { (*cl).state as i32 } == 0 {
            continue;
        }
        // client_t.name is a String now; byte-exact compares at the site (Q_CleanStr
        // works on the local C-copy).
        unsafe {
            if Q_stricmpBytes(
                (*cl).name.as_bytes(),
                s.as_bytes(),
            ) == 0
            {
                return cl;
            }

            let mut cleanName = [0 as c_char; 64];
            let clean_len = cleanName.len();
            Q_strncpyzBytes(
                &mut cleanName,
                (*cl).name.as_bytes(),
                clean_len,
            );
            Q_CleanStr(cleanName.as_mut_ptr());
            if Q_stricmpBytes(
                core::ffi::CStr::from_ptr(cleanName.as_ptr()).to_bytes(),
                s.as_bytes(),
            ) == 0
            {
                return cl;
            }
        }
    }

    com_printf(common, &format!("Player {s} is not on the server\n"));

    core::ptr::null_mut()
}

/// Raven `SV_GetPlayerByNum`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:89-125`
pub fn SV_GetPlayerByNum(common: &mut Common, sv: &mut Server) -> *mut client_t {
    // make sure server is running
    if common.cvar(common.com_sv_running).integer == 0 {
        return core::ptr::null_mut();
    }

    if Cmd_Argc(common) < 2 {
        com_printf(common, "No player specified.\n");
        return core::ptr::null_mut();
    }

    let s = Cmd_Argv(common, 1).to_owned();

    for c in s.bytes() {
        if !c.is_ascii_digit() {
            com_printf(common, &format!("Bad slot number: {s}\n"));
            return core::ptr::null_mut();
        }
    }
    let idnum = atoi(&s);
    if idnum < 0 || idnum >= common.cvar(common.sv_maxclients).integer {
        com_printf(common, &format!("Bad client slot: {}\n", idnum));
        return core::ptr::null_mut();
    }

    let cl = &mut sv.svs.clients[idnum as usize] as *mut client_t;
    if unsafe { (*cl).state as i32 } == 0 {
        com_printf(common, &format!("Client {} is not active\n", idnum));
        return core::ptr::null_mut();
    }
    cl
}

/// Raven `SV_KickByName`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:389-446`
pub fn SV_KickByName(common: &mut Common, sv: &mut Server, name: &str) {
    // make sure server is running
    if common.cvar(common.com_sv_running).integer == 0 {
        return;
    }

    let cl = SV_GetPlayerByFedName(common, sv, name);
    if cl.is_null() {
        if Q_stricmp(name, "all") == 0 {
            let n = sv.svs.clients.as_mut_ptr();
            for i in 0..common.cvar(common.sv_maxclients).integer {
                let cl = unsafe { n.offset(i as isize) };
                if unsafe { (*cl).state as i32 } == 0 {
                    continue;
                }
                if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
                    continue;
                }
                let reason = SV_GetStringEdString(sv, "MP_SVGAME", "WAS_KICKED");
                SV_DropClient(common, sv, cl, &reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        } else if Q_stricmp(name, "allbots") == 0 {
            let n = sv.svs.clients.as_mut_ptr();
            for i in 0..common.cvar(common.sv_maxclients).integer {
                let cl = unsafe { n.offset(i as isize) };
                if unsafe { (*cl).state as i32 } == 0 {
                    continue;
                }
                if unsafe { (*cl).netchan.remoteAddress.r#type } != netadrtype_t::NA_BOT {
                    continue;
                }
                let reason = SV_GetStringEdString(sv, "MP_SVGAME", "WAS_KICKED");
                SV_DropClient(common, sv, cl, &reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        }
        return;
    }
    if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
        // SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        let reason = SV_GetStringEdString(sv, "MP_SVGAME", "CANNOT_KICK_HOST");
        SV_SendServerCommand(
            common,
            sv,
            core::ptr::null_mut(),
            &format!("print \"{reason}\""),
        );
        return;
    }

    let reason = SV_GetStringEdString(sv, "MP_SVGAME", "WAS_KICKED");
    SV_DropClient(common, sv, cl, &reason); // "was kicked"
    unsafe {
        (*cl).lastPacketTime = sv.svs.time;
    } // in case there is a funny zombie
}

/// Raven `SV_Status_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:669-750`
pub fn SV_Status_f(view: &mut EngineHostView, sv: &mut Server) {
    let mut avoidTruncation = qfalse;

    // make sure server is running
    if view.common.cvar(view.common.com_sv_running).integer == 0 {
        let msg = SE_GetString(view, "STR_SERVER_SERVER_NOT_RUNNING");
        com_printf(view.common, &msg);
        return;
    }

    if Cmd_Argc(view.common) > 1 {
        if Q_stricmp("notrunc", Cmd_Argv(view.common, 1)) == 0 {
            avoidTruncation = qtrue;
        }
    }

    com_printf(
        view.common,
        &format!("map: {}\n", view.common.cvar(view.common.sv_mapname).string),
    );

    com_printf(
        view.common,
        "num score ping name            lastmsg address               qport rate\n",
    );
    com_printf(
        view.common,
        "--- ----- ---- --------------- ------- --------------------- ----- -----\n",
    );
    for i in 0..view.common.cvar(view.common.sv_maxclients).integer {
        let cl = &sv.svs.clients[i as usize] as *const client_t;
        if unsafe { (*cl).state as i32 } == 0 {
            continue;
        }

        let state = if unsafe { (*cl).state } == clientState_t::CS_CONNECTED {
            "CNCT ".to_string()
        } else if unsafe { (*cl).state } == clientState_t::CS_ZOMBIE {
            "ZMBI ".to_string()
        } else {
            let ping = if unsafe { (*cl).ping } < 9999 {
                unsafe { (*cl).ping }
            } else {
                9999
            };
            format!("{:4}", ping)
        };

        let ps = SV_GameClientNum(sv, i);
        let s = unsafe {
            core::ffi::CStr::from_ptr(NET_AdrToString(view.common, (*cl).netchan.remoteAddress))
                .to_string_lossy()
                .into_owned()
        };

        let name = unsafe { (*cl).name.clone() };
        if avoidTruncation == qfalse {
            com_printf(
                view.common,
                &format!(
                    "{:3} {:5} {} {:<15.15} {:7} {:>21} {:5} {:5}\n",
                    i,
                    unsafe { (*ps).persistant[PERS_SCORE as usize] },
                    state,
                    name,
                    unsafe { sv.svs.time - (*cl).lastPacketTime },
                    s,
                    unsafe { (*cl).netchan.qport },
                    unsafe { (*cl).rate },
                ),
            );
        } else {
            com_printf(
                view.common,
                &format!(
                    "{:3} {:5} {} {} {:7} {:>21} {:5} {:5}\n",
                    i,
                    unsafe { (*ps).persistant[PERS_SCORE as usize] },
                    state,
                    name,
                    unsafe { sv.svs.time - (*cl).lastPacketTime },
                    s,
                    unsafe { (*cl).netchan.qport },
                    unsafe { (*cl).rate },
                ),
            );
        }
    }
    com_printf(view.common, "\n");
}

/// Raven `SV_ConSay_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:757-787`
pub fn SV_ConSay_f(common: &mut Common, sv: &mut Server) {
    if common.cvar(common.com_dedicated).integer == 0 {
        com_printf(common, "Server is not dedicated.\n");
        return;
    }

    // make sure server is running
    if common.cvar(common.com_sv_running).integer == 0 {
        com_printf(common, "Server is not running.\n");
        return;
    }

    if Cmd_Argc(common) < 2 {
        return;
    }

    let mut text = "Server: ".to_string();
    let mut p = Cmd_Args(common);

    if p.starts_with('"') {
        p.remove(0);
        if !p.is_empty() {
            p.pop();
        }
    }

    text.push_str(&p);

    SV_SendServerCommand(
        common,
        sv,
        core::ptr::null_mut(),
        &format!("chat \"{}\n\"", text),
    );
}

/// Raven `SV_ForceToggle_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:817-867`
pub fn SV_ForceToggle_f(view: &mut EngineHostView, sv: &mut Server) {
    let _ = sv;
    let mut fpDisabled = Cvar_VariableValue(view.common, "g_forcePowerDisable") as c_int;
    let targetPower: c_int;
    let mut powerDisabled;

    if Cmd_Argc(view.common) < 2 {
        // no argument supplied, spit out a list of force powers and their numbers
        let mut i: c_int = 0;
        while i < NUM_FORCE_POWERS {
            powerDisabled = if fpDisabled & (1 << i) != 0 {
                "Disabled"
            } else {
                "Enabled"
            };

            com_printf(
                view.common,
                &format!(
                    "{} - {} - Status: {}\n",
                    i, FORCE_TOGGLE_NAME_PRINTS[i as usize], powerDisabled
                ),
            );
            i += 1;
        }

        com_printf(
            view.common,
            "Example usage: forcetoggle 3\n(toggles PUSH)\n",
        );
        return;
    }

    targetPower = atoi(Cmd_Argv(view.common, 1));

    if targetPower < 0 || targetPower >= NUM_FORCE_POWERS {
        com_printf(
            view.common,
            "Specified a power that does not exist.\nExample usage: forcetoggle 3\n(toggles PUSH)\n",
        );
        return;
    }

    if fpDisabled & (1 << targetPower) != 0 {
        powerDisabled = "enabled";
        fpDisabled &= !(1 << targetPower);
    } else {
        powerDisabled = "disabled";
        fpDisabled |= 1 << targetPower;
    }

    Cvar_Set(view, "g_forcePowerDisable", &format!("{}", fpDisabled));

    com_printf(
        view.common,
        &format!(
            "{} has been {}.\n",
            FORCE_TOGGLE_NAME_PRINTS[targetPower as usize], powerDisabled
        ),
    );
}

/// Raven `forceToggleNamePrints[]` — file-scope const table (fork-3 kind 1).
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:789-810`
const FORCE_TOGGLE_NAME_PRINTS: [&str; NUM_FORCE_POWERS as usize] = [
    "HEAL",           // FP_HEAL
    "JUMP",           // FP_LEVITATION
    "SPEED",          // FP_SPEED
    "PUSH",           // FP_PUSH
    "PULL",           // FP_PULL
    "MINDTRICK",      // FP_TELEPATHY
    "GRIP",           // FP_GRIP
    "LIGHTNING",      // FP_LIGHTNING
    "DARK RAGE",      // FP_RAGE
    "PROTECT",        // FP_PROTECT
    "ABSORB",         // FP_ABSORB
    "TEAM HEAL",      // FP_TEAM_HEAL
    "TEAM REPLENISH", // FP_TEAM_FORCE
    "DRAIN",          // FP_DRAIN
    "SEEING",         // FP_SEE
    "SABER OFFENSE",  // FP_SABER_OFFENSE
    "SABER DEFENSE",  // FP_SABER_DEFENSE
    "SABER THROW",    // FP_SABERTHROW
];

/// Raven `SV_KillServer_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:947-949`
pub fn SV_KillServer_f(view: &mut EngineHostView) {
    // `SV_Shutdown` is a hook target (`fn(&mut EngineHostView, &str)`) that
    // sources the `Server`/`RmManager` from the view's own slots, so the real
    // `sv` receiver this forwarder used to thread collapses into the view.
    SV_Shutdown(view, "killserver");
}

/// Raven `SV_Kick_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:455-511`
pub fn SV_Kick_f(common: &mut Common, sv: &mut Server) {
    // make sure server is running
    if common.cvar(common.com_sv_running).integer == 0 {
        com_printf(common, "Server is not running.\n");
        return;
    }

    if Cmd_Argc(common) != 2 {
        com_printf(
            common,
            "Usage: kick <player name>\nkick all = kick everyone\nkick allbots = kick all bots\n",
        );
        return;
    }

    if Q_stricmp(Cmd_Argv(common, 1), "Padawan") == 0 {
        // if you try to kick the default name, also try to kick ""
        SV_KickByName(common, sv, "");
    }

    let cl = SV_GetPlayerByName(common, sv);
    if cl.is_null() {
        if Q_stricmp(Cmd_Argv(common, 1), "all") == 0 {
            let n = sv.svs.clients.as_mut_ptr();
            for i in 0..common.cvar(common.sv_maxclients).integer {
                let cl = unsafe { n.offset(i as isize) };
                if unsafe { (*cl).state as i32 } == 0 {
                    continue;
                }
                if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
                    continue;
                }
                let reason = SV_GetStringEdString(sv, "MP_SVGAME", "WAS_KICKED");
                SV_DropClient(common, sv, cl, &reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        } else if Q_stricmp(Cmd_Argv(common, 1), "allbots") == 0 {
            let n = sv.svs.clients.as_mut_ptr();
            for i in 0..common.cvar(common.sv_maxclients).integer {
                let cl = unsafe { n.offset(i as isize) };
                if unsafe { (*cl).state as i32 } == 0 {
                    continue;
                }
                if unsafe { (*cl).netchan.remoteAddress.r#type } != netadrtype_t::NA_BOT {
                    continue;
                }
                let reason = SV_GetStringEdString(sv, "MP_SVGAME", "WAS_KICKED");
                SV_DropClient(common, sv, cl, &reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        }
        return;
    }
    if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
        // SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        let reason = SV_GetStringEdString(sv, "MP_SVGAME", "CANNOT_KICK_HOST");
        SV_SendServerCommand(
            common,
            sv,
            core::ptr::null_mut(),
            &format!("print \"{reason}\""),
        );
        return;
    }

    let reason = SV_GetStringEdString(sv, "MP_SVGAME", "WAS_KICKED");
    SV_DropClient(common, sv, cl, &reason); // "was kicked"
    unsafe {
        (*cl).lastPacketTime = sv.svs.time;
    } // in case there is a funny zombie
}

/// Raven `SV_KickNum_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:636-662`
pub fn SV_KickNum_f(common: &mut Common, sv: &mut Server) {
    // make sure server is running
    if common.cvar(common.com_sv_running).integer == 0 {
        com_printf(common, "Server is not running.\n");
        return;
    }

    if Cmd_Argc(common) != 2 {
        com_printf(common, "Usage: kicknum <client number>\n");
        return;
    }

    let cl = SV_GetPlayerByNum(common, sv);
    if cl.is_null() {
        return;
    }
    if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
        // SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        let reason = SV_GetStringEdString(sv, "MP_SVGAME", "CANNOT_KICK_HOST");
        SV_SendServerCommand(
            common,
            sv,
            core::ptr::null_mut(),
            &format!("print \"{reason}\""),
        );
        return;
    }

    let reason = SV_GetStringEdString(sv, "MP_SVGAME", "WAS_KICKED");
    SV_DropClient(common, sv, cl, &reason); // "was kicked"
    unsafe {
        (*cl).lastPacketTime = sv.svs.time;
    } // in case there is a funny zombie
}

/// Raven `SV_Serverinfo_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:888-894`
pub fn SV_Serverinfo_f(common: &mut Common) {
    com_printf(common, "Server info settings:\n");
    let info = Cvar_InfoString(common, CVAR_SERVERINFO);
    let info_c = CString::new(info.as_str()).unwrap_or_default();
    Info_Print(common, info_c.as_ptr());
    // NOTE: com_sv_running is threaded through `Common` per the Cvar_Get
    // registration precedent elsewhere in this crate, not `Server`, since
    // this fn takes no `sv` receiver (LAW per resolved signature).
    if common.cvar(common.com_sv_running).integer == 0 {
        com_printf(common, "Server is not running.\n");
    }
}

/// Raven `SV_Systeminfo_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:904-907`
pub fn SV_Systeminfo_f(common: &mut Common) {
    com_printf(common, "System info settings:\n");
    let info = Cvar_InfoString(common, CVAR_SYSTEMINFO);
    let info_c = CString::new(info.as_str()).unwrap_or_default();
    Info_Print(common, info_c.as_ptr());
}

/// Raven `SV_DumpUser_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:917-939`
pub fn SV_DumpUser_f(common: &mut Common, sv: &mut Server) {
    // make sure server is running
    if common.cvar(common.com_sv_running).integer == 0 {
        com_printf(common, "Server is not running.\n");
        return;
    }

    if Cmd_Argc(common) != 2 {
        com_printf(common, "Usage: info <userid>\n");
        return;
    }

    let cl = SV_GetPlayerByName(common, sv);
    if cl.is_null() {
        return;
    }

    com_printf(common, "userinfo\n");
    com_printf(common, "--------\n");
    // `Info_Print` keeps its `*const c_char` seam; bridge the owned userinfo
    // string through a NUL-terminated holder.
    let userinfo_c = CString::new(unsafe { (*cl).userinfo.as_str() }).unwrap_or_default();
    Info_Print(common, userinfo_c.as_ptr());
}

/// Raven `SV_Map_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:138-223`
pub fn SV_Map_f(view: &mut EngineHostView, sv: &mut Server, g2: &mut Ghoul2System) {
    let map = Cmd_Argv(view.common, 1).to_owned();

    // make sure the level exists before trying to change, so that
    // a typo at the server console won't end the game
    if map.contains('\\') {
        com_printf(view.common, "Can't have mapnames with a \\\n");
        return;
    }

    let expanded = format!("maps/{map}.bsp");
    if FS_ReadFile(view, &expanded, core::ptr::null_mut()) == -1 {
        com_printf(view.common, &format!("Can't find map {expanded}\n"));
        return;
    }

    // force latched values to get set
    Cvar_Get(view, "g_gametype", "0", CVAR_SERVERINFO | CVAR_LATCH);

    let mut cmd = Cmd_Argv(view.common, 0).to_owned();
    let (cheat, killBots);
    if Q_stricmpn(&cmd, "sp", 2) == 0 {
        Cvar_SetValue(
            view,
            "g_gametype",
            mp_bg::public::gametype::GT_SINGLE_PLAYER as c_int as f32,
        );
        Cvar_SetValue(view, "g_doWarmup", 0.0);
        // may not set sv_maxclients directly, always set latched
        Cvar_SetLatched(view, "sv_maxclients", "8");
        cmd = cmd[2..].to_string();
        cheat = qfalse;
        killBots = qtrue;
    } else {
        if Q_stricmpn(&cmd, "devmap", 6) == 0 || Q_stricmp(&cmd, "spdevmap") == 0 {
            cheat = qtrue;
            killBots = qtrue;
        } else {
            cheat = qfalse;
            killBots = qfalse;
        }
        // if( sv_gametype->integer == GT_SINGLE_PLAYER ) {
        //     Cvar_SetValue( "g_gametype", GT_FFA );
        // }
    }

    // save the map name here cause on a map restart we reload the jampconfig.cfg
    // and thus nuke the arguments of the map command
    // (Raven's Q_strncpyz into a MAX_QPATH buffer; the owned snapshot keeps
    // the same value — console tokens never reach the 64-byte bound.)
    let mapname = map.clone();

    let mut eForceReload = ForceReload_e::eForceReload_NOTHING;

    // if ( !Q_stricmp( cmd, "devmapbsp") ) {	// not relevant in MP codebase
    //     eForceReload = eForceReload_BSP;
    // }
    // else
    if Q_stricmp(&cmd, "devmapmdl") == 0 {
        eForceReload = ForceReload_e::eForceReload_MODELS;
    } else if Q_stricmp(&cmd, "devmapall") == 0 {
        eForceReload = ForceReload_e::eForceReload_ALL;
    }

    // start up the map
    SV_SpawnServer(view, sv, g2, &mapname, killBots, eForceReload);

    // set the cheat value
    // if the level was started with "map <levelname>", then
    // cheats will not be allowed.  If started with "devmap <levelname>"
    // then cheats will be allowed
    if cheat == qtrue {
        Cvar_Set(view, "sv_cheats", "1");
    } else {
        Cvar_Set(view, "sv_cheats", "0");
    }
}

/// Raven `SV_MapRestart_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:234-343`
pub fn SV_MapRestart_f(view: &mut EngineHostView, sv: &mut Server, g2: &mut Ghoul2System) {
    // make sure we aren't restarting twice in the same frame
    if view.common.com_frameTime == sv.sv.serverId {
        return;
    }

    // make sure server is running
    if view.common.cvar(view.common.com_sv_running).integer == 0 {
        com_printf(view.common, "Server is not running.\n");
        return;
    }

    if sv.sv.restartTime != 0 {
        return;
    }

    let delay = if Cmd_Argc(view.common) > 1 {
        atoi(Cmd_Argv(view.common, 1))
    } else {
        5
    };
    if delay != 0 {
        sv.sv.restartTime = sv.svs.time + delay * 1000;
        SV_SetConfigstring(
            view,
            sv,
            mp_bg::public::configstring::CS_WARMUP,
            format!("{}\0", sv.sv.restartTime).as_ptr() as *const c_char,
        );
        return;
    }

    // check for changes in variables that can't just be restarted
    // check for maxclients change
    if view.common.cvar(view.common.sv_maxclients).modified
        || view.common.cvar(view.common.sv_gametype).modified
    {
        // restart the map the slow way
        // (Raven's Q_strncpyz into a MAX_QPATH buffer becomes the owned
        // snapshot — the value must outlive the jampconfig.cfg reload.)
        let mapname = Cvar_VariableString(view.common, "mapname").to_owned();

        com_printf(view.common, "variable change -- restarting.\n");

        SV_SpawnServer(
            view,
            sv,
            g2,
            &mapname,
            qfalse,
            ForceReload_e::eForceReload_NOTHING,
        );
        return;
    }

    // toggle the server bit so clients can detect that a
    // map_restart has happened
    sv.svs.snapFlagServerBit ^= SNAPFLAG_SERVERCOUNT;

    // generate a new serverid
    sv.sv.restartedServerId = sv.sv.serverId;
    sv.sv.serverId = view.common.com_frameTime;
    Cvar_Set(view, "sv_serverid", &format!("{}", sv.sv.serverId));

    // reset all the vm data in place without changing memory allocation
    // note that we do NOT set sv.state = SS_LOADING, so configstrings that
    // had been changed from their default values will generate broadcast updates
    sv.sv.state = serverState_t::SS_LOADING;
    sv.sv.restarting = qtrue;

    SV_RestartGameProgs(view, sv);

    // run a few frames to allow everything to settle
    for _ in 0..3 {
        VM_Call(
            view.common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_RUN_FRAME as c_int,
            &[sv.svs.time as isize],
        );
        sv.svs.time += 100;
    }

    sv.sv.state = serverState_t::SS_GAME;
    sv.sv.restarting = qfalse;

    // connect and begin all the clients
    for i in 0..view.common.cvar(view.common.sv_maxclients).integer {
        let client = &mut sv.svs.clients[i as usize] as *mut client_t;

        // send the new gamestate to all connected clients
        if unsafe { (*client).state as i32 } < clientState_t::CS_CONNECTED as i32 {
            continue;
        }

        let isBot = if unsafe { (*client).netchan.remoteAddress.r#type } == netadrtype_t::NA_BOT {
            qtrue
        } else {
            qfalse
        };

        // add the map_restart command
        SV_AddServerCommand(view.common, sv, client, "map_restart\n");

        // connect the client again, without the firstTime flag
        let connect_ret = VM_Call(
            view.common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_CLIENT_CONNECT as c_int,
            &[i as isize, qfalse as isize, isBot as isize],
        );
        let denied = VM_ExplicitArgPtr(view.common, sv.gvm, connect_ret) as *mut c_char;
        if !denied.is_null() {
            // this generally shouldn't happen, because the client
            // was connected before the level change
            // (module-memory seam: convert the denial text at the arm)
            let denied = unsafe { core::ffi::CStr::from_ptr(denied) }.to_string_lossy();
            SV_DropClient(view.common, sv, client, &denied);
            com_printf(
                view.common,
                &format!(
                    "SV_MapRestart_f({}): dropped client {} - denied!\n",
                    delay, i
                ),
            ); // bk010125
            continue;
        }

        unsafe {
            (*client).state = clientState_t::CS_ACTIVE;
        }

        SV_ClientEnterWorld(view.common, sv, client, unsafe {
            &mut (*client).lastUsercmd
        });
    }

    // run another frame to allow things to look at all the players
    VM_Call(
        view.common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_RUN_FRAME as c_int,
        &[sv.svs.time as isize],
    );
    sv.svs.time += 100;
}

/// Cast the view's type-erased `g2` slot back to the live `Ghoul2System`,
/// inside a registered-command handler body.
///
/// SAFETY: view-constructor slot, single-threaded, no other live cast of this
/// slot for the borrow's duration.
unsafe fn g2_from_view<'a>(view: &mut EngineHostView) -> &'a mut Ghoul2System {
    &mut *(view.g2.as_raw() as *mut Ghoul2System)
}

/// Register one console command whose handler is a view-threaded forwarding
/// closure (host-seam restructure, user 2026-07-11); thin wrapper over
/// `Cmd_AddCommand` that supplies `Some(function)`.
fn add(view: &mut EngineHostView, name: &str, function: CmdFunction) {
    Cmd_AddCommand(view, name, Some(function));
}

/// Raven `SV_AddOperatorCommands`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:958-996`
pub fn SV_AddOperatorCommands(view: &mut EngineHostView, _sv: &mut Server) {
    // Function-scope static `initialized` (fork-3 kind 3: genuine cross-frame
    // state) belongs on the owning host struct; `Common` is the only threaded
    // receiver here, so it homes there pending the campaign's receiver-cleanup
    // pass (STATE-D4-style deferral, matches this crate's existing precedent).
    if view.common.sv_ccmds_operator_commands_initialized == qtrue {
        return;
    }
    view.common.sv_ccmds_operator_commands_initialized = qtrue;

    // Each registered handler is a non-capturing forwarding closure matching
    // `CmdFunction`'s `fn(&mut EngineHostView)` shape; where it needs the real
    // `Server`/`Ghoul2System` it casts the view's type-erased slot back
    // (`sv_from_view`/`g2_from_view`, the single documented unsafe pair) and
    // calls the real receiver-threaded command body.
    add(view, "heartbeat", |view| {
        let sv = unsafe { sv_from_view(view) };
        SV_Heartbeat_f(sv)
    });
    add(view, "kick", |view| {
        let sv = unsafe { sv_from_view(view) };
        SV_Kick_f(view.common, sv)
    });
    // #ifdef USE_CD_KEY
    //     Cmd_AddCommand ("banUser", SV_Ban_f);
    //     Cmd_AddCommand ("banClient", SV_BanNum_f);
    // #endif	// USE_CD_KEY

    add(view, "clientkick", |view| {
        let sv = unsafe { sv_from_view(view) };
        SV_KickNum_f(view.common, sv)
    });
    add(view, "status", |view| {
        let sv = unsafe { sv_from_view(view) };
        SV_Status_f(view, sv)
    });
    add(view, "serverinfo", |view| SV_Serverinfo_f(view.common));
    add(view, "systeminfo", |view| SV_Systeminfo_f(view.common));
    add(view, "dumpuser", |view| {
        let sv = unsafe { sv_from_view(view) };
        SV_DumpUser_f(view.common, sv)
    });
    // Engine referee divergence UX (lockstep plan G5; see sv_referee.rs).
    add(view, "ref_step", |view| {
        let sv = unsafe { sv_from_view(view) };
        crate::sv_referee::ref_step_cmd(view, sv)
    });
    add(view, "ref_resume", |view| {
        let sv = unsafe { sv_from_view(view) };
        crate::sv_referee::ref_resume_cmd(view, sv)
    });
    add(view, "ref_diff", |view| {
        let sv = unsafe { sv_from_view(view) };
        crate::sv_referee::ref_diff_cmd(view, sv)
    });
    add(view, "map_restart", SV_MapRestart_f_cmd);
    add(view, "sectorlist", |view| {
        let sv = unsafe { sv_from_view(view) };
        SV_SectorList_f(view.common, sv)
    });
    add(view, "map", SV_Map_f_cmd);
    // #ifndef PRE_RELEASE_DEMO
    add(view, "devmap", SV_Map_f_cmd);
    add(view, "spmap", SV_Map_f_cmd);
    add(view, "spdevmap", SV_Map_f_cmd);
    // Cmd_AddCommand ("devmapbsp", SV_Map_f);	// not used in MP codebase, no server BSP_cacheing
    add(view, "devmapmdl", SV_Map_f_cmd);
    add(view, "devmapall", SV_Map_f_cmd);
    // #endif
    add(view, "killserver", |view| SV_KillServer_f(view));
    // if( com_dedicated->integer )
    {
        add(view, "svsay", |view| {
            let sv = unsafe { sv_from_view(view) };
            SV_ConSay_f(view.common, sv)
        });
    }

    add(view, "forcetoggle", |view| {
        let sv = unsafe { sv_from_view(view) };
        SV_ForceToggle_f(view, sv)
    });
}

// `SV_Map_f`/`SV_MapRestart_f` need the real `Server` + `Ghoul2System`; these
// forwarders match `CmdFunction`'s `fn(&mut EngineHostView)` shape and cast the
// view's type-erased `sv`/`g2` slots back to their real receivers before
// reaching the full command bodies (host-seam restructure, user 2026-07-11).
fn SV_Map_f_cmd(view: &mut EngineHostView) {
    let sv = unsafe { sv_from_view(view) };
    let g2 = unsafe { g2_from_view(view) };
    SV_Map_f(view, sv, g2)
}

fn SV_MapRestart_f_cmd(view: &mut EngineHostView) {
    let sv = unsafe { sv_from_view(view) };
    let g2 = unsafe { g2_from_view(view) };
    SV_MapRestart_f(view, sv, g2)
}
