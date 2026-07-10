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
use mp_engine_ghoul2::Ghoul2System;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::Common;
use mp_engine_renderer::RenderModels;
use mp_engine_rmg::RmManager;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::shared::qboolean;

use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server::server_state_t::serverState_t;
use crate::Server;

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

/// Raven `SV_GetPlayerByFedName`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:354-387`
pub fn SV_GetPlayerByFedName(
    common: &mut Common,
    sv: &mut Server,
    name: *const c_char,
) -> *mut client_t {
    let _ = common;
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        return core::ptr::null_mut();
    }

    // check for a name match
    let n = sv.svs.clients;
    for i in 0..sv.sv_maxclients {
        let cl = unsafe { n.offset(i as isize) };
        if unsafe { (*cl).state as i32 } == 0 {
            continue;
        }
        if unsafe { mp_qshared::shared::q_shared::Q_stricmp((*cl).name.as_ptr(), name) } == 0 {
            return cl;
        }

        let mut cleanName = [0 as c_char; 64];
        unsafe {
            mp_qshared::shared::q_shared::Q_strncpyz(
                cleanName.as_mut_ptr(),
                (*cl).name.as_ptr(),
                cleanName.len() as c_int,
            );
            mp_qshared::shared::q_shared::Q_CleanStr(cleanName.as_mut_ptr());
        }
        if unsafe { mp_qshared::shared::q_shared::Q_stricmp(cleanName.as_ptr(), name) } == 0 {
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

    if mp_engine_qcommon::cmd::Cmd_Argc(common) < 2 {
        mp_engine_qcommon::common::common::com_printf(common, "No player specified.\n");
        return core::ptr::null_mut();
    }

    let s = mp_engine_qcommon::cmd::Cmd_Argv(common, 1);

    // check for a name match
    let n = sv.svs.clients;
    for i in 0..sv.sv_maxclients {
        let cl = unsafe { n.offset(i as isize) };
        if unsafe { (*cl).state as i32 } == 0 {
            continue;
        }
        if unsafe { mp_qshared::shared::q_shared::Q_stricmp((*cl).name.as_ptr(), s) } == 0 {
            return cl;
        }

        let mut cleanName = [0 as c_char; 64];
        unsafe {
            mp_qshared::shared::q_shared::Q_strncpyz(
                cleanName.as_mut_ptr(),
                (*cl).name.as_ptr(),
                cleanName.len() as c_int,
            );
            mp_qshared::shared::q_shared::Q_CleanStr(cleanName.as_mut_ptr());
        }
        if unsafe { mp_qshared::shared::q_shared::Q_stricmp(cleanName.as_ptr(), s) } == 0 {
            return cl;
        }
    }

    unsafe {
        mp_engine_qcommon::common::common::com_printf(
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

    if mp_engine_qcommon::cmd::Cmd_Argc(common) < 2 {
        mp_engine_qcommon::common::common::com_printf(common, "No player specified.\n");
        return core::ptr::null_mut();
    }

    let s = mp_engine_qcommon::cmd::Cmd_Argv(common, 1);
    let s_str = unsafe { core::ffi::CStr::from_ptr(s) }
        .to_string_lossy()
        .into_owned();

    for c in s_str.bytes() {
        if !c.is_ascii_digit() {
            mp_engine_qcommon::common::common::com_printf(
                common,
                &format!("Bad slot number: {}\n", s_str),
            );
            return core::ptr::null_mut();
        }
    }
    let idnum = unsafe { mp_qshared::shared::atoi(s) };
    if idnum < 0 || idnum >= sv.sv_maxclients {
        mp_engine_qcommon::common::common::com_printf(
            common,
            &format!("Bad client slot: {}\n", idnum),
        );
        return core::ptr::null_mut();
    }

    let cl = unsafe { sv.svs.clients.offset(idnum as isize) };
    if unsafe { (*cl).state as i32 } == 0 {
        mp_engine_qcommon::common::common::com_printf(
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
        if unsafe { mp_qshared::shared::q_shared::Q_stricmp(name, c"all".as_ptr()) } == 0 {
            let n = sv.svs.clients;
            for i in 0..sv.sv_maxclients {
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
                mp_engine_server::SV_DropClient(common, sv, cl, reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        } else if unsafe { mp_qshared::shared::q_shared::Q_stricmp(name, c"allbots".as_ptr()) } == 0
        {
            let n = sv.svs.clients;
            for i in 0..sv.sv_maxclients {
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
                mp_engine_server::SV_DropClient(common, sv, cl, reason); // "was kicked"
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
        mp_engine_server::SV_SendServerCommand(
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
    mp_engine_server::SV_DropClient(common, sv, cl, reason); // "was kicked"
    unsafe {
        (*cl).lastPacketTime = sv.svs.time;
    } // in case there is a funny zombie
}

/// Raven `SV_Status_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:669-750`
pub fn SV_Status_f(common: &mut Common, sv: &mut Server, host: &mut dyn EngineHost) {
    let mut avoidTruncation = qboolean::qfalse;

    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        mp_engine_qcommon::common::common::com_printf(common, unsafe {
            &core::ffi::CStr::from_ptr(SE_GetString(
                common,
                host,
                c"STR_SERVER_SERVER_NOT_RUNNING".as_ptr(),
            ))
            .to_string_lossy()
        });
        return;
    }

    if mp_engine_qcommon::cmd::Cmd_Argc(common) > 1 {
        if unsafe {
            mp_qshared::shared::q_shared::Q_stricmp(
                c"notrunc".as_ptr(),
                mp_engine_qcommon::cmd::Cmd_Argv(common, 1),
            )
        } == 0
        {
            avoidTruncation = qboolean::qtrue;
        }
    }

    mp_engine_qcommon::common::common::com_printf(
        common,
        &format!("map: {}\n", unsafe {
            core::ffi::CStr::from_ptr((*sv.sv_mapname).string.as_ptr()).to_string_lossy()
        }),
    );

    mp_engine_qcommon::common::common::com_printf(
        common,
        "num score ping name            lastmsg address               qport rate\n",
    );
    mp_engine_qcommon::common::common::com_printf(
        common,
        "--- ----- ---- --------------- ------- --------------------- ----- -----\n",
    );
    for i in 0..sv.sv_maxclients {
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

        let ps = mp_engine_server::SV_GameClientNum(sv, i);
        let s = unsafe {
            core::ffi::CStr::from_ptr(mp_engine_qcommon::net::NET_AdrToString(
                common,
                (*cl).netchan.remoteAddress,
            ))
            .to_string_lossy()
            .into_owned()
        };

        let name = unsafe { core::ffi::CStr::from_ptr((*cl).name.as_ptr()) }.to_string_lossy();
        if avoidTruncation == qboolean::qfalse {
            mp_engine_qcommon::common::common::com_printf(
                common,
                &format!(
                    "{:3} {:5} {} {:<15.15} {:7} {:>21} {:5} {:5}\n",
                    i,
                    unsafe { (*ps).persistant[mp_qshared::shared::PERS_SCORE as usize] },
                    state,
                    name,
                    unsafe { sv.svs.time - (*cl).lastPacketTime },
                    s,
                    unsafe { (*cl).netchan.qport },
                    unsafe { (*cl).rate },
                ),
            );
        } else {
            mp_engine_qcommon::common::common::com_printf(
                common,
                &format!(
                    "{:3} {:5} {} {} {:7} {:>21} {:5} {:5}\n",
                    i,
                    unsafe { (*ps).persistant[mp_qshared::shared::PERS_SCORE as usize] },
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
    mp_engine_qcommon::common::common::com_printf(common, "\n");
}

/// Raven `SV_ConSay_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:757-787`
pub fn SV_ConSay_f(common: &mut Common, sv: &mut Server) {
    if unsafe { (*common.com_dedicated).integer } == 0 {
        mp_engine_qcommon::common::common::com_printf(common, "Server is not dedicated.\n");
        return;
    }

    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        mp_engine_qcommon::common::common::com_printf(common, "Server is not running.\n");
        return;
    }

    if mp_engine_qcommon::cmd::Cmd_Argc(common) < 2 {
        return;
    }

    let mut text = "Server: ".to_string();
    let mut p = unsafe { core::ffi::CStr::from_ptr(mp_engine_qcommon::cmd::Cmd_Args(common)) }
        .to_string_lossy()
        .into_owned();

    if p.starts_with('"') {
        p.remove(0);
        if !p.is_empty() {
            p.pop();
        }
    }

    text.push_str(&p);

    mp_engine_server::SV_SendServerCommand(
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
    let mut fpDisabled =
        mp_engine_qcommon::cvar::Cvar_VariableValue(common, c"g_forcePowerDisable".as_ptr())
            as c_int;
    let mut targetPower: c_int = 0;
    let mut powerDisabled = "Enabled";

    if mp_engine_qcommon::cmd::Cmd_Argc(common) < 2 {
        // no argument supplied, spit out a list of force powers and their numbers
        let mut i: c_int = 0;
        while i < mp_qshared::shared::force_powers::NUM_FORCE_POWERS {
            powerDisabled = if fpDisabled & (1 << i) != 0 {
                "Disabled"
            } else {
                "Enabled"
            };

            mp_engine_qcommon::common::common::com_printf(
                common,
                &format!(
                    "{} - {} - Status: {}\n",
                    i, FORCE_TOGGLE_NAME_PRINTS[i as usize], powerDisabled
                ),
            );
            i += 1;
        }

        mp_engine_qcommon::common::common::com_printf(
            common,
            "Example usage: forcetoggle 3\n(toggles PUSH)\n",
        );
        return;
    }

    targetPower = unsafe { mp_qshared::shared::atoi(mp_engine_qcommon::cmd::Cmd_Argv(common, 1)) };

    if targetPower < 0 || targetPower >= mp_qshared::shared::force_powers::NUM_FORCE_POWERS {
        mp_engine_qcommon::common::common::com_printf(
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

    mp_engine_qcommon::cvar::Cvar_Set(
        common,
        cm,
        rm,
        host,
        c"g_forcePowerDisable".as_ptr() as *mut c_char,
        format!("{}\0", fpDisabled).as_ptr() as *mut c_char,
    );

    mp_engine_qcommon::common::common::com_printf(
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
    mp_qshared::shared::force_powers::NUM_FORCE_POWERS as usize] = [
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
    mp_engine_qcommon::common::common::SV_Shutdown(common, cm, sv, rm, rmg, host, "killserver");
}

/// Raven `SV_Kick_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:455-511`
pub fn SV_Kick_f(common: &mut Common, sv: &mut Server) {
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        mp_engine_qcommon::common::common::com_printf(common, "Server is not running.\n");
        return;
    }

    if mp_engine_qcommon::cmd::Cmd_Argc(common) != 2 {
        mp_engine_qcommon::common::common::com_printf(
            common,
            "Usage: kick <player name>\nkick all = kick everyone\nkick allbots = kick all bots\n",
        );
        return;
    }

    if unsafe {
        mp_qshared::shared::q_shared::Q_stricmp(
            mp_engine_qcommon::cmd::Cmd_Argv(common, 1),
            c"Padawan".as_ptr(),
        )
    } == 0
    {
        // if you try to kick the default name, also try to kick ""
        SV_KickByName(common, sv, c"".as_ptr());
    }

    let cl = SV_GetPlayerByName(common, sv);
    if cl.is_null() {
        if unsafe {
            mp_qshared::shared::q_shared::Q_stricmp(
                mp_engine_qcommon::cmd::Cmd_Argv(common, 1),
                c"all".as_ptr(),
            )
        } == 0
        {
            let n = sv.svs.clients;
            for i in 0..sv.sv_maxclients {
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
                mp_engine_server::SV_DropClient(common, sv, cl, reason); // "was kicked"
                unsafe {
                    (*cl).lastPacketTime = sv.svs.time;
                } // in case there is a funny zombie
            }
        } else if unsafe {
            mp_qshared::shared::q_shared::Q_stricmp(
                mp_engine_qcommon::cmd::Cmd_Argv(common, 1),
                c"allbots".as_ptr(),
            )
        } == 0
        {
            let n = sv.svs.clients;
            for i in 0..sv.sv_maxclients {
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
                mp_engine_server::SV_DropClient(common, sv, cl, reason); // "was kicked"
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
        mp_engine_server::SV_SendServerCommand(
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
    mp_engine_server::SV_DropClient(common, sv, cl, reason); // "was kicked"
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
        mp_engine_qcommon::common::common::com_printf(common, "Server is not running.\n");
        return;
    }

    if mp_engine_qcommon::cmd::Cmd_Argc(common) != 2 {
        mp_engine_qcommon::common::common::com_printf(common, "Usage: kicknum <client number>\n");
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
        mp_engine_server::SV_SendServerCommand(
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
    mp_engine_server::SV_DropClient(common, sv, cl, reason); // "was kicked"
    unsafe {
        (*cl).lastPacketTime = sv.svs.time;
    } // in case there is a funny zombie
}

/// Raven `SV_Serverinfo_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:888-894`
pub fn SV_Serverinfo_f(common: &mut Common) {
    mp_engine_qcommon::common::common::com_printf(common, "Server info settings:\n");
    let info = mp_engine_qcommon::cvar::Cvar_InfoString(
        common,
        mp_game::q_shared_cvar_flags::CVAR_SERVERINFO,
    );
    mp_engine_qcommon::common_fns::Info_Print(common, info.as_ptr() as *const c_char);
    // NOTE: com_sv_running is threaded through `Common` per the Cvar_Get
    // registration precedent elsewhere in this crate, not `Server`, since
    // this fn takes no `sv` receiver (LAW per resolved signature).
    if unsafe { (*common.com_sv_running).integer } == 0 {
        mp_engine_qcommon::common::common::com_printf(common, "Server is not running.\n");
    }
}

/// Raven `SV_Systeminfo_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:904-907`
pub fn SV_Systeminfo_f(common: &mut Common) {
    mp_engine_qcommon::common::common::com_printf(common, "System info settings:\n");
    let info = mp_engine_qcommon::cvar::Cvar_InfoString(
        common,
        mp_game::q_shared_cvar_flags::CVAR_SYSTEMINFO,
    );
    mp_engine_qcommon::common_fns::Info_Print(common, info.as_ptr() as *const c_char);
}

/// Raven `SV_DumpUser_f`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:917-939`
pub fn SV_DumpUser_f(common: &mut Common, sv: &mut Server) {
    // make sure server is running
    if unsafe { (*common.com_sv_running).integer } == 0 {
        mp_engine_qcommon::common::common::com_printf(common, "Server is not running.\n");
        return;
    }

    if mp_engine_qcommon::cmd::Cmd_Argc(common) != 2 {
        mp_engine_qcommon::common::common::com_printf(common, "Usage: info <userid>\n");
        return;
    }

    let cl = SV_GetPlayerByName(common, sv);
    if cl.is_null() {
        return;
    }

    mp_engine_qcommon::common::common::com_printf(common, "userinfo\n");
    mp_engine_qcommon::common::common::com_printf(common, "--------\n");
    mp_engine_qcommon::common_fns::Info_Print(common, unsafe { (*cl).userinfo.as_ptr() });
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
    let map = mp_engine_qcommon::cmd::Cmd_Argv(common, 1);
    if map.is_null() {
        return;
    }

    // make sure the level exists before trying to change, so that
    // a typo at the server console won't end the game
    let map_str = unsafe { core::ffi::CStr::from_ptr(map) }
        .to_string_lossy()
        .into_owned();
    if map_str.contains('\\') {
        mp_engine_qcommon::common::common::com_printf(common, "Can't have mapnames with a \\\n");
        return;
    }

    let expanded = format!("maps/{}.bsp\0", map_str);
    if mp_engine_qcommon::files::FS_ReadFile(
        common,
        cm,
        rm,
        host,
        expanded.as_ptr() as *const c_char,
        core::ptr::null_mut(),
    ) == -1
    {
        mp_engine_qcommon::common::common::com_printf(
            common,
            &format!("Can't find map {}\n", expanded.trim_end_matches('\0')),
        );
        return;
    }

    // force latched values to get set
    mp_engine_qcommon::cvar::Cvar_Get(
        common,
        cm,
        rm,
        host,
        c"g_gametype".as_ptr() as *mut c_char,
        c"0".as_ptr() as *mut c_char,
        mp_game::q_shared_cvar_flags::CVAR_SERVERINFO | mp_game::q_shared_cvar_flags::CVAR_LATCH,
    );

    let mut cmd = unsafe { core::ffi::CStr::from_ptr(mp_engine_qcommon::cmd::Cmd_Argv(common, 0)) }
        .to_string_lossy()
        .into_owned();
    let (cheat, killBots);
    if unsafe {
        mp_qshared::shared::q_shared::Q_stricmpn(
            format!("{}\0", cmd).as_ptr() as *const c_char,
            c"sp".as_ptr(),
            2,
        )
    } == 0
    {
        mp_engine_qcommon::cvar::Cvar_SetValue(
            common,
            cm,
            rm,
            host,
            c"g_gametype".as_ptr() as *const c_char,
            mp_bg::public::gametype::GT_SINGLE_PLAYER as c_int as f32,
        );
        mp_engine_qcommon::cvar::Cvar_SetValue(
            common,
            cm,
            rm,
            host,
            c"g_doWarmup".as_ptr() as *const c_char,
            0.0,
        );
        // may not set sv_maxclients directly, always set latched
        mp_engine_qcommon::cvar::Cvar_SetLatched(
            common,
            cm,
            rm,
            host,
            c"sv_maxclients".as_ptr() as *mut c_char,
            c"8".as_ptr() as *mut c_char,
        );
        cmd = cmd[2..].to_string();
        cheat = qboolean::qfalse;
        killBots = qboolean::qtrue;
    } else {
        let cmd_c = format!("{}\0", cmd);
        if unsafe {
            mp_qshared::shared::q_shared::Q_stricmpn(
                cmd_c.as_ptr() as *const c_char,
                c"devmap".as_ptr(),
                6,
            )
        } == 0
            || unsafe {
                mp_qshared::shared::q_shared::Q_stricmp(
                    cmd_c.as_ptr() as *const c_char,
                    c"spdevmap".as_ptr(),
                )
            } == 0
        {
            cheat = qboolean::qtrue;
            killBots = qboolean::qtrue;
        } else {
            cheat = qboolean::qfalse;
            killBots = qboolean::qfalse;
        }
        // if( sv_gametype->integer == GT_SINGLE_PLAYER ) {
        //     Cvar_SetValue( "g_gametype", GT_FFA );
        // }
    }

    // save the map name here cause on a map restart we reload the jampconfig.cfg
    // and thus nuke the arguments of the map command
    let mut mapname = [0 as c_char; mp_qshared::shared::MAX_QPATH as usize];
    unsafe {
        mp_qshared::shared::q_shared::Q_strncpyz(mapname.as_mut_ptr(), map, mapname.len() as c_int);
    }

    let mut eForceReload = mp_qshared::shared::force_reload::ForceReload_e::eForceReload_NOTHING;

    // if ( !Q_stricmp( cmd, "devmapbsp") ) {	// not relevant in MP codebase
    //     eForceReload = eForceReload_BSP;
    // }
    // else
    let cmd_c = format!("{}\0", cmd);
    if unsafe {
        mp_qshared::shared::q_shared::Q_stricmp(
            cmd_c.as_ptr() as *const c_char,
            c"devmapmdl".as_ptr(),
        )
    } == 0
    {
        eForceReload = mp_qshared::shared::force_reload::ForceReload_e::eForceReload_MODELS;
    } else if unsafe {
        mp_qshared::shared::q_shared::Q_stricmp(
            cmd_c.as_ptr() as *const c_char,
            c"devmapall".as_ptr(),
        )
    } == 0
    {
        eForceReload = mp_qshared::shared::force_reload::ForceReload_e::eForceReload_ALL;
    }

    // start up the map
    mp_engine_server::SV_SpawnServer(
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
    if cheat == qboolean::qtrue {
        mp_engine_qcommon::cvar::Cvar_Set(
            common,
            cm,
            rm,
            host,
            c"sv_cheats".as_ptr() as *mut c_char,
            c"1".as_ptr() as *mut c_char,
        );
    } else {
        mp_engine_qcommon::cvar::Cvar_Set(
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
        mp_engine_qcommon::common::common::com_printf(common, "Server is not running.\n");
        return;
    }

    if sv.sv.restartTime != 0 {
        return;
    }

    let delay = if mp_engine_qcommon::cmd::Cmd_Argc(common) > 1 {
        unsafe { mp_qshared::shared::atoi(mp_engine_qcommon::cmd::Cmd_Argv(common, 1)) }
    } else {
        5
    };
    if delay != 0 {
        sv.sv.restartTime = sv.svs.time + delay * 1000;
        mp_engine_server::SV_SetConfigstring(
            common,
            cm,
            sv,
            rm,
            host,
            mp_bg::public::configstring::CS_WARMUP,
            &format!("{}", sv.sv.restartTime),
        );
        return;
    }

    // check for changes in variables that can't just be restarted
    // check for maxclients change
    if unsafe { (*sv.sv_maxclients).modified != 0 || (*sv.sv_gametype).modified != 0 } {
        // restart the map the slow way
        let mut mapname = [0 as c_char; mp_qshared::shared::MAX_QPATH as usize];
        unsafe {
            mp_qshared::shared::q_shared::Q_strncpyz(
                mapname.as_mut_ptr(),
                mp_engine_qcommon::cvar::Cvar_VariableString(common, c"mapname".as_ptr()),
                mapname.len() as c_int,
            );
        }

        mp_engine_qcommon::common::common::com_printf(common, "variable change -- restarting.\n");

        mp_engine_server::SV_SpawnServer(
            common,
            cm,
            sv,
            rm,
            rmg,
            g2,
            host,
            mapname.as_mut_ptr(),
            qboolean::qfalse,
            mp_qshared::shared::force_reload::ForceReload_e::eForceReload_NOTHING,
        );
        return;
    }

    // toggle the server bit so clients can detect that a
    // map_restart has happened
    sv.svs.snapFlagServerBit ^= SNAPFLAG_SERVERCOUNT;

    // generate a new serverid
    sv.sv.restartedServerId = sv.sv.serverId;
    sv.sv.serverId = common.com_frameTime;
    mp_engine_qcommon::cvar::Cvar_Set(
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
    sv.sv.restarting = qboolean::qtrue;

    mp_engine_server::SV_RestartGameProgs(common, cm, sv, rm, host);

    // run a few frames to allow everything to settle
    for _ in 0..3 {
        mp_engine_qcommon::vm::VM_Call(
            common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_RUN_FRAME as c_int,
            &[sv.svs.time],
        );
        sv.svs.time += 100;
    }

    sv.sv.state = serverState_t::SS_GAME;
    sv.sv.restarting = qboolean::qfalse;

    // connect and begin all the clients
    for i in 0..sv.sv_maxclients {
        let client = unsafe { sv.svs.clients.offset(i as isize) };

        // send the new gamestate to all connected clients
        if unsafe { (*client).state as i32 } < clientState_t::CS_CONNECTED as i32 {
            continue;
        }

        let isBot = if unsafe { (*client).netchan.remoteAddress.r#type } == netadrtype_t::NA_BOT {
            qboolean::qtrue
        } else {
            qboolean::qfalse
        };

        // add the map_restart command
        mp_engine_server::SV_AddServerCommand(common, sv, client, "map_restart\n");

        // connect the client again, without the firstTime flag
        let denied = mp_engine_qcommon::vm::VM_ExplicitArgPtr(
            common,
            sv.gvm,
            mp_engine_qcommon::vm::VM_Call(
                common,
                sv.gvm,
                mp_abi::game::exports::MpGameExport::GAME_CLIENT_CONNECT as c_int,
                &[i, qboolean::qfalse as isize, isBot as isize],
            ),
        ) as *mut c_char;
        if !denied.is_null() {
            // this generally shouldn't happen, because the client
            // was connected before the level change
            mp_engine_server::SV_DropClient(common, sv, client, denied);
            mp_engine_qcommon::common::common::com_printf(
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

        mp_engine_server::SV_ClientEnterWorld(common, sv, client, unsafe {
            &mut (*client).lastUsercmd
        });
    }

    // run another frame to allow things to look at all the players
    mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_RUN_FRAME as c_int,
        &[sv.svs.time],
    );
    sv.svs.time += 100;
}

/// Raven `SNAPFLAG_SERVERCOUNT`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:283` (`svs.snapFlagServerBit ^=
/// SNAPFLAG_SERVERCOUNT`) — escalated: not yet in the rosetta.
// PORT-NOTE(snapflag): flag value transcribed from the Raven public headers'
// well-known bit (`SNAPFLAG_SERVERCOUNT = 4`, `q_shared.h`); escalate if the
// rosetta lands a differing row.
const SNAPFLAG_SERVERCOUNT: i32 = 4;

/// Raven `SV_AddOperatorCommands`.
///
/// Source: `oracle/codemp/server/sv_ccmds.cpp:958-996`
pub fn SV_AddOperatorCommands(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    // Function-scope static `initialized` (fork-3 kind 3: genuine cross-frame
    // state) belongs on the owning host struct; `Common` is the only threaded
    // receiver here, so it homes there pending the campaign's receiver-cleanup
    // pass (STATE-D4-style deferral, matches this crate's existing precedent).
    if common.sv_ccmds_operator_commands_initialized == qboolean::qtrue {
        return;
    }
    common.sv_ccmds_operator_commands_initialized = qboolean::qtrue;

    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "heartbeat", SV_Heartbeat_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "kick", SV_Kick_f_cmd);
    // #ifdef USE_CD_KEY
    //     Cmd_AddCommand ("banUser", SV_Ban_f);
    //     Cmd_AddCommand ("banClient", SV_BanNum_f);
    // #endif	// USE_CD_KEY

    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "clientkick", SV_KickNum_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "status", SV_Status_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "serverinfo", SV_Serverinfo_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "systeminfo", SV_Systeminfo_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "dumpuser", SV_DumpUser_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(
        common,
        cm,
        rm,
        host,
        "map_restart",
        SV_MapRestart_f_cmd,
    );
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "sectorlist", SV_SectorList_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "map", SV_Map_f_cmd);
    // #ifndef PRE_RELEASE_DEMO
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "devmap", SV_Map_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "spmap", SV_Map_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "spdevmap", SV_Map_f_cmd);
    // Cmd_AddCommand ("devmapbsp", SV_Map_f);	// not used in MP codebase, no server BSP_cacheing
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "devmapmdl", SV_Map_f_cmd);
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "devmapall", SV_Map_f_cmd);
    // #endif
    mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "killserver", SV_KillServer_f_cmd);
    // if( com_dedicated->integer )
    {
        mp_engine_qcommon::cmd::Cmd_AddCommand(common, cm, rm, host, "svsay", SV_ConSay_f_cmd);
    }

    mp_engine_qcommon::cmd::Cmd_AddCommand(
        common,
        cm,
        rm,
        host,
        "forcetoggle",
        SV_ForceToggle_f_cmd,
    );
}

// PORT-NOTE(cmd-table-shape, matches common_fns.rs's existing
// Com_Error_f_cmd/Com_Quit_f_cmd precedent): `Cmd_AddCommand`'s resolved
// callee slot is a zero-arg `fn()` (ruling 5's plain dispatch-table shape);
// this crate's ported command bodies are receiver-threaded (`&mut Common`,
// `&mut Server`, …), which cannot satisfy that shape directly without the
// campaign's still-open command-table/Engine-access wiring. Escalated as a
// shape mismatch — the trampolines below are named placeholders for that
// wiring, not stubs of the ported logic (the logic lives in the real fns
// above).
fn SV_Heartbeat_f_cmd() {}
fn SV_Kick_f_cmd() {}
fn SV_KickNum_f_cmd() {}
fn SV_Status_f_cmd() {}
fn SV_Serverinfo_f_cmd() {}
fn SV_Systeminfo_f_cmd() {}
fn SV_DumpUser_f_cmd() {}
fn SV_MapRestart_f_cmd() {}
fn SV_SectorList_f_cmd() {}
fn SV_Map_f_cmd() {}
fn SV_KillServer_f_cmd() {}
fn SV_ConSay_f_cmd() {}
fn SV_ForceToggle_f_cmd() {}

/// Raven `SE_GetString` — `docs/subsystems/stringed.md` §F seam (LIVE,
/// game-module trap at sv_game.cpp:699); not yet landed in this tree.
///
/// Source: `docs/subsystems/stringed.md`
// PORT-NOTE(stringed-seam): genuinely missing — escalated (missing_symbols).
fn SE_GetString(
    _common: &mut Common,
    _host: &mut dyn EngineHost,
    key: *const c_char,
) -> *const c_char {
    key
}
