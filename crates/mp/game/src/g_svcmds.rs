// PORT-COMPLETE: g_svcmds.c
//! Faithful port for `oracle/codemp/game/g_svcmds.c`.
//!
//! Server-side console commands: IP filtering, entity listing, team forcing.
//! IP filter state is owned by `GameWorld`; accessed via `ctx.world.globals`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_main::G_Printf;
use crate::prelude::*;
use crate::trap;
use crate::world::GameWorld;
use native_string::q_string::Q_stricmp;

// IP filter type: holds mask and compare value for IP filtering.
// Source: oracle/codemp/game/g_svcmds.c:41-45
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ipFilter_t {
    pub mask: c_uint,
    pub compare: c_uint,
}

// Constants from the Raven source.
pub const MAX_IPFILTERS: usize = 1024;

/// Format an `ipFilter_t::compare` word as the oracle's dotted-quad plus a
/// trailing space, walking the 4 bytes in native order (Raven's
/// `((byte *)&compare)[0..3]`).
///
/// Source: `oracle/codemp/game/g_svcmds.c:119-122`
fn format_ip(ip: c_uint) -> String {
    let b = ip.to_ne_bytes();
    format!("{}.{}.{}.{} ", b[0], b[1], b[2], b[3])
}

/// Raven `StringToFilter`.
///
/// Parse an IP address string into mask and compare values for IP filtering.
///
/// Source: `oracle/codemp/game/g_svcmds.c:62-102`
// `f` typed to `*mut ipFilter_t`; the port's `*mut c_void` erasure is retired. Kept a raw pointer
// (not `&mut`) because callers pass `&mut world.globals.ipFilters[i]` that aliases the `ctx` argument
// (STAGE-2b irreducible marker at the AddIP/G_LoadIPBans call sites).
pub fn StringToFilter(ctx: &mut GameContext, s: *mut c_char, f: *mut ipFilter_t) -> qboolean {
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
            G_Printf(ctx, &msg);
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
        // `num` was hand-extracted above as a digit-only substring (the loop
        // above only copies ASCII '0'-'9'), so a whole-token integer parse
        // here is equivalent to libc `atoi` (no sign/whitespace/garbage to
        // diverge on) — not re-flagged to `cstr_util::atoi`.
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

    let f_filter = f;
    // Oracle `*(unsigned *)m` / `*(unsigned *)b`: reinterpret the 4 filled bytes as a native-endian
    // u32. `from_ne_bytes` reproduces that exact read (g_svcmds.c:100-101).
    unsafe {
        (*f_filter).mask = u32::from_ne_bytes(m);
        (*f_filter).compare = u32::from_ne_bytes(b);
    }

    qtrue
}

/// Raven `UpdateIPBans`.
///
/// Rebuild the `g_banIPs` cvar from the active IP filters.
///
/// Source: `oracle/codemp/game/g_svcmds.c:109-127`
pub fn UpdateIPBans(ctx: &mut GameContext) {
    let mut iplist = String::new();

    for i in 0..(ctx.world.globals.numIPFilters as usize) {
        if ctx.world.globals.ipFilters[i].compare == 0xffffffff {
            continue;
        }

        iplist.push_str(&format_ip(ctx.world.globals.ipFilters[i].compare));
    }

    trap::Cvar_Set(ctx.engine, "g_banIPs", &iplist);
}

/// Raven `G_FilterPacket`.
///
/// Check if a packet from the given address should be filtered.
///
/// Source: `oracle/codemp/game/g_svcmds.c:134-166`
pub fn G_FilterPacket(ctx: &mut GameContext, from: *mut c_char) -> qboolean {
    let mut m = [0u8; 4];
    let mut i: c_int = 0;
    let mut in_: c_uint;
    let mut p: *const c_char = from as *const c_char;

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

    // Oracle `*(unsigned *)m`: reinterpret the 4 bytes as a native-endian u32 (g_svcmds.c:161).
    in_ = u32::from_ne_bytes(m);

    for i in 0..(ctx.world.globals.numIPFilters as usize) {
        if (in_ & ctx.world.globals.ipFilters[i].mask) == ctx.world.globals.ipFilters[i].compare {
            return if ctx.world.cvars.g_filterBan.integer != 0 {
                qtrue
            } else {
                qfalse
            };
        }
    }

    if ctx.world.cvars.g_filterBan.integer == 0 {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `AddIP`.
///
/// Add an IP filter entry.
///
/// Source: `oracle/codemp/game/g_svcmds.c:173-194`
pub fn AddIP(ctx: &mut GameContext, str: *mut c_char) {
    // Raw pointer (not a tracked borrow) so `ctx` stays free for the
    // `StringToFilter`/`G_Printf`/`UpdateIPBans` calls below — mirrors the
    // Stage-1 `ctx.world_raw()` idiom (see `g_object.rs`).

    // Oracle's index runs to `numIPFilters` when no free slot is found, so the
    // `i == numIPFilters` test below appends a new slot (g_svcmds.c:177-179).
    let mut i: c_int = 0;
    unsafe {
        while i < ctx.world.globals.numIPFilters {
            if (&ctx.world.globals.ipFilters)[i as usize].compare == 0xffffffff {
                break; // free spot
            }
            i += 1;
        }

        if i == ctx.world.globals.numIPFilters {
            if ctx.world.globals.numIPFilters as usize == MAX_IPFILTERS {
                G_Printf(ctx, "IP filter list is full\n");
                return;
            }
            ctx.world.globals.numIPFilters += 1;
        }

        // STAGE-2b: irreducible — `&mut world.globals.ipFilters[i]` is an out-param
        // that aliases the `ctx` passed to the same StringToFilter call; a raw world
        // pointer hoisted before the call keeps it disjoint from ctx's borrow.
        let world_ptr = ctx.world_raw();
        if StringToFilter(
            ctx,
            str,
            &mut (&mut (*world_ptr).globals.ipFilters)[i as usize] as *mut ipFilter_t,
        ) == qfalse
        {
            (&mut ctx.world.globals.ipFilters)[i as usize].compare = 0xffffffffu32;
        }
    }

    UpdateIPBans(ctx);
}

/// Raven `G_ProcessIPBans`.
///
/// Parse space-separated IP addresses from g_banIPs cvar and add them to filters.
///
/// Source: `oracle/codemp/game/g_svcmds.c:201-218`
pub fn G_ProcessIPBans(ctx: &mut GameContext) {
    let ban_ips_str = unsafe { cstr_to_str(ctx.world.cvars.g_banIPs.string.as_ptr()) };

    // Raven scans the string with `strchr(s, ' ')`: only ' ' separates tokens
    // (tab/newline do not), and the loop breaks at the first token with no
    // trailing space, so a final non-space-terminated token is never added.
    let bytes = ban_ips_str.as_bytes();
    let mut t = 0usize;
    while t < bytes.len() {
        let sp = match bytes[t..].iter().position(|&c| c == b' ') {
            Some(p) => t + p,
            None => break,
        };
        if t < sp {
            let token_cstr = cstr(&ban_ips_str[t..sp]);
            AddIP(ctx, token_cstr.as_ptr() as *mut c_char);
        }
        let mut ns = sp;
        while ns < bytes.len() && bytes[ns] == b' ' {
            ns += 1;
        }
        t = ns;
    }
}

/// Raven `Svcmd_AddIP_f`.
///
/// Console command: addip <ip-mask>
///
/// Source: `oracle/codemp/game/g_svcmds.c:226-239`
pub fn Svcmd_AddIP_f(ctx: &mut GameContext) {
    let argc = trap::Argc(ctx.engine);
    if argc < 2 {
        G_Printf(ctx, "Usage:  addip <ip-mask>\n");
        return;
    }

    let str = trap::Argv(ctx.engine, 1, 128);
    let str_cstr = cstr(&str);

    AddIP(ctx, str_cstr.as_ptr() as *mut c_char);
}

/// Raven `Svcmd_RemoveIP_f`.
///
/// Console command: sv removeip <ip-mask>
///
/// Source: `oracle/codemp/game/g_svcmds.c:246-274`
pub fn Svcmd_RemoveIP_f(ctx: &mut GameContext) {
    let argc = trap::Argc(ctx.engine);
    if argc < 2 {
        G_Printf(ctx, "Usage:  sv removeip <ip-mask>\n");
        return;
    }

    let str = trap::Argv(ctx.engine, 1, 128);
    let str_cstr = cstr(&str);

    let mut f = ipFilter_t {
        mask: 0,
        compare: 0,
    };
    if StringToFilter(ctx, str_cstr.as_ptr() as *mut c_char, &mut f as *mut ipFilter_t) == qfalse {
        return;
    }

    for i in 0..(ctx.world.globals.numIPFilters as usize) {
        if ctx.world.globals.ipFilters[i].mask == f.mask
            && ctx.world.globals.ipFilters[i].compare == f.compare
        {
            ctx.world.globals.ipFilters[i].compare = 0xffffffffu32;
            G_Printf(ctx, "Removed.\n");

            UpdateIPBans(ctx);
            return;
        }
    }

    G_Printf(ctx, &format!("Didn't find {}.\n", str));
}

/// Raven `Svcmd_ListIPs_f`.
///
/// Console command: list currently banned IPs.
///
/// Source: `oracle/codemp/game/g_svcmds.c:276-297`
pub fn Svcmd_ListIPs_f(ctx: &mut GameContext) {
    // Raw pointer (not a tracked borrow) so `ctx` stays free for the `G_Printf`
    // calls inside the loop below — mirrors the Stage-1 `ctx.world_raw()`
    // idiom (see `g_object.rs`).

    unsafe {
        G_Printf(
            ctx,
            &format!("{} IP slots used.\n", ctx.world.globals.numIPFilters),
        );

        for i in 0..(ctx.world.globals.numIPFilters as usize) {
            G_Printf(ctx, &format!("{}: ", i as c_int));
            if (&ctx.world.globals.ipFilters)[i].compare == 0xffffffff {
                G_Printf(ctx, "unused\n");
            } else {
                let s = format!("{}\n", format_ip((&ctx.world.globals.ipFilters)[i].compare));
                G_Printf(ctx, &format!("{}\n", s));
            }
        }
    }
}

/// Raven `G_SaveBanIP`.
///
/// Save IP filters to "banip.txt" file.
///
/// Source: `oracle/codemp/game/g_svcmds.c:299-331`
pub fn G_SaveBanIP(ctx: &mut GameContext) {
    let mut fh: i32 = 0;

    trap::FS_FOpenFile(ctx.engine, "banip.txt", &mut fh, FS_WRITE);

    if fh == 0 {
        G_Printf(ctx, "G_SaveBanIP - ERROR: can't open banip.txt\n");
        return;
    }

    let s = format!("{} \n", ctx.world.globals.numIPFilters);
    trap::FS_Write(ctx.engine, s.as_bytes(), fh);

    for i in 0..(ctx.world.globals.numIPFilters as usize) {
        if ctx.world.globals.ipFilters[i].compare == 0xffffffff {
            let unused = "unused \n";
            trap::FS_Write(ctx.engine, unused.as_bytes(), fh);
        } else {
            let s = format!("{}\n", format_ip(ctx.world.globals.ipFilters[i].compare));
            trap::FS_Write(ctx.engine, s.as_bytes(), fh);
        }
    }

    trap::FS_FCloseFile(ctx.engine, fh);
}

/// Raven `G_LoadIPBans`.
///
/// Load IP filters from "banip.txt" file.
///
/// Source: `oracle/codemp/game/g_svcmds.c:333-379`
pub fn G_LoadIPBans(ctx: &mut GameContext) {
    // Raw pointer (not a tracked borrow) so `ctx` stays free for the
    // `StringToFilter`/`G_Printf` calls below — mirrors the Stage-1
    // `ctx.world_raw()` idiom (see `g_object.rs`).
    let mut fh: i32 = 0;
    let mut ban_ip_buffer = vec![0u8; 32 * 1024]; // MAX_IPFILTERS * 32

    let len = trap::FS_FOpenFile(ctx.engine, "banip.txt", &mut fh, FS_READ);

    if fh == 0 {
        G_Printf(ctx, "G_LoadBanIP - ERROR: can't open banip.txt\n");
        return;
    }

    trap::FS_Read(ctx.engine, &mut ban_ip_buffer[..len as usize], fh);
    if (len as usize) < ban_ip_buffer.len() {
        ban_ip_buffer[len as usize] = 0;
    }
    trap::FS_FCloseFile(ctx.engine, fh);

    // §19 DIVERGENCE: oracle passes the uninitialized `char banIPFile[MAX_QPATH]`
    // to COM_BeginParseSession (UB); the name is only used in parse-error text, so
    // we substitute "banip.txt". Source: `oracle/codemp/game/g_svcmds.c:339,352`.
    crate::q_shared::COM_BeginParseSession(&mut ctx.world.bg_state.qs, "banip.txt");

    let mut p: *const c_char = ban_ip_buffer.as_ptr() as *const c_char;
    let token = crate::q_shared::COM_ParseExt(&mut ctx.world.bg_state.qs, &mut p, qtrue);

    if !token.is_null() {
        ctx.world.globals.numIPFilters = atoi(token);

        for i in 0..(ctx.world.globals.numIPFilters as usize) {
            let token = crate::q_shared::COM_ParseExt(&mut ctx.world.bg_state.qs, &mut p, qtrue);
            if !token.is_null() {
                let token_str = unsafe { std::ffi::CStr::from_ptr(token).to_string_lossy() };
                if token_str.to_lowercase() == "unused" {
                    (&mut ctx.world.globals.ipFilters)[i].compare = 0xffffffffu32;
                } else {
                    // STAGE-2b: irreducible — `&mut world.globals.ipFilters[i]` is an
                    // out-param that aliases the `ctx` passed to the same StringToFilter
                    // call; a raw world pointer hoisted before the call keeps it disjoint
                    // from ctx's borrow.
                    let world_ptr = ctx.world_raw();
                    StringToFilter(ctx, token as *mut c_char, unsafe {
                        &mut (&mut (*world_ptr).globals.ipFilters)[i] as *mut ipFilter_t
                    });
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
/// Source: `oracle/codemp/game/g_svcmds.c:386-443`
pub fn Svcmd_EntityList_f(ctx: &mut GameContext) {
    for e in 1..(ctx.world.level.num_entities as usize) {
        let id = EntityId(e as u32);

        if ctx.world.entity(id).inuse == qfalse {
            continue;
        }

        G_Printf(ctx, &format!("{:3}:", e as c_int));

        let eType = ctx.world.entity(id).s.eType;
        let type_str = match eType {
            0 => "ET_GENERAL          ",
            1 => "ET_PLAYER           ",
            2 => "ET_ITEM             ",
            3 => "ET_MISSILE          ",
            // eType 4 (ET_SPECIAL) and 5 (ET_HOLOCRON) have no C case and fall
            // through to the default numeric label.
            6 => "ET_MOVER            ",
            7 => "ET_BEAM             ",
            8 => "ET_PORTAL           ",
            9 => "ET_SPEAKER          ",
            10 => "ET_PUSH_TRIGGER     ",
            11 => "ET_TELEPORT_TRIGGER ",
            12 => "ET_INVISIBLE        ",
            13 => "ET_NPC              ",
            _ => {
                G_Printf(ctx, &format!("{:3}                 ", eType as c_int));
                ""
            }
        };

        if !type_str.is_empty() {
            G_Printf(ctx, type_str);
        }

        let classname = ctx.world.entity(id).classname;
        if !classname.is_null() {
            G_Printf(ctx, &unsafe { cstr_to_str(classname) });
        }

        G_Printf(ctx, "\n");
    }
}

/// Raven `ClientForString`.
///
/// Look up a client by slot number or player name.
///
/// Source: `oracle/codemp/game/g_svcmds.c:445-480`
pub fn ClientForString(ctx: &mut GameContext, s: *const c_char) -> *mut gclient_t {
    // Check if it's a numeric slot
    if unsafe { *s >= b'0' as c_char && *s <= b'9' as c_char } {
        // Plain `atoi(s)`; oracle has no -1 fallback here (g_svcmds.c:452).
        let idnum: c_int = atoi(s);

        if idnum < 0 || idnum >= ctx.world.level.maxclients {
            crate::g_main::Com_Printf(&format!("Bad client slot: {}\n", idnum));
            return std::ptr::null_mut();
        }

        // Check connection status constant (CON_DISCONNECTED = 0)
        if ctx.world.client(idnum as usize).pers.connected == 0 {
            G_Printf(ctx, &format!("Client {} is not connected\n", idnum));
            return std::ptr::null_mut();
        }

        return unsafe { ctx.world.level.clients.add(idnum as usize) as *mut gclient_t };
    }

    // Check for a name match
    for i in 0..(ctx.world.level.maxclients as usize) {
        let cl = ctx.world.client(i);
        if cl.pers.connected == 0 {
            continue;
        }

        if Q_stricmp(&cl.pers.netname, &unsafe { cstr_to_str(s) }) == 0 {
            return unsafe { ctx.world.level.clients.add(i) as *mut gclient_t };
        }
    }

    G_Printf(
        ctx,
        &format!("User {} is not on the server\n", unsafe { cstr_to_str(s) }),
    );
    std::ptr::null_mut()
}

/// Raven `Svcmd_ForceTeam_f`.
///
/// Console command: force a player to a team.
///
/// Source: `oracle/codemp/game/g_svcmds.c:489-503`
pub fn Svcmd_ForceTeam_f(ctx: &mut GameContext) {
    // Get the player identifier
    let str = trap::Argv(ctx.engine, 1, 128);
    let str_cstr = cstr(&str);
    let cl = ClientForString(ctx, str_cstr.as_ptr());
    if cl.is_null() {
        return;
    }

    // Get the team string
    let team = trap::Argv(ctx.engine, 2, 128);

    // Calculate the entity index from the client pointer (`cl - level.clients`);
    // the client slot is its entity number, so it doubles as the `EntityId`.
    let cl_idx =
        { (cl as usize - ctx.world.level.clients as usize) / std::mem::size_of::<gclient_t>() };

    crate::g_cmds::SetTeam(ctx, EntityId(cl_idx as u32), &team);
}

/// Raven `ConsoleCommand`.
///
/// Dispatcher for server console commands.
///
/// Source: `oracle/codemp/game/g_svcmds.c:513-575`
pub fn ConsoleCommand(ctx: &mut GameContext) -> qboolean {
    let cmd = trap::Argv(ctx.engine, 0, 128);

    if Q_stricmp(&cmd, "entitylist") == 0 {
        Svcmd_EntityList_f(ctx);
        return qtrue;
    }

    if Q_stricmp(&cmd, "forceteam") == 0 {
        Svcmd_ForceTeam_f(ctx);
        return qtrue;
    }

    if Q_stricmp(&cmd, "game_memory") == 0 {
        crate::g_mem::Svcmd_GameMem_f(ctx);
        return qtrue;
    }

    if Q_stricmp(&cmd, "addbot") == 0 {
        crate::g_bot::Svcmd_AddBot_f(ctx);
        return qtrue;
    }

    if Q_stricmp(&cmd, "botlist") == 0 {
        crate::g_bot::Svcmd_BotList_f(ctx);
        return qtrue;
    }

    if Q_stricmp(&cmd, "addip") == 0 {
        Svcmd_AddIP_f(ctx);
        return qtrue;
    }

    if Q_stricmp(&cmd, "removeip") == 0 {
        Svcmd_RemoveIP_f(ctx);
        return qtrue;
    }

    if Q_stricmp(&cmd, "listip") == 0 {
        Svcmd_ListIPs_f(ctx);
        return qtrue;
    }

    if ctx.world.cvars.g_dedicated.integer != 0 {
        if Q_stricmp(&cmd, "say") == 0 {
            let msg = crate::g_cmds::ConcatArgs(ctx, 1);
            let cmd_str = format!("print \"server: {}\n\"", msg);
            trap::SendServerCommand(ctx.engine, -1, &cmd_str);
            return qtrue;
        }

        // Everything else will also be printed as a say command
        let msg = crate::g_cmds::ConcatArgs(ctx, 0);
        let cmd_str = format!("print \"server: {}\n\"", msg);
        trap::SendServerCommand(ctx.engine, -1, &cmd_str);
        return qtrue;
    }

    qfalse
}
