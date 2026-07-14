//! `sv_bot.cpp` — the server's botlib interface glue: debug-polygon pool
//! management, botlib shutdown, the bot console-message / snapshot-entity
//! read paths, and bot client-slot allocate/free bookkeeping.
//!
//! Source: `oracle/codemp/server/sv_bot.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_abi::game::exports::MpGameExport;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::cvar_fns::{Cvar_Get, Cvar_VariableIntegerValue};
use mp_engine_qcommon::qcommon::net_limits::{MAX_RELIABLE_COMMANDS, PACKET_MASK};
use mp_engine_qcommon::vm::VM_Call;
use mp_engine_qcommon::vm_fns::BotVMShift;
use mp_engine_qcommon::z_memman_pc::{Z_Free, Z_Malloc};
use mp_qshared::common::mp::botlib::botlib_misc::BOTLIB_API_VERSION;
use mp_qshared::common::mp::game::g_public::SVF_BOT;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::cvar::CVAR_CHEAT;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::ENTITYNUM_NONE;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorMA, _VectorSubtract, vec3_origin, CrossProduct, VectorLength,
    VectorNormalize, VectorSet,
};
use mp_qshared::shared::surface_flags::MASK_SOLID;
use mp_qshared::shared::vec3_t;
use mp_qshared::shared::wpobject::{wpobject_t, MAX_NEIGHBOR_SIZE};
use mp_qshared::shared::{qfalse, qtrue};

use crate::botlib_import::{arm_botlib_slot, botlib_import_table};
use crate::server::bot_debugpoly_t::bot_debugpoly_t;
use crate::server::client_state_t::clientState_t;
use crate::sv_game::SV_GentityNum;
use crate::sv_world::SV_Trace;
use crate::Server;
use mp_engine_botlib::be_interface_fns::GetBotLibAPI;
use mp_engine_botlib::BotLib;
use mp_qshared::shared::q_string::Q_strncpyz;

/// Raven `SV_BotLibSetup`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:595-608`
pub fn SV_BotLibSetup(common: &mut Common, sv: &mut Server, bot: &mut BotLib) -> c_int {
    if common.bot_enable == 0 {
        return 0;
    }

    if sv.botlib_export.is_null() {
        // `S_COLOR_RED` (`^1`).
        com_printf(common, "^1Error: SV_BotLibSetup without SV_BotInitBotLib\n");
        return -1;
    }

    unsafe { ((*sv.botlib_export).BotLibSetup.unwrap())(bot) }
}

/// Raven `SV_BotInitBotLib` — (re)build the botlib debug-polygon pool and hand
/// the `botlib_import_t` engine-service table to the bot library's
/// `GetBotLibAPI` entry point, storing the returned export on `Server`
/// (`sv.botlib_export`).
///
/// The `debugpolygons` pool is (re)allocated from `bot_maxdebugpolys` exactly
/// as Raven does. The `botlib_import` fn-pointer fields are the C-ABI
/// `BotImport_*` engine callbacks (`botlib.h:157-193`); each reaches threaded
/// engine state (`Com_Printf`/`SV_PointContents`/`Z_Malloc`/…) that a bare
/// `extern "C" fn` cannot carry, so the botlib import slot (`botlib_import.rs`)
/// — the state-capture dual of the game `GAME_SLOT` — is armed here with the
/// islands the thunks need, and `botlib_import_table` builds the C-ABI thunk
/// table (four fields deferred; see its doc).
///
/// Source: `oracle/codemp/server/sv_bot.cpp:684-724`
pub fn SV_BotInitBotLib(view: &mut EngineHostView, sv: &mut Server, bot: &mut BotLib) {
    if !sv.bot.debugpolygons.is_null() {
        Z_Free(view.common, sv.bot.debugpolygons as *mut ());
    }
    sv.bot.bot_maxdebugpolys =
        Cvar_VariableIntegerValue(view.common, c"bot_maxdebugpolys".as_ptr());
    sv.bot.debugpolygons = Z_Malloc(
        view,
        core::mem::size_of::<bot_debugpoly_t>() as c_int * sv.bot.bot_maxdebugpolys,
        memtag_t::TAG_BOTLIB,
        qtrue,
        0,
    ) as *mut bot_debugpoly_t;

    // Arm the botlib-import slot with the islands the `BotImport_*` thunks need,
    // then build the `botlib_import_t` table (`botlib_import.rs`) — the
    // state-capture dual of `arm_game_slot`. Raven assigns each field a file-
    // static `BotImport_*` fn (`sv_bot.cpp:691-720`); we assign C-ABI thunks
    // that recover the armed islands and enter the receiver-threaded bodies.
    arm_botlib_slot(view, sv);
    let mut botlib_import = botlib_import_table();

    sv.botlib_export = GetBotLibAPI(bot, BOTLIB_API_VERSION, &mut botlib_import);
    // bk001129 - somehow we end up with a zero import.
    debug_assert!(!sv.botlib_export.is_null());
}

/// Raven `BotImport_DebugPolygonCreate` — claim the first free slot in the
/// debug-polygon pool (starting at index 1, matching Raven), storing the polygon
/// verbatim. Returns the slot index, or `0` if the pool is unallocated/full.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:471-491`
pub fn BotImport_DebugPolygonCreate(
    sv: &mut Server,
    color: c_int,
    numPoints: c_int,
    points: *const vec3_t,
) -> c_int {
    if sv.bot.debugpolygons.is_null() {
        return 0;
    }
    unsafe {
        let mut i: c_int = 1;
        while i < sv.bot.bot_maxdebugpolys {
            if (*sv.bot.debugpolygons.offset(i as isize)).inuse == qfalse {
                break;
            }
            i += 1;
        }
        if i >= sv.bot.bot_maxdebugpolys {
            return 0;
        }
        let poly = sv.bot.debugpolygons.offset(i as isize);
        (*poly).inuse = qtrue;
        (*poly).color = color;
        (*poly).numPoints = numPoints;
        core::ptr::copy_nonoverlapping(points, (*poly).points.as_mut_ptr(), numPoints as usize);
        i
    }
}

/// Raven `BotImport_DebugPolygonShow` — overwrite an existing pool slot by id.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:498-507`
pub fn BotImport_DebugPolygonShow(
    sv: &mut Server,
    id: c_int,
    color: c_int,
    numPoints: c_int,
    points: *const vec3_t,
) {
    if sv.bot.debugpolygons.is_null() {
        return;
    }
    unsafe {
        let poly = sv.bot.debugpolygons.offset(id as isize);
        (*poly).inuse = qtrue;
        (*poly).color = color;
        (*poly).numPoints = numPoints;
        core::ptr::copy_nonoverlapping(points, (*poly).points.as_mut_ptr(), numPoints as usize);
    }
}

/// Raven `BotImport_DebugPolygonDelete`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:514-518`
pub fn BotImport_DebugPolygonDelete(sv: &mut Server, id: c_int) {
    if sv.bot.debugpolygons.is_null() {
        return;
    }
    unsafe {
        (*sv.bot.debugpolygons.offset(id as isize)).inuse = qfalse;
    }
}

/// Raven `BotImport_DebugLineCreate` — a zero-point polygon standing in for a
/// debug line handle.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:525-528`
pub fn BotImport_DebugLineCreate(sv: &mut Server) -> c_int {
    let points: [vec3_t; 1] = [[0.0; 3]];
    BotImport_DebugPolygonCreate(sv, 0, 0, points.as_ptr())
}

/// Raven `BotImport_DebugLineDelete`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:535-537`
pub fn BotImport_DebugLineDelete(sv: &mut Server, line: c_int) {
    BotImport_DebugPolygonDelete(sv, line);
}

/// Raven `BotImport_DebugLineShow` — build a 2-unit-wide quad along the
/// `start`→`end` segment and show it in the given pool slot.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:544-570`
pub fn BotImport_DebugLineShow(
    sv: &mut Server,
    line: c_int,
    start: vec3_t,
    end: vec3_t,
    color: c_int,
) {
    let up: vec3_t = [0.0, 0.0, 1.0];
    let mut points: [vec3_t; 4] = [start, start, end, end];
    let mut dir: vec3_t = [0.0; 3];
    _VectorSubtract(end, start, &mut dir);
    VectorNormalize(&mut dir);
    let dot = _DotProduct(dir, up);
    let mut cross: vec3_t = [0.0; 3];
    if dot > 0.99 || dot < -0.99 {
        VectorSet(&mut cross, 1.0, 0.0, 0.0);
    } else {
        CrossProduct(dir, up, &mut cross);
    }
    VectorNormalize(&mut cross);
    let p0 = points[0];
    _VectorMA(p0, 2.0, cross, &mut points[0]);
    let p1 = points[1];
    _VectorMA(p1, -2.0, cross, &mut points[1]);
    let p2 = points[2];
    _VectorMA(p2, -2.0, cross, &mut points[2]);
    let p3 = points[3];
    _VectorMA(p3, 2.0, cross, &mut points[3]);
    BotImport_DebugPolygonShow(sv, line, color, 4, points.as_ptr());
}

/// Raven `SV_BotLibShutdown`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:619-626`
pub fn SV_BotLibShutdown(sv: &mut Server, bot: &mut BotLib) -> c_int {
    if sv.botlib_export.is_null() {
        return -1;
    }

    unsafe { ((*sv.botlib_export).BotLibShutdown.unwrap())(bot) }
}

/// Raven `SV_BotGetConsoleMessage`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:736-757`
pub fn SV_BotGetConsoleMessage(
    sv: &mut Server,
    client: c_int,
    buf: *mut c_char,
    size: c_int,
) -> c_int {
    unsafe {
        let cl = sv.svs.clients.offset(client as isize);
        (*cl).lastPacketTime = sv.svs.time;

        if (*cl).reliableAcknowledge == (*cl).reliableSequence {
            return qfalse;
        }

        (*cl).reliableAcknowledge += 1;
        let index = ((*cl).reliableAcknowledge & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize;

        if (*cl).reliableCommands[index][0] == 0 {
            return qfalse;
        }

        Q_strncpyz(buf, (*cl).reliableCommands[index].as_ptr(), size);
        qtrue
    }
}

/// Raven `SV_BotGetSnapshotEntity`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:786-796`
pub fn SV_BotGetSnapshotEntity(sv: &mut Server, client: c_int, sequence: c_int) -> c_int {
    unsafe {
        let cl = sv.svs.clients.offset(client as isize);
        let frame = &(*cl).frames[(*cl).netchan.outgoingSequence as usize & PACKET_MASK];
        if sequence < 0 || sequence >= frame.num_entities {
            return -1;
        }
        (*sv.svs
            .snapshotEntities
            .offset(((frame.first_entity + sequence) % sv.svs.numSnapshotEntities) as isize))
        .number
    }
}

/// Raven `MAX_NEIGHBOR_LINK_DISTANCE` — waypoint neighbor-link cutoff distance.
/// Ported locally here (its sole server-side use is [`SV_BotCalculatePaths`]);
/// `mp_game::ai_wpnav` owns the game-side copy of the same `q_shared.h` `#define`.
///
/// Source: `oracle/codemp/game/q_shared.h:996`
const MAX_NEIGHBOR_LINK_DISTANCE: c_int = 128;

/// Raven `DEFAULT_GRID_SPACING` — RMG waypoint grid spacing (see above).
///
/// Source: `oracle/codemp/game/q_shared.h:999`
const DEFAULT_GRID_SPACING: c_int = 400;

/// Raven `NotWithinRange` — file-static waypoint-index proximity test used by
/// [`SV_BotCalculatePaths`] to skip immediate neighbors in the pool ordering.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:25-38`
fn NotWithinRange(base: c_int, extent: c_int) -> c_int {
    if extent > base && base + 5 >= extent {
        return 0;
    }
    if extent < base && base - 5 <= extent {
        return 0;
    }
    1
}

/// Raven `SV_OrgVisibleBox` — trace `org1`→`org2` (a point trace under `rmg`,
/// else a `mins`/`maxs` box trace) and report whether the segment is
/// unobstructed by `MASK_SOLID` world/entities.
///
/// Raven passes `NULL` for `mins`/`maxs` in the `rmg` branch; the by-value
/// `SV_Trace` seam (`sv_world.rs`) can no longer receive `NULL`, so the
/// `vec3_origin` substitution Raven's `SV_Trace` applies for a `NULL` box
/// (`sv_world.cpp:810-812`) is spelled explicitly here — identical behavior.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:40-58`
pub fn SV_OrgVisibleBox(
    view: &mut EngineHostView,
    org1: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    org2: vec3_t,
    ignore: c_int,
    rmg: c_int,
) -> c_int {
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    if rmg != 0 {
        SV_Trace(
            view,
            &mut tr,
            org1,
            vec3_origin,
            vec3_origin,
            org2,
            ignore,
            MASK_SOLID,
            0,
            0,
            10,
        );
    } else {
        SV_Trace(
            view, &mut tr, org1, mins, maxs, org2, ignore, MASK_SOLID, 0, 0, 10,
        );
    }

    if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
        return 1;
    }
    0
}

/// Raven `SV_BotWaypointReception` — receive the game VM's `gWPNum` waypoint
/// objects and cache their (VM-shifted) pointers into the `gWPArray` table for
/// [`SV_BotCalculatePaths`].
///
/// Each `wps[i]` is a game-VM `wpobject_t*`; Raven truncates it to `int` and
/// runs it through `BotVMShift` (always the game VM, `vm.cpp:657-677`),
/// transcribed faithfully (`(int)wps[i]` → `as usize as c_int`, the low-word
/// truncation Raven relies on).
///
/// Source: `oracle/codemp/server/sv_bot.cpp:63-75`
pub fn SV_BotWaypointReception(sv: &mut Server, wpnum: c_int, wps: *mut *mut wpobject_t) {
    sv.bot.gWPNum = wpnum;

    let mut i: c_int = 0;
    while i < sv.bot.gWPNum {
        let wp = unsafe { *wps.offset(i as isize) };
        sv.bot.gWPArray[i as usize] = BotVMShift(sv.gvm, wp as isize) as *mut wpobject_t;
        i += 1;
    }
}

/// Raven `SV_BotCalculatePaths` — rebuild every cached waypoint's neighbor list:
/// clear the old links, then link each pair within `maxNeighborDist`, on the
/// same integer height, and mutually visible (`SV_OrgVisibleBox`).
///
/// `forceJumpable` is Raven's hard-coded `qfalse` (the `CanForceJumpTo` call is
/// commented out at `:139`), so its dependent branches are dead but transcribed
/// verbatim.
///
/// The neighbor-clear loop indexes `neighbors[neighbornum]` with the pool-
/// supplied `neighbornum`, which the build loop can leave at `MAX_NEIGHBOR_SIZE`
/// (the `>= MAX_NEIGHBOR_SIZE` break fires *after* the post-increment); Raven's
/// unchecked C indexing is reproduced with raw-pointer arithmetic (§19) so the
/// in-range cases match byte-for-byte rather than diverging into a bounds panic.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:81-169`
pub fn SV_BotCalculatePaths(view: &mut EngineHostView, sv: &mut Server, rmg: c_int) {
    if sv.bot.gWPNum == 0 {
        return;
    }

    let mut max_neighbor_dist: c_int = MAX_NEIGHBOR_LINK_DISTANCE;
    if rmg != 0 {
        max_neighbor_dist = DEFAULT_GRID_SPACING + (DEFAULT_GRID_SPACING as f32 * 0.5) as c_int;
    }

    let mins: vec3_t = [-15.0, -15.0, -15.0]; //-1
    let maxs: vec3_t = [15.0, 15.0, 15.0]; //1

    unsafe {
        // now clear out all the neighbor data before we recalculate
        let mut i: c_int = 0;
        while i < sv.bot.gWPNum {
            let wp = sv.bot.gWPArray[i as usize];
            if !wp.is_null() && (*wp).inuse != qfalse && (*wp).neighbornum != 0 {
                let neighbors = (*wp).neighbors.as_mut_ptr();
                while (*wp).neighbornum >= 0 {
                    let n = neighbors.offset((*wp).neighbornum as isize);
                    (*n).num = 0;
                    (*n).forceJumpTo = 0;
                    (*wp).neighbornum -= 1;
                }
                (*wp).neighbornum = 0;
            }
            i += 1;
        }

        i = 0;
        while i < sv.bot.gWPNum {
            let wp_i = sv.bot.gWPArray[i as usize];
            if !wp_i.is_null() && (*wp_i).inuse != qfalse {
                let mut c: c_int = 0;
                while c < sv.bot.gWPNum {
                    let wp_c = sv.bot.gWPArray[c as usize];
                    if !wp_c.is_null()
                        && (*wp_c).inuse != qfalse
                        && i != c
                        && NotWithinRange(i, c) != 0
                    {
                        let mut a: vec3_t = [0.0; 3];
                        _VectorSubtract((*wp_i).origin, (*wp_c).origin, &mut a);

                        let n_l_dist = VectorLength(a);
                        let force_jumpable: c_int = qfalse as c_int; //CanForceJumpTo(i, c, nLDist);

                        if (n_l_dist < max_neighbor_dist as f32 || force_jumpable != 0)
                            && ((*wp_i).origin[2] as c_int == (*wp_c).origin[2] as c_int
                                || force_jumpable != 0)
                            && (SV_OrgVisibleBox(
                                view,
                                (*wp_i).origin,
                                mins,
                                maxs,
                                (*wp_c).origin,
                                ENTITYNUM_NONE,
                                rmg,
                            ) != 0
                                || force_jumpable != 0)
                        {
                            let neighbors = (*wp_i).neighbors.as_mut_ptr();
                            let slot = neighbors.offset((*wp_i).neighbornum as isize);
                            (*slot).num = c;
                            if force_jumpable != 0
                                && ((*wp_i).origin[2] as c_int != (*wp_c).origin[2] as c_int
                                    || n_l_dist < max_neighbor_dist as f32)
                            {
                                (*slot).forceJumpTo = 999; //forceJumpable; //FJSR
                            } else {
                                (*slot).forceJumpTo = 0;
                            }
                            (*wp_i).neighbornum += 1;
                        }

                        if (*wp_i).neighbornum >= MAX_NEIGHBOR_SIZE as c_int {
                            break;
                        }
                    }
                    c += 1;
                }
            }
            i += 1;
        }
    }
}

/// Raven `SV_BotAllocateClient`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:178-201`
pub fn SV_BotAllocateClient(common: &mut Common, sv: &mut Server) -> c_int {
    unsafe {
        // find a client slot
        let mut i: c_int = 0;
        let mut cl = sv.svs.clients;
        while i < (*common.sv_maxclients).integer {
            if (*cl).state == clientState_t::CS_FREE {
                break;
            }
            i += 1;
            cl = cl.offset(1);
        }

        if i == (*common.sv_maxclients).integer {
            return -1;
        }

        (*cl).gentity = SV_GentityNum(sv, i);
        (*(*cl).gentity).s.number = i;
        (*cl).state = clientState_t::CS_ACTIVE;
        (*cl).lastPacketTime = sv.svs.time;
        (*cl).netchan.remoteAddress.r#type = netadrtype_t::NA_BOT;
        (*cl).rate = 16384;

        i
    }
}

/// Raven `SV_BotFreeClient`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:208-221`
pub fn SV_BotFreeClient(common: &mut Common, sv: &mut Server, clientNum: c_int) {
    unsafe {
        if clientNum < 0 || clientNum >= (*common.sv_maxclients).integer {
            com_error(
                errorParm_t::ERR_DROP,
                format!("SV_BotFreeClient: bad clientNum: {}", clientNum),
            );
        }
        let cl = sv.svs.clients.offset(clientNum as isize);
        (*cl).state = clientState_t::CS_FREE;
        (*cl).name[0] = 0;
        if !(*cl).gentity.is_null() {
            (*(*cl).gentity).r.svFlags &= !SVF_BOT;
        }
    }
}

/// Raven `SV_BotFrame`.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:586-591`
pub fn SV_BotFrame(common: &mut Common, sv: &mut Server, time: c_int) {
    if common.bot_enable == 0 {
        return;
    }
    // NOTE: maybe the game is already shutdown
    if sv.gvm.is_null() {
        return;
    }
    // Engine-referee replay deliberately does NOT suppress this call: bot
    // CREATION (G_CheckMinimumPlayers) lives INSIDE the module's
    // BotAIStartFrame, so suppressing bot brains also stops bots from ever
    // spawning. Instead the brains re-run — deterministic under the referee's
    // pinned seed + forced msec (verified byte-identical) — and only
    // tape-created human slots are injected (sv_referee.rs).
    VM_Call(
        common,
        sv.gvm,
        MpGameExport::BOTAI_START_FRAME as c_int,
        &[time as isize],
    );
}

/// Raven `SV_BotInitCvars`.
///
/// Initialize bot cvars so they are listed and can be set before the botlib
/// loads.
///
/// Source: `oracle/codemp/server/sv_bot.cpp:633-665`
pub fn SV_BotInitCvars(view: &mut EngineHostView) {
    Cvar_Get(view, c"bot_enable".as_ptr(), c"1".as_ptr(), 0); //enable the bot
    Cvar_Get(view, c"bot_developer".as_ptr(), c"0".as_ptr(), CVAR_CHEAT); //bot developer mode
    Cvar_Get(view, c"bot_debug".as_ptr(), c"0".as_ptr(), CVAR_CHEAT); //enable bot debugging
    Cvar_Get(view, c"bot_maxdebugpolys".as_ptr(), c"2".as_ptr(), 0); //maximum number of debug polys
    Cvar_Get(view, c"bot_groundonly".as_ptr(), c"1".as_ptr(), 0); //only show ground faces of areas
    Cvar_Get(view, c"bot_reachability".as_ptr(), c"0".as_ptr(), 0); //show all reachabilities to other areas
    Cvar_Get(
        view,
        c"bot_visualizejumppads".as_ptr(),
        c"0".as_ptr(),
        CVAR_CHEAT,
    ); //show jumppads
    Cvar_Get(view, c"bot_forceclustering".as_ptr(), c"0".as_ptr(), 0); //force cluster calculations
    Cvar_Get(view, c"bot_forcereachability".as_ptr(), c"0".as_ptr(), 0); //force reachability calculations
    Cvar_Get(view, c"bot_forcewrite".as_ptr(), c"0".as_ptr(), 0); //force writing aas file
    Cvar_Get(view, c"bot_aasoptimize".as_ptr(), c"0".as_ptr(), 0); //no aas file optimisation
    Cvar_Get(view, c"bot_saveroutingcache".as_ptr(), c"0".as_ptr(), 0); //save routing cache
    Cvar_Get(view, c"bot_thinktime".as_ptr(), c"100".as_ptr(), CVAR_CHEAT); //msec the bots thinks
    Cvar_Get(view, c"bot_reloadcharacters".as_ptr(), c"0".as_ptr(), 0); //reload the bot characters each time
    Cvar_Get(view, c"bot_testichat".as_ptr(), c"0".as_ptr(), 0); //test ichats
    Cvar_Get(view, c"bot_testrchat".as_ptr(), c"0".as_ptr(), 0); //test rchats
    Cvar_Get(view, c"bot_testsolid".as_ptr(), c"0".as_ptr(), CVAR_CHEAT); //test for solid areas
    Cvar_Get(
        view,
        c"bot_testclusters".as_ptr(),
        c"0".as_ptr(),
        CVAR_CHEAT,
    ); //test the AAS clusters
    Cvar_Get(view, c"bot_fastchat".as_ptr(), c"0".as_ptr(), 0); //fast chatting bots
    Cvar_Get(view, c"bot_nochat".as_ptr(), c"0".as_ptr(), 0); //disable chats
    Cvar_Get(view, c"bot_pause".as_ptr(), c"0".as_ptr(), CVAR_CHEAT); //pause the bots thinking
    Cvar_Get(view, c"bot_report".as_ptr(), c"0".as_ptr(), CVAR_CHEAT); //get a full report in ctf
    Cvar_Get(view, c"bot_grapple".as_ptr(), c"0".as_ptr(), 0); //enable grapple
    Cvar_Get(view, c"bot_rocketjump".as_ptr(), c"1".as_ptr(), 0); //enable rocket jumping
    Cvar_Get(view, c"bot_challenge".as_ptr(), c"0".as_ptr(), 0); //challenging bot
    Cvar_Get(view, c"bot_minplayers".as_ptr(), c"0".as_ptr(), 0); //minimum players in a team or the game
    Cvar_Get(
        view,
        c"bot_interbreedchar".as_ptr(),
        c"".as_ptr(),
        CVAR_CHEAT,
    ); //bot character used for interbreeding
    Cvar_Get(
        view,
        c"bot_interbreedbots".as_ptr(),
        c"10".as_ptr(),
        CVAR_CHEAT,
    ); //number of bots used for interbreeding
    Cvar_Get(
        view,
        c"bot_interbreedcycle".as_ptr(),
        c"20".as_ptr(),
        CVAR_CHEAT,
    ); //bot interbreeding cycle
    Cvar_Get(
        view,
        c"bot_interbreedwrite".as_ptr(),
        c"".as_ptr(),
        CVAR_CHEAT,
    ); //write interbreeded bots to this file
}
