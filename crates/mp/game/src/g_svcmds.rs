// PORT-COMPLETE: g_svcmds.c 14/14
//! Faithful port for `oracle/oracle/codemp/game/g_svcmds.c`.
//!
//! Server-side console commands: IP filtering, entity listing, team forcing.
//! IP filter state is owned by `GameWorld`; accessed via `ctx.world.globals`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_main::G_Printf;
use crate::prelude::*;
use crate::trap;
use mp_abi::game::syscalls::G_ARGC::GArgcArgs;
use mp_abi::game::syscalls::G_ARGV::GArgvArgs;
use mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs;
use mp_abi::game::syscalls::G_FS_FCLOSE_FILE::GFsFcloseFileArgs;
use mp_abi::game::syscalls::G_FS_FOPEN_FILE::GFsFopenFileArgs;
use mp_abi::game::syscalls::G_FS_READ::GFsReadArgs;
use mp_abi::game::syscalls::G_FS_WRITE::GFsWriteArgs;
use mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs;

// IP filter type: holds mask and compare value for IP filtering.
// Source: oracle/oracle/codemp/game/g_svcmds.c:41-45
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ipFilter_t {
    pub mask: c_uint,
    pub compare: c_uint,
}

// Constants from the Raven source.
pub const MAX_IPFILTERS: usize = 1024;

/// Raven `StringToFilter`.
///
/// Parse an IP address string into mask and compare values for IP filtering.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:62-102`
pub fn StringToFilter(ctx: GameContext<'_>, s: *mut c_char, f: *mut c_void) -> qboolean {
    let mut num = [0u8; 128];
    let mut i: c_int = 0;
    let mut b = [0u8; 4];
    let mut m = [0u8; 4];

    for i_val in 0..4 {
        b[i_val] = 0;
        m[i_val] = 0;
    }

    let mut ptr = s;
    for i_val in 0..4 {
        i = 0;
        if unsafe { *ptr } < b'0' as c_char || unsafe { *ptr } > b'9' as c_char {
            let s_str = unsafe { cstr_to_str(s) };
            let msg = format!("Bad filter address: {}\n", s_str);
            G_Printf(ctx, cstr(&msg).as_ptr());
            return qfalse;
        }

        i = 0;
        while unsafe { *ptr } >= b'0' as c_char && unsafe { *ptr } <= b'9' as c_char {
            num[i as usize] = unsafe { *ptr } as u8;
            i += 1;
            unsafe {
                ptr = ptr.offset(1);
            }
        }
        num[i as usize] = 0;

        let num_str = std::ffi::CStr::from_bytes_until_nul(&num[..])
            .unwrap_or_default()
            .to_string_lossy();
        // Oracle assigns `atoi(num)` (int) into a byte, truncating to the low 8
        // bits (e.g. "300" -> 44); `as u8` reproduces that wrap. g_svcmds.c:89.
        b[i_val] = num_str.parse::<i32>().unwrap_or(0) as u8;

        if b[i_val] != 0 {
            m[i_val] = 255;
        }

        if unsafe { *ptr } == 0 as c_char {
            break;
        }
        unsafe {
            ptr = ptr.offset(1);
        }
    }

    let f_filter = f as *mut ipFilter_t;
    unsafe {
        (*f_filter).mask = *(m.as_ptr() as *const c_uint);
        (*f_filter).compare = *(b.as_ptr() as *const c_uint);
    }

    qtrue
}

/// Raven `UpdateIPBans`.
///
/// Rebuild the `g_banIPs` cvar from the active IP filters.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:109-127`
pub fn UpdateIPBans(ctx: GameContext<'_>) {
    let mut iplist = String::new();
    let world = unsafe { &mut *ctx.world };

    for i in 0..(world.globals.numIPFilters as usize) {
        if world.globals.ipFilters[i].compare == 0xffffffff {
            continue;
        }

        let b_ptr = &world.globals.ipFilters[i].compare as *const c_uint as *const u8;
        let b0 = unsafe { *b_ptr };
        let b1 = unsafe { *b_ptr.offset(1) };
        let b2 = unsafe { *b_ptr.offset(2) };
        let b3 = unsafe { *b_ptr.offset(3) };

        iplist.push_str(&format!("{}.{}.{}.{} ", b0, b1, b2, b3));
    }

    let iplist_cstr = cstr(&iplist);
    trap::Cvar_Set(ctx.engine, GCvarSetArgs::new(cstr("g_banIPs"), iplist_cstr));
}

/// Raven `G_FilterPacket`.
///
/// Check if a packet from the given address should be filtered.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:134-166`
pub fn G_FilterPacket(ctx: GameContext<'_>, from: *mut c_char) -> qboolean {
    let mut m = [0u8; 4];
    let mut i: c_int = 0;
    let mut in_: c_uint;
    let mut p: *const c_char = from as *const c_char;
    let world = unsafe { &*ctx.world };

    for i_val in 0..4 {
        m[i_val] = 0;
    }

    i = 0;
    while unsafe { *p } != 0 && i < 4 {
        while unsafe { *p } >= b'0' as c_char && unsafe { *p } <= b'9' as c_char {
            m[i as usize] = m[i as usize]
                .wrapping_mul(10)
                .wrapping_add((unsafe { *p } - b'0' as c_char) as u8);
            unsafe {
                p = p.offset(1);
            }
        }
        if unsafe { *p } == 0 as c_char || unsafe { *p } == b':' as c_char {
            break;
        }
        i += 1;
        unsafe {
            p = p.offset(1);
        }
    }

    in_ = unsafe { *(m.as_ptr() as *const c_uint) };

    for i in 0..(world.globals.numIPFilters as usize) {
        if (in_ & world.globals.ipFilters[i].mask) == world.globals.ipFilters[i].compare {
            return if world.cvars.g_filterBan.integer != 0 {
                qtrue
            } else {
                qfalse
            };
        }
    }

    if world.cvars.g_filterBan.integer == 0 {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `AddIP`.
///
/// Add an IP filter entry.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:173-194`
pub fn AddIP(ctx: GameContext<'_>, str: *mut c_char) {
    let world = unsafe { &mut *ctx.world };

    // Oracle's index runs to `numIPFilters` when no free slot is found, so the
    // `i == numIPFilters` test below appends a new slot (g_svcmds.c:177-179).
    let mut i: c_int = 0;
    while i < world.globals.numIPFilters {
        if world.globals.ipFilters[i as usize].compare == 0xffffffff {
            break; // free spot
        }
        i += 1;
    }

    if i == world.globals.numIPFilters {
        if world.globals.numIPFilters as usize == MAX_IPFILTERS {
            G_Printf(ctx, c"IP filter list is full\n".as_ptr());
            return;
        }
        world.globals.numIPFilters += 1;
    }

    if StringToFilter(
        ctx,
        str,
        &mut world.globals.ipFilters[i as usize] as *mut _ as *mut c_void,
    ) == qfalse
    {
        world.globals.ipFilters[i as usize].compare = 0xffffffffu32;
    }

    UpdateIPBans(ctx);
}

/// Raven `G_ProcessIPBans`.
///
/// Parse space-separated IP addresses from g_banIPs cvar and add them to filters.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:201-218`
pub fn G_ProcessIPBans(ctx: GameContext<'_>) {
    let world = unsafe { &*ctx.world };
    let ban_ips_str = unsafe { cstr_to_str(world.cvars.g_banIPs.string.as_ptr()) };

    // Parse space-separated tokens from the string
    for token in ban_ips_str.split_whitespace() {
        let token_cstr = cstr(token);
        AddIP(ctx, token_cstr.as_ptr() as *mut c_char);
    }
}

/// Raven `Svcmd_AddIP_f`.
///
/// Console command: addip <ip-mask>
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:226-239`
pub fn Svcmd_AddIP_f(ctx: GameContext<'_>) {
    let argc = trap::Argc(ctx.engine, GArgcArgs::new());
    if argc < 2 {
        G_Printf(ctx, c"Usage:  addip <ip-mask>\n".as_ptr());
        return;
    }

    let mut str = [0 as c_char; 128];
    trap::Argv(ctx.engine, GArgvArgs::new(1, str.as_mut_ptr(), 128));

    AddIP(ctx, str.as_mut_ptr());
}

/// Raven `Svcmd_RemoveIP_f`.
///
/// Console command: sv removeip <ip-mask>
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:246-274`
pub fn Svcmd_RemoveIP_f(ctx: GameContext<'_>) {
    let argc = trap::Argc(ctx.engine, GArgcArgs::new());
    if argc < 2 {
        G_Printf(ctx, c"Usage:  sv removeip <ip-mask>\n".as_ptr());
        return;
    }

    let mut str = [0 as c_char; 128];
    trap::Argv(ctx.engine, GArgvArgs::new(1, str.as_mut_ptr(), 128));

    let mut f = ipFilter_t {
        mask: 0,
        compare: 0,
    };
    if StringToFilter(ctx, str.as_mut_ptr(), &mut f as *mut _ as *mut c_void) == qfalse {
        return;
    }

    let world = unsafe { &mut *ctx.world };
    for i in 0..(world.globals.numIPFilters as usize) {
        if world.globals.ipFilters[i].mask == f.mask
            && world.globals.ipFilters[i].compare == f.compare
        {
            world.globals.ipFilters[i].compare = 0xffffffffu32;
            G_Printf(ctx, c"Removed.\n".as_ptr());

            UpdateIPBans(ctx);
            return;
        }
    }

    G_Printf(
        ctx,
        cstr(&format!("Didn't find {}.\n", unsafe {
            {
                cstr_to_str(str.as_ptr())
            }
        }))
        .as_ptr(),
    );
}

/// Raven `Svcmd_ListIPs_f`.
///
/// Console command: list currently banned IPs.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:276-297`
pub fn Svcmd_ListIPs_f(ctx: GameContext<'_>) {
    let world = unsafe { &*ctx.world };

    G_Printf(
        ctx,
        cstr(&format!("{} IP slots used.\n", world.globals.numIPFilters)).as_ptr(),
    );

    for i in 0..(world.globals.numIPFilters as usize) {
        G_Printf(ctx, cstr(&format!("{}: ", i as c_int)).as_ptr());
        if world.globals.ipFilters[i].compare == 0xffffffff {
            G_Printf(ctx, c"unused\n".as_ptr());
        } else {
            let b_ptr = &world.globals.ipFilters[i].compare as *const c_uint as *const u8;
            let b0 = unsafe { *b_ptr };
            let b1 = unsafe { *b_ptr.offset(1) };
            let b2 = unsafe { *b_ptr.offset(2) };
            let b3 = unsafe { *b_ptr.offset(3) };

            let s = format!("{}.{}.{}.{} \n", b0, b1, b2, b3);
            let s_cstr = cstr(&s);
            G_Printf(ctx, cstr(&format!("{}\n", s)).as_ptr());
        }
    }
}

/// Raven `G_SaveBanIP`.
///
/// Save IP filters to "banip.txt" file.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:299-331`
pub fn G_SaveBanIP(ctx: GameContext<'_>) {
    let world = unsafe { &*ctx.world };
    let mut fh: i32 = 0;

    let banip_cstr = cstr("banip.txt");
    trap::FS_FOpenFile(ctx.engine, unsafe {
        GFsFopenFileArgs::new(banip_cstr, &mut fh, FS_WRITE)
    });

    if fh == 0 {
        G_Printf(ctx, c"G_SaveBanIP - ERROR: can't open banip.txt\n".as_ptr());
        return;
    }

    let s = format!("{} \n", world.globals.numIPFilters);
    let s_cstr = cstr(&s);
    trap::FS_Write(
        ctx.engine,
        GFsWriteArgs::new(s_cstr.as_ptr() as *const u8, s.len() as c_int, fh),
    );

    for i in 0..(world.globals.numIPFilters as usize) {
        if world.globals.ipFilters[i].compare == 0xffffffff {
            let unused = "unused \n";
            let unused_cstr = cstr(unused);
            trap::FS_Write(
                ctx.engine,
                GFsWriteArgs::new(unused_cstr.as_ptr() as *const u8, unused.len() as c_int, fh),
            );
        } else {
            let b_ptr = &world.globals.ipFilters[i].compare as *const c_uint as *const u8;
            let b0 = unsafe { *b_ptr };
            let b1 = unsafe { *b_ptr.offset(1) };
            let b2 = unsafe { *b_ptr.offset(2) };
            let b3 = unsafe { *b_ptr.offset(3) };

            let s = format!("{}.{}.{}.{} \n", b0, b1, b2, b3);
            let s_cstr = cstr(&s);
            trap::FS_Write(
                ctx.engine,
                GFsWriteArgs::new(s_cstr.as_ptr() as *const u8, s.len() as c_int, fh),
            );
        }
    }

    trap::FS_FCloseFile(ctx.engine, GFsFcloseFileArgs::new(fh));
}

/// Raven `G_LoadIPBans`.
///
/// Load IP filters from "banip.txt" file.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:333-379`
pub fn G_LoadIPBans(ctx: GameContext<'_>) {
    let world = unsafe { &mut *ctx.world };
    let mut fh: i32 = 0;
    let mut ban_ip_buffer = vec![0 as c_char; 32 * 1024]; // MAX_IPFILTERS * 32

    let banip_cstr = cstr("banip.txt");
    let len = trap::FS_FOpenFile(ctx.engine, unsafe {
        GFsFopenFileArgs::new(banip_cstr, &mut fh, FS_READ)
    });

    if fh == 0 {
        G_Printf(ctx, c"G_LoadBanIP - ERROR: can't open banip.txt\n".as_ptr());
        return;
    }

    trap::FS_Read(
        ctx.engine,
        GFsReadArgs::new(ban_ip_buffer.as_mut_ptr() as *mut u8, len, fh),
    );
    if (len as usize) < ban_ip_buffer.len() {
        ban_ip_buffer[len as usize] = 0;
    }
    trap::FS_FCloseFile(ctx.engine, GFsFcloseFileArgs::new(fh));

    let banIPFile = cstr("banip.txt");
    crate::q_shared::COM_BeginParseSession(banIPFile.as_ptr());

    let mut p: *const c_char = ban_ip_buffer.as_ptr();
    let token = crate::q_shared::COM_ParseExt(&mut p, qtrue);

    if !token.is_null() {
        world.globals.numIPFilters = unsafe {
            let token_str = std::ffi::CStr::from_ptr(token).to_string_lossy();
            token_str.parse::<c_int>().unwrap_or(0)
        };

        for i in 0..(world.globals.numIPFilters as usize) {
            let token = crate::q_shared::COM_ParseExt(&mut p, qtrue);
            if !token.is_null() {
                let token_str = unsafe { std::ffi::CStr::from_ptr(token).to_string_lossy() };
                if token_str.to_lowercase() == "unused" {
                    world.globals.ipFilters[i].compare = 0xffffffffu32;
                } else {
                    StringToFilter(
                        ctx,
                        token as *mut c_char,
                        &mut world.globals.ipFilters[i] as *mut _ as *mut c_void,
                    );
                }
            } else {
                break;
            }
        }
    }
}

/// Raven `Svcmd_EntityList_f`.
///
/// Console command: list entities on the server.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:386-443`
pub fn Svcmd_EntityList_f(ctx: GameContext<'_>) {
    let world = unsafe { &*ctx.world };

    for e in 1..(world.level.num_entities as usize) {
        let check = unsafe { &(*world.g_entities.as_ptr().add(e)) };

        if check.inuse == qfalse {
            continue;
        }

        G_Printf(ctx, cstr(&format!("{:3}:", e as c_int)).as_ptr());

        let type_str = match check.s.eType {
            0 => "ET_GENERAL          ",
            1 => "ET_PLAYER           ",
            2 => "ET_ITEM             ",
            3 => "ET_MISSILE          ",
            4 => "ET_MOVER            ",
            5 => "ET_BEAM             ",
            6 => "ET_PORTAL           ",
            7 => "ET_SPEAKER          ",
            8 => "ET_PUSH_TRIGGER     ",
            9 => "ET_TELEPORT_TRIGGER ",
            10 => "ET_INVISIBLE        ",
            11 => "ET_NPC              ",
            _ => {
                G_Printf(
                    ctx,
                    cstr(&format!("{:3}                 ", check.s.eType as c_int)).as_ptr(),
                );
                ""
            }
        };

        if !type_str.is_empty() {
            G_Printf(ctx, cstr(type_str).as_ptr());
        }

        if !check.classname.is_null() {
            G_Printf(ctx, check.classname);
        }

        G_Printf(ctx, c"\n".as_ptr());
    }
}

/// Raven `ClientForString`.
///
/// Look up a client by slot number or player name.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:445-480`
pub fn ClientForString(ctx: GameContext<'_>, s: *const c_char) -> *mut gclient_t {
    let world = unsafe { &*ctx.world };

    // Check if it's a numeric slot
    if unsafe { *s >= b'0' as c_char && *s <= b'9' as c_char } {
        let s_str = unsafe { std::ffi::CStr::from_ptr(s).to_string_lossy() };
        let idnum: c_int = s_str.parse().unwrap_or(-1);

        if idnum < 0 || idnum >= world.level.maxclients {
            crate::g_main::Com_Printf(cstr(&format!("Bad client slot: {}\n", idnum)).as_ptr());
            return std::ptr::null_mut();
        }

        let cl = unsafe { &(*world.level.clients.add(idnum as usize)) };

        // Check connection status constant (CON_DISCONNECTED = 0)
        if cl.pers.connected == 0 {
            G_Printf(
                ctx,
                cstr(&format!("Client {} is not connected\n", idnum)).as_ptr(),
            );
            return std::ptr::null_mut();
        }

        return unsafe { world.level.clients.add(idnum as usize) as *mut gclient_t };
    }

    // Check for a name match
    for i in 0..(world.level.maxclients as usize) {
        let cl = unsafe { &(*world.level.clients.add(i)) };
        if cl.pers.connected == 0 {
            continue;
        }

        if crate::q_shared::Q_stricmp(cl.pers.netname.as_ptr(), s) == 0 {
            return unsafe { world.level.clients.add(i) as *mut gclient_t };
        }
    }

    G_Printf(
        ctx,
        cstr(&format!("User {} is not on the server\n", unsafe {
            {
                cstr_to_str(s)
            }
        }))
        .as_ptr(),
    );
    std::ptr::null_mut()
}

/// Raven `Svcmd_ForceTeam_f`.
///
/// Console command: force a player to a team.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:489-503`
pub fn Svcmd_ForceTeam_f(ctx: GameContext<'_>) {
    let mut str = [0 as c_char; 128];
    let world = unsafe { &*ctx.world };

    // Get the player identifier
    trap::Argv(ctx.engine, GArgvArgs::new(1, str.as_mut_ptr(), 128));
    let cl = ClientForString(ctx, str.as_ptr());
    if cl.is_null() {
        return;
    }

    // Get the team string
    trap::Argv(ctx.engine, GArgvArgs::new(2, str.as_mut_ptr(), 128));

    // Calculate the entity index from the client pointer
    let cl_idx =
        unsafe { (cl as usize - world.level.clients as usize) / std::mem::size_of::<gclient_t>() };

    let ent = unsafe { world.g_entities.as_ptr().add(cl_idx) as *mut gentity_t };
    crate::g_cmds::SetTeam(ctx, ent, str.as_mut_ptr());
}

/// Raven `ConsoleCommand`.
///
/// Dispatcher for server console commands.
///
/// Source: `oracle/oracle/codemp/game/g_svcmds.c:513-575`
pub fn ConsoleCommand(ctx: GameContext<'_>) -> qboolean {
    let world = unsafe { &*ctx.world };
    let mut cmd = [0 as c_char; 128];

    trap::Argv(ctx.engine, GArgvArgs::new(0, cmd.as_mut_ptr(), 128));

    if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"entitylist".as_ptr()) == 0 {
        Svcmd_EntityList_f(ctx);
        return qtrue;
    }

    if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"forceteam".as_ptr()) == 0 {
        Svcmd_ForceTeam_f(ctx);
        return qtrue;
    }

    if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"game_memory".as_ptr()) == 0 {
        crate::g_mem::Svcmd_GameMem_f(ctx);
        return qtrue;
    }

    if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"addbot".as_ptr()) == 0 {
        crate::g_bot::Svcmd_AddBot_f(ctx);
        return qtrue;
    }

    if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"botlist".as_ptr()) == 0 {
        crate::g_bot::Svcmd_BotList_f(ctx);
        return qtrue;
    }

    if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"addip".as_ptr()) == 0 {
        Svcmd_AddIP_f(ctx);
        return qtrue;
    }

    if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"removeip".as_ptr()) == 0 {
        Svcmd_RemoveIP_f(ctx);
        return qtrue;
    }

    if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"listip".as_ptr()) == 0 {
        Svcmd_ListIPs_f(ctx);
        return qtrue;
    }

    if world.cvars.g_dedicated.integer != 0 {
        if crate::q_shared::Q_stricmp(cmd.as_ptr(), c"say".as_ptr()) == 0 {
            let msg = crate::g_cmds::ConcatArgs(ctx, 1);
            let cmd_str = format!("print \"server: {}\n\"", unsafe {
                std::ffi::CStr::from_ptr(msg).to_string_lossy()
            });
            let cmd_cstr = cstr(&cmd_str);
            trap::SendServerCommand(ctx.engine, GSendServerCommandArgs::new(-1, cmd_cstr));
            return qtrue;
        }

        // Everything else will also be printed as a say command
        let msg = crate::g_cmds::ConcatArgs(ctx, 0);
        let cmd_str = format!("print \"server: {}\n\"", unsafe {
            std::ffi::CStr::from_ptr(msg).to_string_lossy()
        });
        let cmd_cstr = cstr(&cmd_str);
        trap::SendServerCommand(ctx.engine, GSendServerCommandArgs::new(-1, cmd_cstr));
        return qtrue;
    }

    qfalse
}
