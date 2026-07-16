// PORT-COMPLETE: bg_saga.c 4/16
//! FAITHFUL port of `oracle/codemp/game/bg_saga.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! PORT STATUS (mega-pass fill phase): the 4 pure string-parsing/table-
//! translation functions (no module-scope state, no unresolved bg deps) are
//! fully ported. `BG_SiegeTranslateForcePowers` and the remaining 15
//! functions are parked: the former needs the file-scope `FPTable` const
//! (not a resolved out-of-file symbol in this packet), the rest read/write
//! the siege module-scope globals (`bgSiegeClasses`, `bgNumSiegeClasses`,
//! `bgSiegeTeams`, `bgNumSiegeTeams`, `team1Theme`, `team2Theme` — all
//! `GameWorld` fields) and/or the engine-bearing trap seam
//! (`trap::X(engine, …)`), but the fnskel signatures take raw
//! pointers/scalars with NO `GameContext`/`world`/`engine` channel — same
//! systemic gap as `g_active.c` (see that file's header). Undecidable from
//! the frozen docs + rulings without inventing a channel into a verbatim
//! cross-file signature; parked rather than fabricated.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::q_shared::Q_strcmp;
use crate::q_shared::Q_stricmp;
// `strlen` resolves to the crate's `Q_strlen` (the `g_spawn.rs` precedent for
// aliasing the libc name); `strcpy`/`strcat` are the file-local unchecked
// helpers below, matching the `c_strcpy` house pattern in `q_shared.rs` /
// `bg_saberLoad.rs` (Raven uses raw libc on fixed buffers here).
use crate::q_shared::Q_strlen as strlen;
// Raven `saber_styles_t` variants (`SS_*`) spelled bare in the saber-style table.
use mp_qshared::common::mp::qcommon::saber::saber_styles::saber_styles_t::*;

/// Local helper mirroring libc `strcpy` (copies through the terminating NUL,
/// no bounds check — faithful to Raven's unchecked fixed-buffer usage).
unsafe fn strcpy(dst: *mut c_char, src: *const c_char) {
    let mut i: isize = 0;
    loop {
        let c = *src.offset(i);
        *dst.offset(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
}

/// Local helper mirroring libc `strcat` (appends at the terminating NUL, no
/// bounds check — faithful to Raven's unchecked fixed-buffer usage).
unsafe fn strcat(dst: *mut c_char, src: *const c_char) {
    let mut end: isize = 0;
    while *dst.offset(end) != 0 {
        end += 1;
    }
    strcpy(dst.offset(end), src);
}

/// Raven's `#define SIEGECHAR_TAB 9` (tab literal used by the hand-rolled
/// siege-file parser instead of `'\t'`).
/// Source: `oracle/codemp/game/bg_saga.c:17`
pub const SIEGECHAR_TAB: c_char = 9;

/// Raven `bgSiegeClassFlagNames` — siege class flag name/id table
/// (`ENUM2STRING`-built).
///
/// Source: `oracle/codemp/game/bg_saga.c:45-56`
pub static bgSiegeClassFlagNames: [stringID_table_t; 9] = [
    stringID_table_t {
        name: c"CFL_MORESABERDMG".as_ptr() as *mut c_char,
        id: CFL_MORESABERDMG as c_int,
    },
    stringID_table_t {
        name: c"CFL_STRONGAGAINSTPHYSICAL".as_ptr() as *mut c_char,
        id: CFL_STRONGAGAINSTPHYSICAL as c_int,
    },
    stringID_table_t {
        name: c"CFL_FASTFORCEREGEN".as_ptr() as *mut c_char,
        id: CFL_FASTFORCEREGEN as c_int,
    },
    stringID_table_t {
        name: c"CFL_STATVIEWER".as_ptr() as *mut c_char,
        id: CFL_STATVIEWER as c_int,
    },
    stringID_table_t {
        name: c"CFL_HEAVYMELEE".as_ptr() as *mut c_char,
        id: CFL_HEAVYMELEE as c_int,
    },
    stringID_table_t {
        name: c"CFL_SINGLE_ROCKET".as_ptr() as *mut c_char,
        id: CFL_SINGLE_ROCKET as c_int,
    },
    stringID_table_t {
        name: c"CFL_CUSTOMSKEL".as_ptr() as *mut c_char,
        id: CFL_CUSTOMSKEL as c_int,
    },
    stringID_table_t {
        name: c"CFL_EXTRA_AMMO".as_ptr() as *mut c_char,
        id: CFL_EXTRA_AMMO as c_int,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: -1,
    },
];

/// Raven `StanceTable` — saber stance name/id table.
///
/// Source: `oracle/codemp/game/bg_saga.c:59-69`
pub static StanceTable: [stringID_table_t; 9] = [
    stringID_table_t {
        name: c"SS_NONE".as_ptr() as *mut c_char,
        id: SS_NONE as c_int,
    },
    stringID_table_t {
        name: c"SS_FAST".as_ptr() as *mut c_char,
        id: SS_FAST as c_int,
    },
    stringID_table_t {
        name: c"SS_MEDIUM".as_ptr() as *mut c_char,
        id: SS_MEDIUM as c_int,
    },
    stringID_table_t {
        name: c"SS_STRONG".as_ptr() as *mut c_char,
        id: SS_STRONG as c_int,
    },
    stringID_table_t {
        name: c"SS_DESANN".as_ptr() as *mut c_char,
        id: SS_DESANN as c_int,
    },
    stringID_table_t {
        name: c"SS_TAVION".as_ptr() as *mut c_char,
        id: SS_TAVION as c_int,
    },
    stringID_table_t {
        name: c"SS_DUAL".as_ptr() as *mut c_char,
        id: SS_DUAL as c_int,
    },
    stringID_table_t {
        name: c"SS_STAFF".as_ptr() as *mut c_char,
        id: SS_STAFF as c_int,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: 0,
    },
];

/// Raven `WPTable` — weapon name/id table (also used by NPC parsing).
///
/// Source: `oracle/codemp/game/bg_saga.c:72-97`
pub static WPTable: [stringID_table_t; 22] = [
    stringID_table_t {
        name: c"NULL".as_ptr() as *mut c_char,
        id: WP_NONE,
    },
    stringID_table_t {
        name: c"WP_NONE".as_ptr() as *mut c_char,
        id: WP_NONE,
    },
    stringID_table_t {
        name: c"WP_STUN_BATON".as_ptr() as *mut c_char,
        id: WP_STUN_BATON,
    },
    stringID_table_t {
        name: c"WP_MELEE".as_ptr() as *mut c_char,
        id: WP_MELEE,
    },
    stringID_table_t {
        name: c"WP_SABER".as_ptr() as *mut c_char,
        id: WP_SABER,
    },
    stringID_table_t {
        name: c"WP_BRYAR_PISTOL".as_ptr() as *mut c_char,
        id: WP_BRYAR_PISTOL,
    },
    stringID_table_t {
        name: c"WP_BLASTER_PISTOL".as_ptr() as *mut c_char,
        id: WP_BRYAR_PISTOL,
    },
    stringID_table_t {
        name: c"WP_BLASTER".as_ptr() as *mut c_char,
        id: WP_BLASTER,
    },
    stringID_table_t {
        name: c"WP_DISRUPTOR".as_ptr() as *mut c_char,
        id: WP_DISRUPTOR,
    },
    stringID_table_t {
        name: c"WP_BOWCASTER".as_ptr() as *mut c_char,
        id: WP_BOWCASTER,
    },
    stringID_table_t {
        name: c"WP_REPEATER".as_ptr() as *mut c_char,
        id: WP_REPEATER,
    },
    stringID_table_t {
        name: c"WP_DEMP2".as_ptr() as *mut c_char,
        id: WP_DEMP2,
    },
    stringID_table_t {
        name: c"WP_FLECHETTE".as_ptr() as *mut c_char,
        id: WP_FLECHETTE,
    },
    stringID_table_t {
        name: c"WP_ROCKET_LAUNCHER".as_ptr() as *mut c_char,
        id: WP_ROCKET_LAUNCHER,
    },
    stringID_table_t {
        name: c"WP_THERMAL".as_ptr() as *mut c_char,
        id: WP_THERMAL,
    },
    stringID_table_t {
        name: c"WP_TRIP_MINE".as_ptr() as *mut c_char,
        id: WP_TRIP_MINE,
    },
    stringID_table_t {
        name: c"WP_DET_PACK".as_ptr() as *mut c_char,
        id: WP_DET_PACK,
    },
    stringID_table_t {
        name: c"WP_CONCUSSION".as_ptr() as *mut c_char,
        id: WP_CONCUSSION,
    },
    stringID_table_t {
        name: c"WP_BRYAR_OLD".as_ptr() as *mut c_char,
        id: WP_BRYAR_OLD,
    },
    stringID_table_t {
        name: c"WP_EMPLACED_GUN".as_ptr() as *mut c_char,
        id: WP_EMPLACED_GUN,
    },
    stringID_table_t {
        name: c"WP_TURRET".as_ptr() as *mut c_char,
        id: WP_TURRET,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: 0,
    },
];

/// Raven `FPTable` — force-power name/id table.
///
/// Source: `oracle/codemp/game/bg_saga.c:100-121`
pub static FPTable: [stringID_table_t; 19] = [
    stringID_table_t {
        name: c"FP_HEAL".as_ptr() as *mut c_char,
        id: FP_HEAL,
    },
    stringID_table_t {
        name: c"FP_LEVITATION".as_ptr() as *mut c_char,
        id: FP_LEVITATION,
    },
    stringID_table_t {
        name: c"FP_SPEED".as_ptr() as *mut c_char,
        id: FP_SPEED,
    },
    stringID_table_t {
        name: c"FP_PUSH".as_ptr() as *mut c_char,
        id: FP_PUSH,
    },
    stringID_table_t {
        name: c"FP_PULL".as_ptr() as *mut c_char,
        id: FP_PULL,
    },
    stringID_table_t {
        name: c"FP_TELEPATHY".as_ptr() as *mut c_char,
        id: FP_TELEPATHY,
    },
    stringID_table_t {
        name: c"FP_GRIP".as_ptr() as *mut c_char,
        id: FP_GRIP,
    },
    stringID_table_t {
        name: c"FP_LIGHTNING".as_ptr() as *mut c_char,
        id: FP_LIGHTNING,
    },
    stringID_table_t {
        name: c"FP_RAGE".as_ptr() as *mut c_char,
        id: FP_RAGE,
    },
    stringID_table_t {
        name: c"FP_PROTECT".as_ptr() as *mut c_char,
        id: FP_PROTECT,
    },
    stringID_table_t {
        name: c"FP_ABSORB".as_ptr() as *mut c_char,
        id: FP_ABSORB,
    },
    stringID_table_t {
        name: c"FP_TEAM_HEAL".as_ptr() as *mut c_char,
        id: FP_TEAM_HEAL,
    },
    stringID_table_t {
        name: c"FP_TEAM_FORCE".as_ptr() as *mut c_char,
        id: FP_TEAM_FORCE,
    },
    stringID_table_t {
        name: c"FP_DRAIN".as_ptr() as *mut c_char,
        id: FP_DRAIN,
    },
    stringID_table_t {
        name: c"FP_SEE".as_ptr() as *mut c_char,
        id: FP_SEE,
    },
    stringID_table_t {
        name: c"FP_SABER_OFFENSE".as_ptr() as *mut c_char,
        id: FP_SABER_OFFENSE,
    },
    stringID_table_t {
        name: c"FP_SABER_DEFENSE".as_ptr() as *mut c_char,
        id: FP_SABER_DEFENSE,
    },
    stringID_table_t {
        name: c"FP_SABERTHROW".as_ptr() as *mut c_char,
        id: FP_SABERTHROW,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: -1,
    },
];

/// Raven `HoldableTable` — holdable item name/id table.
///
/// Source: `oracle/codemp/game/bg_saga.c:123-138`
pub static HoldableTable: [stringID_table_t; 13] = [
    stringID_table_t {
        name: c"HI_NONE".as_ptr() as *mut c_char,
        id: HI_NONE,
    },
    stringID_table_t {
        name: c"HI_SEEKER".as_ptr() as *mut c_char,
        id: HI_SEEKER,
    },
    stringID_table_t {
        name: c"HI_SHIELD".as_ptr() as *mut c_char,
        id: HI_SHIELD,
    },
    stringID_table_t {
        name: c"HI_MEDPAC".as_ptr() as *mut c_char,
        id: HI_MEDPAC,
    },
    stringID_table_t {
        name: c"HI_MEDPAC_BIG".as_ptr() as *mut c_char,
        id: HI_MEDPAC_BIG,
    },
    stringID_table_t {
        name: c"HI_BINOCULARS".as_ptr() as *mut c_char,
        id: HI_BINOCULARS,
    },
    stringID_table_t {
        name: c"HI_SENTRY_GUN".as_ptr() as *mut c_char,
        id: HI_SENTRY_GUN,
    },
    stringID_table_t {
        name: c"HI_JETPACK".as_ptr() as *mut c_char,
        id: HI_JETPACK,
    },
    stringID_table_t {
        name: c"HI_HEALTHDISP".as_ptr() as *mut c_char,
        id: HI_HEALTHDISP,
    },
    stringID_table_t {
        name: c"HI_AMMODISP".as_ptr() as *mut c_char,
        id: HI_AMMODISP,
    },
    stringID_table_t {
        name: c"HI_EWEB".as_ptr() as *mut c_char,
        id: HI_EWEB,
    },
    stringID_table_t {
        name: c"HI_CLOAK".as_ptr() as *mut c_char,
        id: HI_CLOAK,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: -1,
    },
];

/// Raven `PowerupTable` — powerup name/id table.
///
/// Source: `oracle/codemp/game/bg_saga.c:142-161`
pub static PowerupTable: [stringID_table_t; 17] = [
    stringID_table_t {
        name: c"PW_NONE".as_ptr() as *mut c_char,
        id: PW_NONE,
    },
    stringID_table_t {
        name: c"PW_QUAD".as_ptr() as *mut c_char,
        id: PW_QUAD,
    },
    stringID_table_t {
        name: c"PW_BATTLESUIT".as_ptr() as *mut c_char,
        id: PW_BATTLESUIT,
    },
    stringID_table_t {
        name: c"PW_PULL".as_ptr() as *mut c_char,
        id: PW_PULL,
    },
    stringID_table_t {
        name: c"PW_REDFLAG".as_ptr() as *mut c_char,
        id: PW_REDFLAG,
    },
    stringID_table_t {
        name: c"PW_BLUEFLAG".as_ptr() as *mut c_char,
        id: PW_BLUEFLAG,
    },
    stringID_table_t {
        name: c"PW_NEUTRALFLAG".as_ptr() as *mut c_char,
        id: PW_NEUTRALFLAG,
    },
    stringID_table_t {
        name: c"PW_SHIELDHIT".as_ptr() as *mut c_char,
        id: PW_SHIELDHIT,
    },
    stringID_table_t {
        name: c"PW_SPEEDBURST".as_ptr() as *mut c_char,
        id: PW_SPEEDBURST,
    },
    stringID_table_t {
        name: c"PW_DISINT_4".as_ptr() as *mut c_char,
        id: PW_DISINT_4,
    },
    stringID_table_t {
        name: c"PW_SPEED".as_ptr() as *mut c_char,
        id: PW_SPEED,
    },
    stringID_table_t {
        name: c"PW_CLOAKED".as_ptr() as *mut c_char,
        id: PW_CLOAKED,
    },
    stringID_table_t {
        name: c"PW_FORCE_ENLIGHTENED_LIGHT".as_ptr() as *mut c_char,
        id: PW_FORCE_ENLIGHTENED_LIGHT,
    },
    stringID_table_t {
        name: c"PW_FORCE_ENLIGHTENED_DARK".as_ptr() as *mut c_char,
        id: PW_FORCE_ENLIGHTENED_DARK,
    },
    stringID_table_t {
        name: c"PW_FORCE_BOON".as_ptr() as *mut c_char,
        id: PW_FORCE_BOON,
    },
    stringID_table_t {
        name: c"PW_YSALAMIRI".as_ptr() as *mut c_char,
        id: PW_YSALAMIRI,
    },
    stringID_table_t {
        name: c"".as_ptr() as *mut c_char,
        id: -1,
    },
];

/// Raven `classTitles` — Raven icon suffix strings for player-class detection,
/// indexed by `siegePlayerClassFlags_t`.
///
/// Source: `oracle/codemp/game/bg_saga.c:748-756`
pub static classTitles: [&core::ffi::CStr; SPC_MAX as usize] = [
    c"infantry",      // SPC_INFANTRY
    c"vanguard",      // SPC_VANGUARD
    c"support",       // SPC_SUPPORT
    c"jedi_general",  // SPC_JEDI
    c"demolitionist", // SPC_DEMOLITIONIST
    c"heavy_weapons", // SPC_HEAVY_WEAPONS
];

/// Raven `BG_SiegeStripTabs`.
///
/// Raven: converts tabs to spaces in place (the compacting index `i_r` always
/// tracks `i` 1:1 in the oracle — no character is ever dropped).
/// Source: `oracle/codemp/game/bg_saga.c:168-189`
pub fn BG_SiegeStripTabs(buf: *mut c_char) {
    unsafe {
        let mut i: isize = 0;
        let mut i_r: isize = 0;

        while *buf.offset(i) != 0 {
            if *buf.offset(i) != SIEGECHAR_TAB {
                // not a tab, just stick it in
                *buf.offset(i_r) = *buf.offset(i);
            } else {
                // If it's a tab, convert it to a space.
                *buf.offset(i_r) = b' ' as c_char;
            }

            i_r += 1;
            i += 1;
        }

        *buf.offset(i_r) = 0;
    }
}

/// Raven `BG_SiegeGetValueGroup`.
///
/// Source: `oracle/codemp/game/bg_saga.c:191-406`
pub fn BG_SiegeGetValueGroup(buf: *mut c_char, group: *mut c_char, outbuf: *mut c_char) -> c_int {
    unsafe {
        let mut i: isize = 0;
        let mut j: isize;
        let mut check_group = [0 as c_char; 4096];
        let mut is_group: bool;
        let mut parse_groups: c_int;

        while *buf.offset(i) != 0 {
            let c = *buf.offset(i);
            if c != b' ' as c_char
                && c != b'{' as c_char
                && c != b'}' as c_char
                && c != b'\n' as c_char
                && c != b'\r' as c_char
                && c != SIEGECHAR_TAB
            {
                // we're on a valid character
                if *buf.offset(i) == b'/' as c_char && *buf.offset(i + 1) == b'/' as c_char {
                    // this is a comment, so skip over it
                    while *buf.offset(i) != 0
                        && *buf.offset(i) != b'\n' as c_char
                        && *buf.offset(i) != b'\r' as c_char
                        && *buf.offset(i) != SIEGECHAR_TAB
                    {
                        i += 1;
                    }
                } else {
                    // parse to the next space/endline/eos and check this value against our group value.
                    j = 0;

                    while *buf.offset(i) != b' ' as c_char
                        && *buf.offset(i) != b'\n' as c_char
                        && *buf.offset(i) != b'\r' as c_char
                        && *buf.offset(i) != SIEGECHAR_TAB
                        && *buf.offset(i) != b'{' as c_char
                        && *buf.offset(i) != 0
                    {
                        if *buf.offset(i) == b'/' as c_char && *buf.offset(i + 1) == b'/' as c_char
                        {
                            // hit a comment, break out.
                            break;
                        }

                        check_group[j as usize] = *buf.offset(i);
                        j += 1;
                        i += 1;
                    }
                    check_group[j as usize] = 0;

                    // Make sure this is a group as opposed to a globally defined value.
                    if *buf.offset(i) == b'/' as c_char && *buf.offset(i + 1) == b'/' as c_char {
                        // stopped on a comment, so first parse to the end of it.
                        while *buf.offset(i) != 0
                            && *buf.offset(i) != b'\n' as c_char
                            && *buf.offset(i) != b'\r' as c_char
                        {
                            i += 1;
                        }
                        while *buf.offset(i) == b'\n' as c_char || *buf.offset(i) == b'\r' as c_char
                        {
                            i += 1;
                        }
                    }

                    if *buf.offset(i) == 0 {
                        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                        panic!("Unexpected EOF while looking for group");
                    }

                    is_group = false;

                    while (*buf.offset(i) != 0 && *buf.offset(i) == b' ' as c_char)
                        || *buf.offset(i) == SIEGECHAR_TAB
                        || *buf.offset(i) == b'\n' as c_char
                        || *buf.offset(i) == b'\r' as c_char
                    {
                        // parse to the next valid character
                        i += 1;
                    }

                    if *buf.offset(i) == b'{' as c_char {
                        // if the next valid character is an opening bracket, then this is indeed a group
                        is_group = true;
                    }

                    // Is this the one we want?
                    if is_group && Q_stricmp(check_group.as_ptr(), group) == 0 {
                        // guess so. Parse until we hit the { indicating the beginning of the group.
                        while *buf.offset(i) != b'{' as c_char && *buf.offset(i) != 0 {
                            i += 1;
                        }

                        if *buf.offset(i) != 0 {
                            // We're at the start of the group now, so parse to the closing bracket.
                            j = 0;
                            parse_groups = 0;

                            while (*buf.offset(i) != b'}' as c_char || parse_groups != 0)
                                && *buf.offset(i) != 0
                            {
                                if *buf.offset(i) == b'{' as c_char {
                                    // increment for the opening bracket.
                                    parse_groups += 1;
                                } else if *buf.offset(i) == b'}' as c_char {
                                    // decrement for the closing bracket
                                    parse_groups -= 1;
                                }

                                if parse_groups < 0 {
                                    // Syntax error, I guess.
                                    panic!("Found a closing bracket without an opening bracket while looking for group");
                                    // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                                }

                                if (*buf.offset(i) != b'{' as c_char || parse_groups > 1)
                                    && (*buf.offset(i) != b'}' as c_char || parse_groups > 0)
                                {
                                    // don't put the start and end brackets for this group into the output buffer
                                    *outbuf.offset(j) = *buf.offset(i);
                                    j += 1;
                                }

                                if *buf.offset(i) == b'}' as c_char && parse_groups == 0 {
                                    // Alright, we can break out now.
                                    break;
                                }

                                i += 1;
                            }
                            *outbuf.offset(j) = 0;

                            // Verify that we ended up on the closing bracket.
                            if *buf.offset(i) != b'}' as c_char {
                                // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                                panic!("Group is missing a closing bracket");
                            }

                            // Strip the tabs so we're friendly for value parsing.
                            BG_SiegeStripTabs(outbuf);

                            return 1; // we got it, so return 1.
                        } else {
                            panic!("Error parsing group in file, unexpected EOF before opening bracket while looking for group");
                            // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                        }
                    } else if !is_group {
                        // if it wasn't a group, parse to the end of the line
                        while *buf.offset(i) != 0
                            && *buf.offset(i) != b'\n' as c_char
                            && *buf.offset(i) != b'\r' as c_char
                        {
                            i += 1;
                        }
                    } else {
                        // this was a group but we not the one we wanted to find, so parse by it.
                        parse_groups = 0;

                        while *buf.offset(i) != 0
                            && (*buf.offset(i) != b'}' as c_char || parse_groups != 0)
                        {
                            if *buf.offset(i) == b'{' as c_char {
                                parse_groups += 1;
                            } else if *buf.offset(i) == b'}' as c_char {
                                parse_groups -= 1;
                            }

                            if parse_groups < 0 {
                                // Syntax error, I guess.
                                panic!("Found a closing bracket without an opening bracket while looking for group");
                                // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                            }

                            if *buf.offset(i) == b'}' as c_char && parse_groups == 0 {
                                // Alright, we can break out now.
                                break;
                            }

                            i += 1;
                        }

                        if *buf.offset(i) != b'}' as c_char {
                            panic!("Found an opening bracket without a matching closing bracket while looking for group");
                            // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                        }

                        i += 1;
                    }
                }
            } else if c == b'{' as c_char {
                // we're in a group that isn't the one we want, so parse to the end.
                parse_groups = 0;

                while *buf.offset(i) != 0 && (*buf.offset(i) != b'}' as c_char || parse_groups != 0)
                {
                    if *buf.offset(i) == b'{' as c_char {
                        parse_groups += 1;
                    } else if *buf.offset(i) == b'}' as c_char {
                        parse_groups -= 1;
                    }

                    if parse_groups < 0 {
                        // Syntax error, I guess.
                        panic!("Found a closing bracket without an opening bracket while looking for group");
                        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                    }

                    if *buf.offset(i) == b'}' as c_char && parse_groups == 0 {
                        // Alright, we can break out now.
                        break;
                    }

                    i += 1;
                }

                if *buf.offset(i) != b'}' as c_char {
                    panic!("Found an opening bracket without a matching closing bracket while looking for group");
                    // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                }
            }

            if *buf.offset(i) == 0 {
                break;
            }
            i += 1;
        }

        0 // guess we never found it.
    }
}

/// Raven `BG_SiegeGetPairedValue`.
///
/// Source: `oracle/codemp/game/bg_saga.c:408-562`
pub fn BG_SiegeGetPairedValue(buf: *mut c_char, key: *mut c_char, outbuf: *mut c_char) -> c_int {
    unsafe {
        let mut i: isize = 0;
        let mut j: isize;
        let mut k: isize;
        let mut check_key = [0 as c_char; 4096];

        while *buf.offset(i) != 0 {
            let c = *buf.offset(i);
            if c != b' ' as c_char
                && c != b'{' as c_char
                && c != b'}' as c_char
                && c != b'\n' as c_char
                && c != b'\r' as c_char
            {
                // we're on a valid character
                if *buf.offset(i) == b'/' as c_char && *buf.offset(i + 1) == b'/' as c_char {
                    // this is a comment, so skip over it
                    while *buf.offset(i) != 0
                        && *buf.offset(i) != b'\n' as c_char
                        && *buf.offset(i) != b'\r' as c_char
                    {
                        i += 1;
                    }
                } else {
                    // parse to the next space/endline/eos and check this value against our key value.
                    j = 0;

                    while *buf.offset(i) != b' ' as c_char
                        && *buf.offset(i) != b'\n' as c_char
                        && *buf.offset(i) != b'\r' as c_char
                        && *buf.offset(i) != SIEGECHAR_TAB
                        && *buf.offset(i) != 0
                    {
                        if *buf.offset(i) == b'/' as c_char && *buf.offset(i + 1) == b'/' as c_char
                        {
                            // hit a comment, break out.
                            break;
                        }

                        check_key[j as usize] = *buf.offset(i);
                        j += 1;
                        i += 1;
                    }
                    check_key[j as usize] = 0;

                    k = i;

                    while *buf.offset(k) != 0
                        && (*buf.offset(k) == b' ' as c_char
                            || *buf.offset(k) == b'\n' as c_char
                            || *buf.offset(k) == b'\r' as c_char)
                    {
                        k += 1;
                    }

                    if *buf.offset(k) == b'{' as c_char {
                        // this is not the start of a value but rather of a group. We don't want to look in subgroups so skip over the whole thing.
                        let mut open_b: c_int = 0;

                        while *buf.offset(i) != 0
                            && (*buf.offset(i) != b'}' as c_char || open_b != 0)
                        {
                            if *buf.offset(i) == b'{' as c_char {
                                open_b += 1;
                            } else if *buf.offset(i) == b'}' as c_char {
                                open_b -= 1;
                            }

                            if open_b < 0 {
                                panic!("Unexpected closing bracket (too many) while parsing to end of group");
                                // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                            }

                            if *buf.offset(i) == b'}' as c_char && open_b == 0 {
                                // this is the end of the group
                                break;
                            }
                            i += 1;
                        }

                        if *buf.offset(i) == b'}' as c_char {
                            i += 1;
                        }
                    } else {
                        // Is this the one we want?
                        if *buf.offset(i) != b'/' as c_char || *buf.offset(i + 1) != b'/' as c_char
                        {
                            // make sure we didn't stop on a comment, if we did then this is considered an error in the file.
                            if Q_stricmp(check_key.as_ptr(), key) == 0 {
                                // guess so. Parse along to the next valid character, then put that into the output buffer and return 1.
                                while (*buf.offset(i) == b' ' as c_char
                                    || *buf.offset(i) == b'\n' as c_char
                                    || *buf.offset(i) == b'\r' as c_char
                                    || *buf.offset(i) == SIEGECHAR_TAB)
                                    && *buf.offset(i) != 0
                                {
                                    i += 1;
                                }

                                if *buf.offset(i) != 0 {
                                    // We're at the start of the value now.
                                    let mut parse_to_quote = false;

                                    if *buf.offset(i) == b'"' as c_char {
                                        // if the value is in quotes, then stop at the next quote instead of ' '
                                        i += 1;
                                        parse_to_quote = true;
                                    }

                                    j = 0;
                                    while (!parse_to_quote
                                        && *buf.offset(i) != b' ' as c_char
                                        && *buf.offset(i) != b'\n' as c_char
                                        && *buf.offset(i) != b'\r' as c_char)
                                        || (parse_to_quote && *buf.offset(i) != b'"' as c_char)
                                    {
                                        if *buf.offset(i) == b'/' as c_char
                                            && *buf.offset(i + 1) == b'/' as c_char
                                        {
                                            // hit a comment after the value? This isn't an ideal way to be writing things, but we'll support it anyway.
                                            break;
                                        }
                                        *outbuf.offset(j) = *buf.offset(i);
                                        j += 1;
                                        i += 1;

                                        if *buf.offset(i) == 0 {
                                            if parse_to_quote {
                                                panic!("Unexpected EOF while looking for endquote, error finding paired value for");
                                            // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                                            } else {
                                                panic!("Unexpected EOF while looking for space or endline, error finding paired value for");
                                                // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                                            }
                                        }
                                    }
                                    *outbuf.offset(j) = 0;

                                    return 1; // we got it, so return 1.
                                } else {
                                    panic!("Error parsing file, unexpected EOF while looking for valud");
                                    // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                                }
                            } else {
                                // if that wasn't the desired key, then make sure we parse to the end of the line, so we don't mistake a value for a key
                                while *buf.offset(i) != 0 && *buf.offset(i) != b'\n' as c_char {
                                    i += 1;
                                }
                            }
                        } else {
                            // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
                            panic!("Error parsing file, found comment, expected value for key");
                        }
                    }
                }
            }

            if *buf.offset(i) == 0 {
                break;
            }
            i += 1;
        }

        0 // guess we never found it.
    }
}

/// Raven `BG_SiegeTranslateForcePowers`.
///
/// Source: `oracle/codemp/game/bg_saga.c:571-682`
pub fn BG_SiegeTranslateForcePowers(buf: *mut c_char, siegeClass: *mut siegeClass_t) {
    unsafe {
        let mut check_power = [0 as c_char; 1024];
        let mut check_level = [0 as c_char; 256];
        let mut l: isize = 0;
        let mut k: isize = 0;
        let mut j: isize = 0;
        let mut i: isize = 0;
        let mut parsed_level: c_int = 0;
        let mut all_powers: bool = false;
        let mut no_powers: bool = false;

        if Q_stricmp(buf, c"FP_ALL".as_ptr()) == 0 {
            all_powers = true;
        }

        if *buf.offset(0) == b'0' as c_char && *buf.offset(1) == 0 {
            no_powers = true;
        }

        while i < NUM_FORCE_POWERS as isize {
            if all_powers {
                (*siegeClass).forcePowerLevels[i as usize] = FORCE_LEVEL_3;
            } else {
                (*siegeClass).forcePowerLevels[i as usize] = 0;
            }
            i += 1;
        }

        if all_powers || no_powers {
            return;
        }

        i = 0;
        while *buf.offset(i) != 0 {
            if *buf.offset(i) != b' ' as c_char && *buf.offset(i) != b'|' as c_char {
                j = 0;

                while *buf.offset(i) != 0
                    && *buf.offset(i) != b' ' as c_char
                    && *buf.offset(i) != b'|' as c_char
                    && *buf.offset(i) != b',' as c_char
                {
                    check_power[j as usize] = *buf.offset(i);
                    j += 1;
                    i += 1;
                }
                check_power[j as usize] = 0;

                if *buf.offset(i) == b',' as c_char {
                    i += 1;
                    l = 0;
                    while *buf.offset(i) != 0
                        && *buf.offset(i) != b' ' as c_char
                        && *buf.offset(i) != b'|' as c_char
                    {
                        check_level[l as usize] = *buf.offset(i);
                        l += 1;
                        i += 1;
                    }
                    check_level[l as usize] = 0;
                    parsed_level = atoi(check_level.as_ptr());

                    if parsed_level < 0 {
                        parsed_level = 0;
                    }
                    if parsed_level > FORCE_LEVEL_5 {
                        parsed_level = FORCE_LEVEL_5;
                    }
                } else {
                    parsed_level = 3;
                }

                if check_power[0] != 0 {
                    k = 0;

                    if Q_stricmp(check_power.as_ptr(), c"FP_JUMP".as_ptr()) == 0 {
                        unsafe {
                            strcpy(check_power.as_mut_ptr(), c"FP_LEVITATION".as_ptr());
                        }
                    }

                    while FPTable[k as usize].id != -1
                        && !FPTable[k as usize].name.is_null()
                        && *FPTable[k as usize].name as u8 != 0
                    {
                        if Q_stricmp(check_power.as_ptr(), FPTable[k as usize].name) == 0 {
                            (*siegeClass).forcePowerLevels[k as usize] = parsed_level;
                            break;
                        }
                        k += 1;
                    }
                }
            }

            if *buf.offset(i) == 0 {
                break;
            }
            i += 1;
        }
    }
}

/// Raven `BG_SiegeTranslateGenericTable`.
///
/// Raven: used for the majority of generic val parsing stuff. `buf` should be
/// the value string, `table` the appropriate string/id table. If `bitflag` is
/// qtrue then the values are accumulated into a bitflag. If `bitflag` is
/// qfalse then the first value is returned as a directly corresponding id and
/// no further parsing is done.
/// Source: `oracle/codemp/game/bg_saga.c:688-746`
pub fn BG_SiegeTranslateGenericTable(
    buf: *mut c_char,
    table: *mut stringID_table_t,
    bitflag: qboolean,
) -> c_int {
    unsafe {
        let mut items: c_int = 0;
        let mut check_item = [0 as c_char; 1024];
        let mut i: isize = 0;
        let mut j: isize;
        let mut k: isize;

        if *buf.offset(0) == b'0' as c_char && *buf.offset(1) == 0 {
            // special case, no items.
            return 0;
        }

        while *buf.offset(i) != 0 {
            // Using basically the same parsing method as we do for weapons and forcepowers.
            if *buf.offset(i) != b' ' as c_char && *buf.offset(i) != b'|' as c_char {
                j = 0;

                while *buf.offset(i) != 0
                    && *buf.offset(i) != b' ' as c_char
                    && *buf.offset(i) != b'|' as c_char
                {
                    check_item[j as usize] = *buf.offset(i);
                    j += 1;
                    i += 1;
                }
                check_item[j as usize] = 0;

                if check_item[0] != 0 {
                    k = 0;

                    while !(*table.offset(k)).name.is_null()
                        && *(*table.offset(k)).name.offset(0) != 0
                    {
                        // go through the list and check the parsed flag name against the hardcoded names
                        if Q_stricmp(check_item.as_ptr(), (*table.offset(k)).name) == 0 {
                            // Got it, so add the value into our items value.
                            if bitflag != 0 {
                                items |= 1 << (*table.offset(k)).id;
                            } else {
                                // return the value directly then.
                                return (*table.offset(k)).id;
                            }
                            break;
                        }
                        k += 1;
                    }
                }
            }

            if *buf.offset(i) == 0 {
                break;
            }

            i += 1;
        }
        items
    }
}

/// Raven `BG_SiegeParseClassFile`.
///
/// Source: `oracle/codemp/game/bg_saga.c:759-1068`
pub fn BG_SiegeParseClassFile(
    filename: *const c_char,
    descBuffer: *mut siegeClassDesc_t,
    bg: &mut BgState,
    traps: &dyn BgTraps,
) {
    unsafe {
        let mut f: fileHandle_t = 0;
        let mut len: c_int;
        let mut i: isize = 0;
        let mut class_info = [0 as c_char; 4096];
        let mut parse_buf = [0 as c_char; 4096];

        len = traps.fs_fopen(filename, &mut f, FS_READ);

        if f == 0 || len >= 4096 {
            return;
        }

        traps.fs_read(class_info.as_mut_ptr() as *mut c_void, len, f);
        traps.fs_fclose(f);
        class_info[len as usize] = 0;

        if !descBuffer.is_null() {
            if BG_SiegeGetPairedValue(
                class_info.as_mut_ptr(),
                c"description".as_ptr() as *mut c_char,
                descBuffer.as_mut().unwrap().desc.as_mut_ptr(),
            ) == 0
            {
                strcpy(
                    (*descBuffer).desc.as_mut_ptr(),
                    c"DESCRIPTION UNAVAILABLE".as_ptr(),
                );
            }
            assert!(strlen((*descBuffer).desc.as_ptr()) < SIEGE_CLASS_DESC_LEN as usize);
        }

        BG_SiegeGetValueGroup(
            class_info.as_mut_ptr(),
            c"ClassInfo".as_ptr() as *mut c_char,
            class_info.as_mut_ptr(),
        );

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"name".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            strcpy(
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize]
                    .name
                    .as_mut_ptr(),
                parse_buf.as_ptr(),
            );
        } else {
            panic!("Siege class without name entry");
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"model".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            strcpy(
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize]
                    .forcedModel
                    .as_mut_ptr(),
                parse_buf.as_ptr(),
            );
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].forcedModel[0] = 0;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"skin".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            strcpy(
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize]
                    .forcedSkin
                    .as_mut_ptr(),
                parse_buf.as_ptr(),
            );
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].forcedSkin[0] = 0;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"saber1".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            strcpy(
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize]
                    .saber1
                    .as_mut_ptr(),
                parse_buf.as_ptr(),
            );
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].saber1[0] = 0;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"saber2".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            strcpy(
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize]
                    .saber2
                    .as_mut_ptr(),
                parse_buf.as_ptr(),
            );
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].saber2[0] = 0;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"saberstyle".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].saberStance =
                BG_SiegeTranslateGenericTable(
                    parse_buf.as_mut_ptr(),
                    StanceTable.as_ptr() as *mut _,
                    qtrue,
                );
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].saberStance = 0;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"sabercolor".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].forcedSaberColor =
                atoi(parse_buf.as_ptr());
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].hasForcedSaberColor = qtrue;
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].hasForcedSaberColor = qfalse;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"saber2color".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].forcedSaber2Color =
                atoi(parse_buf.as_ptr());
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].hasForcedSaber2Color = qtrue;
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].hasForcedSaber2Color = qfalse;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"weapons".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].weapons =
                BG_SiegeTranslateGenericTable(
                    parse_buf.as_mut_ptr(),
                    WPTable.as_ptr() as *mut _,
                    qtrue,
                );
        } else {
            panic!("Siege class without weapons entry");
        }

        if (bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].weapons & (1 << WP_SABER)) == 0 {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].weapons |= 1 << WP_MELEE;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"forcepowers".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            BG_SiegeTranslateForcePowers(
                parse_buf.as_mut_ptr(),
                &mut bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize],
            );
        } else {
            i = 0;
            while i < NUM_FORCE_POWERS as isize {
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].forcePowerLevels[i as usize] = 0;
                i += 1;
            }
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"classflags".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].classflags =
                BG_SiegeTranslateGenericTable(
                    parse_buf.as_mut_ptr(),
                    bgSiegeClassFlagNames.as_ptr() as *mut _,
                    qtrue,
                );
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].classflags = 0;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"maxhealth".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].maxhealth = atoi(parse_buf.as_ptr());
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].maxhealth = 100;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"starthealth".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].starthealth = atoi(parse_buf.as_ptr());
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].starthealth =
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].maxhealth;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"maxarmor".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].maxarmor = atoi(parse_buf.as_ptr());
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].maxarmor = 0;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"startarmor".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].startarmor = atoi(parse_buf.as_ptr());
            if bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].maxarmor == 0 {
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].maxarmor =
                    bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].startarmor;
            }
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].startarmor =
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].maxarmor;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"speed".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].speed =
                atof(parse_buf.as_ptr()) as f32;
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].speed = 1.0f32;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"uishader".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].uiPortraitShader = 0;
            core::ptr::write_bytes(
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize]
                    .uiPortrait
                    .as_mut_ptr(),
                0,
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize]
                    .uiPortrait
                    .len(),
            );
        } else {
            panic!("Siege class without uishader entry");
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"class_shader".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].classShader = 0;
            let title_length: usize = strlen(parse_buf.as_ptr());
            // Oracle only falls back to SPC_INFANTRY when the loop runs to
            // completion (`i >= SPC_MAX`); an early break on `arrayTitleLength >
            // titleLength` leaves playerClass unchanged. bg_saga.c:1034.
            let mut i: i16 = 0;
            while i < SPC_MAX as i16 {
                let array_title_length: usize = strlen(classTitles[i as usize].as_ptr());
                if array_title_length > title_length {
                    break;
                }

                let hold_buf = parse_buf.as_ptr().add(title_length - array_title_length);
                if Q_strcmp(hold_buf, classTitles[i as usize].as_ptr()) == 0 {
                    bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].playerClass = i;
                    break;
                }
                i += 1;
            }

            // In case the icon name doesn't match up
            if i >= SPC_MAX as i16 {
                bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].playerClass = SPC_INFANTRY as i16;
            }
        } else {
            //No entry!  Bad bad bad
            // Oracle only prints; playerClass keeps its prior value (no
            // SPC_INFANTRY default here). Source: `bg_saga.c:1041-1044`
            let s = format!(
                "ERROR: no class_shader defined for class {}\n",
                cstr_to_str(bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].name.as_ptr())
            );
            crate::g_main::Com_Printf(cstr(&s).as_ptr());
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"holdables".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].invenItems =
                BG_SiegeTranslateGenericTable(
                    parse_buf.as_mut_ptr(),
                    HoldableTable.as_ptr() as *mut stringID_table_t,
                    qtrue,
                );
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].invenItems = 0;
        }

        if BG_SiegeGetPairedValue(
            class_info.as_mut_ptr(),
            c"powerups".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].powerups =
                BG_SiegeTranslateGenericTable(
                    parse_buf.as_mut_ptr(),
                    PowerupTable.as_ptr() as *mut stringID_table_t,
                    qtrue,
                );
        } else {
            bg.bgSiegeClasses[bg.bgNumSiegeClasses as usize].powerups = 0;
        }

        bg.bgNumSiegeClasses += 1;
    }
}

/// Raven `BG_SiegeCountBaseClass`.
///
/// Raven: count the number of like base classes.
/// Source: `oracle/codemp/game/bg_saga.c:1071-1092`
pub fn BG_SiegeCountBaseClass(team: c_int, classIndex: c_short, bg: &BgState) -> c_int {
    unsafe {
        let mut count: c_int = 0;
        let mut i: isize;

        let stm = BG_SiegeFindThemeForTeam(team, bg);
        if stm.is_null() {
            return 0;
        }

        i = 0;
        while i < (*stm).numClasses as isize {
            if (*(*stm).classes[i as usize]).playerClass == classIndex {
                count += 1;
            }
            i += 1;
        }
        count
    }
}

/// Raven `BG_GetUIPortraitFile`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1094-1121`
pub fn BG_GetUIPortraitFile(
    team: c_int,
    classIndex: c_short,
    cntIndex: c_short,
    bg: &BgState,
) -> *mut c_char {
    unsafe {
        let mut count: isize = 0;
        let mut i: isize;

        let stm = BG_SiegeFindThemeForTeam(team, bg);
        if stm.is_null() {
            return core::ptr::null_mut();
        }

        i = 0;
        while i < (*stm).numClasses as isize {
            if (*(*stm).classes[i as usize]).playerClass == classIndex {
                if count == cntIndex as isize {
                    return (*(*stm).classes[i as usize]).uiPortrait.as_mut_ptr();
                }
                count += 1;
            }
            i += 1;
        }

        core::ptr::null_mut()
    }
}

/// Raven `BG_GetUIPortrait`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1123-1150`
pub fn BG_GetUIPortrait(
    team: c_int,
    classIndex: c_short,
    cntIndex: c_short,
    bg: &BgState,
) -> c_int {
    unsafe {
        let mut count: isize = 0;
        let mut i: isize;

        let stm = BG_SiegeFindThemeForTeam(team, bg);
        if stm.is_null() {
            return 0;
        }

        i = 0;
        while i < (*stm).numClasses as isize {
            if (*(*stm).classes[i as usize]).playerClass == classIndex {
                if count == cntIndex as isize {
                    return (*(*stm).classes[i as usize]).uiPortraitShader;
                }
                count += 1;
            }
            i += 1;
        }

        0
    }
}

/// Raven `BG_GetClassOnBaseClass`.
///
/// Raven: this is really getting ugly - looking to get the base class (within
/// a class) based on the index passed in.
/// Source: `oracle/codemp/game/bg_saga.c:1153-1179`
pub fn BG_GetClassOnBaseClass(
    team: c_int,
    classIndex: c_short,
    cntIndex: c_short,
    bg: &BgState,
) -> *mut siegeClass_t {
    unsafe {
        let mut count: isize = 0;
        let mut i: isize;

        let stm = BG_SiegeFindThemeForTeam(team, bg);
        if stm.is_null() {
            return core::ptr::null_mut();
        }

        i = 0;
        while i < (*stm).numClasses as isize {
            if (*(*stm).classes[i as usize]).playerClass == classIndex {
                if count == cntIndex as isize {
                    return (*stm).classes[i as usize];
                }
                count += 1;
            }
            i += 1;
        }

        core::ptr::null_mut()
    }
}

/// Raven `BG_SiegeLoadClasses`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1181-1210`
pub fn BG_SiegeLoadClasses(
    descBuffer: *mut siegeClassDesc_t,
    bg: &mut BgState,
    traps: &dyn BgTraps,
) {
    unsafe {
        let mut num_files: c_int;
        let mut filelen: usize;
        let mut filelist = [0 as c_char; 4096];
        let mut filename = [0 as c_char; MAX_QPATH as usize];
        let mut fileptr: *mut c_char;
        let mut i: isize;

        bg.bgNumSiegeClasses = 0;

        num_files = traps.fs_getfilelist(
            c"ext_data/Siege/Classes".as_ptr(),
            c".scl".as_ptr(),
            filelist.as_mut_ptr(),
            4096,
        );
        fileptr = filelist.as_mut_ptr();

        i = 0;
        while i < num_files as isize {
            filelen = strlen(fileptr);
            strcpy(filename.as_mut_ptr(), c"ext_data/Siege/Classes/".as_ptr());
            strcat(filename.as_mut_ptr(), fileptr);

            if !descBuffer.is_null() {
                BG_SiegeParseClassFile(filename.as_ptr(), &mut *descBuffer.offset(i), bg, traps);
            } else {
                BG_SiegeParseClassFile(filename.as_ptr(), core::ptr::null_mut(), bg, traps);
            }

            fileptr = fileptr.offset((filelen + 1) as isize);
            i += 1;
        }
    }
}

/// Raven `BG_SiegeFindClassByName`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1219-1233`
pub fn BG_SiegeFindClassByName(classname: *const c_char, bg: &BgState) -> *mut siegeClass_t {
    unsafe {
        let mut i: isize = 0;

        while i < bg.bgNumSiegeClasses as isize {
            if Q_stricmp(bg.bgSiegeClasses[i as usize].name.as_ptr(), classname) == 0 {
                return &bg.bgSiegeClasses[i as usize] as *const siegeClass_t as *mut siegeClass_t;
            }
            i += 1;
        }

        core::ptr::null_mut()
    }
}

/// Raven `BG_SiegeParseTeamFile`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1235-1312`
pub fn BG_SiegeParseTeamFile(filename: *const c_char, bg: &mut BgState, traps: &dyn BgTraps) {
    unsafe {
        let mut f: fileHandle_t = 0;
        let mut len: c_int;
        let mut team_info = [0 as c_char; 2048];
        let mut parse_buf = [0 as c_char; 1024];
        let mut look_string = [0 as c_char; 256];
        let mut i: isize = 1;
        let mut success: bool = true;

        len = traps.fs_fopen(filename, &mut f, FS_READ);

        if f == 0 || len >= 2048 {
            return;
        }

        traps.fs_read(team_info.as_mut_ptr() as *mut c_void, len, f);
        traps.fs_fclose(f);
        team_info[len as usize] = 0;

        if BG_SiegeGetPairedValue(
            team_info.as_mut_ptr(),
            c"name".as_ptr() as *mut c_char,
            parse_buf.as_mut_ptr(),
        ) != 0
        {
            strcpy(
                bg.bgSiegeTeams[bg.bgNumSiegeTeams as usize]
                    .name
                    .as_mut_ptr(),
                parse_buf.as_ptr(),
            );
        } else {
            panic!("Siege team with no name definition");
        }

        bg.bgSiegeTeams[bg.bgNumSiegeTeams as usize].friendlyShader = 0;

        bg.bgSiegeTeams[bg.bgNumSiegeTeams as usize].numClasses = 0;

        if BG_SiegeGetValueGroup(
            team_info.as_mut_ptr(),
            c"Classes".as_ptr() as *mut c_char,
            team_info.as_mut_ptr(),
        ) != 0
        {
            while success && i < MAX_SIEGE_CLASSES as isize {
                // Build the lookString for class#i
                let look_string_str = format!("class{}", i);
                strcpy(look_string.as_mut_ptr(), c"class".as_ptr());
                let num_str = format!("{}", i);
                strcat(look_string.as_mut_ptr(), cstr(&num_str).as_ptr());

                success = BG_SiegeGetPairedValue(
                    team_info.as_mut_ptr(),
                    look_string.as_mut_ptr(),
                    parse_buf.as_mut_ptr(),
                ) != 0;

                if !success {
                    break;
                }

                let num_classes = bg.bgSiegeTeams[bg.bgNumSiegeTeams as usize].numClasses as usize;
                let found_class = BG_SiegeFindClassByName(parse_buf.as_ptr(), bg);
                bg.bgSiegeTeams[bg.bgNumSiegeTeams as usize].classes[num_classes] = found_class;

                if bg.bgSiegeTeams[bg.bgNumSiegeTeams as usize].classes[num_classes].is_null() {
                    panic!(
                        "Invalid class specified: '{}'",
                        cstr_to_str(parse_buf.as_ptr())
                    );
                }

                bg.bgSiegeTeams[bg.bgNumSiegeTeams as usize].numClasses += 1;

                i += 1;
            }
        }

        if bg.bgSiegeTeams[bg.bgNumSiegeTeams as usize].numClasses == 0 {
            panic!("Team defined with no allowable classes\n");
        }

        bg.bgNumSiegeTeams += 1;
    }
}

/// Raven `BG_SiegeLoadTeams`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1314-1335`
pub fn BG_SiegeLoadTeams(bg: &mut BgState, traps: &dyn BgTraps) {
    unsafe {
        let mut num_files: c_int;
        let mut filelen: usize;
        let mut filelist = [0 as c_char; 4096];
        let mut filename = [0 as c_char; MAX_QPATH as usize];
        let mut fileptr: *mut c_char;
        let mut i: isize;

        bg.bgNumSiegeTeams = 0;

        num_files = traps.fs_getfilelist(
            c"ext_data/Siege/Teams".as_ptr(),
            c".team".as_ptr(),
            filelist.as_mut_ptr(),
            4096,
        );
        fileptr = filelist.as_mut_ptr();

        i = 0;
        while i < num_files as isize {
            filelen = strlen(fileptr);
            strcpy(filename.as_mut_ptr(), c"ext_data/Siege/Teams/".as_ptr());
            strcat(filename.as_mut_ptr(), fileptr);
            BG_SiegeParseTeamFile(filename.as_ptr(), bg, traps);

            fileptr = fileptr.offset((filelen + 1) as isize);
            i += 1;
        }
    }
}

/// Raven `BG_SiegeFindThemeForTeam`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1344-1356`
pub fn BG_SiegeFindThemeForTeam(team: c_int, bg: &BgState) -> *mut siegeTeam_t {
    unsafe {
        if team == SIEGETEAM_TEAM1 {
            return bg.team1Theme;
        } else if team == SIEGETEAM_TEAM2 {
            return bg.team2Theme;
        }

        core::ptr::null_mut()
    }
}

/// Raven `BG_PrecacheSabersForSiegeTeam`.
///
/// Raven: precache all the sabers for the active classes for the team.
/// Source: `oracle/codemp/game/bg_saga.c:1363-1413`
pub fn BG_PrecacheSabersForSiegeTeam(team: c_int, bg: &mut BgState, traps: &dyn BgTraps) {
    unsafe {
        let mut saber: saberInfo_t = core::mem::zeroed();
        let mut saber_name: *mut c_char;
        let mut s_num: isize;

        let t = BG_SiegeFindThemeForTeam(team, bg);

        if !t.is_null() {
            let mut i: isize = 0;

            while i < (*t).numClasses as isize {
                s_num = 0;

                while s_num < MAX_SABERS as isize {
                    saber_name = match s_num {
                        0 => &mut (*(*t).classes[i as usize]).saber1[0] as *mut c_char,
                        1 => &mut (*(*t).classes[i as usize]).saber2[0] as *mut c_char,
                        _ => core::ptr::null_mut(),
                    };

                    if !saber_name.is_null() && *saber_name != 0 {
                        WP_SaberParseParms(saber_name as *const c_char, &mut saber, bg, traps);
                        if Q_stricmp(saber_name as *const c_char, saber.name.as_ptr()) == 0 {
                            if saber.model[0] != 0 {
                                BG_ModelCache(saber.model.as_ptr(), core::ptr::null(), bg, traps);
                            }
                        }
                    }

                    s_num += 1;
                }

                i += 1;
            }
        }
    }
}

/// Raven `BG_SiegeCheckClassLegality`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1416-1453`
pub fn BG_SiegeCheckClassLegality(team: c_int, classname: *mut c_char, bg: &BgState) -> qboolean {
    unsafe {
        let mut team_ptr: *mut *mut siegeTeam_t = core::ptr::null_mut();
        let mut i: isize = 0;

        if team == SIEGETEAM_TEAM1 {
            team_ptr = &bg.team1Theme as *const *mut siegeTeam_t as *mut *mut siegeTeam_t;
        } else if team == SIEGETEAM_TEAM2 {
            team_ptr = &bg.team2Theme as *const *mut siegeTeam_t as *mut *mut siegeTeam_t;
        } else {
            return qtrue;
        }

        if team_ptr.is_null() || (*team_ptr).is_null() {
            return qtrue;
        }

        while i < (**team_ptr).numClasses as isize {
            if Q_stricmp(classname, (**team_ptr).classes[i as usize].cast()) == 0 {
                return qtrue;
            }
            i += 1;
        }

        strcpy(classname, (*(**team_ptr).classes[0]).name.as_ptr());

        qfalse
    }
}

/// Raven `BG_SiegeFindTeamForTheme`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1455-1471`
pub fn BG_SiegeFindTeamForTheme(themeName: *mut c_char, bg: &BgState) -> *mut siegeTeam_t {
    unsafe {
        let mut i: isize = 0;

        while i < bg.bgNumSiegeTeams as isize {
            if bg.bgSiegeTeams[i as usize].name[0] != 0
                && Q_stricmp(bg.bgSiegeTeams[i as usize].name.as_ptr(), themeName) == 0
            {
                return &bg.bgSiegeTeams[i as usize] as *const siegeTeam_t as *mut siegeTeam_t;
            }

            i += 1;
        }

        core::ptr::null_mut()
    }
}

/// Raven `BG_SiegeSetTeamTheme`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1473-1487`
pub fn BG_SiegeSetTeamTheme(team: c_int, themeName: *mut c_char, bg: &mut BgState) {
    unsafe {
        let mut team_ptr: *mut *mut siegeTeam_t = core::ptr::null_mut();

        if team == SIEGETEAM_TEAM1 {
            team_ptr = &mut bg.team1Theme as *mut *mut siegeTeam_t;
        } else {
            team_ptr = &mut bg.team2Theme as *mut *mut siegeTeam_t;
        }

        *team_ptr = BG_SiegeFindTeamForTheme(themeName, bg);
    }
}

/// Raven `BG_SiegeFindClassIndexByName`.
///
/// Source: `oracle/codemp/game/bg_saga.c:1489-1503`
pub fn BG_SiegeFindClassIndexByName(classname: *const c_char, bg: &BgState) -> c_int {
    unsafe {
        let mut i: isize = 0;

        while i < bg.bgNumSiegeClasses as isize {
            if Q_stricmp(bg.bgSiegeClasses[i as usize].name.as_ptr(), classname) == 0 {
                return i as c_int;
            }
            i += 1;
        }

        -1
    }
}
