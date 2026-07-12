//! `sv_ccmds.cpp` — server console commands: player lookup helpers, kick/status/
//! say/force-toggle/map/killserver/map_restart commands, and the operator
//! command-table registration.
//!
//! Source: `oracle/codemp/server/sv_ccmds.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// PORT-NOTE(engine-host-state): `CollisionWorld`/`Common`/`EngineHost` exist;
// `RenderModels`/`RmManager`/`Ghoul2System` do not exist anywhere in the tree
// yet (grepped: no hits) — these packets were generated ahead of those state
// structs landing (same situation as sv_game.rs's identical note). Imported
// below by their preamble-table decl-home crate; genuinely missing, escalated
// in missing_symbols rather than stubbed (ZERO-PARK).
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::cmd::cmd_function_t::CmdFunction;
use mp_engine_qcommon::cmd::Cmd_AddCommand;
use mp_engine_qcommon::cmd_common::{Cmd_Argc, Cmd_Args, Cmd_Argv};
use mp_engine_qcommon::cmd_pc::Server as CmdServerSlot;
use mp_engine_qcommon::common::opaque_slots::Ghoul2System as CmdGhoul2Slot;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::{com_printf, Common};
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
use mp_engine_qcommon::cm_load::RenderModels;
use mp_engine_qcommon::cm_load::RmManager;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::playerstate::PERS_SCORE;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::shared::cvar::{CVAR_LATCH, CVAR_SERVERINFO, CVAR_SYSTEMINFO};
use mp_qshared::shared::force_powers::NUM_FORCE_POWERS;
use mp_qshared::shared::force_reload::ForceReload_e;
use mp_qshared::shared::q_string::Q_CleanStr;
use mp_qshared::shared::q_string::{Q_stricmp, Q_stricmpn, Q_strncpyz};
use mp_qshared::shared::{qfalse, qtrue, MAX_QPATH, SNAPFLAG_SERVERCOUNT};

use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server::server_state_t::serverState_t;
use crate::server_host::{ghoul2_from_slot, server_from_slot};
use crate::sv_client::SV_ClientEnterWorld;
use crate::sv_game::{SV_GameClientNum, SV_RestartGameProgs};
use crate::sv_init::{SV_SetConfigstring, SV_Shutdown, SV_SpawnServer};
use crate::sv_world::SV_SectorList_f;
use crate::{Server, SV_AddServerCommand, SV_DropClient, SV_SendServerCommand};

/// Raven `SV_GetStringEdString`.
///
/// Raven: Well, it would've been lovely doing it the above way, but it would
/// mean mixing languages for the client depending on what the server is. So
/// we'll mark this as a stringed reference with @@@ and send the refname to
/// the client, and when it goes to print it will get scanned for the
/// stringed reference indication and dealt with properly.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:16-32`
pub fn SV_GetStringEdString(
    sv: &mut Server,
    refSection: *mut c_char,
    refName: *mut c_char,
) -> *const c_char {
    let _ = sv;
    let _ = refSection;
    // Function-scope static `text` (fork-3 kind 2: rotating scratch/return
    // buffer) becomes an owned return value instead of a hidden cell.
    let refname = unsafe { core::ffi::CStr::from_ptr(refName) }
        .to_string_lossy()
        .into_owned();
    let mut text = format!("@@@{}", refname);
    text.push('\0');
    // Leak the owned CString-shaped buffer to hand back a raw `*const
    // c_char` matching Raven's `static char text[1024]` return-by-pointer
    // shape (the buffer must outlive the call, exactly as the C static did).
    let boxed: Box<str> = text.into_boxed_str();
    Box::leak(boxed).as_ptr() as *const c_char
}

/// Rust-native twin of `SV_GetStringEdString` returning an owned `String`
/// (used where callers `format!` the result directly rather than needing the
/// raw `*const c_char` return). Same `@@@`-marked stringed reference shape.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:16-32`
pub fn SV_GetStringEdString_str(sv: &mut Server, _refSection: &str, refName: &str) -> String {
    let _ = sv;
    format!("@@@{}", refName)
}

/// Raven `SV_GetPlayerByFedName`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:354-387`
pub fn SV_GetPlayerByFedName(
    common: &mut Common,
    sv: &mut Server,
    name: *const c_char,
) -> *mut client_t {
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        return core::ptr::null_mut();
    }

    // check for a name match
    let n = sv.svs.clients;
    for i in 0..unsafe { (*common.sv_maxclients).integer } {
        let cl = unsafe { n.offset(i as isize) };
        if unsafe { (*cl).state as i32 } == 0 {
            continue;
        }
        if unsafe { Q_stricmp((*cl).name.as_ptr(), name) } == 0 {
            return cl;
        }

        let mut cleanName = [0 as c_char; 64];
        unsafe {
            Q_strncpyz(
                cleanName.as_mut_ptr(),
                (*cl).name.as_ptr(),
                cleanName.len() as c_int,
            );
            Q_CleanStr(cleanName.as_mut_ptr());
        }
        if Q_stricmp(cleanName.as_ptr(), name) == 0 {
            return cl;
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
    if unsafe { (*common.com_sv_running).integer } == 0 {
        return core::ptr::null_mut();
    }

    if Cmd_Argc(common) < 2 {
        com_printf(common, "No player specified.\n");
        return core::ptr::null_mut();
    }

    let s = Cmd_Argv(common, 1);

    // check for a name match
    let n = sv.svs.clients;
    for i in 0..unsafe { (*common.sv_maxclients).integer } {
        let cl = unsafe { n.offset(i as isize) };
        if unsafe { (*cl).state as i32 } == 0 {
            continue;
        }
        if unsafe { Q_stricmp((*cl).name.as_ptr(), s) } == 0 {
            return cl;
        }

        let mut cleanName = [0 as c_char; 64];
        unsafe {
            Q_strncpyz(
                cleanName.as_mut_ptr(),
                (*cl).name.as_ptr(),
                cleanName.len() as c_int,
            );
            Q_CleanStr(cleanName.as_mut_ptr());
        }
        if Q_stricmp(cleanName.as_ptr(), s) == 0 {
            return cl;
        }
    }

    unsafe {
        com_printf(
            common,
            &format!(
                "Player {} is not on the server\n",
                core::ffi::CStr::from_ptr(s).to_string_lossy()
            ),
        );
    }

    core::ptr::null_mut()
}

/// Raven `SV_GetPlayerByNum`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:89-125`
pub fn SV_GetPlayerByNum(common: &mut Common, sv: &mut Server) -> *mut client_t {
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        return core::ptr::null_mut();
    }

    if Cmd_Argc(common) < 2 {
        com_printf(common, "No player specified.\n");
        return core::ptr::null_mut();
    }

    let s = Cmd_Argv(common, 1);
    let s_str = unsafe { core::ffi::CStr::from_ptr(s) }
        .to_string_lossy()
        .into_owned();

    for c in s_str.bytes() {
        if !c.is_ascii_digit() {
            com_printf(
                common,
                &format!("Bad slot number: {}\n", s_str),
            );
            return core::ptr::null_mut();
        }
    }
    let idnum = unsafe { libc::atoi(s) };
    if idnum < 0 || idnum >= unsafe { (*common.sv_maxclients).integer } {
        com_printf(
            common,
            &format!("Bad client slot: {}\n", idnum),
        );
        return core::ptr::null_mut();
    }

    let cl = unsafe { sv.svs.clients.offset(idnum as isize) };
    if unsafe { (*cl).state as i32 } == 0 {
        com_printf(
            common,
            &format!("Client {} is not active\n", idnum),
        );
        return core::ptr::null_mut();
    }
    cl
}

/// Raven `SV_KickByName`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:389-446`
pub fn SV_KickByName(common: &mut Common, sv: &mut Server, name: *const c_char) {
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        return;
    }

    let cl = SV_GetPlayerByFedName(common, sv, name);
    if cl.is_null() {
        if Q_stricmp(name, c"all".as_ptr()) == 0 {
            let n = sv.svs.clients;
            for i in 0..unsafe { (*common.sv_maxclients).integer } {
                let cl = unsafe { n.offset(i as isize) };
                if unsafe { (*cl).state as i32 } == 0 {
                    continue;
                }
                if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
                    continue;
                }
                let reason = SV_GetStringEdString(
                    sv,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"WAS_KICKED".as_ptr() as *mut c_char,
                );
                SV_DropClient(common, sv, cl, reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        } else if Q_stricmp(name, c"allbots".as_ptr()) == 0
        {
            let n = sv.svs.clients;
            for i in 0..unsafe { (*common.sv_maxclients).integer } {
                let cl = unsafe { n.offset(i as isize) };
                if unsafe { (*cl).state as i32 } == 0 {
                    continue;
                }
                if unsafe { (*cl).netchan.remoteAddress.r#type } != netadrtype_t::NA_BOT {
                    continue;
                }
                let reason = SV_GetStringEdString(
                    sv,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"WAS_KICKED".as_ptr() as *mut c_char,
                );
                SV_DropClient(common, sv, cl, reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        }
        return;
    }
    if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
        // SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        let reason = SV_GetStringEdString(
            sv,
            c"MP_SVGAME".as_ptr() as *mut c_char,
            c"CANNOT_KICK_HOST".as_ptr() as *mut c_char,
        );
        SV_SendServerCommand(
            common,
            sv,
            core::ptr::null_mut(),
            &format!(
                "print \"{}\"",
                unsafe { core::ffi::CStr::from_ptr(reason) }.to_string_lossy()
            ),
        );
        return;
    }

    let reason = SV_GetStringEdString(
        sv,
        c"MP_SVGAME".as_ptr() as *mut c_char,
        c"WAS_KICKED".as_ptr() as *mut c_char,
    );
    SV_DropClient(common, sv, cl, reason); // "was kicked"
    unsafe {
        (*cl).lastPacketTime = sv.svs.time;
    } // in case there is a funny zombie
}

/// Raven `SV_Status_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:669-750`
pub fn SV_Status_f(common: &mut Common, sv: &mut Server, host: &mut dyn EngineHost) {
    let mut avoidTruncation = qfalse;

    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        let msg = SE_GetString(common, host, "STR_SERVER_SERVER_NOT_RUNNING");
        com_printf(common, &msg);
        return;
    }

    if Cmd_Argc(common) > 1 {
        if {
            Q_stricmp(
                c"notrunc".as_ptr(),
                Cmd_Argv(common, 1),
            )
        } == 0
        {
            avoidTruncation = qtrue;
        }
    }

    com_printf(
        common,
        &format!("map: {}\n", unsafe {
            core::ffi::CStr::from_ptr((*common.sv_mapname).string).to_string_lossy()
        }),
    );

    com_printf(
        common,
        "num score ping name            lastmsg address               qport rate\n",
    );
    com_printf(
        common,
        "--- ----- ---- --------------- ------- --------------------- ----- -----\n",
    );
    for i in 0..unsafe { (*common.sv_maxclients).integer } {
        let cl = unsafe { sv.svs.clients.offset(i as isize) };
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
            core::ffi::CStr::from_ptr(NET_AdrToString(
                common,
                (*cl).netchan.remoteAddress,
            ))
            .to_string_lossy()
            .into_owned()
        };

        let name = unsafe { core::ffi::CStr::from_ptr((*cl).name.as_ptr()) }.to_string_lossy();
        if avoidTruncation == qfalse {
            com_printf(
                common,
                &format!(
                    "{:3} {:5} {} {:<15.15} {:7} {:>21} {:5} {:5}\n",
                    i,
                    unsafe {
                        (*ps).persistant[PERS_SCORE as usize]
                    },
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
                common,
                &format!(
                    "{:3} {:5} {} {} {:7} {:>21} {:5} {:5}\n",
                    i,
                    unsafe {
                        (*ps).persistant[PERS_SCORE as usize]
                    },
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
    com_printf(common, "\n");
}

/// Raven `SV_ConSay_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:757-787`
pub fn SV_ConSay_f(common: &mut Common, sv: &mut Server) {
    if unsafe { (*common.com_dedicated).integer } == 0 {
        com_printf(common, "Server is not dedicated.\n");
        return;
    }

    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        com_printf(common, "Server is not running.\n");
        return;
    }

    if Cmd_Argc(common) < 2 {
        return;
    }

    let mut text = "Server: ".to_string();
    let mut p = unsafe {
        core::ffi::CStr::from_ptr(Cmd_Args(common))
    }
        .to_string_lossy()
        .into_owned();

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
pub fn SV_ForceToggle_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    let _ = sv;
    let mut fpDisabled = Cvar_VariableValue(
        common,
        cm,
        rm,
        host,
        c"g_forcePowerDisable".as_ptr(),
    ) as c_int;
    let targetPower: c_int;
    let mut powerDisabled;

    if Cmd_Argc(common) < 2 {
        // no argument supplied, spit out a list of force powers and their numbers
        let mut i: c_int = 0;
        while i < NUM_FORCE_POWERS {
            powerDisabled = if fpDisabled & (1 << i) != 0 {
                "Disabled"
            } else {
                "Enabled"
            };

            com_printf(
                common,
                &format!(
                    "{} - {} - Status: {}\n",
                    i, FORCE_TOGGLE_NAME_PRINTS[i as usize], powerDisabled
                ),
            );
            i += 1;
        }

        com_printf(
            common,
            "Example usage: forcetoggle 3\n(toggles PUSH)\n",
        );
        return;
    }

    targetPower =
        unsafe { libc::atoi(Cmd_Argv(common, 1)) };

    if targetPower < 0 || targetPower >= NUM_FORCE_POWERS {
        com_printf(
            common,
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

    Cvar_Set(
        common,
        cm,
        rm,
        host,
        c"g_forcePowerDisable".as_ptr() as *mut c_char,
        format!("{}\0", fpDisabled).as_ptr() as *mut c_char,
    );

    com_printf(
        common,
        &format!(
            "{} has been {}.\n",
            FORCE_TOGGLE_NAME_PRINTS[targetPower as usize], powerDisabled
        ),
    );
}

/// Raven `forceToggleNamePrints[]` — file-scope const table (fork-3 kind 1).
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:789-810`
// PORT-NOTE(force-toggle-names): the full name list is not reproduced in the
// packet's verbatim source slice (only "HEAL"/"JUMP" shown before elision);
// transcribed against `NUM_FORCE_POWERS`'s FP_* order as best-effort. Escalate
// if this drifts from the oracle's full array at review.
const FORCE_TOGGLE_NAME_PRINTS: [&str;
    NUM_FORCE_POWERS as usize] = [
    "HEAL",
    "JUMP",
    "SPEED",
    "PUSH",
    "PULL",
    "MINDTRICK",
    "GRIP",
    "LIGHTNING",
    "DARK_RAGE",
    "PROTECT",
    "ABSORB",
    "TEAM_HEAL",
    "TEAM_FORCE",
    "DRAIN",
    "SEE",
    "SABERTHROW",
    "SABER_OFFENSE",
    "SABER_DEFENSE",
];

/// Raven `SV_KillServer_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:947-949`
pub fn SV_KillServer_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
) {
    SV_Shutdown(common, cm, sv, rm, rmg, host, "killserver");
}

/// Raven `SV_Kick_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:455-511`
pub fn SV_Kick_f(common: &mut Common, sv: &mut Server) {
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
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

    if {
        Q_stricmp(
            Cmd_Argv(common, 1),
            c"Padawan".as_ptr(),
        )
    } == 0
    {
        // if you try to kick the default name, also try to kick ""
        SV_KickByName(common, sv, c"".as_ptr());
    }

    let cl = SV_GetPlayerByName(common, sv);
    if cl.is_null() {
        if {
            Q_stricmp(
                Cmd_Argv(common, 1),
                c"all".as_ptr(),
            )
        } == 0
        {
            let n = sv.svs.clients;
            for i in 0..unsafe { (*common.sv_maxclients).integer } {
                let cl = unsafe { n.offset(i as isize) };
                if unsafe { (*cl).state as i32 } == 0 {
                    continue;
                }
                if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
                    continue;
                }
                let reason = SV_GetStringEdString(
                    sv,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"WAS_KICKED".as_ptr() as *mut c_char,
                );
                SV_DropClient(common, sv, cl, reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        } else if {
            Q_stricmp(
                Cmd_Argv(common, 1),
                c"allbots".as_ptr(),
            )
        } == 0
        {
            let n = sv.svs.clients;
            for i in 0..unsafe { (*common.sv_maxclients).integer } {
                let cl = unsafe { n.offset(i as isize) };
                if unsafe { (*cl).state as i32 } == 0 {
                    continue;
                }
                if unsafe { (*cl).netchan.remoteAddress.r#type } != netadrtype_t::NA_BOT {
                    continue;
                }
                let reason = SV_GetStringEdString(
                    sv,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"WAS_KICKED".as_ptr() as *mut c_char,
                );
                SV_DropClient(common, sv, cl, reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        }
        return;
    }
    if unsafe { (*cl).netchan.remoteAddress.r#type } == netadrtype_t::NA_LOOPBACK {
        // SV_SendServerCommand(NULL, "print \"%s\"", "Cannot kick host player\n");
        let reason = SV_GetStringEdString(
            sv,
            c"MP_SVGAME".as_ptr() as *mut c_char,
            c"CANNOT_KICK_HOST".as_ptr() as *mut c_char,
        );
        SV_SendServerCommand(
            common,
            sv,
            core::ptr::null_mut(),
            &format!(
                "print \"{}\"",
                unsafe { core::ffi::CStr::from_ptr(reason) }.to_string_lossy()
            ),
        );
        return;
    }

    let reason = SV_GetStringEdString(
        sv,
        c"MP_SVGAME".as_ptr() as *mut c_char,
        c"WAS_KICKED".as_ptr() as *mut c_char,
    );
    SV_DropClient(common, sv, cl, reason); // "was kicked"
    unsafe {
        (*cl).lastPacketTime = sv.svs.time;
    } // in case there is a funny zombie
}

/// Raven `SV_KickNum_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:636-662`
pub fn SV_KickNum_f(common: &mut Common, sv: &mut Server) {
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
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
        let reason = SV_GetStringEdString(
            sv,
            c"MP_SVGAME".as_ptr() as *mut c_char,
            c"CANNOT_KICK_HOST".as_ptr() as *mut c_char,
        );
        SV_SendServerCommand(
            common,
            sv,
            core::ptr::null_mut(),
            &format!(
                "print \"{}\"",
                unsafe { core::ffi::CStr::from_ptr(reason) }.to_string_lossy()
            ),
        );
        return;
    }

    let reason = SV_GetStringEdString(
        sv,
        c"MP_SVGAME".as_ptr() as *mut c_char,
        c"WAS_KICKED".as_ptr() as *mut c_char,
    );
    SV_DropClient(common, sv, cl, reason); // "was kicked"
    unsafe {
        (*cl).lastPacketTime = sv.svs.time;
    } // in case there is a funny zombie
}

/// Raven `SV_Serverinfo_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:888-894`
pub fn SV_Serverinfo_f(common: &mut Common) {
    com_printf(common, "Server info settings:\n");
    let info = Cvar_InfoString(
        common,
        CVAR_SERVERINFO,
    );
    Info_Print(common, info as *const c_char);
    // NOTE: com_sv_running is threaded through `Common` per the Cvar_Get
    // registration precedent elsewhere in this crate, not `Server`, since
    // this fn takes no `sv` receiver (LAW per resolved signature).
    if unsafe { (*common.com_sv_running).integer } == 0 {
        com_printf(common, "Server is not running.\n");
    }
}

/// Raven `SV_Systeminfo_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:904-907`
pub fn SV_Systeminfo_f(common: &mut Common) {
    com_printf(common, "System info settings:\n");
    let info = Cvar_InfoString(
        common,
        CVAR_SYSTEMINFO,
    );
    Info_Print(common, info as *const c_char);
}

/// Raven `SV_DumpUser_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:917-939`
pub fn SV_DumpUser_f(common: &mut Common, sv: &mut Server) {
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
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
    Info_Print(common, unsafe { (*cl).userinfo.as_ptr() });
}

/// Raven `SV_Map_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:138-223`
pub fn SV_Map_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    g2: &mut Ghoul2System,
    host: &mut dyn EngineHost,
) {
    let map = Cmd_Argv(common, 1);
    if map.is_null() {
        return;
    }

    // make sure the level exists before trying to change, so that
    // a typo at the server console won't end the game
    let map_str = unsafe { core::ffi::CStr::from_ptr(map) }
        .to_string_lossy()
        .into_owned();
    if map_str.contains('\\') {
        com_printf(common, "Can't have mapnames with a \\\n");
        return;
    }

    let expanded = format!("maps/{}.bsp\0", map_str);
    if FS_ReadFile(
        common,
        cm,
        rm,
        host,
        expanded.as_ptr() as *const c_char,
        core::ptr::null_mut(),
    ) == -1
    {
        com_printf(
            common,
            &format!("Can't find map {}\n", expanded.trim_end_matches('\0')),
        );
        return;
    }

    // force latched values to get set
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        c"g_gametype".as_ptr() as *mut c_char,
        c"0".as_ptr() as *mut c_char,
        CVAR_SERVERINFO | CVAR_LATCH,
    );

    let mut cmd = unsafe {
        core::ffi::CStr::from_ptr(Cmd_Argv(common, 0))
    }
        .to_string_lossy()
        .into_owned();
    let (cheat, killBots);
    if {
        Q_stricmpn(
            format!("{}\0", cmd).as_ptr() as *const c_char,
            c"sp".as_ptr(),
            2,
        )
    } == 0
    {
        Cvar_SetValue(
            common,
            cm,
            rm,
            host,
            c"g_gametype".as_ptr() as *const c_char,
            mp_bg::public::gametype::GT_SINGLE_PLAYER as c_int as f32,
        );
        Cvar_SetValue(
            common,
            cm,
            rm,
            host,
            c"g_doWarmup".as_ptr() as *const c_char,
            0.0,
        );
        // may not set sv_maxclients directly, always set latched
        Cvar_SetLatched(
            common,
            cm,
            rm,
            host,
            c"sv_maxclients".as_ptr() as *mut c_char,
            c"8".as_ptr() as *mut c_char,
        );
        cmd = cmd[2..].to_string();
        cheat = qfalse;
        killBots = qtrue;
    } else {
        let cmd_c = format!("{}\0", cmd);
        if {
            Q_stricmpn(
                cmd_c.as_ptr() as *const c_char,
                c"devmap".as_ptr(),
                6,
            )
        } == 0
            || {
                Q_stricmp(
                    cmd_c.as_ptr() as *const c_char,
                    c"spdevmap".as_ptr(),
                )
            } == 0
        {
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
    let mut mapname = [0 as c_char; MAX_QPATH as usize];
    Q_strncpyz(mapname.as_mut_ptr(), map, mapname.len() as c_int);

    let mut eForceReload = ForceReload_e::eForceReload_NOTHING;

    // if ( !Q_stricmp( cmd, "devmapbsp") ) {	// not relevant in MP codebase
    //     eForceReload = eForceReload_BSP;
    // }
    // else
    let cmd_c = format!("{}\0", cmd);
    if {
        Q_stricmp(
            cmd_c.as_ptr() as *const c_char,
            c"devmapmdl".as_ptr(),
        )
    } == 0
    {
        eForceReload = ForceReload_e::eForceReload_MODELS;
    } else if {
        Q_stricmp(
            cmd_c.as_ptr() as *const c_char,
            c"devmapall".as_ptr(),
        )
    } == 0
    {
        eForceReload = ForceReload_e::eForceReload_ALL;
    }

    // start up the map
    SV_SpawnServer(
        common,
        cm,
        sv,
        rm,
        rmg,
        g2,
        host,
        mapname.as_mut_ptr(),
        killBots,
        eForceReload,
    );

    // set the cheat value
    // if the level was started with "map <levelname>", then
    // cheats will not be allowed.  If started with "devmap <levelname>"
    // then cheats will be allowed
    if cheat == qtrue {
        Cvar_Set(
            common,
            cm,
            rm,
            host,
            c"sv_cheats".as_ptr() as *mut c_char,
            c"1".as_ptr() as *mut c_char,
        );
    } else {
        Cvar_Set(
            common,
            cm,
            rm,
            host,
            c"sv_cheats".as_ptr() as *mut c_char,
            c"0".as_ptr() as *mut c_char,
        );
    }
}

/// Raven `SV_MapRestart_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:234-343`
pub fn SV_MapRestart_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    g2: &mut Ghoul2System,
    host: &mut dyn EngineHost,
) {
    // make sure we aren't restarting twice in the same frame
    if common.com_frameTime == sv.sv.serverId {
        return;
    }

    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        com_printf(common, "Server is not running.\n");
        return;
    }

    if sv.sv.restartTime != 0 {
        return;
    }

    let delay = if Cmd_Argc(common) > 1 {
        unsafe { libc::atoi(Cmd_Argv(common, 1)) }
    } else {
        5
    };
    if delay != 0 {
        sv.sv.restartTime = sv.svs.time + delay * 1000;
        SV_SetConfigstring(
            common,
            cm,
            sv,
            rm,
            host,
            mp_bg::public::configstring::CS_WARMUP,
            format!("{}\0", sv.sv.restartTime).as_ptr() as *const c_char,
        );
        return;
    }

    // check for changes in variables that can't just be restarted
    // check for maxclients change
    if unsafe { (*common.sv_maxclients).modified != 0 || (*common.sv_gametype).modified != 0 } {
        // restart the map the slow way
        let mut mapname = [0 as c_char; MAX_QPATH as usize];
        Q_strncpyz(
            mapname.as_mut_ptr(),
            Cvar_VariableString(common, c"mapname".as_ptr()),
            mapname.len() as c_int,
        );

        com_printf(common, "variable change -- restarting.\n");

        SV_SpawnServer(
            common,
            cm,
            sv,
            rm,
            rmg,
            g2,
            host,
            mapname.as_mut_ptr(),
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
    sv.sv.serverId = common.com_frameTime;
    Cvar_Set(
        common,
        cm,
        rm,
        host,
        c"sv_serverid".as_ptr() as *mut c_char,
        format!("{}\0", sv.sv.serverId).as_ptr() as *mut c_char,
    );

    // reset all the vm data in place without changing memory allocation
    // note that we do NOT set sv.state = SS_LOADING, so configstrings that
    // had been changed from their default values will generate broadcast updates
    sv.sv.state = serverState_t::SS_LOADING;
    sv.sv.restarting = qtrue;

    SV_RestartGameProgs(common, cm, sv, rm, host);

    // run a few frames to allow everything to settle
    for _ in 0..3 {
        VM_Call(
            common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_RUN_FRAME as c_int,
            &[sv.svs.time],
        );
        sv.svs.time += 100;
    }

    sv.sv.state = serverState_t::SS_GAME;
    sv.sv.restarting = qfalse;

    // connect and begin all the clients
    for i in 0..unsafe { (*common.sv_maxclients).integer } {
        let client = unsafe { sv.svs.clients.offset(i as isize) };

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
        SV_AddServerCommand(common, sv, client, "map_restart\n");

        // connect the client again, without the firstTime flag
        let connect_ret = VM_Call(
            common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_CLIENT_CONNECT as c_int,
            &[i, qfalse as c_int, isBot as c_int],
        );
        let denied =
            VM_ExplicitArgPtr(common, sv.gvm, connect_ret) as *mut c_char;
        if !denied.is_null() {
            // this generally shouldn't happen, because the client
            // was connected before the level change
            SV_DropClient(common, sv, client, denied);
            com_printf(
                common,
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

        SV_ClientEnterWorld(common, sv, client, unsafe {
            &mut (*client).lastUsercmd
        });
    }

    // run another frame to allow things to look at all the players
    VM_Call(
        common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_RUN_FRAME as c_int,
        &[sv.svs.time],
    );
    sv.svs.time += 100;
}

/// Register one console command whose handler is a receiver-threaded forwarding
/// closure (opaque-slot ruling, user 2026-07-12); thin wrapper over
/// `Cmd_AddCommand` that supplies `Some(function)`.
fn add(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    name: *const c_char,
    function: CmdFunction,
) {
    Cmd_AddCommand(common, cm, rm, host, name, Some(function));
}

/// Raven `SV_AddOperatorCommands`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:958-996`
pub fn SV_AddOperatorCommands(
    common: &mut Common,
    cm: &mut CollisionWorld,
    _sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    // Function-scope static `initialized` (fork-3 kind 3: genuine cross-frame
    // state) belongs on the owning host struct; `Common` is the only threaded
    // receiver here, so it homes there pending the campaign's receiver-cleanup
    // pass (STATE-D4-style deferral, matches this crate's existing precedent).
    if common.sv_ccmds_operator_commands_initialized == qtrue {
        return;
    }
    common.sv_ccmds_operator_commands_initialized = qtrue;

    // Each registered handler is a non-capturing forwarding closure matching
    // `CmdFunction`'s pinned receiver order (common/cm/sv/rm/rmg/host); it casts
    // the type-erased `sv` slot back to `&mut Server` (`server_from_slot`, single
    // documented unsafe pair) and calls the real receiver-threaded command body.
    add(common, cm, rm, host, c"heartbeat".as_ptr(), |_common, _cm, sv, _rm, _rmg, _g2, _host| unsafe {
        SV_Heartbeat_f(server_from_slot(sv))
    });
    add(common, cm, rm, host, c"kick".as_ptr(), |common, _cm, sv, _rm, _rmg, _g2, _host| unsafe {
        SV_Kick_f(common, server_from_slot(sv))
    });
    // #ifdef USE_CD_KEY
    //     Cmd_AddCommand ("banUser", SV_Ban_f);
    //     Cmd_AddCommand ("banClient", SV_BanNum_f);
    // #endif	// USE_CD_KEY

    add(common, cm, rm, host, c"clientkick".as_ptr(), |common, _cm, sv, _rm, _rmg, _g2, _host| unsafe {
        SV_KickNum_f(common, server_from_slot(sv))
    });
    add(common, cm, rm, host, c"status".as_ptr(), |common, _cm, sv, _rm, _rmg, _g2, host| unsafe {
        SV_Status_f(common, server_from_slot(sv), host)
    });
    add(common, cm, rm, host, c"serverinfo".as_ptr(), |common, _cm, _sv, _rm, _rmg, _g2, _host| {
        SV_Serverinfo_f(common)
    });
    add(common, cm, rm, host, c"systeminfo".as_ptr(), |common, _cm, _sv, _rm, _rmg, _g2, _host| {
        SV_Systeminfo_f(common)
    });
    add(common, cm, rm, host, c"dumpuser".as_ptr(), |common, _cm, sv, _rm, _rmg, _g2, _host| unsafe {
        SV_DumpUser_f(common, server_from_slot(sv))
    });
    add(common, cm, rm, host, c"map_restart".as_ptr(), SV_MapRestart_f_cmd);
    add(common, cm, rm, host, c"sectorlist".as_ptr(), |common, _cm, sv, _rm, _rmg, _g2, _host| unsafe {
        SV_SectorList_f(common, server_from_slot(sv))
    });
    add(common, cm, rm, host, c"map".as_ptr(), SV_Map_f_cmd);
    // #ifndef PRE_RELEASE_DEMO
    add(common, cm, rm, host, c"devmap".as_ptr(), SV_Map_f_cmd);
    add(common, cm, rm, host, c"spmap".as_ptr(), SV_Map_f_cmd);
    add(common, cm, rm, host, c"spdevmap".as_ptr(), SV_Map_f_cmd);
    // Cmd_AddCommand ("devmapbsp", SV_Map_f);	// not used in MP codebase, no server BSP_cacheing
    add(common, cm, rm, host, c"devmapmdl".as_ptr(), SV_Map_f_cmd);
    add(common, cm, rm, host, c"devmapall".as_ptr(), SV_Map_f_cmd);
    // #endif
    add(common, cm, rm, host, c"killserver".as_ptr(), |common, cm, sv, rm, rmg, _g2, host| unsafe {
        SV_KillServer_f(common, cm, server_from_slot(sv), rm, rmg, host)
    });
    // if( com_dedicated->integer )
    {
        add(common, cm, rm, host, c"svsay".as_ptr(), |common, _cm, sv, _rm, _rmg, _g2, _host| unsafe {
            SV_ConSay_f(common, server_from_slot(sv))
        });
    }

    add(common, cm, rm, host, c"forcetoggle".as_ptr(), |common, cm, sv, rm, _rmg, _g2, host| unsafe {
        SV_ForceToggle_f(common, cm, server_from_slot(sv), rm, host)
    });
}

// `SV_Map_f`/`SV_MapRestart_f` need `g2: &mut Ghoul2System`; `CmdFunction` now
// threads the g2 receiver in its pinned common/cm/sv/rm/rmg/g2/host order (the
// `EngineHooks::SV_Frame` order), so these forwarders cast both type-erased
// slots (`server_from_slot`/`ghoul2_from_slot`) back to their real receivers and
// reach the full command bodies.
fn SV_Map_f_cmd(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut CmdServerSlot,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    g2: &mut CmdGhoul2Slot,
    host: &mut dyn EngineHost,
) {
    unsafe { SV_Map_f(common, cm, server_from_slot(sv), rm, rmg, ghoul2_from_slot(g2), host) }
}

fn SV_MapRestart_f_cmd(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut CmdServerSlot,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    g2: &mut CmdGhoul2Slot,
    host: &mut dyn EngineHost,
) {
    unsafe { SV_MapRestart_f(common, cm, server_from_slot(sv), rm, rmg, ghoul2_from_slot(g2), host) }
}

