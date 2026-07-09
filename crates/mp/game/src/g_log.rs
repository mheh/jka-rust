//! Weapon-statistics logging (`oracle/oracle/codemp/game/g_log.c`).
//!
//! `g_log.c` opens with an unconditional `#define LOGGING_WEAPONS` (line 3), so
//! every `#ifdef LOGGING_WEAPONS` block in the file compiles into the shipped
//! jampgame DLL — the per-player pickup/fire/damage/kill/death counters and the
//! match-end `G_LogWeaponOutput` dump are all live code, not dead stubs. The
//! file-scope `G_WeaponLog*` accumulator arrays live in `GameGlobals` (reached
//! via `ctx.world.globals`). `G_LogWeaponOutput` and `CalculateAwards` stay
//! gated at *runtime* by the `g_statLog` cvar (default `"0"`), not at compile
//! time.
//! Source: `oracle/oracle/codemp/game/g_log.c:1-1752`
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_combat::modNames;
use crate::g_main::G_LogPrintf;
use crate::prelude::*;
use crate::q_shared::Info_ValueForKey;
use crate::w_saber::HasSetSaberOnly;
use mp_abi::game::syscalls::G_FS_FCLOSE_FILE::GFsFcloseFileArgs;
use mp_abi::game::syscalls::G_FS_FOPEN_FILE::GFsFopenFileArgs;
use mp_abi::game::syscalls::G_FS_WRITE::GFsWriteArgs;
use mp_abi::game::syscalls::G_GET_SERVERINFO::GGetServerinfoArgs;
use std::ffi::{CStr, CString};

/// Raven `char *weaponNameFromIndex[WP_NUM_WEAPONS]` — display names for the
/// per-weapon log tables written by `G_LogWeaponOutput`. Raven's initializer
/// list has only 16 entries but the array is `[WP_NUM_WEAPONS]` (19), so the
/// trailing 3 slots are C zero-init NULL (glibc `printf("%s", NULL)` renders
/// those as `(null)`). The names are also index-shifted relative to the modern
/// `weapon_t` enum (they predate `WP_MELEE`) — a faithful Raven quirk, preserved.
/// Source: `oracle/oracle/codemp/game/g_log.c:79-97`
const weaponNameFromIndex: [*const c_char; WP_NUM_WEAPONS as usize] = [
    c"No Weapon".as_ptr(),
    c"Stun Baton".as_ptr(),
    c"Saber".as_ptr(),
    c"Bryar Pistol".as_ptr(),
    c"Blaster".as_ptr(),
    c"Disruptor".as_ptr(),
    c"Bowcaster".as_ptr(),
    c"Repeater".as_ptr(),
    c"Demp2".as_ptr(),
    c"Flechette".as_ptr(),
    c"Rocket Launcher".as_ptr(),
    c"Thermal".as_ptr(),
    c"Tripmine".as_ptr(),
    c"Detpack".as_ptr(),
    c"Emplaced gun".as_ptr(),
    c"Turret".as_ptr(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
];

/// Raven `G_LogWeaponInit` — zeroes every per-player weapon-log accumulator at
/// level start. Faithfully omits `G_WeaponLogClientTouch` (the oracle's
/// `memset` list skips it).
/// Source: `oracle/oracle/codemp/game/g_log.c:108-121`
pub fn G_LogWeaponInit(ctx: GameContext<'_>) {
    unsafe {
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogPickups = [[0; WP_NUM_WEAPONS as usize]; MAX_CLIENTS];
        g.G_WeaponLogFired = [[0; WP_NUM_WEAPONS as usize]; MAX_CLIENTS];
        g.G_WeaponLogDamage.0 = [[0; meansOfDeath_t::MOD_MAX as usize]; MAX_CLIENTS];
        g.G_WeaponLogKills.0 = [[0; meansOfDeath_t::MOD_MAX as usize]; MAX_CLIENTS];
        g.G_WeaponLogDeaths = [[0; WP_NUM_WEAPONS as usize]; MAX_CLIENTS];
        g.G_WeaponLogFrags = [[0; MAX_CLIENTS]; MAX_CLIENTS];
        g.G_WeaponLogTime = [[0; WP_NUM_WEAPONS as usize]; MAX_CLIENTS];
        g.G_WeaponLogLastTime = [0; MAX_CLIENTS];
        g.G_WeaponLogPowerups = [[0; HI_NUM_HOLDABLE as usize]; MAX_CLIENTS];
        g.G_WeaponLogItems = [[0; PW_NUM_POWERUPS as usize]; MAX_CLIENTS];
    }
}

/// Raven `G_LogWeaponPickup`.
/// Source: `oracle/oracle/codemp/game/g_log.c:123-129`
pub fn G_LogWeaponPickup(ctx: GameContext<'_>, client: c_int, weaponid: c_int) {
    unsafe {
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogPickups[client as usize][weaponid as usize] += 1;
        g.G_WeaponLogClientTouch[client as usize] = qtrue;
    }
}

/// Raven `G_LogWeaponFire` — records a shot and the (capped at 5s) time since
/// this client's last logged weapon action.
/// Source: `oracle/oracle/codemp/game/g_log.c:131-145`
pub fn G_LogWeaponFire(ctx: GameContext<'_>, client: c_int, weaponid: c_int) {
    unsafe {
        let time = (*ctx.world).level.time;
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogFired[client as usize][weaponid as usize] += 1;
        let dur = time - g.G_WeaponLogLastTime[client as usize];
        if dur > 5000 {
            // 5 second max.
            g.G_WeaponLogTime[client as usize][weaponid as usize] += 5000;
        } else {
            g.G_WeaponLogTime[client as usize][weaponid as usize] += dur;
        }
        g.G_WeaponLogLastTime[client as usize] = time;
        g.G_WeaponLogClientTouch[client as usize] = qtrue;
    }
}

/// Raven `G_LogWeaponDamage`.
/// Source: `oracle/oracle/codemp/game/g_log.c:147-155`
pub fn G_LogWeaponDamage(ctx: GameContext<'_>, client: c_int, r#mod: c_int, amount: c_int) {
    if client >= MAX_CLIENTS as c_int {
        return;
    }
    unsafe {
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogDamage.0[client as usize][r#mod as usize] += amount;
        g.G_WeaponLogClientTouch[client as usize] = qtrue;
    }
}

/// Raven `G_LogWeaponKill`.
/// Source: `oracle/oracle/codemp/game/g_log.c:157-165`
pub fn G_LogWeaponKill(ctx: GameContext<'_>, client: c_int, r#mod: c_int) {
    if client >= MAX_CLIENTS as c_int {
        return;
    }
    unsafe {
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogKills.0[client as usize][r#mod as usize] += 1;
        g.G_WeaponLogClientTouch[client as usize] = qtrue;
    }
}

/// Raven `G_LogWeaponFrag`.
/// Source: `oracle/oracle/codemp/game/g_log.c:167-175`
pub fn G_LogWeaponFrag(ctx: GameContext<'_>, attacker: c_int, deadguy: c_int) {
    if attacker >= MAX_CLIENTS as c_int || deadguy >= MAX_CLIENTS as c_int {
        return;
    }
    unsafe {
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogFrags[attacker as usize][deadguy as usize] += 1;
        g.G_WeaponLogClientTouch[attacker as usize] = qtrue;
    }
}

/// Raven `G_LogWeaponDeath`.
/// Source: `oracle/oracle/codemp/game/g_log.c:177-185`
pub fn G_LogWeaponDeath(ctx: GameContext<'_>, client: c_int, weaponid: c_int) {
    if client >= MAX_CLIENTS as c_int {
        return;
    }
    unsafe {
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogDeaths[client as usize][weaponid as usize] += 1;
        g.G_WeaponLogClientTouch[client as usize] = qtrue;
    }
}

/// Raven `G_LogWeaponPowerup`.
/// Source: `oracle/oracle/codemp/game/g_log.c:187-195`
pub fn G_LogWeaponPowerup(ctx: GameContext<'_>, client: c_int, powerupid: c_int) {
    if client >= MAX_CLIENTS as c_int {
        return;
    }
    unsafe {
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogPowerups[client as usize][powerupid as usize] += 1;
        g.G_WeaponLogClientTouch[client as usize] = qtrue;
    }
}

/// Raven `G_LogWeaponItem`.
/// Source: `oracle/oracle/codemp/game/g_log.c:197-205`
pub fn G_LogWeaponItem(ctx: GameContext<'_>, client: c_int, itemid: c_int) {
    if client >= MAX_CLIENTS as c_int {
        return;
    }
    unsafe {
        let g = &mut (*ctx.world).globals;
        g.G_WeaponLogItems[client as usize][itemid as usize] += 1;
        g.G_WeaponLogClientTouch[client as usize] = qtrue;
    }
}

/// Raven `G_LogWeaponOutput` — prints the aggregate weapon statistics to the
/// console log and appends the per-player tables to `g_statLogFile`. Runtime-
/// gated by the `g_statLog` cvar: returns immediately when it is `0`.
/// Source: `oracle/oracle/codemp/game/g_log.c:227-821`
pub fn G_LogWeaponOutput(ctx: GameContext<'_>) {
    const WPN: usize = WP_NUM_WEAPONS as usize;
    const MODN: usize = meansOfDeath_t::MOD_MAX as usize;
    unsafe {
        if (*ctx.world).cvars.g_statLog.integer == 0 {
            return;
        }

        G_LogPrintf(
            ctx,
            cstr("*****************************Weapon Log:\n").as_ptr(),
        );

        let mut totalpickups = [0i32; WPN];
        let mut totaltime = [0i32; WPN];
        let mut totaldeaths = [0i32; WPN];
        let mut totaldamage_mod = [0i32; MODN];
        let mut totalkills_mod = [0i32; MODN];
        let mut totaldamage = [0i32; WPN];
        let mut totalkills = [0i32; WPN];
        let mut totalshots = [0i32; WPN];

        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                // Ignore any entity/clients we don't care about!
                for j in 0..WPN {
                    totalpickups[j] += (*ctx.world).globals.G_WeaponLogPickups[i][j];
                    totaltime[j] += (*ctx.world).globals.G_WeaponLogTime[i][j];
                    totaldeaths[j] += (*ctx.world).globals.G_WeaponLogDeaths[i][j];
                    totalshots[j] += (*ctx.world).globals.G_WeaponLogFired[i][j];
                }
                for j in 0..MODN {
                    totaldamage_mod[j] += (*ctx.world).globals.G_WeaponLogDamage.0[i][j];
                    totalkills_mod[j] += (*ctx.world).globals.G_WeaponLogKills.0[i][j];
                }
            }
        }

        // Now total the weapon data from the MOD data.
        for j in 0..MODN {
            if j <= MOD_SENTRY as usize {
                let curwp = weaponFromMOD[j] as usize;
                totaldamage[curwp] += totaldamage_mod[j];
                totalkills[curwp] += totalkills_mod[j];
            }
        }

        G_LogPrintf(ctx, cstr("\n****Data by Weapon:\n").as_ptr());
        for j in 0..WPN {
            G_LogPrintf(
                ctx,
                cstr(&format!(
                    "{:>15}:  Pickups: {:>4},  Time:  {:>5},  Deaths: {:>5}\n",
                    weaponName(j),
                    totalpickups[j],
                    totaltime[j] / 1000,
                    totaldeaths[j]
                ))
                .as_ptr(),
            );
        }

        G_LogPrintf(ctx, cstr("\n****Combat Data by Weapon:\n").as_ptr());
        for j in 0..WPN {
            let pershot = if totalshots[j] > 0 {
                totaldamage[j] as f32 / totalshots[j] as f32
            } else {
                0.0
            };
            G_LogPrintf(
                ctx,
                cstr(&format!(
                    "{:>15}:  Damage: {:>6},  Kills: {:>5},  Dmg per Shot: {:.6}\n",
                    weaponName(j), totaldamage[j], totalkills[j], pershot
                ))
                .as_ptr(),
            );
        }

        G_LogPrintf(ctx, cstr("\n****Combat Data By Damage Type:\n").as_ptr());
        for j in 0..MODN {
            G_LogPrintf(
                ctx,
                cstr(&format!(
                    "{:>25}:  Damage: {:>6},  Kills: {:>5}\n",
                    modName(j),
                    totaldamage_mod[j],
                    totalkills_mod[j]
                ))
                .as_ptr(),
            );
        }

        G_LogPrintf(ctx, cstr("\n").as_ptr());

        // Write the whole weapon statistic log out to a file.
        let mut weaponfile: fileHandle_t = 0;
        let log_name = CStr::from_ptr((*ctx.world).cvars.g_statLogFile.string.as_ptr()).to_owned();
        trap::FS_FOpenFile(
            ctx.engine,
            GFsFopenFileArgs::new(log_name, &mut weaponfile, FS_APPEND),
        );
        if weaponfile == 0 {
            // failed to open file, let's not crash, shall we?
            return;
        }

        let write = |s: &str| {
            let b = s.as_bytes();
            trap::FS_Write(
                ctx.engine,
                GFsWriteArgs::new(b.as_ptr(), b.len() as c_int, weaponfile),
            );
        };

        // Write out the level name.
        let mut info: [c_char; 1024] = [0; 1024];
        trap::GetServerinfo(ctx.engine, GGetServerinfoArgs::new(info.as_mut_ptr(), 1024));
        let mapname_full =
            cstr_to_str(Info_ValueForKey(info.as_ptr(), cstr("mapname").as_ptr())).to_string();
        // strncpy(mapname, ..., sizeof(mapname)-1) -> 127-byte cap.
        let mapname: String = mapname_full.chars().take(127).collect();

        write(&format!("\n\n\nLevel:\t{}\n\n\n", mapname));

        // Helper: the player's netname (or "<Unknown>" when clientless).
        let player_name = |i: usize| -> String {
            let pc = (*ctx.world).g_entities[i].client as *mut gclient_t;
            if !pc.is_null() {
                cstr_to_str((*pc).pers.netname.as_ptr()).to_string()
            } else {
                "<Unknown>".to_string()
            }
        };

        // --- Weapon Pickups per Player ---
        write("Weapon Pickups per Player:\n\n");
        write("Player");
        for j in 0..WPN {
            write(&format!("\t{}", weaponName(j)));
        }
        write("\n");
        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                write(&player_name(i));
                for j in 0..WPN {
                    write(&format!("\t{}", (*ctx.world).globals.G_WeaponLogPickups[i][j]));
                }
                write("\n");
            }
        }
        write("\n***TOTAL:");
        for j in 0..WPN {
            write(&format!("\t{}", totalpickups[j]));
        }
        write("\n\n\n");

        // --- Weapon Shots per Player ---
        write("Weapon Shots per Player:\n\n");
        write("Player");
        for j in 0..WPN {
            write(&format!("\t{}", weaponName(j)));
        }
        write("\n");
        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                write(&player_name(i));
                for j in 0..WPN {
                    write(&format!("\t{}", (*ctx.world).globals.G_WeaponLogFired[i][j]));
                }
                write("\n");
            }
        }
        write("\n***TOTAL:");
        for j in 0..WPN {
            write(&format!("\t{}", totalshots[j]));
        }
        write("\n\n\n");

        // --- Weapon Use Time per Player ---
        write("Weapon Use Time per Player:\n\n");
        write("Player");
        for j in 0..WPN {
            write(&format!("\t{}", weaponName(j)));
        }
        write("\n");
        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                write(&player_name(i));
                for j in 0..WPN {
                    write(&format!("\t{}", (*ctx.world).globals.G_WeaponLogTime[i][j]));
                }
                write("\n");
            }
        }
        write("\n***TOTAL:");
        for j in 0..WPN {
            write(&format!("\t{}", totaltime[j]));
        }
        write("\n\n\n");

        // --- Weapon Deaths per Player ---
        write("Weapon Deaths per Player:\n\n");
        write("Player");
        for j in 0..WPN {
            write(&format!("\t{}", weaponName(j)));
        }
        write("\n");
        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                write(&player_name(i));
                for j in 0..WPN {
                    write(&format!("\t{}", (*ctx.world).globals.G_WeaponLogDeaths[i][j]));
                }
                write("\n");
            }
        }
        write("\n***TOTAL:");
        for j in 0..WPN {
            write(&format!("\t{}", totaldeaths[j]));
        }
        write("\n\n\n");

        // --- Weapon Damage per Player (MOD damage folded onto weapons) ---
        write("Weapon Damage per Player:\n\n");
        write("Player");
        for j in 0..WPN {
            write(&format!("\t{}", weaponName(j)));
        }
        write("\n");
        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                // Grab the totals from the damage types for the player and map them to the weapons.
                let mut percharacter = [0i32; WPN];
                for j in 0..MODN {
                    if j <= MOD_SENTRY as usize {
                        let curwp = weaponFromMOD[j] as usize;
                        percharacter[curwp] += (*ctx.world).globals.G_WeaponLogDamage.0[i][j];
                    }
                }
                write(&player_name(i));
                for j in 0..WPN {
                    write(&format!("\t{}", percharacter[j]));
                }
                write("\n");
            }
        }
        write("\n***TOTAL:");
        for j in 0..WPN {
            write(&format!("\t{}", totaldamage[j]));
        }
        write("\n\n\n");

        // --- Weapon Kills per Player (MOD kills folded onto weapons) ---
        write("Weapon Kills per Player:\n\n");
        write("Player");
        for j in 0..WPN {
            write(&format!("\t{}", weaponName(j)));
        }
        write("\n");
        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                let mut percharacter = [0i32; WPN];
                for j in 0..MODN {
                    if j <= MOD_SENTRY as usize {
                        let curwp = weaponFromMOD[j] as usize;
                        percharacter[curwp] += (*ctx.world).globals.G_WeaponLogKills.0[i][j];
                    }
                }
                write(&player_name(i));
                for j in 0..WPN {
                    write(&format!("\t{}", percharacter[j]));
                }
                write("\n");
            }
        }
        write("\n***TOTAL:");
        for j in 0..WPN {
            write(&format!("\t{}", totalkills[j]));
        }
        write("\n\n\n");

        // --- Typed Damage per Player ---
        write("Typed Damage per Player:\n\n");
        write("Player");
        for j in 0..MODN {
            write(&format!("\t{}", modName(j)));
        }
        write("\n");
        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                write(&player_name(i));
                for j in 0..MODN {
                    write(&format!("\t{}", (*ctx.world).globals.G_WeaponLogDamage.0[i][j]));
                }
                write("\n");
            }
        }
        write("\n***TOTAL:");
        for j in 0..MODN {
            write(&format!("\t{}", totaldamage_mod[j]));
        }
        write("\n\n\n");

        // --- Damage-Typed Kills per Player ---
        write("Damage-Typed Kills per Player:\n\n");
        write("Player");
        for j in 0..MODN {
            write(&format!("\t{}", modName(j)));
        }
        write("\n");
        for i in 0..MAX_CLIENTS {
            if (*ctx.world).globals.G_WeaponLogClientTouch[i] != qfalse {
                write(&player_name(i));
                for j in 0..MODN {
                    write(&format!("\t{}", (*ctx.world).globals.G_WeaponLogKills.0[i][j]));
                }
                write("\n");
            }
        }
        write("\n***TOTAL:");
        for j in 0..MODN {
            write(&format!("\t{}", totalkills_mod[j]));
        }
        write("\n\n\n");

        trap::FS_FCloseFile(ctx.engine, GFsFcloseFileArgs::new(weaponfile));
    }
}

/// `modNames[j]` rendered for output. Raven leaves the last three `[MOD_MAX]`
/// slots NULL; glibc `printf("%s", NULL)` renders those as `(null)`.
fn modName(j: usize) -> String {
    let p = modNames[j];
    if p.is_null() {
        "(null)".to_string()
    } else {
        unsafe { cstr_to_str(p) }
    }
}

/// `weaponName(j)` rendered for output, with the same `(null)` fallback
/// for the trailing zero-init slots.
fn weaponName(j: usize) -> String {
    let p = weaponNameFromIndex[j];
    if p.is_null() {
        "(null)".to_string()
    } else {
        unsafe { cstr_to_str(p) }
    }
}

/// Raven `CalculateEfficiency` — awards the accuracy leader if their hit ratio
/// tops 50%. Writes the winner's efficiency percentage through `efficiency`.
/// Source: `oracle/oracle/codemp/game/g_log.c:824-863`
pub fn CalculateEfficiency(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    efficiency: *mut c_int,
) -> qboolean {
    unsafe {
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        let mut f_best_ratio = 0.0f32;
        let mut n_best_player: c_int = -1;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            if player.inuse == qfalse {
                continue;
            }
            let pc = player.client as *mut gclient_t;
            let n_shots_fired = (*pc).accuracy_shots;
            let n_shots_hit = (*pc).accuracy_hits;
            let f_accuracy_ratio = n_shots_hit as f32 / n_shots_fired as f32;
            if f_accuracy_ratio > f_best_ratio {
                f_best_ratio = f_accuracy_ratio;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            let temp_eff = (100.0 * f_best_ratio) as c_int;
            if temp_eff > 50 {
                *efficiency = temp_eff;
                return qtrue;
            }
            return qfalse;
        }
        qfalse
    }
}

/// Raven `CalculateSharpshooter` — awards the sniper-kills leader if they and
/// the passed player both averaged at least one sniper kill per minute.
/// Source: `oracle/oracle/codemp/game/g_log.c:866-903`
pub fn CalculateSharpshooter(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    frags: *mut c_int,
) -> qboolean {
    unsafe {
        let ec = (*ent).client as *mut gclient_t;
        let play_time = ((*ctx.world).level.time - (*ec).pers.enterTime) / 60000;
        let ent_idx = (*ent).s.number as usize;

        // if this guy didn't get one kill per minute, reject him right now
        let my_kills = (*ctx.world).globals.G_WeaponLogKills.0[ent_idx]
            [meansOfDeath_t::MOD_DISRUPTOR_SNIPER as usize];
        if (my_kills as f32) / (play_time as f32) < 1.0 {
            return qfalse;
        }

        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        let mut n_most_kills: c_int = 0;
        let mut n_best_player: c_int = -1;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            if player.inuse == qfalse {
                continue;
            }
            let n_kills = (*ctx.world).globals.G_WeaponLogKills.0[i as usize]
                [meansOfDeath_t::MOD_DISRUPTOR_SNIPER as usize];
            if n_kills > n_most_kills {
                n_most_kills = n_kills;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            *frags = n_most_kills;
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateUntouchable` — the "perfect game" award: at least two points
/// per minute and never killed.
/// Source: `oracle/oracle/codemp/game/g_log.c:906-928`
pub fn CalculateUntouchable(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let ec = (*ent).client as *mut gclient_t;
        let play_time = ((*ctx.world).level.time - (*ec).pers.enterTime) / 60000;

        if (*ctx.world).cvars.g_gametype.integer == GT_JEDIMASTER && (*ec).ps.isJediMaster != qfalse
        {
            // Jedi Master (was Borg queen) can only be killed once anyway
            return qfalse;
        }
        // MUST HAVE ACHIEVED 2 KILLS PER MINUTE
        if ((*ec).ps.persistant[persEnum_t::PERS_SCORE as usize] as f32) / (play_time as f32) < 2.0
            || play_time == 0
        {
            return qfalse;
        }
        // if this guy was never killed... Award Away!!!
        if (*ec).ps.persistant[persEnum_t::PERS_KILLED as usize] == 0 {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateLogistics` — awards the player who used the most items and
/// powerups, provided they used at least four distinct kinds.
/// Source: `oracle/oracle/codemp/game/g_log.c:931-982`
pub fn CalculateLogistics(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    stuffUsed: *mut c_int,
) -> qboolean {
    unsafe {
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        let mut n_best_player: c_int = -1;
        let mut n_most_stuff_used: c_int = 0;
        let mut n_most_different: c_int = 0;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            if player.inuse == qfalse {
                continue;
            }
            let mut n_stuff_used: c_int = 0;
            let mut n_different: c_int = 0;
            for j in (HI_NONE as usize + 1)..HI_NUM_HOLDABLE as usize {
                if (*ctx.world).globals.G_WeaponLogPowerups[i as usize][j] != 0 {
                    n_different += 1;
                }
                n_stuff_used += (*ctx.world).globals.G_WeaponLogPowerups[i as usize][j];
            }
            for j in (PW_NONE as usize + 1)..PW_NUM_POWERUPS as usize {
                if (*ctx.world).globals.G_WeaponLogItems[i as usize][j] != 0 {
                    n_different += 1;
                }
                n_stuff_used += (*ctx.world).globals.G_WeaponLogItems[i as usize][j];
            }
            if n_different >= 4 && n_different >= n_most_different && n_stuff_used > n_most_stuff_used
            {
                n_most_different = n_different;
                n_most_stuff_used = n_stuff_used;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            *stuffUsed = n_most_different;
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateTactician` — awards the player who got a kill with every
/// weapon available on the map (and the most such kills), given a two-per-minute
/// score rate and no saber-only / Jedi-Master restriction.
/// Source: `oracle/oracle/codemp/game/g_log.c:988-1081`
pub fn CalculateTactician(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    kills: *mut c_int,
) -> qboolean {
    unsafe {
        let ec = (*ent).client as *mut gclient_t;
        let play_time = ((*ctx.world).level.time - (*ec).pers.enterTime) / 60000;

        if HasSetSaberOnly(ctx) != qfalse {
            // duh, only 1 weapon
            return qfalse;
        }
        if (*ctx.world).cvars.g_gametype.integer == GT_JEDIMASTER && (*ec).ps.isJediMaster != qfalse
        {
            // Jedi Master (was Borg queen) has only 1 weapon
            return qfalse;
        }
        // MUST HAVE ACHIEVED 2 KILLS PER MINUTE
        if (play_time as f32) < 0.3 {
            return qfalse;
        }
        if ((*ec).ps.persistant[persEnum_t::PERS_SCORE as usize] as f32) / (play_time as f32) < 2.0 {
            return qfalse;
        }

        let maxclients = (*ctx.world).cvars.g_maxclients.integer;

        // FOR EVERY WEAPON, ADD UP TOTAL PICKUPS
        let mut was_picked_up = [0i32; WP_NUM_WEAPONS as usize];
        for person in 0..maxclients {
            for weapon in 0..WP_NUM_WEAPONS as usize {
                if (*ctx.world).globals.G_WeaponLogPickups[person as usize][weapon] > 0 {
                    was_picked_up[weapon] += 1;
                }
            }
        }

        let mut n_most_kills: c_int = 0;
        let mut n_best_player: c_int = -1;

        // FOR EVERY PERSON, CHECK FOR CANDIDATE
        for person in 0..maxclients {
            let player = &(*ctx.world).g_entities[person as usize];
            if player.inuse == qfalse {
                continue;
            }
            let mut n_kills: c_int = 0;
            // One extra slot: Raven's `while` loop reads killsWithWeapon[weapon]
            // one-past-end (a UB stack read); the zero-init slot gives that read a
            // defined value of 0 (porting-rules §19).
            let mut kills_with_weapon = [0i32; WP_NUM_WEAPONS as usize + 1];
            for i in 0..meansOfDeath_t::MOD_MAX as usize {
                let weapon = weaponFromMOD[i] as usize;
                kills_with_weapon[weapon] += (*ctx.world).globals.G_WeaponLogKills.0[person as usize][i];
            }

            let mut weapon = WP_STUN_BATON;
            // keep looking through weapons if weapon is not on map, or if it is and we used it
            while weapon < WP_NUM_WEAPONS
                && (was_picked_up[weapon as usize] == 0 || kills_with_weapon[weapon as usize] > 0)
            {
                weapon += 1;
                n_kills += kills_with_weapon[weapon as usize];
            }
            if weapon >= WP_NUM_WEAPONS && n_kills > n_most_kills {
                n_most_kills = n_kills;
                n_best_player = person;
            }
        }

        if n_best_player == (*ent).s.number {
            *kills = n_most_kills;
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateDemolitionist` — awards the explosive-kills leader, provided
/// they averaged at least two explosive kills per minute (`playTime` measured
/// from the passed player, a faithful Raven quirk).
/// Source: `oracle/oracle/codemp/game/g_log.c:1087-1134`
pub fn CalculateDemolitionist(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    kills: *mut c_int,
) -> qboolean {
    unsafe {
        let ec = (*ent).client as *mut gclient_t;
        let play_time = ((*ctx.world).level.time - (*ec).pers.enterTime) / 60000;
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        let mut n_most_kills: c_int = 0;
        let mut n_best_player: c_int = -1;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            if player.inuse == qfalse {
                continue;
            }
            let k = (*ctx.world).globals.G_WeaponLogKills.0[i as usize];
            let mut n_kills = k[meansOfDeath_t::MOD_THERMAL as usize];
            n_kills += k[meansOfDeath_t::MOD_THERMAL_SPLASH as usize];
            n_kills += k[meansOfDeath_t::MOD_ROCKET as usize];
            n_kills += k[meansOfDeath_t::MOD_ROCKET_SPLASH as usize];
            n_kills += k[meansOfDeath_t::MOD_ROCKET_HOMING as usize];
            n_kills += k[meansOfDeath_t::MOD_ROCKET_HOMING_SPLASH as usize];
            n_kills += k[meansOfDeath_t::MOD_TRIP_MINE_SPLASH as usize];
            n_kills += k[meansOfDeath_t::MOD_TIMED_MINE_SPLASH as usize];
            n_kills += k[meansOfDeath_t::MOD_DET_PACK_SPLASH as usize];

            // if this guy didn't get two explosive kills per minute, reject him right now
            if (n_kills as f32) / (play_time as f32) < 2.0 {
                continue;
            }
            if n_kills > n_most_kills {
                n_most_kills = n_kills;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            *kills = n_most_kills;
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateStreak`.
///
/// Raven: the live body is `#if 0`'d out (dead streak-award code — the
/// comment above it says "No streak calculation, at least for now"); the
/// only compiled statement is the trailing `return 0`.
/// Source: `oracle/oracle/codemp/game/g_log.c:1136-1158`
pub fn CalculateStreak(ent: *mut gentity_t) -> c_int {
    0
}

/// Raven `CalculateTeamMVP`.
///
/// Source: `oracle/oracle/codemp/game/g_log.c:1160-1188`
pub fn CalculateTeamMVP(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let team = (*client).ps.persistant[persEnum_t::PERS_TEAM as usize];
        let mut n_best_player: c_int = -1;
        let mut n_highest_score: c_int = 0;
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            let pc = player.client as *mut gclient_t;
            if player.inuse == qfalse || (*pc).ps.persistant[persEnum_t::PERS_TEAM as usize] != team
            {
                continue;
            }
            let n_score = (*pc).ps.persistant[persEnum_t::PERS_SCORE as usize];
            if n_score > n_highest_score {
                n_highest_score = n_score;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateTeamMVPByRank`.
///
/// Raven: the `team == PERS_TEAM && PERS_CLASS == PC_BORG` Borg-queen
/// special case is commented out in the oracle (`#if 0`-equivalent
/// `/* */`) — dead source, not ported.
/// Source: `oracle/oracle/codemp/game/g_log.c:1190-1240`
pub fn CalculateTeamMVPByRank(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let team = (*client).ps.persistant[persEnum_t::PERS_RANK as usize] + 1;
        let b_tied = team == 3;
        let mut n_best_player: c_int = -1;
        let mut n_highest_score: c_int = 0;
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            if player.inuse == qfalse {
                continue;
            }
            let pc = player.client as *mut gclient_t;
            if !b_tied && (*pc).ps.persistant[persEnum_t::PERS_TEAM as usize] != team {
                continue;
            }
            let n_score = (*pc).ps.persistant[persEnum_t::PERS_SCORE as usize];
            if n_score > n_highest_score {
                n_highest_score = n_score;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateTeamDefender`.
///
/// Raven: the `CalculateTeamMVP(ent)` short-circuit is commented out in the
/// oracle — dead source, not ported.
/// Source: `oracle/oracle/codemp/game/g_log.c:1242-1276`
pub fn CalculateTeamDefender(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let team = (*client).ps.persistant[persEnum_t::PERS_TEAM as usize];
        let mut n_best_player: c_int = -1;
        let mut n_highest_score: c_int = 0;
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            let pc = player.client as *mut gclient_t;
            if player.inuse == qfalse || (*pc).ps.persistant[persEnum_t::PERS_TEAM as usize] != team
            {
                continue;
            }
            let n_score = (*pc).pers.teamState.basedefense;
            if n_score > n_highest_score {
                n_highest_score = n_score;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateTeamWarrior`.
///
/// Raven: the `CalculateTeamMVP(ent) || CalculateTeamDefender(ent)`
/// short-circuit is commented out in the oracle — dead source, not ported.
/// Source: `oracle/oracle/codemp/game/g_log.c:1278-1312`
pub fn CalculateTeamWarrior(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let team = (*client).ps.persistant[persEnum_t::PERS_TEAM as usize];
        let mut n_best_player: c_int = -1;
        let mut n_highest_score: c_int = 0;
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            let pc = player.client as *mut gclient_t;
            if player.inuse == qfalse || (*pc).ps.persistant[persEnum_t::PERS_TEAM as usize] != team
            {
                continue;
            }
            let n_score = (*pc).ps.persistant[persEnum_t::PERS_SCORE as usize];
            if n_score > n_highest_score {
                n_highest_score = n_score;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateTeamCarrier`.
///
/// Raven: the `CalculateTeamMVP/Defender/Warrior(ent)` short-circuit is
/// commented out in the oracle — dead source, not ported.
/// Source: `oracle/oracle/codemp/game/g_log.c:1314-1348`
pub fn CalculateTeamCarrier(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let team = (*client).ps.persistant[persEnum_t::PERS_TEAM as usize];
        let mut n_best_player: c_int = -1;
        let mut n_highest_score: c_int = 0;
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            let pc = player.client as *mut gclient_t;
            if player.inuse == qfalse || (*pc).ps.persistant[persEnum_t::PERS_TEAM as usize] != team
            {
                continue;
            }
            let n_score = (*pc).pers.teamState.captures;
            if n_score > n_highest_score {
                n_highest_score = n_score;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateTeamInterceptor`.
///
/// Raven: the `CalculateTeamMVP/Defender/Warrior/Carrier(ent)`
/// short-circuit is commented out in the oracle — dead source, not ported.
/// Source: `oracle/oracle/codemp/game/g_log.c:1350-1386`
pub fn CalculateTeamInterceptor(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let team = (*client).ps.persistant[persEnum_t::PERS_TEAM as usize];
        let mut n_best_player: c_int = -1;
        let mut n_highest_score: c_int = 0;
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            let pc = player.client as *mut gclient_t;
            if player.inuse == qfalse || (*pc).ps.persistant[persEnum_t::PERS_TEAM as usize] != team
            {
                continue;
            }
            let mut n_score = (*pc).pers.teamState.flagrecovery;
            n_score += (*pc).pers.teamState.fragcarrier;
            if n_score > n_highest_score {
                n_highest_score = n_score;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `CalculateTeamRedShirt`.
///
/// Raven: the `CalculateTeamMVP/Defender/Warrior/Carrier/Interceptor(ent)`
/// short-circuit is commented out in the oracle — dead source, not ported.
/// Source: `oracle/oracle/codemp/game/g_log.c:1388-1424`
pub fn CalculateTeamRedShirt(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let team = (*client).ps.persistant[persEnum_t::PERS_TEAM as usize];
        let mut n_best_player: c_int = -1;
        let mut n_highest_score: c_int = 0;
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            let pc = player.client as *mut gclient_t;
            if player.inuse == qfalse || (*pc).ps.persistant[persEnum_t::PERS_TEAM as usize] != team
            {
                continue;
            }
            // Raven: suicides don't count, you big cheater.
            let mut n_score = (*pc).ps.persistant[persEnum_t::PERS_KILLED as usize];
            n_score -= (*pc).ps.fd.suicides;
            if n_score > n_highest_score {
                n_highest_score = n_score;
                n_best_player = i;
            }
        }
        if n_best_player == -1 {
            return qfalse;
        }
        if n_best_player == (*ent).s.number {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `teamAward_e`.
///
/// Raven comments: TEAM_NONE "ha ha! you suck!"; TEAM_MVP "most overall
/// points"; TEAM_DEFENDER "killed the most baddies near your flag";
/// TEAM_WARRIOR "most frags"; TEAM_CARRIER "infected the most people with
/// plague"; TEAM_INTERCEPTOR "returned your own flag the most";
/// TEAM_BRAVERY "Red Shirt Award (tm). you died more than anybody."
/// Source: `oracle/oracle/codemp/game/g_log.c:1438-1447`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamAward_e {
    TeamNone = 0,
    TeamMvp,
    TeamDefender,
    TeamWarrior,
    TeamCarrier,
    TeamInterceptor,
    TeamBravery,
    TeamMax,
}

/// Raven `CalculateTeamAward`.
///
/// Source: `oracle/oracle/codemp/game/g_log.c:1451-1484`
pub fn CalculateTeamAward(ctx: GameContext<'_>, ent: *mut gentity_t) -> c_int {
    unsafe {
        let mut team_awards: c_int = 0;

        if CalculateTeamMVP(ctx, ent) != qfalse {
            team_awards |= 1 << TeamAward_e::TeamMvp as i32;
        }
        if (*ctx.world).cvars.g_gametype.integer == GT_CTF
            || (*ctx.world).cvars.g_gametype.integer == GT_CTY
        {
            if CalculateTeamDefender(ctx, ent) != qfalse {
                team_awards |= 1 << TeamAward_e::TeamDefender as i32;
            }
            if CalculateTeamWarrior(ctx, ent) != qfalse {
                team_awards |= 1 << TeamAward_e::TeamWarrior as i32;
            }
            if CalculateTeamCarrier(ctx, ent) != qfalse {
                team_awards |= 1 << TeamAward_e::TeamCarrier as i32;
            }
            if CalculateTeamInterceptor(ctx, ent) != qfalse {
                team_awards |= 1 << TeamAward_e::TeamInterceptor as i32;
            }
        }
        if team_awards == 0 && CalculateTeamRedShirt(ctx, ent) != qfalse {
            // if you got nothing else and died a lot, at least get bravery
            team_awards |= 1 << TeamAward_e::TeamBravery as i32;
        }
        team_awards
    }
}

/// Raven `CalculateSection31Award` — the all-around "god" award: sharpshooter,
/// untouchable, and ≥75 efficiency all at once.
/// Source: `oracle/oracle/codemp/game/g_log.c:1486-1514`
pub fn CalculateSection31Award(ctx: GameContext<'_>, ent: *mut gentity_t) -> qboolean {
    unsafe {
        let maxclients = (*ctx.world).cvars.g_maxclients.integer;
        let mut efficiency: c_int = 0;
        let mut frags: c_int = 0;
        for i in 0..maxclients {
            let player = &(*ctx.world).g_entities[i as usize];
            if player.inuse == qfalse {
                continue;
            }
            CalculateEfficiency(ctx, ent, &mut efficiency);
            if CalculateSharpshooter(ctx, ent, &mut frags) == qfalse
                || CalculateUntouchable(ctx, ent) == qfalse
                || efficiency < 75
            {
                continue;
            }
            return qtrue;
        }
        qfalse
    }
}

/// Raven `awardType_t` — award bit indices for `CalculateAwards`' flag word.
/// Source: `oracle/oracle/codemp/game/g_log.c:1426-1437`
#[allow(non_camel_case_types)]
#[repr(i32)]
#[derive(Clone, Copy)]
enum awardType_t {
    AWARD_EFFICIENCY = 0, // Accuracy
    AWARD_SHARPSHOOTER,   // Most compression rifle frags
    AWARD_UNTOUCHABLE,    // Perfect (no deaths)
    AWARD_LOGISTICS,      // Most pickups
    AWARD_TACTICIAN,      // Kills with all weapons
    AWARD_DEMOLITIONIST,  // Most explosive damage kills
    AWARD_STREAK,         // Ace/Expert/Master/Champion
    AWARD_TEAM,           // MVP/Defender/Warrior/Carrier/Interceptor/Bravery
    AWARD_SECTION31,      // All-around god
    AWARD_MAX,
}

/// Raven `#define AWARDS_MSG_LENGTH 256`.
/// Source: `oracle/oracle/codemp/game/g_log.c:1516`
const AWARDS_MSG_LENGTH: usize = 256;

/// Raven `CalculateAwards` — appends this player's award flag word and per-award
/// values onto `msg` (a fixed `AWARDS_MSG_LENGTH` buffer, truncated like
/// `Com_sprintf`).
/// Source: `oracle/oracle/codemp/game/g_log.c:1518-1587`
pub fn CalculateAwards(ctx: GameContext<'_>, ent: *mut gentity_t, msg: *mut c_char) {
    unsafe {
        let old = cstr_to_str(msg).to_string();
        let mut buf1 = String::new();
        let mut award_flags: c_int = 0;
        let mut efficiency: c_int = 0;
        let mut stuff_used: c_int = 0;
        let mut kills: c_int = 0;

        if CalculateEfficiency(ctx, ent, &mut efficiency) != qfalse {
            award_flags |= 1 << awardType_t::AWARD_EFFICIENCY as i32;
            buf1 = format!(" {}", efficiency);
        }
        if CalculateSharpshooter(ctx, ent, &mut kills) != qfalse {
            award_flags |= 1 << awardType_t::AWARD_SHARPSHOOTER as i32;
            buf1 = format!("{} {}", buf1, kills);
        }
        if CalculateUntouchable(ctx, ent) != qfalse {
            award_flags |= 1 << awardType_t::AWARD_UNTOUCHABLE as i32;
            buf1 = format!("{} {}", buf1, 0);
        }
        if CalculateLogistics(ctx, ent, &mut stuff_used) != qfalse {
            award_flags |= 1 << awardType_t::AWARD_LOGISTICS as i32;
            buf1 = format!("{} {}", buf1, stuff_used);
        }
        if CalculateTactician(ctx, ent, &mut kills) != qfalse {
            award_flags |= 1 << awardType_t::AWARD_TACTICIAN as i32;
            buf1 = format!("{} {}", buf1, kills);
        }
        if CalculateDemolitionist(ctx, ent, &mut kills) != qfalse {
            award_flags |= 1 << awardType_t::AWARD_DEMOLITIONIST as i32;
            buf1 = format!("{} {}", buf1, kills);
        }
        let streak = CalculateStreak(ent);
        if streak != 0 {
            award_flags |= 1 << awardType_t::AWARD_STREAK as i32;
            buf1 = format!("{} {}", buf1, streak);
        }
        if (*ctx.world).cvars.g_gametype.integer >= GT_TEAM {
            let team_awards = CalculateTeamAward(ctx, ent);
            if team_awards != 0 {
                award_flags |= 1 << awardType_t::AWARD_TEAM as i32;
                buf1 = format!("{} {}", buf1, team_awards);
            }
        }
        if CalculateSection31Award(ctx, ent) != qfalse {
            award_flags |= 1 << awardType_t::AWARD_SECTION31 as i32;
            buf1 = format!("{} {}", buf1, 0);
        }

        // Com_sprintf(msg, AWARDS_MSG_LENGTH, "%s %d%s", old_msg, awardFlags, buf1)
        let result = format!("{} {}{}", old, award_flags, buf1);
        let bytes = result.as_bytes();
        let n = bytes.len().min(AWARDS_MSG_LENGTH - 1);
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), msg as *mut u8, n);
        *msg.add(n) = 0;
    }
}

/// Raven `GetMaxDeathsForClient`.
///
/// Source: `oracle/oracle/codemp/game/g_log.c:1589-1605`
pub fn GetMaxDeathsForClient(ctx: GameContext<'_>, nClient: c_int) -> c_int {
    if nClient < 0 || nClient >= MAX_CLIENTS as c_int {
        return 0;
    }
    unsafe {
        let mut n_most_deaths: c_int = 0;
        for i in 0..MAX_CLIENTS as c_int {
            let v = (*ctx.world).globals.G_WeaponLogFrags[i as usize][nClient as usize];
            if v > n_most_deaths {
                n_most_deaths = v;
            }
        }
        n_most_deaths
    }
}

/// Raven `GetMaxKillsForClient`.
///
/// Source: `oracle/oracle/codemp/game/g_log.c:1607-1623`
pub fn GetMaxKillsForClient(ctx: GameContext<'_>, nClient: c_int) -> c_int {
    if nClient < 0 || nClient >= MAX_CLIENTS as c_int {
        return 0;
    }
    unsafe {
        let mut n_most_kills: c_int = 0;
        for i in 0..MAX_CLIENTS as c_int {
            let v = (*ctx.world).globals.G_WeaponLogFrags[nClient as usize][i as usize];
            if v > n_most_kills {
                n_most_kills = v;
            }
        }
        n_most_kills
    }
}

/// Raven `GetFavoriteTargetForClient`.
///
/// Source: `oracle/oracle/codemp/game/g_log.c:1625-1642`
pub fn GetFavoriteTargetForClient(ctx: GameContext<'_>, nClient: c_int) -> c_int {
    if nClient < 0 || nClient >= MAX_CLIENTS as c_int {
        return 0;
    }
    unsafe {
        let mut n_most_kills: c_int = 0;
        let mut n_favorite_target: c_int = -1;
        for i in 0..MAX_CLIENTS as c_int {
            let v = (*ctx.world).globals.G_WeaponLogFrags[nClient as usize][i as usize];
            if v > n_most_kills {
                n_most_kills = v;
                n_favorite_target = i;
            }
        }
        n_favorite_target
    }
}

/// Raven `GetWorstEnemyForClient`.
///
/// Raven: "If there is a tie for most deaths, we want to choose anybody
/// else over the client... I.E. Most deaths should not tie with yourself
/// and have yourself show up..."
/// Source: `oracle/oracle/codemp/game/g_log.c:1644-1666`
pub fn GetWorstEnemyForClient(ctx: GameContext<'_>, nClient: c_int) -> c_int {
    if nClient < 0 || nClient >= MAX_CLIENTS as c_int {
        return 0;
    }
    unsafe {
        let mut n_most_deaths: c_int = 0;
        let mut n_worst_enemy: c_int = -1;
        for i in 0..MAX_CLIENTS as c_int {
            let v = (*ctx.world).globals.G_WeaponLogFrags[i as usize][nClient as usize];
            if v > n_most_deaths || (v == n_most_deaths && i != nClient && n_most_deaths != 0) {
                n_most_deaths = v;
                n_worst_enemy = i;
            }
        }
        n_worst_enemy
    }
}

/// Raven `weaponFromMOD`.
///
/// MOD-weapon mapping array. Raven declares it `int weaponFromMOD[MOD_MAX]`
/// (not a named enum), so this stays a plain array of `c_int`, not an enum.
/// The designated-initializer list only covers the first 38 of `MOD_MAX`(45)
/// entries — the trailing entries (`MOD_FORCE_DARK`..`MOD_TRIGGER_HURT`) are
/// C zero-init, which is `WP_NONE` (0) anyway, so this array is faithfully
/// padded with `WP_NONE`.
/// Source: `oracle/oracle/codemp/game/g_log.c:35-77`
pub const weaponFromMOD: [c_int; meansOfDeath_t::MOD_MAX as usize] = {
    let mut table = [WP_NONE; meansOfDeath_t::MOD_MAX as usize];
    table[meansOfDeath_t::MOD_UNKNOWN as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_STUN_BATON as usize] = WP_STUN_BATON;
    table[meansOfDeath_t::MOD_MELEE as usize] = WP_MELEE;
    table[meansOfDeath_t::MOD_SABER as usize] = WP_SABER;
    table[meansOfDeath_t::MOD_BRYAR_PISTOL as usize] = WP_BRYAR_PISTOL;
    table[meansOfDeath_t::MOD_BRYAR_PISTOL_ALT as usize] = WP_BRYAR_PISTOL;
    table[meansOfDeath_t::MOD_BLASTER as usize] = WP_BLASTER;
    table[meansOfDeath_t::MOD_TURBLAST as usize] = WP_TURRET;
    table[meansOfDeath_t::MOD_DISRUPTOR as usize] = WP_DISRUPTOR;
    table[meansOfDeath_t::MOD_DISRUPTOR_SPLASH as usize] = WP_DISRUPTOR;
    table[meansOfDeath_t::MOD_DISRUPTOR_SNIPER as usize] = WP_DISRUPTOR;
    table[meansOfDeath_t::MOD_BOWCASTER as usize] = WP_BOWCASTER;
    table[meansOfDeath_t::MOD_REPEATER as usize] = WP_REPEATER;
    table[meansOfDeath_t::MOD_REPEATER_ALT as usize] = WP_REPEATER;
    table[meansOfDeath_t::MOD_REPEATER_ALT_SPLASH as usize] = WP_REPEATER;
    table[meansOfDeath_t::MOD_DEMP2 as usize] = WP_DEMP2;
    table[meansOfDeath_t::MOD_DEMP2_ALT as usize] = WP_DEMP2;
    table[meansOfDeath_t::MOD_FLECHETTE as usize] = WP_FLECHETTE;
    table[meansOfDeath_t::MOD_FLECHETTE_ALT_SPLASH as usize] = WP_FLECHETTE;
    table[meansOfDeath_t::MOD_ROCKET as usize] = WP_ROCKET_LAUNCHER;
    table[meansOfDeath_t::MOD_ROCKET_SPLASH as usize] = WP_ROCKET_LAUNCHER;
    table[meansOfDeath_t::MOD_ROCKET_HOMING as usize] = WP_ROCKET_LAUNCHER;
    table[meansOfDeath_t::MOD_ROCKET_HOMING_SPLASH as usize] = WP_ROCKET_LAUNCHER;
    table[meansOfDeath_t::MOD_THERMAL as usize] = WP_THERMAL;
    table[meansOfDeath_t::MOD_THERMAL_SPLASH as usize] = WP_THERMAL;
    table[meansOfDeath_t::MOD_TRIP_MINE_SPLASH as usize] = WP_TRIP_MINE;
    table[meansOfDeath_t::MOD_TIMED_MINE_SPLASH as usize] = WP_TRIP_MINE;
    table[meansOfDeath_t::MOD_DET_PACK_SPLASH as usize] = WP_DET_PACK;
    table[meansOfDeath_t::MOD_FORCE_DARK as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_SENTRY as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_WATER as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_SLIME as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_LAVA as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_CRUSH as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_TELEFRAG as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_FALLING as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_COLLISION as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_SUICIDE as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_TARGET_LASER as usize] = WP_NONE;
    table[meansOfDeath_t::MOD_TRIGGER_HURT as usize] = WP_NONE;
    table
};

/// Raven `GetFavoriteWeaponForClient`.
///
/// Source: `oracle/oracle/codemp/game/g_log.c:1668-1702`
pub fn GetFavoriteWeaponForClient(ctx: GameContext<'_>, nClient: c_int) -> c_int {
    let mut n_most_kills: c_int = 0;
    let mut fav: c_int = 0;
    let mut weapon: c_int;
    let mut kills_with_weapon: [c_int; WP_NUM_WEAPONS as usize] = [0; WP_NUM_WEAPONS as usize];

    // First thing we need to do is cycle through all the MOD types and convert
    // number of kills to a single weapon.
    // ----------------------------------------------------------------
    for weapon_idx in 0..WP_NUM_WEAPONS as usize {
        kills_with_weapon[weapon_idx] = 0; // CLEAR
    }

    unsafe {
        for i in MOD_STUN_BATON as c_int..=MOD_FORCE_DARK as c_int {
            weapon = weaponFromMOD[i as usize]; // Select Weapon

            if weapon != WP_NONE as c_int {
                kills_with_weapon[weapon as usize] +=
                    (*ctx.world).globals.G_WeaponLogKills.0[nClient as usize][i as usize];
                // Store Num Kills With Weapon
            }
        }
    }

    // now look through our list of kills per weapon and pick the biggest
    // ----------------------------------------------------------------
    // Oracle does not reset `fav` here (only `nMostKills`), so a client with zero
    // recorded kills returns WP_NONE (0). g_log.c:1692.
    n_most_kills = 0;
    weapon = WP_STUN_BATON as c_int;
    while weapon < WP_NUM_WEAPONS as c_int {
        if kills_with_weapon[weapon as usize] > n_most_kills {
            n_most_kills = kills_with_weapon[weapon as usize];
            fav = weapon;
        }
        weapon += 1;
    }
    fav
}

/// Raven `G_ClearClientLog`.
///
/// Raven: kef -- if a client leaves the game, clear out all counters he may
/// have set.
/// Source: `oracle/oracle/codemp/game/g_log.c:1705-1751`
pub fn G_ClearClientLog(ctx: GameContext<'_>, client: c_int) {
    unsafe {
        let g = &mut (*ctx.world).globals;
        let c = client as usize;
        for i in 0..WP_NUM_WEAPONS as usize {
            g.G_WeaponLogPickups[c][i] = 0;
        }
        for i in 0..WP_NUM_WEAPONS as usize {
            g.G_WeaponLogFired[c][i] = 0;
        }
        for i in 0..meansOfDeath_t::MOD_MAX as usize {
            g.G_WeaponLogDamage.0[c][i] = 0;
        }
        for i in 0..meansOfDeath_t::MOD_MAX as usize {
            g.G_WeaponLogKills.0[c][i] = 0;
        }
        for i in 0..WP_NUM_WEAPONS as usize {
            g.G_WeaponLogDeaths[c][i] = 0;
        }
        for i in 0..MAX_CLIENTS {
            g.G_WeaponLogFrags[c][i] = 0;
        }
        for i in 0..MAX_CLIENTS {
            g.G_WeaponLogFrags[i][c] = 0;
        }
        for i in 0..WP_NUM_WEAPONS as usize {
            g.G_WeaponLogTime[c][i] = 0;
        }
        g.G_WeaponLogLastTime[c] = 0;
        g.G_WeaponLogClientTouch[c] = qfalse;
        for i in 0..HI_NUM_HOLDABLE as usize {
            g.G_WeaponLogPowerups[c][i] = 0;
        }
        for i in 0..PW_NUM_POWERUPS as usize {
            g.G_WeaponLogItems[c][i] = 0;
        }
    }
}
