// PORT-COMPLETE: g_cmds.c
//! FAITHFUL port of `oracle/codemp/game/g_cmds.c`.
//!
//! Filled by the jampgame mega-pass, pass-2 retrofitted with `ctx: GameContext`,
//! and pass-3 blind-transcribed against the settled fork rulings (the
//! va/printf mapping table, EntityId/fn-enum idioms).
//!
//! Pass-3 status: every fn has a real body. `ClientCommand`'s dispatch tail
//! drops the `#ifdef _DEBUG`/`VM_MEMALLOC_DEBUG` branches as dead surface
//! (§20 — neither macro is defined in any target build).
#![allow(non_snake_case, unused, clippy::all)]

use crate::client::client_connected::CON_CONNECTED;
use crate::client::player_team_state::playerTeamStateState_t;
use crate::g_team::{COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA};
use crate::prelude::*;
use crate::trap;

/// Raven `SAY_ALL`/`SAY_TEAM`/`SAY_TELL` chat-mode `#define`s.
///
/// Source: `oracle/codemp/game/q_shared.h:3064-3066`
const SAY_ALL: c_int = 0;
const SAY_TEAM: c_int = 1;
const SAY_TELL: c_int = 2;

/// Raven `LAST_USEABLE_WEAPON` — `WP_BRYAR_OLD`.
///
/// Source: `oracle/codemp/game/bg_weapons.h:43`
const LAST_USEABLE_WEAPON: c_int = WP_BRYAR_OLD;

/// Raven `gc_orders[]` — canned "game command" voice-order strings.
///
/// Source: `oracle/codemp/game/g_cmds.c:1812-1820`
static gc_orders: [&core::ffi::CStr; 7] = [
    c"hold your position",
    c"hold this position",
    c"come here",
    c"cover me",
    c"guard location",
    c"search and destroy",
    c"report",
];

/// Raven `gameNames[]` — display names for each `gametype_t`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1851-1862`
static gameNames: [&core::ffi::CStr; 10] = [
    c"Free For All",
    c"Holocron FFA",
    c"Jedi Master",
    c"Duel",
    c"Power Duel",
    c"Single Player",
    c"Team FFA",
    c"Siege",
    c"Capture the Flag",
    c"Capture the Ysalamiri",
];

/// Byte-exact C-string equality (`!strcmp(a, b)`), used by the name-lookup
/// walkers below in place of Raven's `strcmp`.
unsafe fn cstr_eq(mut a: *const c_char, mut b: *const c_char) -> bool {
    loop {
        if *a != *b {
            return false;
        }
        if *a == 0 {
            return true;
        }
        a = a.add(1);
        b = b.add(1);
    }
}

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `DeathmatchScoreboardMessage`.
///
/// Source: `oracle/codemp/game/g_cmds.c:25-88`
pub fn DeathmatchScoreboardMessage(ctx: &mut GameContext, ent: EntityId) {
    use mp_qshared::common::mp::playerstate::{
        PERS_ASSIST_COUNT, PERS_CAPTURES, PERS_DEFEND_COUNT, PERS_EXCELLENT_COUNT,
        PERS_GAUNTLET_FRAG_COUNT, PERS_IMPRESSIVE_COUNT, PERS_KILLED, PERS_RANK, PERS_SCORE,
    };

    let mut string = String::new();
    let mut stringlength: usize = 0;
    let scoreFlags: c_int = 0;

    let mut numSorted = ctx.world.level.numConnectedClients;
    if numSorted > MAX_CLIENT_SCORE_SEND {
        numSorted = MAX_CLIENT_SCORE_SEND;
    }

    for i in 0..numSorted {
        let cnum = ctx.world.level.sortedClients[i as usize] as usize;

        let ping = if ctx.world.client(cnum).pers.connected
            == crate::client::client_connected::CON_CONNECTING
        {
            -1
        } else if ctx.world.client(cnum).ps.ping < 999 {
            ctx.world.client(cnum).ps.ping
        } else {
            999
        };

        let accuracy = if ctx.world.client(cnum).accuracy_shots != 0 {
            ctx.world.client(cnum).accuracy_hits * 100 / ctx.world.client(cnum).accuracy_shots
        } else {
            0
        };
        let perfect = if ctx.world.client(cnum).ps.persistant[PERS_RANK as usize] == 0
            && ctx.world.client(cnum).ps.persistant[PERS_KILLED as usize] == 0
        {
            1
        } else {
            0
        };

        let sortedClientIdx = ctx.world.level.sortedClients[i as usize];
        let entry = format!(
            " {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            sortedClientIdx,
            ctx.world.client(cnum).ps.persistant[PERS_SCORE as usize],
            ping,
            (ctx.world.level.time - ctx.world.client(cnum).pers.enterTime) / 60000,
            scoreFlags,
            ctx.world.g_entities[sortedClientIdx as usize].s.powerups,
            accuracy,
            ctx.world.client(cnum).ps.persistant[PERS_IMPRESSIVE_COUNT as usize],
            ctx.world.client(cnum).ps.persistant[PERS_EXCELLENT_COUNT as usize],
            ctx.world.client(cnum).ps.persistant[PERS_GAUNTLET_FRAG_COUNT as usize],
            ctx.world.client(cnum).ps.persistant[PERS_DEFEND_COUNT as usize],
            ctx.world.client(cnum).ps.persistant[PERS_ASSIST_COUNT as usize],
            perfect,
            ctx.world.client(cnum).ps.persistant[PERS_CAPTURES as usize],
        );
        let j = entry.len();
        if stringlength + j > 1022 {
            break;
        }
        string.push_str(&entry);
        stringlength += j;
    }

    // still want to know the total # of clients
    let i = ctx.world.level.numConnectedClients;

    let cmd = format!(
        "scores {} {} {}{}",
        i,
        ctx.world.level.teamScores[TEAM_RED as usize],
        ctx.world.level.teamScores[TEAM_BLUE as usize],
        string
    );
    trap::SendServerCommand(
        ctx.engine,
        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
            ent.index() as c_int,
            cstr(&cmd),
        ),
    );
}

/// Raven `MAX_CLIENT_SCORE_SEND`.
///
/// No workspace canonical exists yet (belongs in `mp_bg` from `bg_public.h`);
/// kept local at the oracle value. Consolidation candidate.
/// Source: `oracle/codemp/game/bg_public.h:51`
const MAX_CLIENT_SCORE_SEND: c_int = 20;

/// Raven `Cmd_Score_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:98-100`
pub fn Cmd_Score_f(ctx: &mut GameContext, ent: EntityId) {
    DeathmatchScoreboardMessage(ctx, ent);
}

/// Raven `CheatsOk`.
///
/// Source: `oracle/codemp/game/g_cmds.c:109-119`
pub fn CheatsOk(ctx: &mut GameContext, ent: EntityId) -> qboolean {
    unsafe {
        if ctx.world.cvars.g_cheats.integer == 0 {
            let msg = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOCHEATS".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(msg));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return qfalse;
        }
        if ctx.world.entity(ent).health <= 0 {
            let msg = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"MUSTBEALIVE".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(msg));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return qfalse;
        }
        qtrue
    }
}

/// Raven `ConcatArgs`.
///
/// Source: `oracle/codemp/game/g_cmds.c:127-152`
pub fn ConcatArgs(ctx: &mut GameContext, start: c_int) -> *mut c_char {
    // DIVERGENCE: Raven returns a pointer into file-scope
    // `static char line[MAX_STRING_CHARS]`; ported callers all consume the
    // result immediately (as `*const c_char`), so a `ctx`-owned scratch buffer
    // on `GameWorld` stands in for the static and is returned as a raw pointer
    // valid until the next `ConcatArgs` call (mirrors the Raven rotating-buffer
    // lifetime discipline used elsewhere for `va`).
    unsafe {
        let c = trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new());
        let mut len: usize = 0;
        let mut out: Vec<u8> = Vec::new();
        for i in start..c {
            let mut arg = [0 as c_char; MAX_STRING_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    i,
                    arg.as_mut_ptr(),
                    MAX_STRING_CHARS as c_int,
                ),
            );
            // Raw-byte `strlen`/`memcpy`: routing through `cstr_to_str`
            // (from_utf8_lossy) inflates 0x80-0xFF bytes to U+FFFD and shifts the
            // `MAX_STRING_CHARS - 1` break point, so copy the argv bytes verbatim.
            // Source: `oracle/codemp/game/g_cmds.c:137-146`
            let tlen = arg.iter().position(|&b| b == 0).unwrap_or(MAX_STRING_CHARS);
            if len + tlen >= MAX_STRING_CHARS - 1 {
                break;
            }
            out.extend(arg[..tlen].iter().map(|&b| b as u8));
            len += tlen;
            if i != c - 1 {
                out.push(b' ');
                len += 1;
            }
        }
        ctx.world.bg_state.concat_args_line = out;
        ctx.world.bg_state.concat_args_line.push(0);
        ctx.world.bg_state.concat_args_line.as_mut_ptr() as *mut c_char
    }
}

/// Raven `SanitizeString`.
///
/// Remove case and control characters.
///
/// Source: `oracle/codemp/game/g_cmds.c:161-175`
pub fn SanitizeString(r#in: *mut c_char, out: *mut c_char) {
    unsafe {
        let mut i = r#in;
        let mut o = out;
        loop {
            let c = *i as u8;
            if c == 0 {
                break;
            }
            if c == 27 {
                // skip color code
                i = i.add(2);
                continue;
            }
            if (c as i8) < 32 {
                i = i.add(1);
                continue;
            }
            *o = (c as char).to_ascii_lowercase() as c_char;
            o = o.add(1);
            i = i.add(1);
        }
        *o = 0;
    }
}

/// Raven `ClientNumberFromString`.
///
/// Returns a player number for either a number or name string. Returns -1 if invalid.
///
/// Source: `oracle/codemp/game/g_cmds.c:185-221`
pub fn ClientNumberFromString(ctx: &mut GameContext, to: EntityId, s: *mut c_char) -> c_int {
    unsafe {
        let ss = cstr_to_str(s);

        if let Some(c0) = ss.as_bytes().first() {
            if (*c0 as char).is_ascii_digit() {
                // Source: oracle/codemp/game/g_cmds.c:193 — plain `atoi(s)`.
                let idnum: c_int = atoi(s as *const c_char);
                if idnum < 0 || idnum >= ctx.world.level.maxclients {
                    let msg = format!("print \"Bad client slot: {}\n\"", idnum);
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                            to.index() as c_int,
                            cstr(&msg),
                        ),
                    );
                    return -1;
                }

                if ctx.world.client(idnum as usize).pers.connected != CON_CONNECTED {
                    let msg = format!("print \"Client {} is not active\n\"", idnum);
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                            to.index() as c_int,
                            cstr(&msg),
                        ),
                    );
                    return -1;
                }
                return idnum;
            }
        }

        // check for a name match
        let mut s2 = [0 as c_char; MAX_STRING_CHARS];
        SanitizeString(s, s2.as_mut_ptr());
        for idnum in 0..ctx.world.level.maxclients {
            if ctx.world.client(idnum as usize).pers.connected != CON_CONNECTED {
                continue;
            }
            let mut n2 = [0 as c_char; MAX_STRING_CHARS];
            SanitizeString(
                ctx.world.client(idnum as usize).pers.netname.as_ptr() as *mut c_char,
                n2.as_mut_ptr(),
            );
            if cstr_eq(n2.as_ptr(), s2.as_ptr()) {
                return idnum;
            }
        }

        let msg = format!("print \"User {} is not on the server\n\"", ss);
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                to.index() as c_int,
                cstr(&msg),
            ),
        );
        -1
    }
}

/// Raven `Cmd_Give_f`.
///
/// Give items to a client.
///
/// Source: `oracle/codemp/game/g_cmds.c:230-392`
pub fn Cmd_Give_f(ctx: &mut GameContext, cmdent: EntityId, baseArg: c_int) {
    unsafe {
        if CheatsOk(ctx, cmdent) == qfalse {
            return;
        }

        let ent: EntityId;
        if baseArg != 0 {
            let mut otherindex = [0 as c_char; MAX_TOKEN_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    1,
                    otherindex.as_mut_ptr(),
                    MAX_TOKEN_CHARS as c_int,
                ),
            );

            if otherindex[0] == 0 {
                crate::g_main::Com_Printf(
                    cstr("giveother requires that the second argument be a client index number.\n")
                        .as_ptr(),
                );
                return;
            }

            let i: c_int = atoi_str(&cstr_to_str(otherindex.as_ptr()));

            if !(0..MAX_CLIENTS as c_int).contains(&i) {
                crate::g_main::Com_Printf(cstr(&format!("{} is not a client index\n", i)).as_ptr());
                return;
            }

            let iid = EntityId(i as u32);

            if ctx.world.entity(iid).inuse == qfalse || ctx.world.entity(iid).client.is_null() {
                crate::g_main::Com_Printf(
                    cstr(&format!("{} is not an active client\n", i)).as_ptr(),
                );
                return;
            }
            ent = iid;
        } else {
            ent = cmdent;
        }

        // `ent` is provably a real client slot here (either `cmdent`, the
        // commanding player, or a `[0, MAX_CLIENTS)` index whose `.client` was
        // just null-checked), so its client index is `ent.index()`.
        let cidx = ent.index();

        let mut name = [0 as c_char; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1 + baseArg,
                name.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );
        let name_str = cstr_to_str(name.as_ptr());

        let give_all = name_str.eq_ignore_ascii_case("all");

        if give_all {
            for i in 0..HI_NUM_HOLDABLE {
                ctx.world.client_mut(cidx).ps.stats[STAT_HOLDABLE_ITEMS as usize] |= 1 << i;
            }
        }

        if give_all || name_str.eq_ignore_ascii_case("health") {
            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new())
                == 3 + baseArg
            {
                let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        2 + baseArg,
                        arg.as_mut_ptr(),
                        MAX_TOKEN_CHARS as c_int,
                    ),
                );
                ctx.world.entity_mut(ent).health = atoi_str(&cstr_to_str(arg.as_ptr()));
                let max_health = ctx.world.client(cidx).ps.stats[STAT_MAX_HEALTH as usize];
                if ctx.world.entity(ent).health > max_health {
                    ctx.world.entity_mut(ent).health = max_health;
                }
            } else {
                ctx.world.entity_mut(ent).health =
                    ctx.world.client(cidx).ps.stats[STAT_MAX_HEALTH as usize];
            }
            if !give_all {
                return;
            }
        }

        if give_all || name_str.eq_ignore_ascii_case("weapons") {
            ctx.world.client_mut(cidx).ps.stats[STAT_WEAPONS as usize] =
                (1 << (LAST_USEABLE_WEAPON + 1)) - (1 << WP_NONE);
            if !give_all {
                return;
            }
        }

        if !give_all && name_str.eq_ignore_ascii_case("weaponnum") {
            let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    2 + baseArg,
                    arg.as_mut_ptr(),
                    MAX_TOKEN_CHARS as c_int,
                ),
            );
            let n: c_int = atoi_str(&cstr_to_str(arg.as_ptr()));
            ctx.world.client_mut(cidx).ps.stats[STAT_WEAPONS as usize] |= 1 << n;
            return;
        }

        if give_all || name_str.eq_ignore_ascii_case("ammo") {
            let mut num = 999;
            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new())
                == 3 + baseArg
            {
                let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        2 + baseArg,
                        arg.as_mut_ptr(),
                        MAX_TOKEN_CHARS as c_int,
                    ),
                );
                num = atoi_str(&cstr_to_str(arg.as_ptr()));
            }
            for i in 0..MAX_WEAPONS as usize {
                ctx.world.client_mut(cidx).ps.ammo[i] = num;
            }
            if !give_all {
                return;
            }
        }

        if give_all || name_str.eq_ignore_ascii_case("armor") {
            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new())
                == 3 + baseArg
            {
                let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        2 + baseArg,
                        arg.as_mut_ptr(),
                        MAX_TOKEN_CHARS as c_int,
                    ),
                );
                ctx.world.client_mut(cidx).ps.stats[STAT_ARMOR as usize] =
                    atoi_str(&cstr_to_str(arg.as_ptr()));
            } else {
                let max_health = ctx.world.client(cidx).ps.stats[STAT_MAX_HEALTH as usize];
                ctx.world.client_mut(cidx).ps.stats[STAT_ARMOR as usize] = max_health;
            }

            if !give_all {
                return;
            }
        }

        if name_str.eq_ignore_ascii_case("excellent") {
            ctx.world.client_mut(cidx).ps.persistant[PERS_EXCELLENT_COUNT as usize] += 1;
            return;
        }
        if name_str.eq_ignore_ascii_case("impressive") {
            ctx.world.client_mut(cidx).ps.persistant[PERS_IMPRESSIVE_COUNT as usize] += 1;
            return;
        }
        if name_str.eq_ignore_ascii_case("gauntletaward") {
            ctx.world.client_mut(cidx).ps.persistant[PERS_GAUNTLET_FRAG_COUNT as usize] += 1;
            return;
        }
        if name_str.eq_ignore_ascii_case("defend") {
            ctx.world.client_mut(cidx).ps.persistant[PERS_DEFEND_COUNT as usize] += 1;
            return;
        }
        if name_str.eq_ignore_ascii_case("assist") {
            ctx.world.client_mut(cidx).ps.persistant[PERS_ASSIST_COUNT as usize] += 1;
            return;
        }

        // spawn a specific item right on the player
        if !give_all {
            let it = crate::bg_misc::BG_FindItem(name.as_ptr());
            if it.is_null() {
                return;
            }

            let it_ent = crate::g_utils::G_Spawn(ctx);
            let it_id = ctx.entity_id_of(it_ent).unwrap();
            let origin = ctx.world.entity(ent).r.currentOrigin;
            crate::q_math::_VectorCopy(origin, &mut ctx.world.entity_mut(it_id).s.origin);
            // FLAG: `it` is a raw `*gitem_t` (BG item table, not an entity/client
            // arena); its deref stays raw per recipe 2c.
            let classname = (*it).classname;
            ctx.world.entity_mut(it_id).classname = classname;
            crate::g_items::G_SpawnItem(ctx, it_id, it);
            crate::g_items::FinishSpawningItem(ctx, it_id);
            let mut trace: trace_t = core::mem::zeroed();
            crate::g_items::Touch_Item(ctx, it_id, Some(ent), &mut trace);
            if ctx.world.entity(it_id).inuse != qfalse {
                crate::g_utils::G_FreeEntity(ctx, Some(it_id));
            }
        }
    }
}

/// Raven `Cmd_God_f`.
///
/// Sets client to godmode.
///
/// argv(0) god
///
/// Source: `oracle/codemp/game/g_cmds.c:403-418`
pub fn Cmd_God_f(ctx: &mut GameContext, ent: EntityId) {
    if CheatsOk(ctx, ent) == qfalse {
        return;
    }

    ctx.world.entity_mut(ent).flags ^= FL_GODMODE;
    let msg = if ctx.world.entity(ent).flags & FL_GODMODE == 0 {
        "godmode OFF\n"
    } else {
        "godmode ON\n"
    };

    let s = format!("print \"{}\"", msg);
    trap::SendServerCommand(
        ctx.engine,
        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
            ent.index() as c_int,
            cstr(&s),
        ),
    );
}

/// Raven `Cmd_Notarget_f`.
///
/// Sets client to notarget.
///
/// argv(0) notarget
///
/// Source: `oracle/codemp/game/g_cmds.c:430-444`
pub fn Cmd_Notarget_f(ctx: &mut GameContext, ent: EntityId) {
    if CheatsOk(ctx, ent) == qfalse {
        return;
    }

    ctx.world.entity_mut(ent).flags ^= FL_NOTARGET;
    let msg = if ctx.world.entity(ent).flags & FL_NOTARGET == 0 {
        "notarget OFF\n"
    } else {
        "notarget ON\n"
    };

    let s = format!("print \"{}\"", msg);
    trap::SendServerCommand(
        ctx.engine,
        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
            ent.index() as c_int,
            cstr(&s),
        ),
    );
}

/// Raven `Cmd_Noclip_f`.
///
/// argv(0) noclip
///
/// Source: `oracle/codemp/game/g_cmds.c:454-469`
pub fn Cmd_Noclip_f(ctx: &mut GameContext, ent: EntityId) {
    if CheatsOk(ctx, ent) == qfalse {
        return;
    }

    let noclip = ctx.world.client(ent.index()).noclip;
    let msg = if noclip != qfalse {
        "noclip OFF\n"
    } else {
        "noclip ON\n"
    };
    ctx.world.client_mut(ent.index()).noclip = if noclip != qfalse { qfalse } else { qtrue };

    let s = format!("print \"{}\"", msg);
    trap::SendServerCommand(
        ctx.engine,
        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
            ent.index() as c_int,
            cstr(&s),
        ),
    );
}

/// Raven `Cmd_LevelShot_f`.
///
/// This is just to help generate the level pictures for the menus. It goes to
/// the intermission immediately and sends over a command to the client to
/// resize the view, hide the scoreboard, and take a special screenshot.
///
/// Source: `oracle/codemp/game/g_cmds.c:482-496`
pub fn Cmd_LevelShot_f(ctx: &mut GameContext, ent: EntityId) {
    if CheatsOk(ctx, ent) == qfalse {
        return;
    }

    // doesn't work in single player
    if ctx.world.cvars.g_gametype.integer != 0 {
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                ent.index() as c_int,
                cstr("print \"Must be in g_gametype 0 for levelshot\n\""),
            ),
        );
        return;
    }

    crate::g_main::BeginIntermission(ctx);
    trap::SendServerCommand(
        ctx.engine,
        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
            ent.index() as c_int,
            cstr("clientLevelShot"),
        ),
    );
}

/// Raven `Cmd_TeamTask_f`.
///
/// From TA.
///
/// Source: `oracle/codemp/game/g_cmds.c:506-522`
pub fn Cmd_TeamTask_f(ctx: &mut GameContext, ent: EntityId) {
    // Canonical in `mp_qshared::shared::limits` (value 1024).
    // Source: `oracle/codemp/game/q_shared.h:384`
    use mp_qshared::shared::limits::MAX_INFO_STRING;

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let clientNum = ent.index() as c_int;

        if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) != 2 {
            return;
        }
        let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                arg.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );
        let task: c_int = atoi_str(&cstr_to_str(arg.as_ptr()));

        let mut userinfo = [0 as c_char; MAX_INFO_STRING];
        trap::GetUserinfo(
            ctx.engine,
            mp_abi::game::syscalls::G_GET_USERINFO::GGetUserinfoArgs::new(
                clientNum,
                userinfo.as_mut_ptr(),
                MAX_INFO_STRING as c_int,
            ),
        );
        let value = format!("{}", task);
        crate::q_shared::Info_SetValueForKey(
            userinfo.as_mut_ptr(),
            c"teamtask".as_ptr(),
            cstr(&value).as_ptr(),
        );
        // Pass the raw userinfo buffer through unchanged; C hands `trap_SetUserinfo`
        // the buffer directly, so build the CString from the raw bytes rather than
        // a lossy UTF-8 round-trip. Source: `oracle/codemp/game/g_cmds.c:520`
        trap::SetUserinfo(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_USERINFO::GSetUserinfoArgs::new(
                clientNum,
                cstr_from_chars(&userinfo).to_owned(),
            ),
        );
        crate::g_client::ClientUserinfoChanged(ctx, clientNum);
    }
}

/// Raven `G_CheckTKAutoKickBan`.
///
/// Source: `oracle/codemp/game/g_cmds.c:527-573`
pub fn G_CheckTKAutoKickBan(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        if ctx.world.entity(ent).client.is_null()
            || ctx.world.entity(ent).s.number >= MAX_CLIENTS as c_int
        {
            return;
        }

        // Past the guard `ent` is a real client slot (`s.number < MAX_CLIENTS`),
        // so its client index is `ent.index()`.
        let cidx = ent.index();
        let auto_kick = ctx.world.cvars.g_autoKickTKSpammers.integer;
        let auto_ban = ctx.world.cvars.g_autoBanTKSpammers.integer;

        if auto_kick > 0 || auto_ban > 0 {
            ctx.world.client_mut(cidx).sess.TKCount += 1;
            let tkcount = ctx.world.client(cidx).sess.TKCount;
            if auto_ban > 0 && tkcount >= auto_ban {
                // Oracle guards with `if ( ent->client->sess.IPstring )`, but
                // IPstring is a `char[32]` array whose address is never null, so
                // this ban runs unconditionally. Preserve the always-true quirk.
                let ipstr = ctx.world.client_mut(cidx).sess.IPstring.as_mut_ptr();
                crate::g_svcmds::AddIP(ctx, ipstr);

                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                    c"TKBAN".as_ptr() as *mut c_char,
                );
                let s = format!(
                    "print \"{} {}\n\"",
                    cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                    cstr_to_str(m)
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        -1,
                        cstr(&s),
                    ),
                );
                let cc = format!("clientkick {}\n", ent.index());
                trap::SendConsoleCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_CONSOLE_COMMAND::GSendConsoleCommandArgs::new(
                        EXEC_INSERT as c_int,
                        cstr(&cc),
                    ),
                );
                return;
            }
            if auto_kick > 0 && tkcount >= auto_kick {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                    c"TKKICK".as_ptr() as *mut c_char,
                );
                let s = format!(
                    "print \"{} {}\n\"",
                    cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                    cstr_to_str(m)
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        -1,
                        cstr(&s),
                    ),
                );
                let cc = format!("clientkick {}\n", ent.index());
                trap::SendConsoleCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_CONSOLE_COMMAND::GSendConsoleCommandArgs::new(
                        EXEC_INSERT as c_int,
                        cstr(&cc),
                    ),
                );
                return;
            }
            // okay, not gone (yet), but warn them...
            if auto_ban > 0 && (auto_kick <= 0 || auto_ban < auto_kick) {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                    c"WARNINGTKBAN".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s),
                    ),
                );
            } else if auto_kick > 0 {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                    c"WARNINGTKKICK".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s),
                    ),
                );
            }
        }
    }
}

/// Raven `Cmd_Kill_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:583-643`
pub fn Cmd_Kill_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL};

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR {
            return;
        }
        if ctx.world.entity(ent).health <= 0 {
            return;
        }

        let g_gametype = ctx.world.cvars.g_gametype.integer;
        if (g_gametype == GT_DUEL || g_gametype == GT_POWERDUEL)
            && ctx.world.level.numPlayingClients > 1
            && ctx.world.level.warmupTime == 0
        {
            if ctx.world.cvars.g_allowDuelSuicide.integer == 0 {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"ATTEMPTDUELKILL".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s),
                    ),
                );
                return;
            }
        }

        let auto_kick = ctx.world.cvars.g_autoKickKillSpammers.integer;
        let auto_ban = ctx.world.cvars.g_autoBanKillSpammers.integer;
        if auto_kick > 0 || auto_ban > 0 {
            ctx.world.client_mut(cidx).sess.killCount += 1;
            let killcount = ctx.world.client(cidx).sess.killCount;
            if auto_ban > 0 && killcount >= auto_ban {
                // Oracle guards with `if ( ent->client->sess.IPstring )`, but
                // IPstring is a `char[32]` array whose address is never null, so
                // this ban runs unconditionally. Preserve the always-true quirk.
                let ipstr = ctx.world.client_mut(cidx).sess.IPstring.as_mut_ptr();
                crate::g_svcmds::AddIP(ctx, ipstr);
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                    c"SUICIDEBAN".as_ptr() as *mut c_char,
                );
                let s = format!(
                    "print \"{} {}\n\"",
                    cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                    cstr_to_str(m)
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        -1,
                        cstr(&s),
                    ),
                );
                let cc = format!("clientkick {}\n", ent.index());
                trap::SendConsoleCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_CONSOLE_COMMAND::GSendConsoleCommandArgs::new(
                        EXEC_INSERT as c_int,
                        cstr(&cc),
                    ),
                );
                return;
            }
            if auto_kick > 0 && killcount >= auto_kick {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                    c"SUICIDEKICK".as_ptr() as *mut c_char,
                );
                let s = format!(
                    "print \"{} {}\n\"",
                    cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                    cstr_to_str(m)
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        -1,
                        cstr(&s),
                    ),
                );
                let cc = format!("clientkick {}\n", ent.index());
                trap::SendConsoleCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_CONSOLE_COMMAND::GSendConsoleCommandArgs::new(
                        EXEC_INSERT as c_int,
                        cstr(&cc),
                    ),
                );
                return;
            }
            if auto_ban > 0 && (auto_kick <= 0 || auto_ban < auto_kick) {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                    c"WARNINGSUICIDEBAN".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s),
                    ),
                );
            } else if auto_kick > 0 {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME_ADMIN".as_ptr() as *mut c_char,
                    c"WARNINGSUICIDEKICK".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s),
                    ),
                );
            }
        }
        ctx.world.entity_mut(ent).flags &= !FL_GODMODE;
        ctx.world.client_mut(cidx).ps.stats[STAT_HEALTH as usize] = -999;
        ctx.world.entity_mut(ent).health = -999;
        crate::g_combat::player_die(ctx, ent, Some(ent), Some(ent), 100000, MOD_SUICIDE as c_int);
    }
}

/// Raven `G_GetDuelWinner`.
///
/// Source: `oracle/codemp/game/g_cmds.c:645-661`
pub fn G_GetDuelWinner(ctx: &mut GameContext, ent: EntityId) -> *mut gentity_t {
    // Raven's `client` param is a `level.clients` slot; the EntityId port carries
    // the owning entity, whose client index is `ent.index()`.
    let cidx = ent.index();
    for i in 0..ctx.world.level.maxclients {
        // Faithful to Raven's `if (wCl && wCl != client && …)` — `wCl != client`
        // is the slot-identity test (a different client than `ent`'s).
        if i as usize != cidx
            && ctx.world.client(i as usize).pers.connected == CON_CONNECTED
            && ctx.world.client(i as usize).sess.sessionTeam != TEAM_SPECTATOR
        {
            let cn = ctx.world.client(i as usize).ps.clientNum;
            return &mut ctx.world.g_entities[cn as usize] as *mut gentity_t;
        }
    }

    core::ptr::null_mut()
}

/// Raven `BroadcastTeamChange`.
///
/// Let everyone know about a team change.
///
/// Source: `oracle/codemp/game/g_cmds.c:670-718`
pub fn BroadcastTeamChange(ctx: &mut GameContext, ent: EntityId, oldTeam: c_int) {
    use mp_bg::public::gametype::GT_SIEGE;

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();
        ctx.world.client_mut(cidx).ps.fd.forceDoInit = 1;

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            return;
        }

        if ctx.world.client(cidx).sess.sessionTeam == TEAM_RED {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"JOINEDTHEREDTEAM".as_ptr() as *mut c_char,
            );
            let s = format!(
                "cp \"{}{}{} {}\n\"",
                cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                "^7",
                "",
                cstr_to_str(m)
            );
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    -1,
                    cstr(&s),
                ),
            );
        } else if ctx.world.client(cidx).sess.sessionTeam == TEAM_BLUE {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"JOINEDTHEBLUETEAM".as_ptr() as *mut c_char,
            );
            let s = format!(
                "cp \"{}{} {}\n\"",
                cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                "^7",
                cstr_to_str(m)
            );
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    -1,
                    cstr(&s),
                ),
            );
        } else if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR
            && oldTeam != TEAM_SPECTATOR
        {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"JOINEDTHESPECTATORS".as_ptr() as *mut c_char,
            );
            let s = format!(
                "cp \"{}{} {}\n\"",
                cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                "^7",
                cstr_to_str(m)
            );
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    -1,
                    cstr(&s),
                ),
            );
        } else if ctx.world.client(cidx).sess.sessionTeam == TEAM_FREE {
            use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL};
            if ctx.world.cvars.g_gametype.integer == GT_DUEL
                || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
            {
                // NOTE: Just doing a vs. once it counts two players up — Raven leaves
                // this branch as commented-out dead code (a currentWinner vs. print).
            } else {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"JOINEDTHEBATTLE".as_ptr() as *mut c_char,
                );
                let s = format!(
                    "cp \"{}{} {}\n\"",
                    cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                    "^7",
                    cstr_to_str(m)
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        -1,
                        cstr(&s),
                    ),
                );
            }
        }

        let idx = ent.index() as c_int;
        let team = ctx.world.client(cidx).sess.sessionTeam;
        let msg = format!(
            "setteam:  {} {} {}\n",
            idx,
            cstr_to_str(crate::g_team::TeamName(oldTeam)),
            cstr_to_str(crate::g_team::TeamName(team)),
        );
        crate::g_main::G_LogPrintf(ctx, cstr(&msg).as_ptr());
    }
}

/// Raven `G_PowerDuelCheckFail`.
///
/// Source: `oracle/codemp/game/g_cmds.c:720-743`
pub fn G_PowerDuelCheckFail(ctx: &mut GameContext, ent: EntityId) -> qboolean {
    // Raven `duelTeam_t` (`bg_public.h:1019-1025`); `gclient_t::sess.duelTeam` is
    // stored as plain `c_int`, so the enum discriminants are transcribed as consts.
    const DUELTEAM_FREE: c_int = 0;
    const DUELTEAM_LONE: c_int = 1;
    const DUELTEAM_DOUBLE: c_int = 2;

    // `ent` is the commanding player, so its client slot is `ent.index()`.
    let cidx = ent.index();
    if ctx.world.entity(ent).client.is_null()
        || ctx.world.client(cidx).sess.duelTeam == DUELTEAM_FREE
    {
        return qtrue;
    }

    let mut loners: c_int = 0;
    let mut doubles: c_int = 0;
    crate::g_main::G_PowerDuelCount(ctx, &mut loners, &mut doubles, qfalse);

    if ctx.world.client(cidx).sess.duelTeam == DUELTEAM_LONE && loners >= 1 {
        return qtrue;
    }

    if ctx.world.client(cidx).sess.duelTeam == DUELTEAM_DOUBLE && doubles >= 2 {
        return qtrue;
    }

    qfalse
}

/// Raven `SetTeam`.
///
/// Source: `oracle/codemp/game/g_cmds.c:752-1022`
pub fn SetTeam(ctx: &mut GameContext, ent: EntityId, s: *mut c_char) {
    use crate::client::spectator_state::spectatorState_t::*;
    use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL, GT_SIEGE, GT_TEAM};

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();
        let clientNum = ent.index() as c_int;
        let mut specClient: c_int = 0;
        let mut specState = SPECTATOR_NOT;
        let ss = cstr_to_str(s);
        let mut team: c_int;

        if ss.eq_ignore_ascii_case("scoreboard") || ss.eq_ignore_ascii_case("score") {
            team = TEAM_SPECTATOR;
            specState = SPECTATOR_SCOREBOARD;
        } else if ss.eq_ignore_ascii_case("follow1") {
            team = TEAM_SPECTATOR;
            specState = SPECTATOR_FOLLOW;
            specClient = -1;
        } else if ss.eq_ignore_ascii_case("follow2") {
            team = TEAM_SPECTATOR;
            specState = SPECTATOR_FOLLOW;
            specClient = -2;
        } else if ss.eq_ignore_ascii_case("spectator") || ss.eq_ignore_ascii_case("s") {
            team = TEAM_SPECTATOR;
            specState = SPECTATOR_FREE;
        } else if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
            specState = SPECTATOR_NOT;
            if ss.eq_ignore_ascii_case("red") || ss.eq_ignore_ascii_case("r") {
                team = TEAM_RED;
            } else if ss.eq_ignore_ascii_case("blue") || ss.eq_ignore_ascii_case("b") {
                team = TEAM_BLUE;
            } else {
                team = crate::g_client::PickTeam(ctx, clientNum) as c_int;
            }

            if ctx.world.cvars.g_teamForceBalance.integer != 0
                && ctx.world.cvars.g_trueJedi.integer == 0
            {
                let ps_clientnum = ctx.world.client(cidx).ps.clientNum;
                let mut counts = [0 as c_int; TEAM_NUM_TEAMS as usize];
                counts[TEAM_BLUE as usize] =
                    crate::g_client::TeamCount(ctx, ps_clientnum, TEAM_BLUE) as c_int;
                counts[TEAM_RED as usize] =
                    crate::g_client::TeamCount(ctx, ps_clientnum, TEAM_RED) as c_int;

                if team == TEAM_RED && counts[TEAM_RED as usize] - counts[TEAM_BLUE as usize] > 1 {
                    let m = crate::g_main::G_GetStringEdString(
                        ctx,
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"TOOMANYRED".as_ptr() as *mut c_char,
                    );
                    let msg = format!("print \"{}\n\"", cstr_to_str(m));
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                            ps_clientnum,
                            cstr(&msg),
                        ),
                    );
                    return; // ignore the request
                }
                if team == TEAM_BLUE && counts[TEAM_BLUE as usize] - counts[TEAM_RED as usize] > 1 {
                    let m = crate::g_main::G_GetStringEdString(
                        ctx,
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"TOOMANYBLUE".as_ptr() as *mut c_char,
                    );
                    let msg = format!("print \"{}\n\"", cstr_to_str(m));
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                            ps_clientnum,
                            cstr(&msg),
                        ),
                    );
                    return; // ignore the request
                }
            }
        } else {
            team = TEAM_FREE;
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            if ctx.world.client(cidx).tempSpectate >= ctx.world.level.time && team == TEAM_SPECTATOR
            {
                // sorry, can't do that.
                return;
            }

            ctx.world.client_mut(cidx).sess.siegeDesiredTeam = team;

            if ctx.world.client(cidx).sess.sessionTeam != TEAM_SPECTATOR && team != TEAM_SPECTATOR {
                let doBegin = ctx.world.client(cidx).tempSpectate < ctx.world.level.time;

                if doBegin {
                    if ctx.world.entity(ent).health > 0 {
                        ctx.world.entity_mut(ent).flags &= !FL_GODMODE;
                        ctx.world.client_mut(cidx).ps.stats[STAT_HEALTH as usize] = 0;
                        ctx.world.entity_mut(ent).health = 0;
                        crate::g_combat::player_die(
                            ctx,
                            ent,
                            Some(ent),
                            Some(ent),
                            100000,
                            MOD_TEAM_CHANGE as c_int,
                        );
                    }
                }

                if ctx.world.client(cidx).sess.sessionTeam
                    != ctx.world.client(cidx).sess.siegeDesiredTeam
                {
                    let sdt = ctx.world.client(cidx).sess.siegeDesiredTeam;
                    crate::g_saga::SetTeamQuick(ctx, ent, sdt, qfalse);
                }

                return;
            }
        }

        // override decision if limiting the players
        if ctx.world.cvars.g_gametype.integer == GT_DUEL
            && ctx.world.level.numNonSpectatorClients >= 2
        {
            team = TEAM_SPECTATOR;
        } else if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
            && (ctx.world.level.numPlayingClients >= 3 || G_PowerDuelCheckFail(ctx, ent) != qfalse)
        {
            team = TEAM_SPECTATOR;
        } else if ctx.world.cvars.g_maxGameClients.integer > 0
            && ctx.world.level.numNonSpectatorClients >= ctx.world.cvars.g_maxGameClients.integer
        {
            team = TEAM_SPECTATOR;
        }

        let oldTeam = ctx.world.client(cidx).sess.sessionTeam;
        if team == oldTeam && team != TEAM_SPECTATOR {
            return;
        }

        // if the player was dead leave the body
        if ctx.world.client(cidx).ps.stats[STAT_HEALTH as usize] <= 0
            && ctx.world.client(cidx).sess.sessionTeam != TEAM_SPECTATOR
        {
            crate::g_client::MaintainBodyQueue(ctx, ent);
        }

        // he starts at 'base'
        ctx.world.client_mut(cidx).pers.teamState.state = playerTeamStateState_t::TEAM_BEGIN;
        if oldTeam != TEAM_SPECTATOR {
            ctx.world.entity_mut(ent).flags &= !FL_GODMODE;
            ctx.world.client_mut(cidx).ps.stats[STAT_HEALTH as usize] = 0;
            ctx.world.entity_mut(ent).health = 0;
            ctx.world.globals.g_dontPenalizeTeam = qtrue;
            crate::g_combat::player_die(
                ctx,
                ent,
                Some(ent),
                Some(ent),
                100000,
                MOD_SUICIDE as c_int,
            );
            ctx.world.globals.g_dontPenalizeTeam = qfalse;
        }
        if team == TEAM_SPECTATOR {
            if ctx.world.cvars.g_gametype.integer != GT_DUEL || oldTeam != TEAM_SPECTATOR {
                ctx.world.client_mut(cidx).sess.spectatorTime = ctx.world.level.time;
            }
        }

        ctx.world.client_mut(cidx).sess.sessionTeam = team;
        ctx.world.client_mut(cidx).sess.spectatorState = specState;
        ctx.world.client_mut(cidx).sess.spectatorClient = specClient;

        ctx.world.client_mut(cidx).sess.teamLeader = qfalse;
        if team == TEAM_RED || team == TEAM_BLUE {
            let teamLeader = crate::g_client::TeamLeader(ctx, team);
            if teamLeader == -1
                || (ctx.world.g_entities[clientNum as usize].r.svFlags & SVF_BOT == 0
                    && ctx.world.g_entities[teamLeader as usize].r.svFlags & SVF_BOT != 0)
            {
                //SetLeader( team, clientNum );
            }
        }
        if oldTeam == TEAM_RED || oldTeam == TEAM_BLUE {
            crate::g_main::CheckTeamLeader(ctx, oldTeam);
        }

        BroadcastTeamChange(ctx, ent, oldTeam);

        if oldTeam != TEAM_SPECTATOR {
            let origin = ctx.world.client(cidx).ps.origin;
            let tent = crate::g_utils::G_TempEntity(ctx, origin, EV_PLAYER_TELEPORT_OUT as c_int);
            let tent_id = ctx.entity_id_of(tent).unwrap();
            ctx.world.entity_mut(tent_id).s.clientNum = clientNum;
        }

        crate::g_client::ClientUserinfoChanged(ctx, clientNum);

        if ctx.world.globals.g_preventTeamBegin == qfalse {
            crate::g_client::ClientBegin(ctx, clientNum, qfalse);
        }
    }
}

/// Raven `StopFollowing`.
///
/// If the client being followed leaves the game, or you just want to drop
/// to free floating spectator mode.
///
/// Source: `oracle/codemp/game/g_cmds.c:1032-1051`
pub fn StopFollowing(ctx: &mut GameContext, ent: EntityId) {
    use crate::client::spectator_state::spectatorState_t::SPECTATOR_FREE;

    // `ent` is the commanding player, so its client slot is `ent.index()`.
    let cidx = ent.index();
    ctx.world.client_mut(cidx).ps.persistant[PERS_TEAM as usize] = TEAM_SPECTATOR;
    ctx.world.client_mut(cidx).sess.sessionTeam = TEAM_SPECTATOR;
    ctx.world.client_mut(cidx).sess.spectatorState = SPECTATOR_FREE;
    ctx.world.client_mut(cidx).ps.pm_flags &= !PMF_FOLLOW;
    ctx.world.entity_mut(ent).r.svFlags &= !SVF_BOT;
    ctx.world.client_mut(cidx).ps.clientNum = ent.index() as c_int;
    ctx.world.client_mut(cidx).ps.weapon = WP_NONE;
    ctx.world.client_mut(cidx).ps.m_iVehicleNum = 0;
    ctx.world.client_mut(cidx).ps.viewangles[ROLL as usize] = 0.0f32;
    ctx.world.client_mut(cidx).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
    ctx.world.client_mut(cidx).ps.forceHandExtendTime = 0;
    ctx.world.client_mut(cidx).ps.zoomMode = 0;
    ctx.world.client_mut(cidx).ps.zoomLocked = 0;
    ctx.world.client_mut(cidx).ps.zoomLockTime = 0;
    ctx.world.client_mut(cidx).ps.legsAnim = 0;
    ctx.world.client_mut(cidx).ps.legsTimer = 0;
    ctx.world.client_mut(cidx).ps.torsoAnim = 0;
    ctx.world.client_mut(cidx).ps.torsoTimer = 0;
}

/// Raven `Cmd_Team_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1058-1112`
pub fn Cmd_Team_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL};

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) != 2 {
            let oldTeam = ctx.world.client(cidx).sess.sessionTeam;
            let key = match oldTeam {
                TEAM_BLUE => Some("PRINTBLUETEAM"),
                TEAM_RED => Some("PRINTREDTEAM"),
                TEAM_FREE => Some("PRINTFREETEAM"),
                TEAM_SPECTATOR => Some("PRINTSPECTEAM"),
                _ => None,
            };
            if let Some(k) = key {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    cstr(k).as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s),
                    ),
                );
            }
            return;
        }

        if ctx.world.client(cidx).switchTeamTime > ctx.world.level.time {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOSWITCH".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        if ctx.world.globals.gEscaping != qfalse {
            return;
        }

        if ctx.world.cvars.g_gametype.integer == GT_DUEL
            && ctx.world.client(cidx).sess.sessionTeam == TEAM_FREE
        {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"Cannot switch teams in Duel\n\""),
                ),
            );
            return;
        }

        if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"Cannot switch teams in Power Duel\n\""),
                ),
            );
            return;
        }

        let mut s = [0 as c_char; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                s.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );

        SetTeam(ctx, ent, s.as_mut_ptr());

        ctx.world.client_mut(cidx).switchTeamTime = ctx.world.level.time + 5000;
    }
}

/// Raven `Cmd_DuelTeam_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1119-1204`
pub fn Cmd_DuelTeam_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_POWERDUEL;

    const DUELTEAM_FREE: c_int = 0;
    const DUELTEAM_LONE: c_int = 1;
    const DUELTEAM_DOUBLE: c_int = 2;

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if ctx.world.cvars.g_gametype.integer != GT_POWERDUEL {
            return;
        }

        if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) != 2 {
            let oldTeam = ctx.world.client(cidx).sess.duelTeam;
            let msg = match oldTeam {
                DUELTEAM_FREE => Some("print \"None\n\""),
                DUELTEAM_LONE => Some("print \"Single\n\""),
                DUELTEAM_DOUBLE => Some("print \"Double\n\""),
                _ => None,
            };
            if let Some(m) = msg {
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(m),
                    ),
                );
            }
            return;
        }

        if ctx.world.client(cidx).switchDuelTeamTime > ctx.world.level.time {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOSWITCH".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        let mut s = [0 as c_char; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                s.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );
        let ss = cstr_to_str(s.as_ptr());

        let oldTeam = ctx.world.client(cidx).sess.duelTeam;

        if ss.eq_ignore_ascii_case("free") {
            ctx.world.client_mut(cidx).sess.duelTeam = DUELTEAM_FREE;
        } else if ss.eq_ignore_ascii_case("single") {
            ctx.world.client_mut(cidx).sess.duelTeam = DUELTEAM_LONE;
        } else if ss.eq_ignore_ascii_case("double") {
            ctx.world.client_mut(cidx).sess.duelTeam = DUELTEAM_DOUBLE;
        } else {
            let msg = format!("print \"'{}' not a valid duel team.\n\"", ss);
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&msg),
                ),
            );
        }

        if oldTeam == ctx.world.client(cidx).sess.duelTeam {
            return;
        }

        if ctx.world.client(cidx).sess.sessionTeam != TEAM_SPECTATOR {
            let curTeam = ctx.world.client(cidx).sess.duelTeam;
            ctx.world.client_mut(cidx).sess.duelTeam = oldTeam;
            let origin = ctx.world.client(cidx).ps.origin;
            crate::g_combat::G_Damage(
                ctx,
                Some(ent),
                Some(ent),
                Some(ent),
                None,
                origin,
                99999,
                DAMAGE_NO_PROTECTION,
                MOD_SUICIDE as c_int,
            );
            ctx.world.client_mut(cidx).sess.duelTeam = curTeam;
        }
        ctx.world.client_mut(cidx).sess.wins = 0;
        ctx.world.client_mut(cidx).sess.losses = 0;

        crate::g_client::ClientUserinfoChanged(ctx, ent.index() as c_int);

        ctx.world.client_mut(cidx).switchDuelTeamTime = ctx.world.level.time + 5000;
    }
}

/// Raven `G_TeamForSiegeClass`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1206-1244`
pub fn G_TeamForSiegeClass(ctx: &mut GameContext, clName: *const c_char) -> c_int {
    // Siege team ids (distinct from `team_t` RED/BLUE) and the 128-class cap,
    // canonical in `mp_bg::saga`. The former local `MAX_SIEGE_CLASSES` was
    // wrongly 12 (oracle is 128).
    // Source: `oracle/codemp/game/bg_saga.h:3-4,12`
    use mp_bg::saga::siege_class_t::MAX_SIEGE_CLASSES;
    use mp_bg::saga::siege_team_t::{SIEGETEAM_TEAM1, SIEGETEAM_TEAM2};

    unsafe {
        let bg = &ctx.world.bg_state;
        let mut team = SIEGETEAM_TEAM1;
        let mut i: c_int = 0;
        let mut stm = crate::bg_saga::BG_SiegeFindThemeForTeam(team, bg);

        if stm.is_null() {
            return 0;
        }

        while team <= SIEGETEAM_TEAM2 {
            let scl = (*stm).classes[i as usize];

            if !scl.is_null() && (*scl).name[0] != 0 {
                if crate::q_shared::Q_stricmp(clName, (*scl).name.as_ptr()) == 0 {
                    return team;
                }
            }

            i += 1;
            if i >= MAX_SIEGE_CLASSES as c_int || i >= (*stm).numClasses {
                if team == SIEGETEAM_TEAM2 {
                    break;
                }
                team = SIEGETEAM_TEAM2;
                stm = crate::bg_saga::BG_SiegeFindThemeForTeam(team, bg);
                i = 0;
            }
        }

        0
    }
}

/// Raven `Cmd_SiegeClass_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1251-1348`
pub fn Cmd_SiegeClass_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_SIEGE;

    unsafe {
        let mut className = [0 as c_char; 64];
        let mut startedAsSpec = qfalse;

        if ctx.world.cvars.g_gametype.integer != GT_SIEGE {
            return;
        }

        if ctx.world.entity(ent).client.is_null() {
            return;
        }

        if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) < 1 {
            return;
        }

        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if ctx.world.client(cidx).switchClassTime > ctx.world.level.time {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOCLASSSWITCH".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR {
            startedAsSpec = qtrue;
        }

        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(1, className.as_mut_ptr(), 64),
        );

        let team = G_TeamForSiegeClass(ctx, className.as_ptr());

        if team == 0 {
            return;
        }

        if ctx.world.client(cidx).sess.sessionTeam != team {
            ctx.world.globals.g_preventTeamBegin = qtrue;
            if team == TEAM_RED {
                SetTeam(ctx, ent, c"red".as_ptr() as *mut c_char);
            } else if team == TEAM_BLUE {
                SetTeam(ctx, ent, c"blue".as_ptr() as *mut c_char);
            }
            ctx.world.globals.g_preventTeamBegin = qfalse;

            if ctx.world.client(cidx).sess.sessionTeam != team {
                if ctx.world.client(cidx).sess.sessionTeam != TEAM_SPECTATOR
                    || ctx.world.client(cidx).sess.siegeDesiredTeam != team
                {
                    let m = crate::g_main::G_GetStringEdString(
                        ctx,
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"NOCLASSTEAM".as_ptr() as *mut c_char,
                    );
                    let s = format!("print \"{}\n\"", cstr_to_str(m));
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                            ent.index() as c_int,
                            cstr(&s),
                        ),
                    );
                    return;
                }
            }
        }

        let preScore = ctx.world.client(cidx).ps.persistant[PERS_SCORE as usize];

        crate::bg_saga::BG_SiegeCheckClassLegality(
            team,
            className.as_mut_ptr(),
            &mut ctx.world.bg_state,
        );

        let cn = cstr_to_str(className.as_ptr());
        write_cstr_field(&mut ctx.world.client_mut(cidx).sess.siegeClass, &cn);

        crate::g_client::ClientUserinfoChanged(ctx, ent.index() as c_int);

        if ctx.world.client(cidx).tempSpectate < ctx.world.level.time {
            if ctx.world.entity(ent).health > 0 && startedAsSpec == qfalse {
                ctx.world.entity_mut(ent).flags &= !FL_GODMODE;
                ctx.world.client_mut(cidx).ps.stats[STAT_HEALTH as usize] = 0;
                ctx.world.entity_mut(ent).health = 0;
                crate::g_combat::player_die(
                    ctx,
                    ent,
                    Some(ent),
                    Some(ent),
                    100000,
                    MOD_SUICIDE as c_int,
                );
            }

            if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR || startedAsSpec != qfalse
            {
                crate::g_client::ClientBegin(ctx, ent.index() as c_int, qfalse);
            }
        }
        ctx.world.client_mut(cidx).ps.persistant[PERS_SCORE as usize] = preScore;

        ctx.world.client_mut(cidx).switchClassTime = ctx.world.level.time + 5000;
    }
}

/// Raven `Cmd_ForceChanged_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1355-1392`
pub fn Cmd_ForceChanged_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL};

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        // Raven's `goto argCheck` is preserved here as the natural fall-through of
        // the if/else below (both arms reach the same trailing logic) — see §C10.
        if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR {
            crate::w_force::WP_InitForcePowers(ctx, Some(ent));
        } else {
            let buf = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"FORCEPOWERCHANGED".as_ptr() as *mut c_char,
            );
            let fpChStr = cstr_to_str(buf);
            let s = format!(
                "print \"{}{}\n\n\"",
                S_COLOR_GREEN.to_string_lossy(),
                fpChStr
            );
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );

            ctx.world.client_mut(cidx).ps.fd.forceDoInit = 1;
        }

        if ctx.world.cvars.g_gametype.integer == GT_DUEL
            || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
        {
            return;
        }

        if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
            let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    1,
                    arg.as_mut_ptr(),
                    MAX_TOKEN_CHARS as c_int,
                ),
            );

            if arg[0] != 0 {
                Cmd_Team_f(ctx, ent);
            }
        }
    }
}

/// Raven `G_SetSaber`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1396-1455`
pub fn G_SetSaber(
    ctx: &mut GameContext,
    ent: EntityId,
    saberNum: c_int,
    saberName: *mut c_char,
    siegeOverride: qboolean,
) -> qboolean {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    use mp_bg::public::gametype::GT_SIEGE;

    unsafe {
        let client = (*ent).client;
        let mut truncSaberName = [0 as c_char; 64];
        let mut i: usize = 0;

        let bgSiegeClasses = &ctx.world.bg_state.bgSiegeClasses;

        if siegeOverride == qfalse
            && ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (*client).siegeClass != -1
            && (bgSiegeClasses[(*client).siegeClass as usize].saberStance != 0
                || bgSiegeClasses[(*client).siegeClass as usize].saber1[0] != 0
                || bgSiegeClasses[(*client).siegeClass as usize].saber2[0] != 0)
        {
            return qfalse;
        }

        while *saberName.add(i) != 0 && i < 64 - 1 {
            truncSaberName[i] = *saberName.add(i);
            i += 1;
        }
        truncSaberName[i] = 0;

        if saberNum == 0
            && (crate::q_shared::Q_stricmp(c"none".as_ptr(), truncSaberName.as_ptr()) == 0
                || crate::q_shared::Q_stricmp(c"remove".as_ptr(), truncSaberName.as_ptr()) == 0)
        {
            write_cstr_field(&mut truncSaberName, "Kyle");
        }

        let mut callbacks = crate::bg_channel::GameCallbacksImpl {
            // SEAM-BG-REENTRY (DEC-28, sanctioned) — GameCallbacksImpl.world is a `*mut GameWorld`
            // field aliasing bg_state; a raw store is required (bg-seam re-entry).
            world: ctx.world_raw(),
            engine: ctx.engine,
        };
        crate::bg_saberLoad::WP_SetSaber(
            (*ent).s.number,
            (*client).saber.as_mut_ptr(),
            saberNum,
            truncSaberName.as_ptr(),
            &mut ctx.world.bg_state,
            &crate::bg_channel::GameBgTraps::new(ctx.engine),
            &mut callbacks,
        );

        if (*client).saber[0].model[0] == 0 {
            debug_assert!(false, "should never happen!"); // Raven `assert(0)`
            write_cstr_field(&mut (*client).sess.saberType, "none");
        } else {
            write_cstr_field(
                &mut (*client).sess.saberType,
                &cstr_to_str((*client).saber[0].name.as_ptr()),
            );
        }

        if (*client).saber[1].model[0] == 0 {
            write_cstr_field(&mut (*client).sess.saber2Type, "none");
        } else {
            write_cstr_field(
                &mut (*client).sess.saber2Type,
                &cstr_to_str((*client).saber[1].name.as_ptr()),
            );
        }

        if crate::bg_saberLoad::WP_SaberStyleValidForSaber(
            &mut (*client).saber[0],
            &mut (*client).saber[1],
            (*client).ps.saberHolstered,
            (*client).ps.fd.saberAnimLevel,
        ) == qfalse
        {
            crate::bg_saberLoad::WP_UseFirstValidSaberStyle(
                &mut (*client).saber[0],
                &mut (*client).saber[1],
                (*client).ps.saberHolstered,
                &mut (*client).ps.fd.saberAnimLevel,
            );
            (*client).ps.fd.saberAnimLevelBase = (*client).ps.fd.saberAnimLevel;
            (*client).saberCycleQueue = (*client).ps.fd.saberAnimLevel;
        }

        qtrue
    }
}

/// Raven `Cmd_Follow_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1462-1503`
pub fn Cmd_Follow_f(ctx: &mut GameContext, ent: EntityId) {
    use crate::client::spectator_state::spectatorState_t::*;
    use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL};

    // `ent` is the commanding player, so its client slot is `ent.index()`.
    let cidx = ent.index();

    if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) != 2 {
        if ctx.world.client(cidx).sess.spectatorState == SPECTATOR_FOLLOW {
            StopFollowing(ctx, ent);
        }
        return;
    }

    let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
    trap::Argv(
        ctx.engine,
        mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
            1,
            arg.as_mut_ptr(),
            MAX_TOKEN_CHARS as c_int,
        ),
    );
    let i = ClientNumberFromString(ctx, ent, arg.as_mut_ptr());
    if i == -1 {
        return;
    }

    // can't follow self — Raven's `clients+i == client` is a slot-identity test.
    if i as usize == cidx {
        return;
    }

    // can't follow another spectator
    if ctx.world.client(i as usize).sess.sessionTeam == TEAM_SPECTATOR {
        return;
    }

    // if they are playing a tournement game, count as a loss
    if (ctx.world.cvars.g_gametype.integer == GT_DUEL
        || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL)
        && ctx.world.client(cidx).sess.sessionTeam == TEAM_FREE
    {
        //WTF???
        ctx.world.client_mut(cidx).sess.losses += 1;
    }

    // first set them to spectator
    if ctx.world.client(cidx).sess.sessionTeam != TEAM_SPECTATOR {
        SetTeam(ctx, ent, c"spectator".as_ptr() as *mut c_char);
    }

    ctx.world.client_mut(cidx).sess.spectatorState = SPECTATOR_FOLLOW;
    ctx.world.client_mut(cidx).sess.spectatorClient = i;
}

/// Raven `Cmd_FollowCycle_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1510-1557`
pub fn Cmd_FollowCycle_f(ctx: &mut GameContext, ent: EntityId, dir: c_int) {
    use crate::client::spectator_state::spectatorState_t::*;
    use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL};

    // `ent` is the commanding player, so its client slot is `ent.index()`.
    let cidx = ent.index();

    if (ctx.world.cvars.g_gametype.integer == GT_DUEL
        || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL)
        && ctx.world.client(cidx).sess.sessionTeam == TEAM_FREE
    {
        ctx.world.client_mut(cidx).sess.losses += 1;
    }
    if ctx.world.client(cidx).sess.spectatorState == SPECTATOR_NOT {
        SetTeam(ctx, ent, c"spectator".as_ptr() as *mut c_char);
    }

    if dir != 1 && dir != -1 {
        // Raven calls G_Error (aborts the game) here; ported as a panic.
        panic!("Cmd_FollowCycle_f: bad dir {}", dir);
    }

    let mut clientnum = ctx.world.client(cidx).sess.spectatorClient;
    let original = clientnum;
    loop {
        clientnum += dir;
        if clientnum >= ctx.world.level.maxclients {
            clientnum = 0;
        }
        if clientnum < 0 {
            clientnum = ctx.world.level.maxclients - 1;
        }

        if ctx.world.client(clientnum as usize).pers.connected != CON_CONNECTED {
            if clientnum == original {
                break;
            }
            continue;
        }

        if ctx.world.client(clientnum as usize).sess.sessionTeam == TEAM_SPECTATOR {
            if clientnum == original {
                break;
            }
            continue;
        }

        ctx.world.client_mut(cidx).sess.spectatorClient = clientnum;
        ctx.world.client_mut(cidx).sess.spectatorState = SPECTATOR_FOLLOW;
        return;
    }
    // leave it where it was
}

/// Raven `G_SayTo`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1566-1614`
pub fn G_SayTo(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    mode: c_int,
    color: c_int,
    name: *const c_char,
    message: *const c_char,
    locMsg: *mut c_char,
) {
    use mp_bg::public::gametype::GT_SIEGE;

    unsafe {
        let other = match other {
            Some(o) => o,
            None => return,
        };
        if ctx.world.entity(other).inuse == qfalse {
            return;
        }
        if ctx.world.entity(other).client.is_null() {
            return;
        }
        // `other` is the recipient client entity, so its client slot is
        // `other.index()`; `ent` is the sender player (`ent.index()`).
        let oidx = other.index();
        let cidx = ent.index();
        if ctx.world.client(oidx).pers.connected != CON_CONNECTED {
            return;
        }
        if mode == SAY_TEAM && crate::g_team::OnSameTeam(ctx, Some(ent), Some(other)) == qfalse {
            return;
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && !ctx.world.entity(ent).client.is_null()
            && (ctx.world.client(cidx).tempSpectate >= ctx.world.level.time
                || ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR)
            && ctx.world.client(oidx).sess.sessionTeam != TEAM_SPECTATOR
            && ctx.world.client(oidx).tempSpectate < ctx.world.level.time
        {
            return;
        }

        let name_str = cstr_to_str(name);
        let message_str = cstr_to_str(message);

        if !locMsg.is_null() {
            let locMsg_str = cstr_to_str(locMsg);
            let cmdname = if mode == SAY_TEAM { "ltchat" } else { "lchat" };
            let colorc = (color as u8) as char;
            let s = format!(
                "{} \"{}\" \"{}\" \"{}\" \"{}\"",
                cmdname, name_str, locMsg_str, colorc, message_str
            );
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    other.index() as c_int,
                    cstr(&s),
                ),
            );
        } else {
            let cmdname = if mode == SAY_TEAM { "tchat" } else { "chat" };
            let s = format!(
                "{} \"{}{}{}{}\"",
                cmdname,
                name_str,
                Q_COLOR_ESCAPE as u8 as char,
                (color as u8) as char,
                message_str
            );
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    other.index() as c_int,
                    cstr(&s),
                ),
            );
        }
    }
}

/// Raven `G_Say`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1618-1687`
pub fn G_Say(
    ctx: &mut GameContext,
    ent: EntityId,
    target: Option<EntityId>,
    mode: c_int,
    chatText: *const c_char,
) {
    use mp_bg::public::gametype::GT_TEAM;

    unsafe {
        // `ent` is the sending player, so its client slot is `ent.index()`.
        let cidx = ent.index();
        let mut mode = mode;

        if ctx.world.cvars.g_gametype.integer < GT_TEAM && mode == SAY_TEAM {
            mode = SAY_ALL;
        }

        let netname = cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr());
        let chat = cstr_to_str(chatText);
        let mut locMsg: Option<String> = None;
        let name: String;
        let color: c_int;

        match mode {
            SAY_TEAM => {
                let logmsg = format!("sayteam: {}: {}\n", netname, chat);
                crate::g_main::G_LogPrintf(ctx, cstr(&logmsg).as_ptr());

                let mut location = [0 as c_char; 64];
                if crate::g_team::Team_GetLocationMsg(ctx, ent, location.as_mut_ptr(), 64) != qfalse
                {
                    // Raven's EC macro is the literal byte 0x19, distinct from
                    // the ^7 color escape (Q_COLOR_ESCAPE + COLOR_WHITE).
                    name = format!("\u{19}({}^7\u{19})\u{19}: ", netname);
                    locMsg = Some(cstr_to_str(location.as_ptr()));
                } else {
                    name = format!("\u{19}({}^7\u{19})\u{19}: ", netname);
                }
                color = COLOR_CYAN;
            }
            SAY_TELL => {
                // Raven's `targetClient = target ? target->client : NULL`, then a
                // same-team check against the sender.
                let same_team = match target {
                    Some(t) => {
                        ctx.world.cvars.g_gametype.integer >= GT_TEAM
                            && !ctx.world.entity(t).client.is_null()
                            && ctx.world.client(t.index()).sess.sessionTeam
                                == ctx.world.client(cidx).sess.sessionTeam
                    }
                    None => false,
                };
                if same_team {
                    let mut location = [0 as c_char; 64];
                    if crate::g_team::Team_GetLocationMsg(ctx, ent, location.as_mut_ptr(), 64)
                        != qfalse
                    {
                        // EC is the literal byte 0x19, distinct from the ^7
                        // color escape (Q_COLOR_ESCAPE + COLOR_WHITE).
                        name = format!("\u{19}[{}^7\u{19}]\u{19}: ", netname);
                        locMsg = Some(cstr_to_str(location.as_ptr()));
                    } else {
                        name = format!("\u{19}[{}^7\u{19}]\u{19}: ", netname);
                    }
                } else {
                    name = format!("\u{19}[{}^7\u{19}]\u{19}: ", netname);
                }
                color = COLOR_MAGENTA;
            }
            _ => {
                // SAY_ALL and default
                let logmsg = format!("say: {}: {}\n", netname, chat);
                crate::g_main::G_LogPrintf(ctx, cstr(&logmsg).as_ptr());
                // Trailing EC is the literal byte 0x19, distinct from the ^7
                // color escape (Q_COLOR_ESCAPE + COLOR_WHITE).
                name = format!("{}^7\u{19}: ", netname);
                color = COLOR_GREEN;
            }
        }

        let mut text = [0 as c_char; MAX_SAY_TEXT];
        write_cstr_field(&mut text, &chat);
        let text_str = cstr_to_str(text.as_ptr());

        if target.is_some() {
            let lm = locMsg.map(|s| {
                let mut b: Vec<c_char> = s.bytes().map(|c| c as c_char).collect();
                b.push(0);
                b
            });
            let mut lm_buf = lm;
            let lm_ptr = lm_buf
                .as_mut()
                .map(|v| v.as_mut_ptr())
                .unwrap_or(std::ptr::null_mut());
            G_SayTo(
                ctx,
                ent,
                target,
                mode,
                color,
                cstr(&name).as_ptr(),
                cstr(&text_str).as_ptr(),
                lm_ptr,
            );
            return;
        }

        // echo the text to the console
        if ctx.world.cvars.g_dedicated.integer != 0 {
            let msg = format!("{}{}\n", name, text_str);
            crate::g_main::G_Printf(ctx, cstr(&msg).as_ptr());
        }

        // send it to all the apropriate clients
        for j in 0..ctx.world.level.maxclients {
            let other = EntityId(j as u32);
            let lm = locMsg.clone().map(|s| {
                let mut b: Vec<c_char> = s.bytes().map(|c| c as c_char).collect();
                b.push(0);
                b
            });
            let mut lm_buf = lm;
            let lm_ptr = lm_buf
                .as_mut()
                .map(|v| v.as_mut_ptr())
                .unwrap_or(std::ptr::null_mut());
            G_SayTo(
                ctx,
                ent,
                Some(other),
                mode,
                color,
                cstr(&name).as_ptr(),
                cstr(&text_str).as_ptr(),
                lm_ptr,
            );
        }
    }
}

/// Raven `Cmd_Say_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1695-1712`
pub fn Cmd_Say_f(ctx: &mut GameContext, ent: EntityId, mode: c_int, arg0: qboolean) {
    if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) < 2
        && arg0 == qfalse
    {
        return;
    }

    let p = if arg0 != qfalse {
        ConcatArgs(ctx, 0)
    } else {
        ConcatArgs(ctx, 1)
    };

    G_Say(ctx, ent, None, mode, p as *const c_char);
}

/// Raven `Cmd_Tell_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1719-1749`
pub fn Cmd_Tell_f(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) < 2 {
            return;
        }

        let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                arg.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );
        let targetNum: c_int = atoi_str(&cstr_to_str(arg.as_ptr()));
        if targetNum < 0 || targetNum >= ctx.world.level.maxclients {
            return;
        }

        // `targetNum` is a `[0, maxclients)` slot, so it is a real client entity.
        let target = EntityId(targetNum as u32);
        if ctx.world.entity(target).inuse == qfalse || ctx.world.entity(target).client.is_null() {
            return;
        }

        let p = ConcatArgs(ctx, 2);

        let netname_ent = cstr_to_str(ctx.world.client(ent.index()).pers.netname.as_ptr());
        let netname_target =
            cstr_to_str(ctx.world.client(targetNum as usize).pers.netname.as_ptr());
        let logmsg = format!(
            "tell: {} to {}: {}\n",
            netname_ent,
            netname_target,
            cstr_to_str(p)
        );
        crate::g_main::G_LogPrintf(ctx, cstr(&logmsg).as_ptr());
        G_Say(ctx, ent, Some(target), SAY_TELL, p as *const c_char);
        // don't tell to the player self if it was already directed to this player
        // also don't send the chat back to a bot
        if ent != target && ctx.world.entity(ent).r.svFlags & SVF_BOT == 0 {
            G_Say(ctx, ent, Some(ent), SAY_TELL, p as *const c_char);
        }
    }
}

/// Raven `Cmd_VoiceCommand_f`.
///
/// Siege voice command.
///
/// Source: `oracle/codemp/game/g_cmds.c:1752-1809`
pub fn Cmd_VoiceCommand_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_TEAM;

    // Oracle value is 30 (was wrongly 32, which could index past the 30-entry
    // `bg_customSiegeSoundNames`). No legal workspace canonical (the `mp_cgame`
    // copy is off-limits; `mp_bg` has only the array, not a const). Consolidation
    // candidate for `mp_bg`.
    // Source: `oracle/codemp/game/bg_public.h:140`
    const MAX_CUSTOM_SIEGE_SOUNDS: usize = 30;

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if ctx.world.cvars.g_gametype.integer < GT_TEAM {
            return;
        }

        if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) < 2 {
            return;
        }

        if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR
            || ctx.world.client(cidx).tempSpectate >= ctx.world.level.time
        {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOVOICECHATASSPEC".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        let mut arg = [0 as c_char; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                arg.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );

        if arg[0] as u8 == b'*' {
            return;
        }

        let s = format!("*{}", cstr_to_str(arg.as_ptr()));

        let mut i: usize = 0;
        let names = &mp_bg::local::bg_customSiegeSoundNames;
        while i < MAX_CUSTOM_SIEGE_SOUNDS {
            if names[i].is_none() {
                break;
            }
            if crate::q_shared::Q_stricmp(names[i].unwrap().as_ptr(), cstr(&s).as_ptr()) == 0 {
                break;
            }
            i += 1;
        }

        if i == MAX_CUSTOM_SIEGE_SOUNDS || names[i].is_none() {
            return;
        }

        let te = crate::g_utils::G_TempEntity(ctx, [0.0f32, 0.0, 0.0], EV_VOICECMD_SOUND as c_int);
        let te_id = ctx.entity_id_of(te).unwrap();
        ctx.world.entity_mut(te_id).s.groundEntityNum = ent.index() as c_int;
        ctx.world.entity_mut(te_id).s.eventParm =
            crate::g_utils::G_SoundIndex(names[i].unwrap().as_ptr());
        ctx.world.entity_mut(te_id).r.svFlags |= SVF_BROADCAST;
    }
}

/// Raven `Cmd_GameCommand_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1822-1840`
pub fn Cmd_GameCommand_f(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let mut s = [0 as c_char; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                s.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );
        let player: c_int = atoi_str(&cstr_to_str(s.as_ptr()));
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                2,
                s.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );
        let order: c_int = atoi_str(&cstr_to_str(s.as_ptr()));

        if player < 0 || player >= MAX_CLIENTS as c_int {
            return;
        }
        // C's guard `order > sizeof(gc_orders)/sizeof(char*)` is off by one and
        // lets order == 7 index a 7-element array (UB read). Bound at `>=` so the
        // out-of-range case is a deterministic no-op instead of a panic.
        if order < 0 || order as usize >= gc_orders.len() {
            return;
        }
        // `player` is a `[0, MAX_CLIENTS)` slot → a real client entity.
        let target = EntityId(player as u32);
        G_Say(
            ctx,
            ent,
            Some(target),
            SAY_TELL,
            gc_orders[order as usize].as_ptr(),
        );
        G_Say(
            ctx,
            ent,
            Some(ent),
            SAY_TELL,
            gc_orders[order as usize].as_ptr(),
        );
    }
}

/// Raven `Cmd_Where_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1847-1849`
pub fn Cmd_Where_f(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let origin = ctx.world.entity(ent).s.origin;
        let v = crate::g_utils::vtos(ctx, origin);
        let s = format!("print \"{}\n\"", cstr_to_str(v));
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                ent.index() as c_int,
                cstr(&s),
            ),
        );
    }
}

/// Raven `G_ClientNumberFromName`.
///
/// Finds the client number of the client with the given name.
///
/// Source: `oracle/codemp/game/g_cmds.c:1871-1890`
pub fn G_ClientNumberFromName(ctx: &mut GameContext, name: *const c_char) -> c_int {
    unsafe {
        let mut s2 = [0 as c_char; MAX_STRING_CHARS];
        let mut n2 = [0 as c_char; MAX_STRING_CHARS];

        // check for a name match
        SanitizeString(name as *mut c_char, s2.as_mut_ptr());
        for i in 0..ctx.world.level.numConnectedClients {
            SanitizeString(
                ctx.world.client(i as usize).pers.netname.as_ptr() as *mut c_char,
                n2.as_mut_ptr(),
            );
            if cstr_eq(n2.as_ptr(), s2.as_ptr()) {
                return i;
            }
        }

        -1
    }
}

/// Raven `SanitizeString2`.
///
/// Rich's revised version of `SanitizeString`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1899-1937`
pub fn SanitizeString2(r#in: *mut c_char, out: *mut c_char) {
    unsafe {
        let mut i: isize = 0;
        let mut r: isize = 0;

        loop {
            let c = *r#in.offset(i) as u8;
            if c == 0 {
                break;
            }
            if i as usize >= MAX_NAME_LENGTH - 1 {
                // the ui truncates the name here..
                break;
            }

            if c == b'^' {
                let next = *r#in.offset(i + 1) as u8;
                if next >= b'0' && next <= b'9' {
                    // only skip it if there's a number after it for the color
                    i += 2;
                    continue;
                } else {
                    // just skip the ^
                    i += 1;
                    continue;
                }
            }

            if (c as i8) < 32 {
                i += 1;
                continue;
            }

            *out.offset(r) = c as c_char;
            r += 1;
            i += 1;
        }

        *out.offset(r) = 0;
    }
}

/// Raven `G_ClientNumberFromStrippedName`.
///
/// Same as `G_ClientNumberFromName`, but strips special characters out of the
/// names before comparing.
///
/// Source: `oracle/codemp/game/g_cmds.c:1946-1965`
pub fn G_ClientNumberFromStrippedName(ctx: &mut GameContext, name: *const c_char) -> c_int {
    unsafe {
        let mut s2 = [0 as c_char; MAX_STRING_CHARS];
        let mut n2 = [0 as c_char; MAX_STRING_CHARS];

        // check for a name match
        SanitizeString2(name as *mut c_char, s2.as_mut_ptr());
        for i in 0..ctx.world.level.numConnectedClients {
            SanitizeString2(
                ctx.world.client(i as usize).pers.netname.as_ptr() as *mut c_char,
                n2.as_mut_ptr(),
            );
            if cstr_eq(n2.as_ptr(), s2.as_ptr()) {
                return i;
            }
        }

        -1
    }
}

/// Raven `Cmd_CallVote_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:1974-2156`
pub fn Cmd_CallVote_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::{
        GT_DUEL, GT_FFA, GT_MAX_GAME_TYPE, GT_POWERDUEL, GT_SINGLE_PLAYER,
    };

    // Oracle `MAX_VOTE_COUNT` is 3 (was wrongly 5), canonical in
    // `client_persistant`. Source: `oracle/codemp/game/g_local.h:439`
    use crate::client::client_persistant::MAX_VOTE_COUNT;

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if ctx.world.cvars.g_allowVote.integer == 0 {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOVOTE".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        if ctx.world.level.voteTime != 0 || ctx.world.level.voteExecuteTime >= ctx.world.level.time
        {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"VOTEINPROGRESS".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }
        if ctx.world.client(cidx).pers.voteCount >= MAX_VOTE_COUNT {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"MAXVOTES".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        if ctx.world.cvars.g_gametype.integer != GT_DUEL
            && ctx.world.cvars.g_gametype.integer != GT_POWERDUEL
        {
            if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOSPECVOTE".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s),
                    ),
                );
                return;
            }
        }

        let mut arg1 = [0 as c_char; MAX_STRING_TOKENS];
        let mut arg2 = [0 as c_char; MAX_STRING_TOKENS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                arg1.as_mut_ptr(),
                MAX_STRING_TOKENS as c_int,
            ),
        );
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                2,
                arg2.as_mut_ptr(),
                MAX_STRING_TOKENS as c_int,
            ),
        );
        let arg1_s = cstr_to_str(arg1.as_ptr());
        let arg2_s = cstr_to_str(arg2.as_ptr());

        if arg1_s.contains(';') || arg2_s.contains(';') {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"Invalid vote string.\n\""),
                ),
            );
            return;
        }

        let valid = [
            "map_restart",
            "nextmap",
            "map",
            "g_gametype",
            "kick",
            "clientkick",
            "g_doWarmup",
            "timelimit",
            "fraglimit",
        ];
        if !valid.iter().any(|v| arg1_s.eq_ignore_ascii_case(v)) {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"Invalid vote string.\n\""),
                ),
            );
            trap::SendServerCommand(ctx.engine, mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(ent.index() as c_int, cstr("print \"Vote commands are: map_restart, nextmap, map <mapname>, g_gametype <n>, kick <player>, clientkick <clientnum>, g_doWarmup, timelimit <time>, fraglimit <frags>.\n\"")));
            return;
        }

        if ctx.world.level.voteExecuteTime != 0 {
            ctx.world.level.voteExecuteTime = 0;
            let cc = format!("{}\n", cstr_to_str(ctx.world.level.voteString.as_ptr()));
            trap::SendConsoleCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_CONSOLE_COMMAND::GSendConsoleCommandArgs::new(
                    EXEC_APPEND as c_int,
                    cstr(&cc),
                ),
            );
        }

        if arg1_s.eq_ignore_ascii_case("g_gametype") {
            let i: c_int = atoi_str(&arg2_s);
            if i == GT_SINGLE_PLAYER || i < GT_FFA || i >= GT_MAX_GAME_TYPE {
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr("print \"Invalid gametype.\n\""),
                    ),
                );
                return;
            }

            ctx.world.level.votingGametype = qtrue;
            ctx.world.level.votingGametypeTo = i;

            write_cstr_field(
                &mut ctx.world.level.voteString,
                &format!("{} {}", arg1_s, i),
            );
            write_cstr_field(
                &mut ctx.world.level.voteDisplayString,
                &format!("{} {}", arg1_s, cstr_to_str(gameNames[i as usize].as_ptr())),
            );
        } else if arg1_s.eq_ignore_ascii_case("map") {
            let mut s = [0 as c_char; MAX_STRING_CHARS];
            let gametype = trap::Cvar_VariableIntegerValue(ctx.engine, mp_abi::game::syscalls::G_CVAR_VARIABLE_INTEGER_VALUE::GCvarVariableIntegerValueArgs::new(cstr("g_gametype")));
            if crate::g_bot::G_DoesMapSupportGametype(ctx, cstr(&arg2_s).as_ptr(), gametype)
                == qfalse
            {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOVOTE_MAPNOTSUPPORTEDBYGAME".as_ptr() as *mut c_char,
                );
                let msg = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&msg),
                    ),
                );
                return;
            }

            trap::Cvar_VariableStringBuffer(ctx.engine, mp_abi::game::syscalls::G_CVAR_VARIABLE_STRING_BUFFER::GCvarVariableStringBufferArgs::new(cstr("nextmap"), s.as_mut_ptr(), MAX_STRING_CHARS as c_int));
            let s_str = cstr_to_str(s.as_ptr());
            if !s_str.is_empty() {
                write_cstr_field(
                    &mut ctx.world.level.voteString,
                    &format!("{} {}; set nextmap \"{}\"", arg1_s, arg2_s, s_str),
                );
            } else {
                write_cstr_field(
                    &mut ctx.world.level.voteString,
                    &format!("{} {}", arg1_s, arg2_s),
                );
            }

            let arenaInfo = crate::g_bot::G_GetArenaInfoByMap(ctx, cstr(&arg2_s).as_ptr());
            let mut mapName = if !arenaInfo.is_null() {
                crate::q_shared::Info_ValueForKey(
                    &mut ctx.world.bg_state.qs,
                    arenaInfo,
                    cstr("longname").as_ptr(),
                )
            } else {
                std::ptr::null_mut()
            };
            let mapName_str = if mapName.is_null() || *mapName == 0 {
                "ERROR".to_string()
            } else {
                cstr_to_str(mapName)
            };

            write_cstr_field(
                &mut ctx.world.level.voteDisplayString,
                &format!("map {}", mapName_str),
            );
        } else if arg1_s.eq_ignore_ascii_case("clientkick") {
            let n: c_int = atoi_str(&arg2_s);
            if n < 0 || n >= MAX_CLIENTS as c_int {
                let msg = format!("print \"invalid client number {}.\n\"", n);
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&msg),
                    ),
                );
                return;
            }

            let nclient = ctx.world.g_entities[n as usize].client;
            if (*nclient).pers.connected == crate::client::client_connected::CON_DISCONNECTED {
                let msg = format!(
                    "print \"there is no client with the client number {}.\n\"",
                    n
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&msg),
                    ),
                );
                return;
            }

            write_cstr_field(
                &mut ctx.world.level.voteString,
                &format!("{} {}", arg1_s, arg2_s),
            );
            write_cstr_field(
                &mut ctx.world.level.voteDisplayString,
                &format!("kick {}", cstr_to_str((*nclient).pers.netname.as_ptr())),
            );
        } else if arg1_s.eq_ignore_ascii_case("kick") {
            let mut clientid = G_ClientNumberFromName(ctx, cstr(&arg2_s).as_ptr());
            if clientid == -1 {
                clientid = G_ClientNumberFromStrippedName(ctx, cstr(&arg2_s).as_ptr());
                if clientid == -1 {
                    let msg = format!(
                        "print \"there is no client named '{}' currently on the server.\n\"",
                        arg2_s
                    );
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                            ent.index() as c_int,
                            cstr(&msg),
                        ),
                    );
                    return;
                }
            }

            write_cstr_field(
                &mut ctx.world.level.voteString,
                &format!("clientkick {}", clientid),
            );
            let ncl = ctx.world.g_entities[clientid as usize].client;
            write_cstr_field(
                &mut ctx.world.level.voteDisplayString,
                &format!("kick {}", cstr_to_str((*ncl).pers.netname.as_ptr())),
            );
        } else if arg1_s.eq_ignore_ascii_case("nextmap") {
            let mut s = [0 as c_char; MAX_STRING_CHARS];
            trap::Cvar_VariableStringBuffer(ctx.engine, mp_abi::game::syscalls::G_CVAR_VARIABLE_STRING_BUFFER::GCvarVariableStringBufferArgs::new(cstr("nextmap"), s.as_mut_ptr(), MAX_STRING_CHARS as c_int));
            if cstr_to_str(s.as_ptr()).is_empty() {
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr("print \"nextmap not set.\n\""),
                    ),
                );
                return;
            }
            crate::g_saga::SiegeClearSwitchData(ctx);
            write_cstr_field(&mut ctx.world.level.voteString, "vstr nextmap");
            let vs = cstr_to_str(ctx.world.level.voteString.as_ptr());
            write_cstr_field(&mut ctx.world.level.voteDisplayString, &vs);
        } else {
            write_cstr_field(
                &mut ctx.world.level.voteString,
                &format!("{} \"{}\"", arg1_s, arg2_s),
            );
            let vs = cstr_to_str(ctx.world.level.voteString.as_ptr());
            write_cstr_field(&mut ctx.world.level.voteDisplayString, &vs);
        }

        let m = crate::g_main::G_GetStringEdString(
            ctx,
            c"MP_SVGAME".as_ptr() as *mut c_char,
            c"PLCALLEDVOTE".as_ptr() as *mut c_char,
        );
        let s = format!(
            "print \"{}^7 {}\n\"",
            cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
            cstr_to_str(m)
        );
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                -1,
                cstr(&s),
            ),
        );

        ctx.world.level.voteTime = ctx.world.level.time;
        ctx.world.level.voteYes = 1;
        ctx.world.level.voteNo = 0;

        for i in 0..ctx.world.level.maxclients {
            ctx.world.client_mut(i as usize).mGameFlags &= !(PSG_VOTED as u32);
        }
        ctx.world.client_mut(cidx).mGameFlags |= PSG_VOTED as u32;

        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_VOTE_TIME,
                cstr(&format!("{}", ctx.world.level.voteTime)),
            ),
        );
        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_VOTE_STRING,
                cstr(&cstr_to_str(ctx.world.level.voteDisplayString.as_ptr())),
            ),
        );
        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_VOTE_YES,
                cstr(&format!("{}", ctx.world.level.voteYes)),
            ),
        );
        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_VOTE_NO,
                cstr(&format!("{}", ctx.world.level.voteNo)),
            ),
        );
    }
}

/// Raven `Cmd_Vote_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2163-2199`
pub fn Cmd_Vote_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL};

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if ctx.world.level.voteTime == 0 {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOVOTEINPROG".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }
        if ctx.world.client(cidx).mGameFlags & (PSG_VOTED as u32) != 0 {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"VOTEALREADY".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }
        if ctx.world.cvars.g_gametype.integer != GT_DUEL
            && ctx.world.cvars.g_gametype.integer != GT_POWERDUEL
        {
            if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"NOVOTEASSPEC".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s),
                    ),
                );
                return;
            }
        }

        let m = crate::g_main::G_GetStringEdString(
            ctx,
            c"MP_SVGAME".as_ptr() as *mut c_char,
            c"PLVOTECAST".as_ptr() as *mut c_char,
        );
        let s = format!("print \"{}\n\"", cstr_to_str(m));
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                ent.index() as c_int,
                cstr(&s),
            ),
        );

        ctx.world.client_mut(cidx).mGameFlags |= PSG_VOTED as u32;

        let mut msg = [0 as c_char; 64];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(1, msg.as_mut_ptr(), 64),
        );

        if msg[0] as u8 == b'y' || msg[1] as u8 == b'Y' || msg[1] as u8 == b'1' {
            ctx.world.level.voteYes += 1;
            trap::SetConfigstring(
                ctx.engine,
                mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                    CS_VOTE_YES,
                    cstr(&format!("{}", ctx.world.level.voteYes)),
                ),
            );
        } else {
            ctx.world.level.voteNo += 1;
            trap::SetConfigstring(
                ctx.engine,
                mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                    CS_VOTE_NO,
                    cstr(&format!("{}", ctx.world.level.voteNo)),
                ),
            );
        }
        // a majority will be determined in CheckVote, which will also account
        // for players entering or leaving
    }
}

/// Raven `Cmd_CallTeamVote_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2206-2363`
pub fn Cmd_CallTeamVote_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_TEAM;

    // `MAX_NETNAME`/`MAX_VOTE_COUNT` canonical in `client_persistant`;
    // `ENTITYNUM_NONE` in `mp_qshared::shared::limits` (all value-correct here).
    // Sources: `oracle/codemp/game/g_local.h:438-439`, `q_shared.h:2014`
    use crate::client::client_persistant::{MAX_NETNAME, MAX_VOTE_COUNT};
    use mp_qshared::shared::limits::ENTITYNUM_NONE;

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if ctx.world.cvars.g_gametype.integer < GT_TEAM {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"Cannot call a team vote in a non-team gametype!\n\""),
                ),
            );
            return;
        }
        let team = ctx.world.client(cidx).sess.sessionTeam;
        let cs_offset: c_int = if team == TEAM_RED {
            0
        } else if team == TEAM_BLUE {
            1
        } else {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"Cannot call a team vote if not on a team!\n\""),
                ),
            );
            return;
        };

        if ctx.world.cvars.g_allowTeamVote.integer == 0 {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOVOTE".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        if ctx.world.level.teamVoteTime[cs_offset as usize] != 0 {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"TEAMVOTEALREADY".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }
        if ctx.world.client(cidx).pers.teamVoteCount >= MAX_VOTE_COUNT {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"MAXTEAMVOTES".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }
        if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOSPECVOTE".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        let mut arg1 = [0 as c_char; MAX_STRING_TOKENS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                arg1.as_mut_ptr(),
                MAX_STRING_TOKENS as c_int,
            ),
        );
        let arg1_s = cstr_to_str(arg1.as_ptr());
        let mut arg2_s = String::new();
        let argc = trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new());
        for i in 2..argc {
            if i > 2 {
                arg2_s.push(' ');
            }
            let mut a = [0 as c_char; MAX_STRING_TOKENS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    i,
                    a.as_mut_ptr(),
                    MAX_STRING_TOKENS as c_int,
                ),
            );
            arg2_s.push_str(&cstr_to_str(a.as_ptr()));
        }

        if arg1_s.contains(';') || arg2_s.contains(';') {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"Invalid vote string.\n\""),
                ),
            );
            return;
        }

        let mut targetClientNum: c_int = ENTITYNUM_NONE;
        if arg1_s.eq_ignore_ascii_case("leader") || arg1_s.eq_ignore_ascii_case("kick") {
            if arg2_s.is_empty() {
                targetClientNum = ctx.world.client(cidx).ps.clientNum;
            } else {
                // C scans only the first up-to-3 chars: numeric slot iff those
                // are all digits (i reaches 3) or the string ends inside them.
                let bytes = arg2_s.as_bytes();
                let mut i = 0usize;
                while i < 3 {
                    let c = if i < bytes.len() { bytes[i] } else { 0 };
                    if c == 0 || !c.is_ascii_digit() {
                        break;
                    }
                    i += 1;
                }
                let numeric = i >= 3 || i >= bytes.len();
                if numeric {
                    // Source: oracle/codemp/game/g_cmds.c:2273 — plain `atoi(arg2)`.
                    targetClientNum = atoi_str(&arg2_s);
                    if targetClientNum < 0 || targetClientNum >= ctx.world.level.maxclients {
                        let msg = format!("print \"Bad client slot: {}\n\"", targetClientNum);
                        trap::SendServerCommand(ctx.engine, mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(ent.index() as c_int, cstr(&msg)));
                        return;
                    }
                    if ctx.world.g_entities[targetClientNum as usize].inuse == qfalse {
                        let msg = format!("print \"Client {} is not active\n\"", targetClientNum);
                        trap::SendServerCommand(ctx.engine, mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(ent.index() as c_int, cstr(&msg)));
                        return;
                    }
                } else {
                    let mut target = arg2_s.clone();
                    let mut tbuf: Vec<c_char> = target.bytes().map(|b| b as c_char).collect();
                    tbuf.push(0);
                    crate::q_shared::Q_CleanStr(tbuf.as_mut_ptr());
                    target = cstr_to_str(tbuf.as_ptr());

                    targetClientNum = ctx.world.level.maxclients;
                    for i in 0..ctx.world.level.maxclients {
                        if ctx.world.client(i as usize).pers.connected
                            == crate::client::client_connected::CON_DISCONNECTED
                        {
                            continue;
                        }
                        if ctx.world.client(i as usize).sess.sessionTeam != team {
                            continue;
                        }
                        let mut nbuf: Vec<c_char> =
                            cstr_to_str(ctx.world.client(i as usize).pers.netname.as_ptr())
                                .bytes()
                                .map(|b| b as c_char)
                                .collect();
                        nbuf.push(0);
                        crate::q_shared::Q_CleanStr(nbuf.as_mut_ptr());
                        let netname = cstr_to_str(nbuf.as_ptr());
                        if netname.eq_ignore_ascii_case(&target) {
                            targetClientNum = i;
                            break;
                        }
                    }
                    if targetClientNum >= ctx.world.level.maxclients {
                        let msg =
                            format!("print \"{} is not a valid player on your team.\n\"", arg2_s);
                        trap::SendServerCommand(ctx.engine, mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(ent.index() as c_int, cstr(&msg)));
                        return;
                    }
                }
            }
            if targetClientNum >= MAX_CLIENTS as c_int {
                let msg = format!("print \"{} is not a valid player on your team.\n\"", arg2_s);
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&msg),
                    ),
                );
                return;
            }
            if ctx.world.client(targetClientNum as usize).sess.sessionTeam
                != ctx.world.client(cidx).sess.sessionTeam
            {
                let msg = format!(
                    "print \"Cannot call a team vote on someone not on your team ({}).\n\"",
                    cstr_to_str(
                        ctx.world
                            .client(targetClientNum as usize)
                            .pers
                            .netname
                            .as_ptr()
                    )
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&msg),
                    ),
                );
                return;
            }
            arg2_s = format!("{}", targetClientNum);
        } else {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"Invalid vote string.\n\""),
                ),
            );
            trap::SendServerCommand(ctx.engine, mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(ent.index() as c_int, cstr("print \"Team vote commands are: leader <player on your team> OR kick <player on your team>.\n\"")));
            return;
        }

        if arg1_s.eq_ignore_ascii_case("kick") {
            write_cstr_field(
                &mut ctx.world.level.teamVoteString[cs_offset as usize],
                &format!("clientkick {}", arg2_s),
            );
        } else {
            write_cstr_field(
                &mut ctx.world.level.teamVoteString[cs_offset as usize],
                &format!("{} {}", arg1_s, arg2_s),
            );
        }

        for i in 0..ctx.world.level.maxclients {
            if ctx.world.client(i as usize).pers.connected
                == crate::client::client_connected::CON_DISCONNECTED
            {
                continue;
            }
            if ctx.world.client(i as usize).sess.sessionTeam == team {
                let msg = format!(
                    "print \"{} called a team vote.\n\"",
                    cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr())
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        i,
                        cstr(&msg),
                    ),
                );
            }
        }

        ctx.world.level.teamVoteTime[cs_offset as usize] = ctx.world.level.time;
        ctx.world.level.teamVoteYes[cs_offset as usize] = 1;
        ctx.world.level.teamVoteNo[cs_offset as usize] = 0;

        for i in 0..ctx.world.level.maxclients {
            if ctx.world.client(i as usize).sess.sessionTeam == team {
                ctx.world.client_mut(i as usize).mGameFlags &= !(PSG_TEAMVOTED as u32);
            }
        }
        ctx.world.client_mut(cidx).mGameFlags |= PSG_TEAMVOTED as u32;

        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_TEAMVOTE_TIME + cs_offset,
                cstr(&format!(
                    "{}",
                    ctx.world.level.teamVoteTime[cs_offset as usize]
                )),
            ),
        );
        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_TEAMVOTE_STRING + cs_offset,
                cstr(&cstr_to_str(
                    ctx.world.level.teamVoteString[cs_offset as usize].as_ptr(),
                )),
            ),
        );
        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_TEAMVOTE_YES + cs_offset,
                cstr(&format!(
                    "{}",
                    ctx.world.level.teamVoteYes[cs_offset as usize]
                )),
            ),
        );
        trap::SetConfigstring(
            ctx.engine,
            mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                CS_TEAMVOTE_NO + cs_offset,
                cstr(&format!(
                    "{}",
                    ctx.world.level.teamVoteNo[cs_offset as usize]
                )),
            ),
        );
    }
}

/// Raven `Cmd_TeamVote_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2370-2411`
pub fn Cmd_TeamVote_f(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();
        let team = ctx.world.client(cidx).sess.sessionTeam;
        let cs_offset: c_int = if team == TEAM_RED {
            0
        } else if team == TEAM_BLUE {
            1
        } else {
            return;
        };

        if ctx.world.level.teamVoteTime[cs_offset as usize] == 0 {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOTEAMVOTEINPROG".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }
        if ctx.world.client(cidx).mGameFlags & (PSG_TEAMVOTED as u32) != 0 {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"TEAMVOTEALREADYCAST".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }
        if ctx.world.client(cidx).sess.sessionTeam == TEAM_SPECTATOR {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOVOTEASSPEC".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        let m = crate::g_main::G_GetStringEdString(
            ctx,
            c"MP_SVGAME".as_ptr() as *mut c_char,
            c"PLTEAMVOTECAST".as_ptr() as *mut c_char,
        );
        let s = format!("print \"{}\n\"", cstr_to_str(m));
        trap::SendServerCommand(
            ctx.engine,
            mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                ent.index() as c_int,
                cstr(&s),
            ),
        );

        ctx.world.client_mut(cidx).mGameFlags |= PSG_TEAMVOTED as u32;

        let mut msg = [0 as c_char; 64];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(1, msg.as_mut_ptr(), 64),
        );

        if msg[0] as u8 == b'y' || msg[1] as u8 == b'Y' || msg[1] as u8 == b'1' {
            ctx.world.level.teamVoteYes[cs_offset as usize] += 1;
            trap::SetConfigstring(
                ctx.engine,
                mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                    CS_TEAMVOTE_YES + cs_offset,
                    cstr(&format!(
                        "{}",
                        ctx.world.level.teamVoteYes[cs_offset as usize]
                    )),
                ),
            );
        } else {
            ctx.world.level.teamVoteNo[cs_offset as usize] += 1;
            trap::SetConfigstring(
                ctx.engine,
                mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs::new(
                    CS_TEAMVOTE_NO + cs_offset,
                    cstr(&format!(
                        "{}",
                        ctx.world.level.teamVoteNo[cs_offset as usize]
                    )),
                ),
            );
        }
        // a majority will be determined in TeamCheckVote, which will also account
        // for players entering or leaving
    }
}

/// Raven `Cmd_SetViewpos_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2419-2443`
pub fn Cmd_SetViewpos_f(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let mut origin: vec3_t = [0.0, 0.0, 0.0];
        let mut angles: vec3_t = [0.0, 0.0, 0.0];
        let mut buffer = [0 as c_char; MAX_TOKEN_CHARS];

        if ctx.world.cvars.g_cheats.integer == 0 {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NOCHEATS".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }
        if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) != 5 {
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr("print \"usage: setviewpos x y z yaw\n\""),
                ),
            );
            return;
        }

        for i in 0..3usize {
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    i as c_int + 1,
                    buffer.as_mut_ptr(),
                    MAX_TOKEN_CHARS as c_int,
                ),
            );
            origin[i] = crate::bg_lib::atof(buffer.as_ptr()) as f32;
        }

        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                4,
                buffer.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );
        angles[YAW as usize] = crate::bg_lib::atof(buffer.as_ptr()) as f32;

        crate::g_misc::TeleportPlayer(ctx, ent, origin, angles);
    }
}

/// Raven `Cmd_Stats_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2452-2467`
// Raven's body is entirely `#if 0`-style commented out (dead code, kept for
// reference in the oracle) — the compiled function is a callable no-op.
// Source: `oracle/codemp/game/g_cmds.c:2453-2466`
// STAGE-1: ctx-free leaf borrow &gentity_t (body ignores `ent` — empty stub).
pub fn Cmd_Stats_f(ent: &gentity_t) {}

/// Raven `G_ItemUsable`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2469-2591`
pub fn G_ItemUsable(ctx: &mut GameContext, ps: *mut playerState_t, forcedUse: c_int) -> c_int {
    unsafe {
        let mut forcedUse = forcedUse;

        if (*ps).m_iVehicleNum != 0 {
            return 0;
        }

        if (*ps).pm_flags & PMF_USE_ITEM_HELD != 0 {
            return 0;
        }

        if forcedUse == 0 {
            forcedUse = bg_itemlist[(*ps).stats[STAT_HOLDABLE_ITEM as usize] as usize].giTag;
        }

        if crate::bg_misc::BG_IsItemSelectable(ps, forcedUse) == qfalse {
            return 0;
        }

        match forcedUse {
            HI_MEDPAC | HI_MEDPAC_BIG => {
                if (*ps).stats[STAT_HEALTH as usize] >= (*ps).stats[STAT_MAX_HEALTH as usize] {
                    return 0;
                }
                if (*ps).stats[STAT_HEALTH as usize] <= 0 {
                    return 0;
                }
                1
            }
            HI_SEEKER => {
                if (*ps).eFlags & EF_SEEKERDRONE != 0 {
                    crate::g_utils::G_AddEvent(
                        &mut *(&mut ctx.world.g_entities[(*ps).clientNum as usize]),
                        EV_ITEMUSEFAIL as c_int,
                        mp_qshared::shared::itemUseFail_t::SEEKER_ALREADYDEPLOYED as c_int,
                    );
                    return 0;
                }
                1
            }
            HI_SENTRY_GUN => {
                if (*ps).fd.sentryDeployed != 0 {
                    crate::g_utils::G_AddEvent(
                        &mut *(&mut ctx.world.g_entities[(*ps).clientNum as usize]),
                        EV_ITEMUSEFAIL as c_int,
                        mp_qshared::shared::itemUseFail_t::SENTRY_ALREADYPLACED as c_int,
                    );
                    return 0;
                }

                let mut yawonly: vec3_t = [0.0, 0.0, 0.0];
                yawonly[ROLL as usize] = 0.0;
                yawonly[PITCH as usize] = 0.0;
                yawonly[YAW as usize] = (*ps).viewangles[YAW as usize];

                let mut mins: vec3_t = [-8.0, -8.0, 0.0];
                let mut maxs: vec3_t = [8.0, 8.0, 24.0];

                let mut fwd: vec3_t = [0.0, 0.0, 0.0];
                crate::q_math::AngleVectors(yawonly, Some(&mut fwd), None, None);

                let mut fwdorg: vec3_t = [
                    (*ps).origin[0] + fwd[0] * 64.0,
                    (*ps).origin[1] + fwd[1] * 64.0,
                    (*ps).origin[2] + fwd[2] * 64.0,
                ];
                let trtest: vec3_t = [
                    fwdorg[0] + fwd[0] * 16.0,
                    fwdorg[1] + fwd[1] * 16.0,
                    fwdorg[2] + fwd[2] * 16.0,
                ];

                let mut tr: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*ps).origin as *const vec3_t,
                        &mins as *const vec3_t,
                        &maxs as *const vec3_t,
                        &trtest as *const vec3_t,
                        (*ps).clientNum,
                        MASK_PLAYERSOLID,
                    ),
                );

                if (tr.fraction != 1.0 && tr.entityNum != (*ps).clientNum as c_short)
                    || tr.startsolid != qfalse as u8
                    || tr.allsolid != qfalse as u8
                {
                    crate::g_utils::G_AddEvent(
                        &mut *(&mut ctx.world.g_entities[(*ps).clientNum as usize]),
                        EV_ITEMUSEFAIL as c_int,
                        mp_qshared::shared::itemUseFail_t::SENTRY_NOROOM as c_int,
                    );
                    return 0;
                }

                1
            }
            HI_SHIELD => {
                let mins: vec3_t = [-8.0, -8.0, 0.0];
                let maxs: vec3_t = [8.0, 8.0, 8.0];

                let mut fwd: vec3_t = [0.0, 0.0, 0.0];
                crate::q_math::AngleVectors((*ps).viewangles, Some(&mut fwd), None, None);
                fwd[2] = 0.0;
                let mut dest: vec3_t = [0.0, 0.0, 0.0];
                crate::q_math::_VectorMA((*ps).origin, 64.0, fwd, &mut dest);

                let mut tr: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*ps).origin as *const vec3_t,
                        &mins as *const vec3_t,
                        &maxs as *const vec3_t,
                        &dest as *const vec3_t,
                        (*ps).clientNum,
                        mp_qshared::shared::surface_flags::MASK_SHOT,
                    ),
                );
                if tr.fraction > 0.9 && tr.startsolid == qfalse as u8 && tr.allsolid == qfalse as u8
                {
                    let pos = tr.endpos;
                    let dest2: vec3_t = [pos[0], pos[1], pos[2] - 4096.0];
                    let mut tr2: trace_t = core::mem::zeroed();
                    trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut tr2 as *mut trace_t,
                            &pos as *const vec3_t,
                            &mins as *const vec3_t,
                            &maxs as *const vec3_t,
                            &dest2 as *const vec3_t,
                            (*ps).clientNum,
                            MASK_SOLID,
                        ),
                    );
                    if tr2.startsolid == qfalse as u8 && tr2.allsolid == qfalse as u8 {
                        return 1;
                    }
                }
                crate::g_utils::G_AddEvent(
                    &mut *(&mut ctx.world.g_entities[(*ps).clientNum as usize]),
                    EV_ITEMUSEFAIL as c_int,
                    mp_qshared::shared::itemUseFail_t::SHIELD_NOROOM as c_int,
                );
                0
            }
            HI_JETPACK | HI_HEALTHDISP | HI_AMMODISP | HI_EWEB | HI_CLOAK => 1,
            _ => 1,
        }
    }
}

/// Raven `Cmd_ToggleSaber_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2595-2670`
pub fn Cmd_ToggleSaber_f(ctx: &mut GameContext, ent: EntityId) {
    // `ent` is the commanding player, so its client slot is `ent.index()`.
    let cidx = ent.index();
    let level_time = ctx.world.level.time;

    if ctx.world.client(cidx).ps.fd.forceGripCripple != 0 {
        // if they are being gripped, don't let them unholster their saber
        if ctx.world.client(cidx).ps.saberHolstered != 0 {
            return;
        }
    }

    if ctx.world.client(cidx).ps.saberInFlight != qfalse {
        if ctx.world.client(cidx).ps.saberEntityNum != 0 {
            // turn it off in midair
            let saberent = EntityId(ctx.world.client(cidx).ps.saberEntityNum as u32);
            crate::w_saber::saberKnockDown(ctx, saberent, ent, ent);
        }
        return;
    }

    if ctx.world.client(cidx).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
        return;
    }

    if ctx.world.client(cidx).ps.weapon != WP_SABER {
        return;
    }

    if ctx.world.client(cidx).ps.duelTime >= level_time {
        return;
    }

    if ctx.world.client(cidx).ps.saberLockTime >= level_time {
        return;
    }

    if ctx.world.client(cidx).ps.weaponTime < 1 {
        if ctx.world.client(cidx).ps.saberHolstered == 2 {
            ctx.world.client_mut(cidx).ps.saberHolstered = 0;

            let s0 = ctx.world.client(cidx).saber[0].soundOn;
            if s0 != 0 {
                crate::g_utils::G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, s0);
            }
            let s1 = ctx.world.client(cidx).saber[1].soundOn;
            if s1 != 0 {
                crate::g_utils::G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, s1);
            }
        } else {
            ctx.world.client_mut(cidx).ps.saberHolstered = 2;
            let s0 = ctx.world.client(cidx).saber[0].soundOff;
            if s0 != 0 {
                crate::g_utils::G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, s0);
            }
            let s1 = ctx.world.client(cidx).saber[1].soundOff;
            if s1 != 0 && ctx.world.client(cidx).saber[1].model[0] != 0 {
                crate::g_utils::G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, s1);
            }
            // prevent anything from being done for 400ms after holster
            ctx.world.client_mut(cidx).ps.weaponTime = 400;
        }
    }
}

/// Raven `Cmd_SaberAttackCycle_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2675-2873`
pub fn Cmd_SaberAttackCycle_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::GT_SIEGE;

    unsafe {
        let mut selectLevel: c_int = 0;
        let mut usingSiegeStyle = qfalse;

        if ctx.world.entity(ent).client.is_null() {
            return;
        }
        // FLAG(2c): the `(*client).saber[..]` accesses below are passed as pairs of
        // `&mut client->saber[0/1]` into the `bg_saberLoad` helpers
        // (`WP_SaberCanTurnOffSomeBlades`, `WP_UseFirstValidSaberStyle`) — two
        // simultaneous `&mut` into one client's saber array, which the single-borrow
        // `client_mut(idx)` accessor cannot express (recipe step 4). The client
        // pointer is read once via the safe entity borrow and its saber fields stay
        // raw, exactly as Raven does.
        let client = ctx.world.entity(ent).client;

        if (*client).saber[0].model[0] != 0 && (*client).saber[1].model[0] != 0 {
            // no cycling for akimbo
            if crate::bg_saberLoad::WP_SaberCanTurnOffSomeBlades(&mut (*client).saber[1]) != qfalse
            {
                if (*client).ps.saberHolstered == 1 {
                    crate::g_utils::G_Sound(
                        ctx,
                        Some(ent),
                        CHAN_AUTO as c_int,
                        (*client).saber[1].soundOn,
                    );
                    (*client).ps.saberHolstered = 0;
                    (*client).ps.fd.saberAnimLevel = saber_styles_t::SS_DUAL as c_int;
                } else if (*client).ps.saberHolstered == 0 {
                    if (*client).saber[1].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE != 0 {
                        // can't turn it off manually
                    } else if (*client).saber[1].bladeStyle2Start > 0
                        && (*client).saber[1].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE2 != 0
                    {
                        // can't turn it off manually
                    } else {
                        crate::g_utils::G_Sound(
                            ctx,
                            Some(ent),
                            CHAN_AUTO as c_int,
                            (*client).saber[1].soundOff,
                        );
                        (*client).ps.saberHolstered = 1;
                        (*client).ps.fd.saberAnimLevel = saber_styles_t::SS_FAST as c_int;
                    }
                }

                if ctx.world.cvars.d_saberStanceDebug.integer != 0 {
                    trap::SendServerCommand(ctx.engine, mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(ent.index() as c_int, cstr("print \"SABERSTANCEDEBUG: Attempted to toggle dual saber blade.\n\"")));
                }
                return;
            }
        } else if (*client).saber[0].numBlades > 1
            && crate::bg_saberLoad::WP_SaberCanTurnOffSomeBlades(&mut (*client).saber[0]) != qfalse
        {
            if (*client).ps.saberHolstered == 1 {
                if (*client).ps.saberInFlight != qfalse {
                    if ctx.world.cvars.d_saberStanceDebug.integer != 0 {
                        trap::SendServerCommand(ctx.engine, mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(ent.index() as c_int, cstr("print \"SABERSTANCEDEBUG: Attempted to toggle staff blade in air.\n\"")));
                    }
                    return;
                }
                crate::g_utils::G_Sound(
                    ctx,
                    Some(ent),
                    CHAN_AUTO as c_int,
                    (*client).saber[0].soundOn,
                );
                (*client).ps.saberHolstered = 0;
                if (*client).saber[0].stylesForbidden != 0 {
                    crate::bg_saberLoad::WP_UseFirstValidSaberStyle(
                        &mut (*client).saber[0],
                        &mut (*client).saber[1],
                        (*client).ps.saberHolstered,
                        &mut selectLevel,
                    );
                    if (*client).ps.weaponTime <= 0 {
                        (*client).ps.fd.saberAnimLevel = selectLevel;
                    } else {
                        (*client).saberCycleQueue = selectLevel;
                    }
                }
            } else if (*client).ps.saberHolstered == 0 {
                if (*client).saber[0].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE != 0 {
                    // can't turn it off manually
                } else if (*client).saber[0].bladeStyle2Start > 0
                    && (*client).saber[0].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE2 != 0
                {
                    // can't turn it off manually
                } else {
                    crate::g_utils::G_Sound(
                        ctx,
                        Some(ent),
                        CHAN_AUTO as c_int,
                        (*client).saber[0].soundOff,
                    );
                    (*client).ps.saberHolstered = 1;
                    if (*client).saber[0].singleBladeStyle != saber_styles_t::SS_NONE {
                        if (*client).ps.weaponTime <= 0 {
                            (*client).ps.fd.saberAnimLevel =
                                (*client).saber[0].singleBladeStyle as c_int;
                        } else {
                            (*client).saberCycleQueue =
                                (*client).saber[0].singleBladeStyle as c_int;
                        }
                    }
                }
            }
            if ctx.world.cvars.d_saberStanceDebug.integer != 0 {
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr("print \"SABERSTANCEDEBUG: Attempted to toggle staff blade.\n\""),
                    ),
                );
            }
            return;
        }

        if (*client).saberCycleQueue != 0 {
            selectLevel = (*client).saberCycleQueue;
        } else {
            selectLevel = (*client).ps.fd.saberAnimLevel;
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE
            && (*client).siegeClass != -1
            && (&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize].saberStance != 0
        {
            let mut i = selectLevel + 1;
            usingSiegeStyle = qtrue;

            while i != selectLevel {
                if i >= saber_styles_t::SS_NUM_SABER_STYLES as c_int {
                    i = saber_styles_t::SS_FAST as c_int;
                }

                if (&ctx.world.bg_state.bgSiegeClasses)[(*client).siegeClass as usize].saberStance
                    & (1 << i)
                    != 0
                {
                    selectLevel = i;
                    break;
                }
                i += 1;
            }

            if ctx.world.cvars.d_saberStanceDebug.integer != 0 {
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(
                            "print \"SABERSTANCEDEBUG: Attempted to cycle given class stance.\n\"",
                        ),
                    ),
                );
            }
        } else {
            selectLevel += 1;
            if selectLevel > (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] {
                selectLevel = FORCE_LEVEL_1;
            }
            if ctx.world.cvars.d_saberStanceDebug.integer != 0 {
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr("print \"SABERSTANCEDEBUG: Attempted to cycle stance normally.\n\""),
                    ),
                );
            }
        }

        if usingSiegeStyle == qfalse {
            crate::bg_saberLoad::WP_UseFirstValidSaberStyle(
                &mut (*client).saber[0],
                &mut (*client).saber[1],
                (*client).ps.saberHolstered,
                &mut selectLevel,
            );
        }

        if (*client).ps.weaponTime <= 0 {
            (*client).ps.fd.saberAnimLevelBase = selectLevel;
            (*client).ps.fd.saberAnimLevel = selectLevel;
        } else {
            (*client).ps.fd.saberAnimLevelBase = selectLevel;
            (*client).saberCycleQueue = selectLevel;
        }
    }
}

/// Raven `G_OtherPlayersDueling`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2875-2892`
pub fn G_OtherPlayersDueling(ctx: &mut GameContext) -> qboolean {
    for i in 0..MAX_CLIENTS {
        let id = EntityId(i as u32);
        if ctx.world.entity(id).inuse != qfalse
            && !ctx.world.entity(id).client.is_null()
            && ctx.world.client(i).ps.duelInProgress != qfalse
        {
            return qtrue;
        }
    }

    qfalse
}

/// Raven `Cmd_EngageDuel_f`.
///
/// Source: `oracle/codemp/game/g_cmds.c:2894-3042`
pub fn Cmd_EngageDuel_f(ctx: &mut GameContext, ent: EntityId) {
    use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL, GT_TEAM};

    unsafe {
        // `ent` is the commanding player, so its client slot is `ent.index()`.
        let cidx = ent.index();

        if ctx.world.cvars.g_privateDuel.integer == 0 {
            return;
        }

        if ctx.world.cvars.g_gametype.integer == GT_DUEL
            || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
        {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NODUEL_GAMETYPE".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"NODUEL_GAMETYPE".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        if ctx.world.client(cidx).ps.duelTime >= ctx.world.level.time {
            return;
        }

        if ctx.world.client(cidx).ps.weapon != WP_SABER {
            return;
        }

        if ctx.world.client(cidx).ps.saberInFlight != qfalse {
            return;
        }

        if ctx.world.client(cidx).ps.duelInProgress != qfalse {
            return;
        }

        if ctx.world.client(cidx).ps.fd.privateDuelTime > ctx.world.level.time {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"CANTDUEL_JUSTDID".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        if G_OtherPlayersDueling(ctx) != qfalse {
            let m = crate::g_main::G_GetStringEdString(
                ctx,
                c"MP_SVGAME".as_ptr() as *mut c_char,
                c"CANTDUEL_BUSY".as_ptr() as *mut c_char,
            );
            let s = format!("print \"{}\n\"", cstr_to_str(m));
            trap::SendServerCommand(
                ctx.engine,
                mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                    ent.index() as c_int,
                    cstr(&s),
                ),
            );
            return;
        }

        let mut forward: vec3_t = [0.0, 0.0, 0.0];
        let viewangles = ctx.world.client(cidx).ps.viewangles;
        crate::q_math::AngleVectors(viewangles, Some(&mut forward), None, None);

        let origin = ctx.world.client(cidx).ps.origin;
        let viewheight = ctx.world.client(cidx).ps.viewheight;
        let fwdOrg: vec3_t = [
            origin[0] + forward[0] * 256.0,
            origin[1] + forward[1] * 256.0,
            (origin[2] + viewheight as f32) + forward[2] * 256.0,
        ];

        let mut tr: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr as *mut trace_t,
                &origin as *const vec3_t,
                std::ptr::null(),
                std::ptr::null(),
                &fwdOrg as *const vec3_t,
                ent.index() as c_int,
                MASK_PLAYERSOLID,
            ),
        );

        if tr.fraction != 1.0 && tr.entityNum < MAX_CLIENTS as c_short {
            // `tr.entityNum < MAX_CLIENTS`, so the hit entity is a real client slot.
            let challenged = EntityId(tr.entityNum as u32);
            let chidx = tr.entityNum as usize;

            if ctx.world.entity(challenged).client.is_null()
                || ctx.world.entity(challenged).inuse == qfalse
                || ctx.world.entity(challenged).health < 1
            {
                return;
            }
            if ctx.world.client(chidx).ps.stats[STAT_HEALTH as usize] < 1
                || ctx.world.client(chidx).ps.weapon != WP_SABER
                || ctx.world.client(chidx).ps.duelInProgress != qfalse
                || ctx.world.client(chidx).ps.saberInFlight != qfalse
            {
                return;
            }

            if ctx.world.cvars.g_gametype.integer >= GT_TEAM
                && crate::g_team::OnSameTeam(ctx, Some(ent), Some(challenged)) != qfalse
            {
                return;
            }

            if ctx.world.client(chidx).ps.duelIndex == ent.index() as c_int
                && ctx.world.client(chidx).ps.duelTime >= ctx.world.level.time
            {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"PLDUELACCEPT".as_ptr() as *mut c_char,
                );
                let s = format!(
                    "print \"{} {} {}!\n\"",
                    cstr_to_str(ctx.world.client(chidx).pers.netname.as_ptr()),
                    cstr_to_str(m),
                    cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr())
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        -1,
                        cstr(&s),
                    ),
                );

                ctx.world.client_mut(cidx).ps.duelInProgress = qtrue;
                ctx.world.client_mut(chidx).ps.duelInProgress = qtrue;

                ctx.world.client_mut(cidx).ps.duelTime = ctx.world.level.time + 2000;
                ctx.world.client_mut(chidx).ps.duelTime = ctx.world.level.time + 2000;

                crate::g_utils::G_AddEvent(ctx.world.entity_mut(ent), EV_PRIVATE_DUEL as c_int, 1);
                crate::g_utils::G_AddEvent(
                    ctx.world.entity_mut(challenged),
                    EV_PRIVATE_DUEL as c_int,
                    1,
                );

                if ctx.world.client(cidx).ps.saberHolstered == 0 {
                    let s0 = ctx.world.client(cidx).saber[0].soundOff;
                    if s0 != 0 {
                        crate::g_utils::G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, s0);
                    }
                    let s1 = ctx.world.client(cidx).saber[1].soundOff;
                    if s1 != 0 && ctx.world.client(cidx).saber[1].model[0] != 0 {
                        crate::g_utils::G_Sound(ctx, Some(ent), CHAN_AUTO as c_int, s1);
                    }
                    ctx.world.client_mut(cidx).ps.weaponTime = 400;
                    ctx.world.client_mut(cidx).ps.saberHolstered = 2;
                }
                if ctx.world.client(chidx).ps.saberHolstered == 0 {
                    let s0 = ctx.world.client(chidx).saber[0].soundOff;
                    if s0 != 0 {
                        crate::g_utils::G_Sound(ctx, Some(challenged), CHAN_AUTO as c_int, s0);
                    }
                    let s1 = ctx.world.client(chidx).saber[1].soundOff;
                    if s1 != 0 && ctx.world.client(chidx).saber[1].model[0] != 0 {
                        crate::g_utils::G_Sound(ctx, Some(challenged), CHAN_AUTO as c_int, s1);
                    }
                    ctx.world.client_mut(chidx).ps.weaponTime = 400;
                    ctx.world.client_mut(chidx).ps.saberHolstered = 2;
                }
            } else {
                let m1 = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"PLDUELCHALLENGE".as_ptr() as *mut c_char,
                );
                let s1 = format!(
                    "cp \"{} {}\n\"",
                    cstr_to_str(ctx.world.client(cidx).pers.netname.as_ptr()),
                    cstr_to_str(m1)
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        challenged.index() as c_int,
                        cstr(&s1),
                    ),
                );

                let m2 = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"PLDUELCHALLENGED".as_ptr() as *mut c_char,
                );
                let s2 = format!(
                    "cp \"{} {}\n\"",
                    cstr_to_str(m2),
                    cstr_to_str(ctx.world.client(chidx).pers.netname.as_ptr())
                );
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        ent.index() as c_int,
                        cstr(&s2),
                    ),
                );
            }

            ctx.world.client_mut(chidx).ps.fd.privateDuelTime = 0;

            ctx.world.client_mut(cidx).ps.forceHandExtend = HANDEXTEND_DUELCHALLENGE as c_int;
            ctx.world.client_mut(cidx).ps.forceHandExtendTime = ctx.world.level.time + 1000;

            ctx.world.client_mut(cidx).ps.duelIndex = challenged.index() as c_int;
            ctx.world.client_mut(cidx).ps.duelTime = ctx.world.level.time + 5000;
        }
    }
}

/// Raven `Cmd_DebugSetSaberMove_f`.
///
/// `#ifndef FINAL_BUILD` debug command.
///
/// Source: `oracle/codemp/game/g_cmds.c:3047-3073`
pub fn Cmd_DebugSetSaberMove_f(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // `self_` is the commanding player, so its client slot is `self_.index()`.
        let cidx = self_.index();
        let argNum = trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new());
        let mut arg = [0 as c_char; MAX_STRING_CHARS];

        if argNum < 2 {
            return;
        }

        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                arg.as_mut_ptr(),
                MAX_STRING_CHARS as c_int,
            ),
        );

        if arg[0] == 0 {
            return;
        }

        ctx.world.client_mut(cidx).ps.saberMove = atoi_str(&cstr_to_str(arg.as_ptr()));
        ctx.world.client_mut(cidx).ps.saberBlocked = BLOCKED_BOUNCE_MOVE as c_int;

        if ctx.world.client(cidx).ps.saberMove >= LS_MOVE_MAX {
            ctx.world.client_mut(cidx).ps.saberMove = LS_MOVE_MAX - 1;
        }

        // §19 DIVERGENCE: oracle clamps only the high end, so a negative arg
        // reads `saberMoveData[negative]` OOB (UB); the Rust index panics instead
        // (dev-only command, compiled out in FINAL_BUILD). Source: `g_cmds.c:3067-3072`.
        let saber_move = ctx.world.client(cidx).ps.saberMove;
        let animIdx = ctx.world.bg_state.saberMoveData[saber_move as usize].animToUse;
        let name = cstr_to_str(animTable[animIdx as usize].name);
        crate::g_main::Com_Printf(cstr(&format!("Anim for move: {}\n", name)).as_ptr());
    }
}

/// Raven `Cmd_DebugSetBodyAnim_f`.
///
/// `#ifndef FINAL_BUILD` debug command.
///
/// Source: `oracle/codemp/game/g_cmds.c:3075-3111`
pub fn Cmd_DebugSetBodyAnim_f(ctx: &mut GameContext, self_: EntityId, flags: c_int) {
    unsafe {
        let argNum = trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new());
        let mut arg = [0 as c_char; MAX_STRING_CHARS];
        let mut i: c_int = 0;

        if argNum < 2 {
            return;
        }

        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                1,
                arg.as_mut_ptr(),
                MAX_STRING_CHARS as c_int,
            ),
        );

        if arg[0] == 0 {
            return;
        }

        while (i as usize) < MAX_ANIMATIONS as usize {
            if crate::q_shared::Q_stricmp(arg.as_ptr(), animTable[i as usize].name) == 0 {
                break;
            }
            i += 1;
        }

        if i as usize == MAX_ANIMATIONS as usize {
            crate::g_main::Com_Printf(
                cstr(&format!(
                    "Animation '{}' does not exist\n",
                    cstr_to_str(arg.as_ptr())
                ))
                .as_ptr(),
            );
            return;
        }

        StandardSetBodyAnim(ctx, self_, i, flags);

        crate::g_main::Com_Printf(
            cstr(&format!("Set body anim to {}\n", cstr_to_str(arg.as_ptr()))).as_ptr(),
        );
    }
}

/// Raven `StandardSetBodyAnim`.
///
/// Source: `oracle/codemp/game/g_cmds.c:3114-3117`
pub fn StandardSetBodyAnim(ctx: &mut GameContext, self_: EntityId, anim: c_int, flags: c_int) {
    // Raven `SETANIM_BOTH` == `SETANIM_TORSO|SETANIM_LEGS` == 3 (was wrongly 2),
    // canonical in `mp_bg::public::set_anim`.
    // Source: `oracle/codemp/game/bg_public.h:500`
    use mp_bg::public::set_anim::SETANIM_BOTH;
    crate::g_utils::G_SetAnim(
        ctx,
        self_,
        std::ptr::null_mut(),
        SETANIM_BOTH,
        anim,
        flags,
        0,
    );
}

/// Raven `G_ClientNumFromNetname`.
///
/// Source: `oracle/codemp/game/g_cmds.c:3128-3146`
pub fn G_ClientNumFromNetname(ctx: &mut GameContext, name: *mut c_char) -> c_int {
    unsafe {
        for i in 0..MAX_CLIENTS {
            let id = EntityId(i as u32);

            if ctx.world.entity(id).inuse != qfalse
                && !ctx.world.entity(id).client.is_null()
                && crate::q_shared::Q_stricmp(ctx.world.client(i).pers.netname.as_ptr(), name) == 0
            {
                return ctx.world.entity(id).s.number;
            }
        }

        -1
    }
}

/// Raven `TryGrapple`.
///
/// Source: `oracle/codemp/game/g_cmds.c:3148-3191`
pub fn TryGrapple(ctx: &mut GameContext, ent: EntityId) -> qboolean {
    use mp_bg::public::anim_number::animNumber_t;
    use mp_bg::public::set_anim::{SETANIM_BOTH, SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE};
    // `animNumber_t` is `#[repr(i32)]`; the anim fields (`torsoAnim`, ...) store
    // the value as `c_int`, so compare/pass the discriminant.
    let kyle_grab: c_int = animNumber_t::BOTH_KYLE_GRAB as c_int;

    // `ent` is the commanding player, so its client slot is `ent.index()`.
    let cidx = ent.index();

    if ctx.world.client(cidx).ps.weaponTime > 0 {
        return qfalse;
    }
    if ctx.world.client(cidx).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
        return qfalse;
    }
    if ctx.world.client(cidx).grappleState != 0 {
        return qfalse;
    }

    if ctx.world.client(cidx).ps.weapon != WP_SABER && ctx.world.client(cidx).ps.weapon != WP_MELEE
    {
        return qfalse;
    }

    if ctx.world.client(cidx).ps.weapon == WP_SABER && ctx.world.client(cidx).ps.saberHolstered == 0
    {
        crate::g_cmds::Cmd_ToggleSaber_f(ctx, ent);
        if ctx.world.client(cidx).ps.saberHolstered == 0 {
            return qfalse;
        }
    }

    let cmd_ptr = &mut ctx.world.client_mut(cidx).pers.cmd as *mut _;
    crate::g_utils::G_SetAnim(
        ctx,
        ent,
        cmd_ptr,
        SETANIM_BOTH,
        kyle_grab,
        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        0,
    );
    if ctx.world.client(cidx).ps.torsoAnim == kyle_grab {
        ctx.world.client_mut(cidx).ps.torsoTimer += 500;
        let torso_timer = ctx.world.client(cidx).ps.torsoTimer;
        if ctx.world.client(cidx).ps.legsAnim == ctx.world.client(cidx).ps.torsoAnim {
            ctx.world.client_mut(cidx).ps.legsTimer = torso_timer;
        }
        ctx.world.client_mut(cidx).ps.weaponTime = torso_timer;
        return qtrue;
    }

    qfalse
}

/// Raven `ClientCommand`.
///
/// Source: `oracle/codemp/game/g_cmds.c:3202-4083`
pub fn ClientCommand(ctx: &mut GameContext, clientNum: c_int) {
    unsafe {
        let ent = &mut ctx.world.g_entities[clientNum as usize] as *mut gentity_t;
        if (*ent).client.is_null() {
            return; // not fully in game yet
        }

        let mut cmd = [0 as c_char; MAX_TOKEN_CHARS];
        trap::Argv(
            ctx.engine,
            mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                0,
                cmd.as_mut_ptr(),
                MAX_TOKEN_CHARS as c_int,
            ),
        );
        let cmd_s = cstr_to_str(cmd.as_ptr());

        // rww - redirect bot commands
        if cmd_s.contains("bot_")
            && crate::ai_wpnav::AcceptBotCommand(ctx, cmd.as_mut_ptr(), ctx.entity_id_of(ent)) != 0
        {
            return;
        }

        if cmd_s.eq_ignore_ascii_case("say") {
            Cmd_Say_f(ctx, ctx.entity_id_of(ent).unwrap(), SAY_ALL, qfalse);
            return;
        }
        if cmd_s.eq_ignore_ascii_case("say_team") {
            if ctx.world.cvars.g_gametype.integer < mp_bg::public::gametype::GT_TEAM {
                Cmd_Say_f(ctx, ctx.entity_id_of(ent).unwrap(), SAY_ALL, qfalse);
            } else {
                Cmd_Say_f(ctx, ctx.entity_id_of(ent).unwrap(), SAY_TEAM, qfalse);
            }
            return;
        }
        if cmd_s.eq_ignore_ascii_case("tell") {
            Cmd_Tell_f(ctx, ctx.entity_id_of(ent).unwrap());
            return;
        }

        // note: these voice_cmds come from the ui/jamp/ingame_voicechat.menu menu
        // file... the strings are in strings/English/menus.str and all start with
        // "VC_"
        if cmd_s.eq_ignore_ascii_case("voice_cmd") {
            Cmd_VoiceCommand_f(ctx, ctx.entity_id_of(ent).unwrap());
            return;
        }

        if cmd_s.eq_ignore_ascii_case("score") {
            Cmd_Score_f(ctx, ctx.entity_id_of(ent).unwrap());
            return;
        }

        // ignore all other commands when at intermission
        if ctx.world.level.intermissiontime != 0 {
            let mut giveError = qfalse;

            let intermission_gated = [
                "give",
                "giveother",
                "god",
                "notarget",
                "noclip",
                "kill",
                "teamtask",
                "levelshot",
                "follow",
                "follownext",
                "followprev",
                "team",
                "duelteam",
                "siegeclass",
                "where",
                "callvote",
                "vote",
                "callteamvote",
                "teamvote",
                "gc",
                "setviewpos",
                "stats",
            ];

            if cmd_s.eq_ignore_ascii_case("forcechanged") {
                // special case: still update force change
                Cmd_ForceChanged_f(ctx, ctx.entity_id_of(ent).unwrap());
                return;
            } else if intermission_gated
                .iter()
                .any(|c| cmd_s.eq_ignore_ascii_case(c))
            {
                giveError = qtrue;
            }

            if giveError != qfalse {
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"CANNOT_TASK_INTERMISSION".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{} ({}) \n\"", cstr_to_str(m), cmd_s);
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        clientNum,
                        cstr(&s),
                    ),
                );
            } else {
                Cmd_Say_f(ctx, ctx.entity_id_of(ent).unwrap(), qfalse, qtrue);
            }
            return;
        }

        if cmd_s.eq_ignore_ascii_case("give") {
            Cmd_Give_f(ctx, ctx.entity_id_of(ent).unwrap(), 0);
        } else if cmd_s.eq_ignore_ascii_case("giveother") {
            // for debugging pretty much
            Cmd_Give_f(ctx, ctx.entity_id_of(ent).unwrap(), 1);
        } else if cmd_s.eq_ignore_ascii_case("t_use")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
        {
            // debug use map object
            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
                let mut sArg = [0 as c_char; MAX_STRING_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        1,
                        sArg.as_mut_ptr(),
                        MAX_STRING_CHARS as c_int,
                    ),
                );

                let targetname_ofs = std::mem::offset_of!(gentity_t, targetname) as c_int;
                let mut targ = crate::g_utils::G_Find(
                    ctx,
                    ctx.entity_id_of(std::ptr::null_mut()),
                    targetname_ofs,
                    sArg.as_ptr(),
                );

                while !targ.is_null() {
                    if let Some(use_fn) = (*targ).use_.get() {
                        crate::ent_fn_enums::dispatch_use(ctx, use_fn, targ, ent, ent);
                    }
                    targ = crate::g_utils::G_Find(
                        ctx,
                        ctx.entity_id_of(targ),
                        targetname_ofs,
                        sArg.as_ptr(),
                    );
                }
            }
        } else if cmd_s.eq_ignore_ascii_case("god") {
            Cmd_God_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("notarget") {
            Cmd_Notarget_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("noclip") {
            Cmd_Noclip_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("NPC")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
        {
            crate::NPC_spawn::Cmd_NPC_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("kill") {
            Cmd_Kill_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("teamtask") {
            Cmd_TeamTask_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("levelshot") {
            Cmd_LevelShot_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("follow") {
            Cmd_Follow_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("follownext") {
            Cmd_FollowCycle_f(ctx, ctx.entity_id_of(ent).unwrap(), 1);
        } else if cmd_s.eq_ignore_ascii_case("followprev") {
            Cmd_FollowCycle_f(ctx, ctx.entity_id_of(ent).unwrap(), -1);
        } else if cmd_s.eq_ignore_ascii_case("team") {
            Cmd_Team_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("duelteam") {
            Cmd_DuelTeam_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("siegeclass") {
            Cmd_SiegeClass_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("forcechanged") {
            Cmd_ForceChanged_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("where") {
            Cmd_Where_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("callvote") {
            Cmd_CallVote_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("vote") {
            Cmd_Vote_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("callteamvote") {
            Cmd_CallTeamVote_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("teamvote") {
            Cmd_TeamVote_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("gc") {
            Cmd_GameCommand_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("setviewpos") {
            Cmd_SetViewpos_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("stats") {
            Cmd_Stats_f(&*ent);
        }
        // for convenient powerduel testing in release
        else if cmd_s.eq_ignore_ascii_case("killother")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
        {
            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
                let mut sArg = [0 as c_char; MAX_STRING_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        1,
                        sArg.as_mut_ptr(),
                        MAX_STRING_CHARS as c_int,
                    ),
                );

                let entNum = G_ClientNumFromNetname(ctx, sArg.as_mut_ptr());

                if entNum >= 0 && (entNum as usize) < MAX_GENTITIES {
                    let kEnt = &mut ctx.world.g_entities[entNum as usize] as *mut gentity_t;
                    if (*kEnt).inuse != qfalse && !(*kEnt).client.is_null() {
                        let kClient = (*kEnt).client;
                        (*kEnt).flags &= !FL_GODMODE;
                        (*kClient).ps.stats[STAT_HEALTH as usize] = -999;
                        (*kEnt).health = -999;
                        crate::g_combat::player_die(
                            ctx,
                            ctx.entity_id_of(kEnt).unwrap(),
                            ctx.entity_id_of(kEnt),
                            ctx.entity_id_of(kEnt),
                            100000,
                            MOD_SUICIDE as c_int,
                        );
                    }
                }
            }
        }
        // §20: `#ifdef _DEBUG` (g_cmds.c:3470-3656) and `#ifdef VM_MEMALLOC_DEBUG`
        // (g_cmds.c:4013-4055) branches are dropped as dead surface — neither macro is defined in any target build.
        else if cmd_s.eq_ignore_ascii_case("thedestroyer")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
            && !ent.is_null()
            && !(*ent).client.is_null()
            && (*((*ent).client)).ps.saberHolstered != 0
            && (*((*ent).client)).ps.weapon == WP_SABER
        {
            Cmd_ToggleSaber_f(ctx, ctx.entity_id_of(ent).unwrap());
        }
        // begin bot debug cmds
        else if cmd_s.eq_ignore_ascii_case("debugBMove_Forward")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
        {
            let mut sarg = [0 as c_char; MAX_STRING_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    1,
                    sarg.as_mut_ptr(),
                    MAX_STRING_CHARS as c_int,
                ),
            );
            let bCl: c_int = atoi_str(&cstr_to_str(sarg.as_ptr()));
            crate::ai_main::Bot_SetForcedMovement(ctx, bCl, 4000, -1, -1);
        } else if cmd_s.eq_ignore_ascii_case("debugBMove_Back")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
        {
            let mut sarg = [0 as c_char; MAX_STRING_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    1,
                    sarg.as_mut_ptr(),
                    MAX_STRING_CHARS as c_int,
                ),
            );
            let bCl: c_int = atoi_str(&cstr_to_str(sarg.as_ptr()));
            crate::ai_main::Bot_SetForcedMovement(ctx, bCl, -4000, -1, -1);
        } else if cmd_s.eq_ignore_ascii_case("debugBMove_Right")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
        {
            let mut sarg = [0 as c_char; MAX_STRING_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    1,
                    sarg.as_mut_ptr(),
                    MAX_STRING_CHARS as c_int,
                ),
            );
            let bCl: c_int = atoi_str(&cstr_to_str(sarg.as_ptr()));
            crate::ai_main::Bot_SetForcedMovement(ctx, bCl, -1, 4000, -1);
        } else if cmd_s.eq_ignore_ascii_case("debugBMove_Left")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
        {
            let mut sarg = [0 as c_char; MAX_STRING_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    1,
                    sarg.as_mut_ptr(),
                    MAX_STRING_CHARS as c_int,
                ),
            );
            let bCl: c_int = atoi_str(&cstr_to_str(sarg.as_ptr()));
            crate::ai_main::Bot_SetForcedMovement(ctx, bCl, -1, -4000, -1);
        } else if cmd_s.eq_ignore_ascii_case("debugBMove_Up")
            && CheatsOk(ctx, ctx.entity_id_of(ent).unwrap()) != qfalse
        {
            let mut sarg = [0 as c_char; MAX_STRING_CHARS];
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    1,
                    sarg.as_mut_ptr(),
                    MAX_STRING_CHARS as c_int,
                ),
            );
            let bCl: c_int = atoi_str(&cstr_to_str(sarg.as_ptr()));
            crate::ai_main::Bot_SetForcedMovement(ctx, bCl, -1, -1, 4000);
        }
        // end bot debug cmds
        else if cmd_s.eq_ignore_ascii_case("debugSetSaberMove") {
            Cmd_DebugSetSaberMove_f(ctx, ctx.entity_id_of(ent).unwrap());
        } else if cmd_s.eq_ignore_ascii_case("debugSetBodyAnim") {
            // Canonical in `mp_bg::public::set_anim` (values match: 1, 2).
            // Source: `oracle/codemp/game/bg_public.h:503-504`
            use mp_bg::public::set_anim::{SETANIM_FLAG_HOLD, SETANIM_FLAG_OVERRIDE};
            Cmd_DebugSetBodyAnim_f(
                ctx,
                ctx.entity_id_of(ent).unwrap(),
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
        } else if cmd_s.eq_ignore_ascii_case("debugDismemberment") {
            Cmd_Kill_f(ctx, ctx.entity_id_of(ent).unwrap());
            if (*ent).health < 1 {
                let mut iArg: c_int = 0;
                if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
                    let mut arg = [0 as c_char; MAX_STRING_CHARS];
                    trap::Argv(
                        ctx.engine,
                        mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                            1,
                            arg.as_mut_ptr(),
                            MAX_STRING_CHARS as c_int,
                        ),
                    );
                    if arg[0] != 0 {
                        iArg = atoi_str(&cstr_to_str(arg.as_ptr()));
                    }
                }
                crate::g_combat::DismembermentByNum(ctx, ctx.entity_id_of(ent).unwrap(), iArg);
            }
        } else if cmd_s.eq_ignore_ascii_case("debugDropSaber") {
            let client = (*ent).client;
            if (*client).ps.weapon == WP_SABER
                && (*client).ps.saberEntityNum != 0
                && (*client).ps.saberInFlight == qfalse
            {
                crate::w_saber::saberKnockOutOfHand(
                    ctx,
                    Some(EntityId(((*client).ps.saberEntityNum) as u32)),
                    ctx.entity_id_of(ent),
                    vec3_origin,
                );
            }
        } else if cmd_s.eq_ignore_ascii_case("debugKnockMeDown") {
            let client = (*ent).client;
            if crate::bg_pmove::BG_KnockDownable(&mut (*client).ps) != qfalse {
                (*client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN as c_int;
                (*client).ps.forceDodgeAnim = 0;
                if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
                    (*client).ps.forceHandExtendTime = ctx.world.level.time + 1100;
                    (*client).ps.quickerGetup = qfalse;
                } else {
                    (*client).ps.forceHandExtendTime = ctx.world.level.time + 700;
                    (*client).ps.quickerGetup = qtrue;
                }
            }
        } else if cmd_s.eq_ignore_ascii_case("debugSaberSwitch") {
            let mut targ: *mut gentity_t = core::ptr::null_mut();

            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
                let mut arg = [0 as c_char; MAX_STRING_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        1,
                        arg.as_mut_ptr(),
                        MAX_STRING_CHARS as c_int,
                    ),
                );

                if arg[0] != 0 {
                    let x = atoi_str(&cstr_to_str(arg.as_ptr()));

                    if x >= 0 && x < MAX_CLIENTS as c_int {
                        targ = &mut ctx.world.g_entities[x as usize] as *mut gentity_t;
                    }
                }
            }

            if !targ.is_null() && (*targ).inuse != qfalse && !(*targ).client.is_null() {
                Cmd_ToggleSaber_f(ctx, ctx.entity_id_of(targ).unwrap());
            }
        } else if cmd_s.eq_ignore_ascii_case("debugIKGrab") {
            let mut targ: *mut gentity_t = core::ptr::null_mut();

            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
                let mut arg = [0 as c_char; MAX_STRING_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        1,
                        arg.as_mut_ptr(),
                        MAX_STRING_CHARS as c_int,
                    ),
                );

                if arg[0] != 0 {
                    let x = atoi_str(&cstr_to_str(arg.as_ptr()));

                    if x >= 0 && x < MAX_CLIENTS as c_int {
                        targ = &mut ctx.world.g_entities[x as usize] as *mut gentity_t;
                    }
                }
            }

            if !targ.is_null()
                && (*targ).inuse != qfalse
                && !(*targ).client.is_null()
                && (*ent).s.number != (*targ).s.number
            {
                let targClient = (*targ).client;
                (*targClient).ps.heldByClient = (*ent).s.number + 1;
            }
        } else if cmd_s.eq_ignore_ascii_case("debugIKBeGrabbedBy") {
            let mut targ: *mut gentity_t = core::ptr::null_mut();

            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
                let mut arg = [0 as c_char; MAX_STRING_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        1,
                        arg.as_mut_ptr(),
                        MAX_STRING_CHARS as c_int,
                    ),
                );

                if arg[0] != 0 {
                    let x = atoi_str(&cstr_to_str(arg.as_ptr()));

                    if x >= 0 && x < MAX_CLIENTS as c_int {
                        targ = &mut ctx.world.g_entities[x as usize] as *mut gentity_t;
                    }
                }
            }

            if !targ.is_null()
                && (*targ).inuse != qfalse
                && !(*targ).client.is_null()
                && (*ent).s.number != (*targ).s.number
            {
                let client = (*ent).client;
                (*client).ps.heldByClient = (*targ).s.number + 1;
            }
        } else if cmd_s.eq_ignore_ascii_case("debugIKRelease") {
            let mut targ: *mut gentity_t = core::ptr::null_mut();

            if trap::Argc(ctx.engine, mp_abi::game::syscalls::G_ARGC::GArgcArgs::new()) > 1 {
                let mut arg = [0 as c_char; MAX_STRING_CHARS];
                trap::Argv(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                        1,
                        arg.as_mut_ptr(),
                        MAX_STRING_CHARS as c_int,
                    ),
                );

                if arg[0] != 0 {
                    let x = atoi_str(&cstr_to_str(arg.as_ptr()));

                    if x >= 0 && x < MAX_CLIENTS as c_int {
                        targ = &mut ctx.world.g_entities[x as usize] as *mut gentity_t;
                    }
                }
            }

            if !targ.is_null() && (*targ).inuse != qfalse && !(*targ).client.is_null() {
                let targClient = (*targ).client;
                (*targClient).ps.heldByClient = 0;
            }
        } else if cmd_s.eq_ignore_ascii_case("debugThrow") {
            let client = (*ent).client;
            let mut tr: trace_t = core::mem::zeroed();
            let mut tTo: vec3_t = [0.0; 3];
            let mut fwd: vec3_t = [0.0; 3];

            if (*client).ps.weaponTime > 0
                || (*client).ps.forceHandExtend != HANDEXTEND_NONE as c_int
                || (*client).ps.groundEntityNum == ENTITYNUM_NONE
                || (*ent).health < 1
            {
                return;
            }

            crate::q_math::AngleVectors((*client).ps.viewangles, Some(&mut fwd), None, None);
            tTo[0] = (*client).ps.origin[0] + fwd[0] * 32.0;
            tTo[1] = (*client).ps.origin[1] + fwd[1] * 32.0;
            tTo[2] = (*client).ps.origin[2] + fwd[2] * 32.0;

            trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*client).ps.origin as *const vec3_t,
                    core::ptr::null(),
                    core::ptr::null(),
                    &tTo as *const vec3_t,
                    (*ent).s.number,
                    MASK_PLAYERSOLID,
                ),
            );

            if tr.fraction != 1.0 {
                let other = &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;
                let otherClient = (*other).client;

                if (*other).inuse != qfalse
                    && !(*other).client.is_null()
                    && (*otherClient).ps.forceHandExtend == HANDEXTEND_NONE as c_int
                    && (*otherClient).ps.groundEntityNum != ENTITYNUM_NONE
                    && (*other).health > 0
                    && (*client).ps.origin[2] as c_int == (*otherClient).ps.origin[2] as c_int
                {
                    let pDif: f32 = 40.0;
                    let mut entAngles: vec3_t = [0.0; 3];
                    let mut entDir: vec3_t = [0.0; 3];
                    let mut otherAngles: vec3_t = [0.0; 3];
                    let mut otherDir: vec3_t = [0.0; 3];
                    let mut intendedOrigin: vec3_t = [0.0; 3];
                    let mut boltOrg: vec3_t = [0.0; 3];
                    let mut pBoltOrg: vec3_t = [0.0; 3];
                    let mut tAngles: vec3_t = [0.0; 3];
                    let mut vDif: vec3_t = [0.0; 3];
                    let mut fwd: vec3_t = [0.0; 3];
                    let mut right: vec3_t = [0.0; 3];
                    let mut tr: trace_t = core::mem::zeroed();
                    let mut tr2: trace_t = core::mem::zeroed();

                    crate::q_math::_VectorSubtract(
                        (*otherClient).ps.origin,
                        (*client).ps.origin,
                        &mut otherDir,
                    );
                    crate::q_math::_VectorCopy((*client).ps.viewangles, &mut entAngles);
                    entAngles[YAW] = vectoyaw(otherDir);
                    crate::g_client::SetClientViewAngle(&mut *ent, entAngles);

                    (*client).ps.forceHandExtend = HANDEXTEND_PRETHROW as c_int;
                    (*client).ps.forceHandExtendTime = ctx.world.level.time + 5000;

                    (*client).throwingIndex = (*other).s.number;
                    (*client).doingThrow = ctx.world.level.time + 5000;
                    (*client).beingThrown = 0;

                    crate::q_math::_VectorSubtract(
                        (*client).ps.origin,
                        (*otherClient).ps.origin,
                        &mut entDir,
                    );
                    crate::q_math::_VectorCopy((*otherClient).ps.viewangles, &mut otherAngles);
                    otherAngles[YAW] = vectoyaw(entDir);
                    crate::g_client::SetClientViewAngle(&mut *other, otherAngles);

                    (*otherClient).ps.forceHandExtend = HANDEXTEND_PRETHROWN as c_int;
                    (*otherClient).ps.forceHandExtendTime = ctx.world.level.time + 5000;

                    (*otherClient).throwingIndex = (*ent).s.number;
                    (*otherClient).beingThrown = ctx.world.level.time + 5000;
                    (*otherClient).doingThrow = 0;

                    //Doing this now at a stage in the throw, isntead of initially.
                    //other->client->ps.heldByClient = ent->s.number+1;

                    crate::g_utils::G_EntitySound(
                        ctx,
                        ctx.entity_id_of(other).unwrap(),
                        CHAN_VOICE as c_int,
                        crate::g_utils::G_SoundIndex(c"*pain100.wav".as_ptr()),
                    );
                    crate::g_utils::G_EntitySound(
                        ctx,
                        ctx.entity_id_of(ent).unwrap(),
                        CHAN_VOICE as c_int,
                        crate::g_utils::G_SoundIndex(c"*jump1.wav".as_ptr()),
                    );
                    crate::g_utils::G_Sound(
                        ctx,
                        ctx.entity_id_of(other),
                        CHAN_AUTO as c_int,
                        crate::g_utils::G_SoundIndex(
                            c"sound/movers/objects/objectHit.wav".as_ptr(),
                        ),
                    );

                    //see if we can move to be next to the hand.. if it's not clear, break the throw.
                    crate::q_math::VectorClear(&mut tAngles);
                    tAngles[YAW] = (*client).ps.viewangles[YAW];
                    crate::q_math::_VectorCopy((*client).ps.origin, &mut pBoltOrg);
                    crate::q_math::AngleVectors(tAngles, Some(&mut fwd), Some(&mut right), None);
                    boltOrg[0] = pBoltOrg[0] + fwd[0] * 8.0 + right[0] * pDif;
                    boltOrg[1] = pBoltOrg[1] + fwd[1] * 8.0 + right[1] * pDif;
                    boltOrg[2] = pBoltOrg[2];

                    crate::q_math::_VectorSubtract(boltOrg, pBoltOrg, &mut vDif);
                    crate::q_math::VectorNormalize(&mut vDif);

                    crate::q_math::VectorClear(&mut (*otherClient).ps.velocity);
                    intendedOrigin[0] = pBoltOrg[0] + vDif[0] * pDif;
                    intendedOrigin[1] = pBoltOrg[1] + vDif[1] * pDif;
                    intendedOrigin[2] = (*otherClient).ps.origin[2];

                    trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut tr as *mut trace_t,
                            &intendedOrigin as *const vec3_t,
                            &(*other).r.mins as *const vec3_t,
                            &(*other).r.maxs as *const vec3_t,
                            &intendedOrigin as *const vec3_t,
                            (*other).s.number,
                            (*other).clipmask,
                        ),
                    );
                    trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut tr2 as *mut trace_t,
                            &(*client).ps.origin as *const vec3_t,
                            &(*ent).r.mins as *const vec3_t,
                            &(*ent).r.maxs as *const vec3_t,
                            &intendedOrigin as *const vec3_t,
                            (*ent).s.number,
                            CONTENTS_SOLID,
                        ),
                    );

                    if tr.fraction == 1.0
                        && tr.startsolid == qfalse as u8
                        && tr2.fraction == 1.0
                        && tr2.startsolid == qfalse as u8
                    {
                        crate::q_math::_VectorCopy(intendedOrigin, &mut (*otherClient).ps.origin);
                    } else {
                        //if the guy can't be put here then it's time to break the throw off.
                        let mut oppDir: vec3_t = [0.0; 3];
                        let strength: c_int = 4;

                        (*otherClient).ps.heldByClient = 0;
                        (*otherClient).beingThrown = 0;
                        (*client).doingThrow = 0;

                        (*client).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                        crate::g_utils::G_EntitySound(
                            ctx,
                            ctx.entity_id_of(ent).unwrap(),
                            CHAN_VOICE as c_int,
                            crate::g_utils::G_SoundIndex(c"*pain25.wav".as_ptr()),
                        );

                        (*otherClient).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                        crate::q_math::_VectorSubtract(
                            (*otherClient).ps.origin,
                            (*client).ps.origin,
                            &mut oppDir,
                        );
                        crate::q_math::VectorNormalize(&mut oppDir);
                        (*otherClient).ps.velocity[0] = oppDir[0] * (strength * 40) as f32;
                        (*otherClient).ps.velocity[1] = oppDir[1] * (strength * 40) as f32;
                        (*otherClient).ps.velocity[2] = 150.0;

                        crate::q_math::_VectorSubtract(
                            (*client).ps.origin,
                            (*otherClient).ps.origin,
                            &mut oppDir,
                        );
                        crate::q_math::VectorNormalize(&mut oppDir);
                        (*client).ps.velocity[0] = oppDir[0] * (strength * 40) as f32;
                        (*client).ps.velocity[1] = oppDir[1] * (strength * 40) as f32;
                        (*client).ps.velocity[2] = 150.0;
                    }
                }
            }
        }
        // Dropped dead surface (porting-rules §20): the `#ifdef VM_MEMALLOC_DEBUG`
        // `debugTestAlloc` branch — `g_cmds.c:4013-4055`. VM_MEMALLOC_DEBUG is
        // never defined in any build we target, so it is not compiled in.
        else if cmd_s.eq_ignore_ascii_case("debugShipDamage") {
            let mut arg = [0 as c_char; MAX_STRING_CHARS];
            let mut arg2 = [0 as c_char; MAX_STRING_CHARS];

            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    1,
                    arg.as_mut_ptr(),
                    MAX_STRING_CHARS as c_int,
                ),
            );
            trap::Argv(
                ctx.engine,
                mp_abi::game::syscalls::G_ARGV::GArgvArgs::new(
                    2,
                    arg2.as_mut_ptr(),
                    MAX_STRING_CHARS as c_int,
                ),
            );
            let shipSurf = SHIPSURF_FRONT + atoi_str(&cstr_to_str(arg.as_ptr()));
            let damageLevel = atoi_str(&cstr_to_str(arg2.as_ptr()));

            crate::g_vehicles::G_SetVehDamageFlags(
                ctx,
                EntityId((*ent).s.m_iVehicleNum as u32),
                shipSurf,
                damageLevel,
            );
        } else {
            if cmd_s.eq_ignore_ascii_case("addbot") {
                // because addbot isn't a recognized command unless you're the
                // server, but it is in the menus regardless
                let m = crate::g_main::G_GetStringEdString(
                    ctx,
                    c"MP_SVGAME".as_ptr() as *mut c_char,
                    c"ONLY_ADD_BOTS_AS_SERVER".as_ptr() as *mut c_char,
                );
                let s = format!("print \"{}.\n\"", cstr_to_str(m));
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        clientNum,
                        cstr(&s),
                    ),
                );
            } else {
                let s = format!("print \"unknown cmd {}\n\"", cmd_s);
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        clientNum,
                        cstr(&s),
                    ),
                );
            }
        }
    }
}
