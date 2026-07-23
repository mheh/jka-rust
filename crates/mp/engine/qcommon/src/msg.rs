#![allow(non_snake_case, non_camel_case_types)]
//! `msg.cpp` — the bit-stream read/write layer (`msg_t`) used for net-channel
//! messages, demo/save serialization, and the entity/playerstate delta coder.
//!
//! Source: `oracle/codemp/qcommon/msg.cpp`

use core::ffi::{c_char, c_int};
use core::mem::offset_of;

use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::shared::force_powers::{FP_LEVITATION, FP_SEE};
use mp_qshared::shared::{errorParm_t, qboolean, qfalse, qtrue};

use crate::qcommon::msg_consts::{FLOAT_INT_BIAS, FLOAT_INT_BITS};
use crate::qcommon::net_field_t::netField_t;

use mp_host_interface::engine_host::EngineHost;
use native_types::byte;

use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;

// Sweep: extern forward-declares eliminated. Real qshared/in-crate callees
// imported.
use crate::common::com_error;

// `MSG_CheckNETFPSFOverrides` callees (netf/psf mod-override reload).
use crate::common::com_printf;
use crate::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Read};
use crate::qcommon::bit_storage_t::bitStorage_t;
use crate::z_memman_pc::Z_Malloc;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::limits::{BIG_INFO_STRING, GENTITYNUM_BITS, MAX_STRING_CHARS};
use native_types::fileHandle_t;
use native_string::{latin1_to_string, string_to_latin1};

// The `sv`/`SV_GentityNum` cross-crate reach (server depends on qcommon) is
// resolved through the sanctioned host edge `EngineHost::
// sv_shownet_entity_classname` (ruling 56c); see `MSG_ReadDeltaEntity`.

/// Raven `MSG_Clear`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:94-98`
pub fn MSG_Clear(buf: *mut msg_t) {
    unsafe {
        (*buf).cursize = 0;
        (*buf).overflowed = qfalse;
        (*buf).bit = 0; //<- in bits
    }
}

/// Raven `MSG_Bitstream`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:101-103`
pub fn MSG_Bitstream(buf: *mut msg_t) {
    unsafe {
        (*buf).oob = qfalse;
    }
}

/// Raven `MSG_BeginReading`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:105-109`
pub fn MSG_BeginReading(msg: *mut msg_t) {
    unsafe {
        (*msg).readcount = 0;
        (*msg).bit = 0;
        (*msg).oob = qfalse;
    }
}

/// Raven `MSG_BeginReadingOOB`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:111-115`
pub fn MSG_BeginReadingOOB(msg: *mut msg_t) {
    unsafe {
        (*msg).readcount = 0;
        (*msg).bit = 0;
        (*msg).oob = qtrue;
    }
}

/// Raven `MSG_shutdownHuffman`. `_NEWHUFFTABLE_`'s `fp`-close body is
/// debug-only tooling with no reachable `fp` global in this port; the
/// `#ifdef` gate is faithfully dead under our build (no `_NEWHUFFTABLE_`).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:3274-3282`
pub fn MSG_shutdownHuffman() {}

/// Raven `msg_hData` — static Huffman frequency table for `MSG_initHuffman`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:2958-3215`
pub const MSG_H_DATA: [i32; 256] = [
    250315, 41193, 6292, 7106, 3730, 3750, 6110, 23283, 33317, 6950, 7838, 9714, 9257, 17259, 3949,
    1778, 8288, 1604, 1590, 1663, 1100, 1213, 1238, 1134, 1749, 1059, 1246, 1149, 1273, 4486, 2805,
    3472, 21819, 1159, 1670, 1066, 1043, 1012, 1053, 1070, 1726, 888, 1180, 850, 960, 780, 1752,
    3296, 10630, 4514, 5881, 2685, 4650, 3837, 2093, 1867, 2584, 1949, 1972, 940, 1134, 1788, 1670,
    1206, 5719, 6128, 7222, 6654, 3710, 3795, 1492, 1524, 2215, 1140, 1355, 971, 2180, 1248, 1328,
    1195, 1770, 1078, 1264, 1266, 1168, 965, 1155, 1186, 1347, 1228, 1529, 1600, 2617, 2048, 2546,
    3275, 2410, 3585, 2504, 2800, 2675, 6146, 3663, 2840, 14253, 3164, 2221, 1687, 3208, 2739,
    3512, 4796, 4091, 3515, 5288, 4016, 7937, 6031, 5360, 3924, 4892, 3743, 4566, 4807, 5852, 6400,
    6225, 8291, 23243, 7838, 7073, 8935, 5437, 4483, 3641, 5256, 5312, 5328, 5370, 3492, 2458,
    1694, 1821, 2121, 1916, 1149, 1516, 1367, 1236, 1029, 1258, 1104, 1245, 1006, 1149, 1025, 1241,
    952, 1287, 997, 1713, 1009, 1187, 879, 1099, 929, 1078, 951, 1656, 930, 1153, 1030, 1262, 1062,
    1214, 1060, 1621, 930, 1106, 912, 1034, 892, 1158, 990, 1175, 850, 1121, 903, 1087, 920, 1144,
    1056, 3462, 2240, 4397, 12136, 7758, 1345, 1307, 3278, 1950, 886, 1023, 1112, 1077, 1042, 1061,
    1071, 1484, 1001, 1096, 915, 1052, 995, 1070, 876, 1111, 851, 1059, 805, 1112, 923, 1103, 817,
    1899, 1872, 976, 841, 1127, 956, 1159, 950, 7791, 954, 1289, 933, 1127, 3207, 1020, 927, 1355,
    768, 1040, 745, 952, 805, 1073, 740, 1013, 805, 1008, 796, 996, 1057, 11457, 13504,
];

/// One `entityStateFields[]`/`playerStateFields[]` row. Raven's `NETF(x)`/
/// `PSF(x)` macros stringize the field expression (`name`) and take its
/// `offsetof` (`offset`); array subscripts add `index * 4` (every indexed field
/// is a 4-byte `int`/`float`).
fn nf(name: &'static str, offset: usize, bits: c_int) -> netField_t {
    netField_t {
        name,
        offset: offset as c_int,
        bits,
        mCount: 0,
    }
}

/// Raven `entityStateFields[]` — the entity-state delta-coder field table
/// (`!_XBOX` branch, including the mod-author `userInt/userFloat/userVec` tail).
/// Order is wire-critical: a field's list index is its network position.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:858-1051`
fn build_entity_state_fields() -> Vec<netField_t> {
    vec![
        nf("pos.trTime", offset_of!(entityState_t, pos.trTime), 32),
        nf(
            "pos.trBase[1]",
            offset_of!(entityState_t, pos.trBase) + 1 * 4,
            0,
        ),
        nf(
            "pos.trBase[0]",
            offset_of!(entityState_t, pos.trBase) + 0 * 4,
            0,
        ),
        nf(
            "apos.trBase[1]",
            offset_of!(entityState_t, apos.trBase) + 1 * 4,
            0,
        ),
        nf(
            "pos.trBase[2]",
            offset_of!(entityState_t, pos.trBase) + 2 * 4,
            0,
        ),
        nf(
            "apos.trBase[0]",
            offset_of!(entityState_t, apos.trBase) + 0 * 4,
            0,
        ),
        nf(
            "pos.trDelta[0]",
            offset_of!(entityState_t, pos.trDelta) + 0 * 4,
            0,
        ),
        nf(
            "pos.trDelta[1]",
            offset_of!(entityState_t, pos.trDelta) + 1 * 4,
            0,
        ),
        nf("eType", offset_of!(entityState_t, eType), 8),
        nf("angles[1]", offset_of!(entityState_t, angles) + 1 * 4, 0),
        nf(
            "pos.trDelta[2]",
            offset_of!(entityState_t, pos.trDelta) + 2 * 4,
            0,
        ),
        nf("origin[0]", offset_of!(entityState_t, origin) + 0 * 4, 0),
        nf("origin[1]", offset_of!(entityState_t, origin) + 1 * 4, 0),
        nf("origin[2]", offset_of!(entityState_t, origin) + 2 * 4, 0),
        nf("weapon", offset_of!(entityState_t, weapon), 8),
        nf("apos.trType", offset_of!(entityState_t, apos.trType), 8),
        nf("legsAnim", offset_of!(entityState_t, legsAnim), 16),
        nf("torsoAnim", offset_of!(entityState_t, torsoAnim), 16),
        nf(
            "genericenemyindex",
            offset_of!(entityState_t, genericenemyindex),
            32,
        ),
        nf("eFlags", offset_of!(entityState_t, eFlags), 32),
        nf(
            "pos.trDuration",
            offset_of!(entityState_t, pos.trDuration),
            32,
        ),
        nf("teamowner", offset_of!(entityState_t, teamowner), 8),
        nf(
            "groundEntityNum",
            offset_of!(entityState_t, groundEntityNum),
            GENTITYNUM_BITS,
        ),
        nf("pos.trType", offset_of!(entityState_t, pos.trType), 8),
        nf("angles[2]", offset_of!(entityState_t, angles) + 2 * 4, 0),
        nf("angles[0]", offset_of!(entityState_t, angles) + 0 * 4, 0),
        nf("solid", offset_of!(entityState_t, solid), 24),
        nf("fireflag", offset_of!(entityState_t, fireflag), 2),
        nf("event", offset_of!(entityState_t, event), 10),
        nf(
            "customRGBA[3]",
            offset_of!(entityState_t, customRGBA) + 3 * 4,
            8,
        ),
        nf(
            "customRGBA[0]",
            offset_of!(entityState_t, customRGBA) + 0 * 4,
            8,
        ),
        nf("speed", offset_of!(entityState_t, speed), 0),
        nf(
            "clientNum",
            offset_of!(entityState_t, clientNum),
            GENTITYNUM_BITS,
        ),
        nf(
            "apos.trBase[2]",
            offset_of!(entityState_t, apos.trBase) + 2 * 4,
            0,
        ),
        nf("apos.trTime", offset_of!(entityState_t, apos.trTime), 32),
        nf(
            "customRGBA[1]",
            offset_of!(entityState_t, customRGBA) + 1 * 4,
            8,
        ),
        nf(
            "customRGBA[2]",
            offset_of!(entityState_t, customRGBA) + 2 * 4,
            8,
        ),
        nf(
            "saberEntityNum",
            offset_of!(entityState_t, saberEntityNum),
            GENTITYNUM_BITS,
        ),
        nf("g2radius", offset_of!(entityState_t, g2radius), 8),
        nf(
            "otherEntityNum2",
            offset_of!(entityState_t, otherEntityNum2),
            GENTITYNUM_BITS,
        ),
        nf("owner", offset_of!(entityState_t, owner), GENTITYNUM_BITS),
        nf("modelindex2", offset_of!(entityState_t, modelindex2), 8),
        nf("eventParm", offset_of!(entityState_t, eventParm), 8),
        nf("saberMove", offset_of!(entityState_t, saberMove), 8),
        nf(
            "apos.trDelta[1]",
            offset_of!(entityState_t, apos.trDelta) + 1 * 4,
            0,
        ),
        nf(
            "boneAngles1[1]",
            offset_of!(entityState_t, boneAngles1) + 1 * 4,
            0,
        ),
        nf("modelindex", offset_of!(entityState_t, modelindex), -16),
        nf(
            "emplacedOwner",
            offset_of!(entityState_t, emplacedOwner),
            32,
        ),
        nf(
            "apos.trDelta[0]",
            offset_of!(entityState_t, apos.trDelta) + 0 * 4,
            0,
        ),
        nf(
            "apos.trDelta[2]",
            offset_of!(entityState_t, apos.trDelta) + 2 * 4,
            0,
        ),
        nf("torsoFlip", offset_of!(entityState_t, torsoFlip), 1),
        nf("angles2[1]", offset_of!(entityState_t, angles2) + 1 * 4, 0),
        nf(
            "lookTarget",
            offset_of!(entityState_t, lookTarget),
            GENTITYNUM_BITS,
        ),
        nf("origin2[2]", offset_of!(entityState_t, origin2) + 2 * 4, 0),
        nf("modelGhoul2", offset_of!(entityState_t, modelGhoul2), 8),
        nf("loopSound", offset_of!(entityState_t, loopSound), 8),
        nf("origin2[0]", offset_of!(entityState_t, origin2) + 0 * 4, 0),
        nf("shouldtarget", offset_of!(entityState_t, shouldtarget), 1),
        nf(
            "trickedentindex",
            offset_of!(entityState_t, trickedentindex),
            16,
        ),
        nf(
            "otherEntityNum",
            offset_of!(entityState_t, otherEntityNum),
            GENTITYNUM_BITS,
        ),
        nf("origin2[1]", offset_of!(entityState_t, origin2) + 1 * 4, 0),
        nf("time2", offset_of!(entityState_t, time2), 32),
        nf("legsFlip", offset_of!(entityState_t, legsFlip), 1),
        nf("bolt2", offset_of!(entityState_t, bolt2), GENTITYNUM_BITS),
        nf(
            "constantLight",
            offset_of!(entityState_t, constantLight),
            32,
        ),
        nf("time", offset_of!(entityState_t, time), 32),
        nf("hasLookTarget", offset_of!(entityState_t, hasLookTarget), 1),
        nf(
            "boneAngles1[2]",
            offset_of!(entityState_t, boneAngles1) + 2 * 4,
            0,
        ),
        nf(
            "activeForcePass",
            offset_of!(entityState_t, activeForcePass),
            6,
        ),
        nf("health", offset_of!(entityState_t, health), 10),
        nf(
            "loopIsSoundset",
            offset_of!(entityState_t, loopIsSoundset),
            1,
        ),
        nf(
            "saberHolstered",
            offset_of!(entityState_t, saberHolstered),
            2,
        ),
        nf("npcSaber1", offset_of!(entityState_t, npcSaber1), 9),
        nf("maxhealth", offset_of!(entityState_t, maxhealth), 10),
        nf(
            "trickedentindex2",
            offset_of!(entityState_t, trickedentindex2),
            16,
        ),
        nf(
            "forcePowersActive",
            offset_of!(entityState_t, forcePowersActive),
            32,
        ),
        nf("iModelScale", offset_of!(entityState_t, iModelScale), 10),
        nf("powerups", offset_of!(entityState_t, powerups), 16),
        nf("soundSetIndex", offset_of!(entityState_t, soundSetIndex), 8),
        nf("brokenLimbs", offset_of!(entityState_t, brokenLimbs), 8),
        nf("csSounds_Std", offset_of!(entityState_t, csSounds_Std), 8),
        nf("saberInFlight", offset_of!(entityState_t, saberInFlight), 1),
        nf("angles2[0]", offset_of!(entityState_t, angles2) + 0 * 4, 0),
        nf("frame", offset_of!(entityState_t, frame), 16),
        nf("angles2[2]", offset_of!(entityState_t, angles2) + 2 * 4, 0),
        nf("forceFrame", offset_of!(entityState_t, forceFrame), 16),
        nf("generic1", offset_of!(entityState_t, generic1), 8),
        nf("boneIndex1", offset_of!(entityState_t, boneIndex1), 6),
        nf("NPC_class", offset_of!(entityState_t, NPC_class), 8),
        nf(
            "apos.trDuration",
            offset_of!(entityState_t, apos.trDuration),
            32,
        ),
        nf("boneOrient", offset_of!(entityState_t, boneOrient), 9),
        nf("bolt1", offset_of!(entityState_t, bolt1), 8),
        nf(
            "trickedentindex3",
            offset_of!(entityState_t, trickedentindex3),
            16,
        ),
        nf(
            "m_iVehicleNum",
            offset_of!(entityState_t, m_iVehicleNum),
            GENTITYNUM_BITS,
        ),
        nf(
            "trickedentindex4",
            offset_of!(entityState_t, trickedentindex4),
            16,
        ),
        nf("surfacesOff", offset_of!(entityState_t, surfacesOff), 32),
        nf("eFlags2", offset_of!(entityState_t, eFlags2), 10),
        nf("isJediMaster", offset_of!(entityState_t, isJediMaster), 1),
        nf("isPortalEnt", offset_of!(entityState_t, isPortalEnt), 1),
        nf("heldByClient", offset_of!(entityState_t, heldByClient), 6),
        nf(
            "ragAttach",
            offset_of!(entityState_t, ragAttach),
            GENTITYNUM_BITS,
        ),
        nf("boltToPlayer", offset_of!(entityState_t, boltToPlayer), 6),
        nf("npcSaber2", offset_of!(entityState_t, npcSaber2), 9),
        nf(
            "csSounds_Combat",
            offset_of!(entityState_t, csSounds_Combat),
            8,
        ),
        nf(
            "csSounds_Extra",
            offset_of!(entityState_t, csSounds_Extra),
            8,
        ),
        nf("csSounds_Jedi", offset_of!(entityState_t, csSounds_Jedi), 8),
        nf("surfacesOn", offset_of!(entityState_t, surfacesOn), 32),
        nf("boneIndex2", offset_of!(entityState_t, boneIndex2), 6),
        nf("boneIndex3", offset_of!(entityState_t, boneIndex3), 6),
        nf("boneIndex4", offset_of!(entityState_t, boneIndex4), 6),
        nf(
            "boneAngles1[0]",
            offset_of!(entityState_t, boneAngles1) + 0 * 4,
            0,
        ),
        nf(
            "boneAngles2[0]",
            offset_of!(entityState_t, boneAngles2) + 0 * 4,
            0,
        ),
        nf(
            "boneAngles2[1]",
            offset_of!(entityState_t, boneAngles2) + 1 * 4,
            0,
        ),
        nf(
            "boneAngles2[2]",
            offset_of!(entityState_t, boneAngles2) + 2 * 4,
            0,
        ),
        nf(
            "boneAngles3[0]",
            offset_of!(entityState_t, boneAngles3) + 0 * 4,
            0,
        ),
        nf(
            "boneAngles3[1]",
            offset_of!(entityState_t, boneAngles3) + 1 * 4,
            0,
        ),
        nf(
            "boneAngles3[2]",
            offset_of!(entityState_t, boneAngles3) + 2 * 4,
            0,
        ),
        nf(
            "boneAngles4[0]",
            offset_of!(entityState_t, boneAngles4) + 0 * 4,
            0,
        ),
        nf(
            "boneAngles4[1]",
            offset_of!(entityState_t, boneAngles4) + 1 * 4,
            0,
        ),
        nf(
            "boneAngles4[2]",
            offset_of!(entityState_t, boneAngles4) + 2 * 4,
            0,
        ),
        nf("userInt1", offset_of!(entityState_t, userInt1), 1),
        nf("userInt2", offset_of!(entityState_t, userInt2), 1),
        nf("userInt3", offset_of!(entityState_t, userInt3), 1),
        nf("userFloat1", offset_of!(entityState_t, userFloat1), 1),
        nf("userFloat2", offset_of!(entityState_t, userFloat2), 1),
        nf("userFloat3", offset_of!(entityState_t, userFloat3), 1),
        nf(
            "userVec1[0]",
            offset_of!(entityState_t, userVec1) + 0 * 4,
            1,
        ),
        nf(
            "userVec1[1]",
            offset_of!(entityState_t, userVec1) + 1 * 4,
            1,
        ),
        nf(
            "userVec1[2]",
            offset_of!(entityState_t, userVec1) + 2 * 4,
            1,
        ),
        nf(
            "userVec2[0]",
            offset_of!(entityState_t, userVec2) + 0 * 4,
            1,
        ),
        nf(
            "userVec2[1]",
            offset_of!(entityState_t, userVec2) + 1 * 4,
            1,
        ),
        nf(
            "userVec2[2]",
            offset_of!(entityState_t, userVec2) + 2 * 4,
            1,
        ),
    ]
}

/// Raven `playerStateFields[]` — the normal-client playerstate delta table.
/// Order is wire-critical. `fd.forcePowerLevel[FP_LEVITATION/FP_SEE]` and
/// `fd.forcePowerDebounce[FP_LEVITATION]` index the nested `forcedata_t`.
///
/// RETAIL-WIRE DIVERGENCE (do NOT regenerate from the oracle source): the
/// source drop's tables (152 rows, both ifdef variants, msg.cpp:1410-1568)
/// postdate the shipped 1.01 build. The retail wire is THIS 137-row set,
/// verified row-identical against a retail-compatible client's compiled
/// tables (TaystJK arm64 binary dump, 2026-07-14; the 152-row table shifts
/// every field index past 66 and drops real clients with
/// `CL_ParsePacketEntities: end of message` when e.g. lookTarget changes).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:1410-1568` (minus the 15
/// never-shipped vehicle rows)
fn build_player_state_fields() -> Vec<netField_t> {
    vec![
        nf("commandTime", offset_of!(playerState_t, commandTime), 32),
        nf("origin[1]", offset_of!(playerState_t, origin) + 1 * 4, 0),
        nf("origin[0]", offset_of!(playerState_t, origin) + 0 * 4, 0),
        nf(
            "viewangles[1]",
            offset_of!(playerState_t, viewangles) + 1 * 4,
            0,
        ),
        nf(
            "viewangles[0]",
            offset_of!(playerState_t, viewangles) + 0 * 4,
            0,
        ),
        nf("origin[2]", offset_of!(playerState_t, origin) + 2 * 4, 0),
        nf(
            "velocity[0]",
            offset_of!(playerState_t, velocity) + 0 * 4,
            0,
        ),
        nf(
            "velocity[1]",
            offset_of!(playerState_t, velocity) + 1 * 4,
            0,
        ),
        nf(
            "velocity[2]",
            offset_of!(playerState_t, velocity) + 2 * 4,
            0,
        ),
        nf("bobCycle", offset_of!(playerState_t, bobCycle), 8),
        nf("weaponTime", offset_of!(playerState_t, weaponTime), -16),
        nf(
            "delta_angles[1]",
            offset_of!(playerState_t, delta_angles) + 1 * 4,
            16,
        ),
        nf("speed", offset_of!(playerState_t, speed), 0),
        nf("legsAnim", offset_of!(playerState_t, legsAnim), 16),
        nf(
            "delta_angles[0]",
            offset_of!(playerState_t, delta_angles) + 0 * 4,
            16,
        ),
        nf("torsoAnim", offset_of!(playerState_t, torsoAnim), 16),
        nf(
            "groundEntityNum",
            offset_of!(playerState_t, groundEntityNum),
            GENTITYNUM_BITS,
        ),
        nf("eFlags", offset_of!(playerState_t, eFlags), 32),
        nf("fd.forcePower", offset_of!(playerState_t, fd.forcePower), 8),
        nf(
            "eventSequence",
            offset_of!(playerState_t, eventSequence),
            16,
        ),
        nf("torsoTimer", offset_of!(playerState_t, torsoTimer), 16),
        nf("legsTimer", offset_of!(playerState_t, legsTimer), 16),
        nf("viewheight", offset_of!(playerState_t, viewheight), -8),
        nf(
            "fd.saberAnimLevel",
            offset_of!(playerState_t, fd.saberAnimLevel),
            4,
        ),
        nf(
            "rocketLockIndex",
            offset_of!(playerState_t, rocketLockIndex),
            GENTITYNUM_BITS,
        ),
        nf(
            "fd.saberDrawAnimLevel",
            offset_of!(playerState_t, fd.saberDrawAnimLevel),
            4,
        ),
        nf(
            "genericEnemyIndex",
            offset_of!(playerState_t, genericEnemyIndex),
            32,
        ),
        nf("events[0]", offset_of!(playerState_t, events) + 0 * 4, 10),
        nf("events[1]", offset_of!(playerState_t, events) + 1 * 4, 10),
        nf(
            "customRGBA[0]",
            offset_of!(playerState_t, customRGBA) + 0 * 4,
            8,
        ),
        nf("movementDir", offset_of!(playerState_t, movementDir), 4),
        nf(
            "saberEntityNum",
            offset_of!(playerState_t, saberEntityNum),
            GENTITYNUM_BITS,
        ),
        nf(
            "customRGBA[3]",
            offset_of!(playerState_t, customRGBA) + 3 * 4,
            8,
        ),
        nf("weaponstate", offset_of!(playerState_t, weaponstate), 4),
        nf("saberMove", offset_of!(playerState_t, saberMove), 32),
        nf("standheight", offset_of!(playerState_t, standheight), 10),
        nf("crouchheight", offset_of!(playerState_t, crouchheight), 10),
        nf("basespeed", offset_of!(playerState_t, basespeed), -16),
        nf("pm_flags", offset_of!(playerState_t, pm_flags), 16),
        nf("jetpackFuel", offset_of!(playerState_t, jetpackFuel), 8),
        nf("cloakFuel", offset_of!(playerState_t, cloakFuel), 8),
        nf("pm_time", offset_of!(playerState_t, pm_time), -16),
        nf(
            "customRGBA[1]",
            offset_of!(playerState_t, customRGBA) + 1 * 4,
            8,
        ),
        nf(
            "clientNum",
            offset_of!(playerState_t, clientNum),
            GENTITYNUM_BITS,
        ),
        nf(
            "duelIndex",
            offset_of!(playerState_t, duelIndex),
            GENTITYNUM_BITS,
        ),
        nf(
            "customRGBA[2]",
            offset_of!(playerState_t, customRGBA) + 2 * 4,
            8,
        ),
        nf("gravity", offset_of!(playerState_t, gravity), 16),
        nf("weapon", offset_of!(playerState_t, weapon), 8),
        nf(
            "delta_angles[2]",
            offset_of!(playerState_t, delta_angles) + 2 * 4,
            16,
        ),
        nf("saberCanThrow", offset_of!(playerState_t, saberCanThrow), 1),
        nf(
            "viewangles[2]",
            offset_of!(playerState_t, viewangles) + 2 * 4,
            0,
        ),
        nf(
            "fd.forcePowersKnown",
            offset_of!(playerState_t, fd.forcePowersKnown),
            32,
        ),
        nf(
            "fd.forcePowerLevel[FP_LEVITATION]",
            offset_of!(playerState_t, fd.forcePowerLevel) + FP_LEVITATION as usize * 4,
            2,
        ),
        nf(
            "fd.forcePowerDebounce[FP_LEVITATION]",
            offset_of!(playerState_t, fd.forcePowerDebounce) + FP_LEVITATION as usize * 4,
            32,
        ),
        nf(
            "fd.forcePowerSelected",
            offset_of!(playerState_t, fd.forcePowerSelected),
            8,
        ),
        nf("torsoFlip", offset_of!(playerState_t, torsoFlip), 1),
        nf(
            "externalEvent",
            offset_of!(playerState_t, externalEvent),
            10,
        ),
        nf("damageYaw", offset_of!(playerState_t, damageYaw), 8),
        nf("damageCount", offset_of!(playerState_t, damageCount), 8),
        nf("inAirAnim", offset_of!(playerState_t, inAirAnim), 1),
        nf(
            "eventParms[1]",
            offset_of!(playerState_t, eventParms) + 1 * 4,
            8,
        ),
        nf("fd.forceSide", offset_of!(playerState_t, fd.forceSide), 2),
        nf(
            "saberAttackChainCount",
            offset_of!(playerState_t, saberAttackChainCount),
            4,
        ),
        nf("pm_type", offset_of!(playerState_t, pm_type), 8),
        nf(
            "externalEventParm",
            offset_of!(playerState_t, externalEventParm),
            8,
        ),
        nf(
            "eventParms[0]",
            offset_of!(playerState_t, eventParms) + 0 * 4,
            -16,
        ),
        nf(
            "lookTarget",
            offset_of!(playerState_t, lookTarget),
            GENTITYNUM_BITS,
        ),
        nf(
            "weaponChargeSubtractTime",
            offset_of!(playerState_t, weaponChargeSubtractTime),
            32,
        ),
        nf(
            "weaponChargeTime",
            offset_of!(playerState_t, weaponChargeTime),
            32,
        ),
        nf("legsFlip", offset_of!(playerState_t, legsFlip), 1),
        nf("damageEvent", offset_of!(playerState_t, damageEvent), 8),
        nf(
            "rocketTargetTime",
            offset_of!(playerState_t, rocketTargetTime),
            32,
        ),
        nf(
            "activeForcePass",
            offset_of!(playerState_t, activeForcePass),
            6,
        ),
        nf(
            "electrifyTime",
            offset_of!(playerState_t, electrifyTime),
            32,
        ),
        nf(
            "fd.forceJumpZStart",
            offset_of!(playerState_t, fd.forceJumpZStart),
            0,
        ),
        nf("loopSound", offset_of!(playerState_t, loopSound), 16),
        nf("hasLookTarget", offset_of!(playerState_t, hasLookTarget), 1),
        nf("saberBlocked", offset_of!(playerState_t, saberBlocked), 8),
        nf("damageType", offset_of!(playerState_t, damageType), 2),
        nf(
            "rocketLockTime",
            offset_of!(playerState_t, rocketLockTime),
            32,
        ),
        nf(
            "forceHandExtend",
            offset_of!(playerState_t, forceHandExtend),
            8,
        ),
        nf(
            "saberHolstered",
            offset_of!(playerState_t, saberHolstered),
            2,
        ),
        nf(
            "fd.forcePowersActive",
            offset_of!(playerState_t, fd.forcePowersActive),
            32,
        ),
        nf("damagePitch", offset_of!(playerState_t, damagePitch), 8),
        nf(
            "m_iVehicleNum",
            offset_of!(playerState_t, m_iVehicleNum),
            GENTITYNUM_BITS,
        ),
        nf("generic1", offset_of!(playerState_t, generic1), 8),
        nf("jumppad_ent", offset_of!(playerState_t, jumppad_ent), 10),
        nf(
            "hasDetPackPlanted",
            offset_of!(playerState_t, hasDetPackPlanted),
            1,
        ),
        nf("saberInFlight", offset_of!(playerState_t, saberInFlight), 1),
        nf(
            "forceDodgeAnim",
            offset_of!(playerState_t, forceDodgeAnim),
            16,
        ),
        nf("zoomMode", offset_of!(playerState_t, zoomMode), 2),
        nf("hackingTime", offset_of!(playerState_t, hackingTime), 32),
        nf("zoomTime", offset_of!(playerState_t, zoomTime), 32),
        nf("brokenLimbs", offset_of!(playerState_t, brokenLimbs), 8),
        nf("zoomLocked", offset_of!(playerState_t, zoomLocked), 1),
        nf("zoomFov", offset_of!(playerState_t, zoomFov), 0),
        nf(
            "fd.forceRageRecoveryTime",
            offset_of!(playerState_t, fd.forceRageRecoveryTime),
            32,
        ),
        nf(
            "fallingToDeath",
            offset_of!(playerState_t, fallingToDeath),
            32,
        ),
        nf(
            "fd.forceMindtrickTargetIndex",
            offset_of!(playerState_t, fd.forceMindtrickTargetIndex),
            16,
        ),
        nf(
            "fd.forceMindtrickTargetIndex2",
            offset_of!(playerState_t, fd.forceMindtrickTargetIndex2),
            16,
        ),
        nf(
            "lastHitLoc[2]",
            offset_of!(playerState_t, lastHitLoc) + 2 * 4,
            0,
        ),
        nf(
            "fd.forceMindtrickTargetIndex3",
            offset_of!(playerState_t, fd.forceMindtrickTargetIndex3),
            16,
        ),
        nf(
            "lastHitLoc[0]",
            offset_of!(playerState_t, lastHitLoc) + 0 * 4,
            0,
        ),
        nf("eFlags2", offset_of!(playerState_t, eFlags2), 10),
        nf(
            "fd.forceMindtrickTargetIndex4",
            offset_of!(playerState_t, fd.forceMindtrickTargetIndex4),
            16,
        ),
        nf(
            "lastHitLoc[1]",
            offset_of!(playerState_t, lastHitLoc) + 1 * 4,
            0,
        ),
        nf(
            "fd.sentryDeployed",
            offset_of!(playerState_t, fd.sentryDeployed),
            1,
        ),
        nf(
            "saberLockTime",
            offset_of!(playerState_t, saberLockTime),
            32,
        ),
        nf(
            "saberLockFrame",
            offset_of!(playerState_t, saberLockFrame),
            16,
        ),
        nf(
            "fd.forcePowerLevel[FP_SEE]",
            offset_of!(playerState_t, fd.forcePowerLevel) + FP_SEE as usize * 4,
            2,
        ),
        nf(
            "saberLockEnemy",
            offset_of!(playerState_t, saberLockEnemy),
            GENTITYNUM_BITS,
        ),
        nf(
            "fd.forceGripCripple",
            offset_of!(playerState_t, fd.forceGripCripple),
            1,
        ),
        nf(
            "emplacedIndex",
            offset_of!(playerState_t, emplacedIndex),
            GENTITYNUM_BITS,
        ),
        nf("holocronBits", offset_of!(playerState_t, holocronBits), 32),
        nf("isJediMaster", offset_of!(playerState_t, isJediMaster), 1),
        nf(
            "forceRestricted",
            offset_of!(playerState_t, forceRestricted),
            1,
        ),
        nf("trueJedi", offset_of!(playerState_t, trueJedi), 1),
        nf("trueNonJedi", offset_of!(playerState_t, trueNonJedi), 1),
        nf("duelTime", offset_of!(playerState_t, duelTime), 32),
        nf(
            "duelInProgress",
            offset_of!(playerState_t, duelInProgress),
            1,
        ),
        nf(
            "saberLockAdvance",
            offset_of!(playerState_t, saberLockAdvance),
            1,
        ),
        nf("heldByClient", offset_of!(playerState_t, heldByClient), 6),
        nf(
            "ragAttach",
            offset_of!(playerState_t, ragAttach),
            GENTITYNUM_BITS,
        ),
        nf("iModelScale", offset_of!(playerState_t, iModelScale), 10),
        nf(
            "hackingBaseTime",
            offset_of!(playerState_t, hackingBaseTime),
            16,
        ),
        nf("userInt1", offset_of!(playerState_t, userInt1), 1),
        nf("userInt2", offset_of!(playerState_t, userInt2), 1),
        nf("userInt3", offset_of!(playerState_t, userInt3), 1),
        nf("userFloat1", offset_of!(playerState_t, userFloat1), 1),
        nf("userFloat2", offset_of!(playerState_t, userFloat2), 1),
        nf("userFloat3", offset_of!(playerState_t, userFloat3), 1),
        nf(
            "userVec1[0]",
            offset_of!(playerState_t, userVec1) + 0 * 4,
            1,
        ),
        nf(
            "userVec1[1]",
            offset_of!(playerState_t, userVec1) + 1 * 4,
            1,
        ),
        nf(
            "userVec1[2]",
            offset_of!(playerState_t, userVec1) + 2 * 4,
            1,
        ),
        nf(
            "userVec2[0]",
            offset_of!(playerState_t, userVec2) + 0 * 4,
            1,
        ),
        nf(
            "userVec2[1]",
            offset_of!(playerState_t, userVec2) + 1 * 4,
            1,
        ),
        nf(
            "userVec2[2]",
            offset_of!(playerState_t, userVec2) + 2 * 4,
            1,
        ),
    ]
}

/// Raven `pilotPlayerStateFields[]` — the pilot-riding-inside-a-vehicle delta
/// table (live: `_OPTIMIZED_VEHICLE_NETWORKING` is unconditionally defined,
/// `q_shared.h:2154`). Order is wire-critical. Only the first
/// `len - 82` (= 58) entries are ever coded — Raven's
/// `sizeof(pilotPlayerStateFields)/sizeof([0]) - 82` — but the full 140-entry
/// table is transcribed faithfully.
///
/// RETAIL-WIRE DIVERGENCE: 140 rows, not the source drop's 152 — see
/// [`build_player_state_fields`]; verified against the same client binary.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:1570-1734` (minus never-shipped
/// vehicle rows)
fn build_pilot_player_state_fields() -> Vec<netField_t> {
    vec![
        nf("commandTime", offset_of!(playerState_t, commandTime), 32),
        nf("origin[1]", offset_of!(playerState_t, origin) + 1 * 4, 0),
        nf("origin[0]", offset_of!(playerState_t, origin) + 0 * 4, 0),
        nf(
            "viewangles[1]",
            offset_of!(playerState_t, viewangles) + 1 * 4,
            0,
        ),
        nf(
            "viewangles[0]",
            offset_of!(playerState_t, viewangles) + 0 * 4,
            0,
        ),
        nf("origin[2]", offset_of!(playerState_t, origin) + 2 * 4, 0),
        nf("weaponTime", offset_of!(playerState_t, weaponTime), -16),
        nf(
            "delta_angles[1]",
            offset_of!(playerState_t, delta_angles) + 1 * 4,
            16,
        ),
        nf(
            "delta_angles[0]",
            offset_of!(playerState_t, delta_angles) + 0 * 4,
            16,
        ),
        nf("eFlags", offset_of!(playerState_t, eFlags), 32),
        nf(
            "eventSequence",
            offset_of!(playerState_t, eventSequence),
            16,
        ),
        nf(
            "rocketLockIndex",
            offset_of!(playerState_t, rocketLockIndex),
            GENTITYNUM_BITS,
        ),
        nf("events[0]", offset_of!(playerState_t, events) + 0 * 4, 10),
        nf("events[1]", offset_of!(playerState_t, events) + 1 * 4, 10),
        nf("weaponstate", offset_of!(playerState_t, weaponstate), 4),
        nf("pm_flags", offset_of!(playerState_t, pm_flags), 16),
        nf("pm_time", offset_of!(playerState_t, pm_time), -16),
        nf(
            "clientNum",
            offset_of!(playerState_t, clientNum),
            GENTITYNUM_BITS,
        ),
        nf("weapon", offset_of!(playerState_t, weapon), 8),
        nf(
            "delta_angles[2]",
            offset_of!(playerState_t, delta_angles) + 2 * 4,
            16,
        ),
        nf(
            "viewangles[2]",
            offset_of!(playerState_t, viewangles) + 2 * 4,
            0,
        ),
        nf(
            "externalEvent",
            offset_of!(playerState_t, externalEvent),
            10,
        ),
        nf(
            "eventParms[1]",
            offset_of!(playerState_t, eventParms) + 1 * 4,
            8,
        ),
        nf("pm_type", offset_of!(playerState_t, pm_type), 8),
        nf(
            "externalEventParm",
            offset_of!(playerState_t, externalEventParm),
            8,
        ),
        nf(
            "eventParms[0]",
            offset_of!(playerState_t, eventParms) + 0 * 4,
            -16,
        ),
        nf(
            "weaponChargeSubtractTime",
            offset_of!(playerState_t, weaponChargeSubtractTime),
            32,
        ),
        nf(
            "weaponChargeTime",
            offset_of!(playerState_t, weaponChargeTime),
            32,
        ),
        nf(
            "rocketTargetTime",
            offset_of!(playerState_t, rocketTargetTime),
            32,
        ),
        nf(
            "fd.forceJumpZStart",
            offset_of!(playerState_t, fd.forceJumpZStart),
            0,
        ),
        nf(
            "rocketLockTime",
            offset_of!(playerState_t, rocketLockTime),
            32,
        ),
        nf(
            "m_iVehicleNum",
            offset_of!(playerState_t, m_iVehicleNum),
            GENTITYNUM_BITS,
        ),
        nf("generic1", offset_of!(playerState_t, generic1), 8),
        nf("eFlags2", offset_of!(playerState_t, eFlags2), 10),
        //===THESE SHOULD NOT BE CHANGING OFTEN====================================================================
        nf("legsAnim", offset_of!(playerState_t, legsAnim), 16),
        nf("torsoAnim", offset_of!(playerState_t, torsoAnim), 16),
        nf("torsoTimer", offset_of!(playerState_t, torsoTimer), 16),
        nf("legsTimer", offset_of!(playerState_t, legsTimer), 16),
        nf("jetpackFuel", offset_of!(playerState_t, jetpackFuel), 8),
        nf("cloakFuel", offset_of!(playerState_t, cloakFuel), 8),
        nf("saberCanThrow", offset_of!(playerState_t, saberCanThrow), 1),
        nf(
            "fd.forcePowerDebounce[FP_LEVITATION]",
            offset_of!(playerState_t, fd.forcePowerDebounce) + FP_LEVITATION as usize * 4,
            32,
        ),
        nf("torsoFlip", offset_of!(playerState_t, torsoFlip), 1),
        nf("legsFlip", offset_of!(playerState_t, legsFlip), 1),
        nf(
            "fd.forcePowersActive",
            offset_of!(playerState_t, fd.forcePowersActive),
            32,
        ),
        nf(
            "hasDetPackPlanted",
            offset_of!(playerState_t, hasDetPackPlanted),
            1,
        ),
        nf(
            "fd.forceRageRecoveryTime",
            offset_of!(playerState_t, fd.forceRageRecoveryTime),
            32,
        ),
        nf("saberInFlight", offset_of!(playerState_t, saberInFlight), 1),
        nf(
            "fd.forceMindtrickTargetIndex",
            offset_of!(playerState_t, fd.forceMindtrickTargetIndex),
            16,
        ),
        nf(
            "fd.forceMindtrickTargetIndex2",
            offset_of!(playerState_t, fd.forceMindtrickTargetIndex2),
            16,
        ),
        nf(
            "fd.forceMindtrickTargetIndex3",
            offset_of!(playerState_t, fd.forceMindtrickTargetIndex3),
            16,
        ),
        nf(
            "fd.forceMindtrickTargetIndex4",
            offset_of!(playerState_t, fd.forceMindtrickTargetIndex4),
            16,
        ),
        nf(
            "fd.sentryDeployed",
            offset_of!(playerState_t, fd.sentryDeployed),
            1,
        ),
        nf(
            "fd.forcePowerLevel[FP_SEE]",
            offset_of!(playerState_t, fd.forcePowerLevel) + FP_SEE as usize * 4,
            2,
        ),
        nf("holocronBits", offset_of!(playerState_t, holocronBits), 32),
        nf("fd.forcePower", offset_of!(playerState_t, fd.forcePower), 8),
        //===THE REST OF THESE SHOULD NOT BE RELEVANT, BUT, FOR SAFETY, INCLUDE THEM ANYWAY, JUST AT THE BOTTOM===============================================================
        nf(
            "velocity[0]",
            offset_of!(playerState_t, velocity) + 0 * 4,
            0,
        ),
        nf(
            "velocity[1]",
            offset_of!(playerState_t, velocity) + 1 * 4,
            0,
        ),
        nf(
            "velocity[2]",
            offset_of!(playerState_t, velocity) + 2 * 4,
            0,
        ),
        nf("bobCycle", offset_of!(playerState_t, bobCycle), 8),
        nf("speed", offset_of!(playerState_t, speed), 0),
        nf(
            "groundEntityNum",
            offset_of!(playerState_t, groundEntityNum),
            GENTITYNUM_BITS,
        ),
        nf("viewheight", offset_of!(playerState_t, viewheight), -8),
        nf(
            "fd.saberAnimLevel",
            offset_of!(playerState_t, fd.saberAnimLevel),
            4,
        ),
        nf(
            "fd.saberDrawAnimLevel",
            offset_of!(playerState_t, fd.saberDrawAnimLevel),
            4,
        ),
        nf(
            "genericEnemyIndex",
            offset_of!(playerState_t, genericEnemyIndex),
            32,
        ),
        nf(
            "customRGBA[0]",
            offset_of!(playerState_t, customRGBA) + 0 * 4,
            8,
        ),
        nf("movementDir", offset_of!(playerState_t, movementDir), 4),
        nf(
            "saberEntityNum",
            offset_of!(playerState_t, saberEntityNum),
            GENTITYNUM_BITS,
        ),
        nf(
            "customRGBA[3]",
            offset_of!(playerState_t, customRGBA) + 3 * 4,
            8,
        ),
        nf("saberMove", offset_of!(playerState_t, saberMove), 32),
        nf("standheight", offset_of!(playerState_t, standheight), 10),
        nf("crouchheight", offset_of!(playerState_t, crouchheight), 10),
        nf("basespeed", offset_of!(playerState_t, basespeed), -16),
        nf(
            "customRGBA[1]",
            offset_of!(playerState_t, customRGBA) + 1 * 4,
            8,
        ),
        nf(
            "duelIndex",
            offset_of!(playerState_t, duelIndex),
            GENTITYNUM_BITS,
        ),
        nf(
            "customRGBA[2]",
            offset_of!(playerState_t, customRGBA) + 2 * 4,
            8,
        ),
        nf("gravity", offset_of!(playerState_t, gravity), 16),
        nf(
            "fd.forcePowersKnown",
            offset_of!(playerState_t, fd.forcePowersKnown),
            32,
        ),
        nf(
            "fd.forcePowerLevel[FP_LEVITATION]",
            offset_of!(playerState_t, fd.forcePowerLevel) + FP_LEVITATION as usize * 4,
            2,
        ),
        nf(
            "fd.forcePowerSelected",
            offset_of!(playerState_t, fd.forcePowerSelected),
            8,
        ),
        nf("damageYaw", offset_of!(playerState_t, damageYaw), 8),
        nf("damageCount", offset_of!(playerState_t, damageCount), 8),
        nf("inAirAnim", offset_of!(playerState_t, inAirAnim), 1),
        nf("fd.forceSide", offset_of!(playerState_t, fd.forceSide), 2),
        nf(
            "saberAttackChainCount",
            offset_of!(playerState_t, saberAttackChainCount),
            4,
        ),
        nf(
            "lookTarget",
            offset_of!(playerState_t, lookTarget),
            GENTITYNUM_BITS,
        ),
        nf("moveDir[1]", offset_of!(playerState_t, moveDir) + 1 * 4, 0),
        nf("moveDir[0]", offset_of!(playerState_t, moveDir) + 0 * 4, 0),
        nf("damageEvent", offset_of!(playerState_t, damageEvent), 8),
        nf("moveDir[2]", offset_of!(playerState_t, moveDir) + 2 * 4, 0),
        nf(
            "activeForcePass",
            offset_of!(playerState_t, activeForcePass),
            6,
        ),
        nf(
            "electrifyTime",
            offset_of!(playerState_t, electrifyTime),
            32,
        ),
        nf("damageType", offset_of!(playerState_t, damageType), 2),
        nf("loopSound", offset_of!(playerState_t, loopSound), 16),
        nf("hasLookTarget", offset_of!(playerState_t, hasLookTarget), 1),
        nf("saberBlocked", offset_of!(playerState_t, saberBlocked), 8),
        nf(
            "forceHandExtend",
            offset_of!(playerState_t, forceHandExtend),
            8,
        ),
        nf(
            "saberHolstered",
            offset_of!(playerState_t, saberHolstered),
            2,
        ),
        nf("damagePitch", offset_of!(playerState_t, damagePitch), 8),
        nf("jumppad_ent", offset_of!(playerState_t, jumppad_ent), 10),
        nf(
            "forceDodgeAnim",
            offset_of!(playerState_t, forceDodgeAnim),
            16,
        ),
        nf("zoomMode", offset_of!(playerState_t, zoomMode), 2),
        nf("hackingTime", offset_of!(playerState_t, hackingTime), 32),
        nf("zoomTime", offset_of!(playerState_t, zoomTime), 32),
        nf("brokenLimbs", offset_of!(playerState_t, brokenLimbs), 8),
        nf("zoomLocked", offset_of!(playerState_t, zoomLocked), 1),
        nf("zoomFov", offset_of!(playerState_t, zoomFov), 0),
        nf(
            "fallingToDeath",
            offset_of!(playerState_t, fallingToDeath),
            32,
        ),
        nf(
            "lastHitLoc[2]",
            offset_of!(playerState_t, lastHitLoc) + 2 * 4,
            0,
        ),
        nf(
            "lastHitLoc[0]",
            offset_of!(playerState_t, lastHitLoc) + 0 * 4,
            0,
        ),
        nf(
            "lastHitLoc[1]",
            offset_of!(playerState_t, lastHitLoc) + 1 * 4,
            0,
        ),
        nf(
            "saberLockTime",
            offset_of!(playerState_t, saberLockTime),
            32,
        ),
        nf(
            "saberLockFrame",
            offset_of!(playerState_t, saberLockFrame),
            16,
        ),
        nf(
            "saberLockEnemy",
            offset_of!(playerState_t, saberLockEnemy),
            GENTITYNUM_BITS,
        ),
        nf(
            "fd.forceGripCripple",
            offset_of!(playerState_t, fd.forceGripCripple),
            1,
        ),
        nf(
            "emplacedIndex",
            offset_of!(playerState_t, emplacedIndex),
            GENTITYNUM_BITS,
        ),
        nf("isJediMaster", offset_of!(playerState_t, isJediMaster), 1),
        nf(
            "forceRestricted",
            offset_of!(playerState_t, forceRestricted),
            1,
        ),
        nf("trueJedi", offset_of!(playerState_t, trueJedi), 1),
        nf("trueNonJedi", offset_of!(playerState_t, trueNonJedi), 1),
        nf("duelTime", offset_of!(playerState_t, duelTime), 32),
        nf(
            "duelInProgress",
            offset_of!(playerState_t, duelInProgress),
            1,
        ),
        nf(
            "saberLockAdvance",
            offset_of!(playerState_t, saberLockAdvance),
            1,
        ),
        nf("heldByClient", offset_of!(playerState_t, heldByClient), 6),
        nf(
            "ragAttach",
            offset_of!(playerState_t, ragAttach),
            GENTITYNUM_BITS,
        ),
        nf("iModelScale", offset_of!(playerState_t, iModelScale), 10),
        nf(
            "hackingBaseTime",
            offset_of!(playerState_t, hackingBaseTime),
            16,
        ),
        //===NEVER SEND THESE, ONLY USED BY VEHICLES============================
        // (Raven's veh* entries here are commented out in the oracle.)
        nf("userInt1", offset_of!(playerState_t, userInt1), 1),
        nf("userInt2", offset_of!(playerState_t, userInt2), 1),
        nf("userInt3", offset_of!(playerState_t, userInt3), 1),
        nf("userFloat1", offset_of!(playerState_t, userFloat1), 1),
        nf("userFloat2", offset_of!(playerState_t, userFloat2), 1),
        nf("userFloat3", offset_of!(playerState_t, userFloat3), 1),
        nf(
            "userVec1[0]",
            offset_of!(playerState_t, userVec1) + 0 * 4,
            1,
        ),
        nf(
            "userVec1[1]",
            offset_of!(playerState_t, userVec1) + 1 * 4,
            1,
        ),
        nf(
            "userVec1[2]",
            offset_of!(playerState_t, userVec1) + 2 * 4,
            1,
        ),
        nf(
            "userVec2[0]",
            offset_of!(playerState_t, userVec2) + 0 * 4,
            1,
        ),
        nf(
            "userVec2[1]",
            offset_of!(playerState_t, userVec2) + 1 * 4,
            1,
        ),
        nf(
            "userVec2[2]",
            offset_of!(playerState_t, userVec2) + 2 * 4,
            1,
        ),
    ]
}

/// Raven `vehPlayerStateFields[]` — the vehicle playerstate delta table (live:
/// `_OPTIMIZED_VEHICLE_NETWORKING` is unconditionally defined,
/// `q_shared.h:2154`). Order is wire-critical; all 69 entries are coded.
///
/// RETAIL-WIRE DIVERGENCE: 69 rows, not the source drop's 80 — see
/// [`build_player_state_fields`]; verified against the same client binary.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:1736-1822` (minus never-shipped
/// vehicle rows)
fn build_veh_player_state_fields() -> Vec<netField_t> {
    vec![
        nf("commandTime", offset_of!(playerState_t, commandTime), 32),
        nf("origin[1]", offset_of!(playerState_t, origin) + 1 * 4, 0),
        nf("origin[0]", offset_of!(playerState_t, origin) + 0 * 4, 0),
        nf(
            "viewangles[1]",
            offset_of!(playerState_t, viewangles) + 1 * 4,
            0,
        ),
        nf(
            "viewangles[0]",
            offset_of!(playerState_t, viewangles) + 0 * 4,
            0,
        ),
        nf("origin[2]", offset_of!(playerState_t, origin) + 2 * 4, 0),
        nf(
            "velocity[0]",
            offset_of!(playerState_t, velocity) + 0 * 4,
            0,
        ),
        nf(
            "velocity[1]",
            offset_of!(playerState_t, velocity) + 1 * 4,
            0,
        ),
        nf(
            "velocity[2]",
            offset_of!(playerState_t, velocity) + 2 * 4,
            0,
        ),
        nf("weaponTime", offset_of!(playerState_t, weaponTime), -16),
        nf(
            "delta_angles[1]",
            offset_of!(playerState_t, delta_angles) + 1 * 4,
            16,
        ),
        nf("speed", offset_of!(playerState_t, speed), 0),
        nf("legsAnim", offset_of!(playerState_t, legsAnim), 16),
        nf(
            "delta_angles[0]",
            offset_of!(playerState_t, delta_angles) + 0 * 4,
            16,
        ),
        nf(
            "groundEntityNum",
            offset_of!(playerState_t, groundEntityNum),
            GENTITYNUM_BITS,
        ),
        nf("eFlags", offset_of!(playerState_t, eFlags), 32),
        nf(
            "eventSequence",
            offset_of!(playerState_t, eventSequence),
            16,
        ),
        nf("legsTimer", offset_of!(playerState_t, legsTimer), 16),
        nf(
            "rocketLockIndex",
            offset_of!(playerState_t, rocketLockIndex),
            GENTITYNUM_BITS,
        ),
        nf("events[0]", offset_of!(playerState_t, events) + 0 * 4, 10),
        nf("events[1]", offset_of!(playerState_t, events) + 1 * 4, 10),
        nf("weaponstate", offset_of!(playerState_t, weaponstate), 4),
        nf("pm_flags", offset_of!(playerState_t, pm_flags), 16),
        nf("pm_time", offset_of!(playerState_t, pm_time), -16),
        nf(
            "clientNum",
            offset_of!(playerState_t, clientNum),
            GENTITYNUM_BITS,
        ),
        nf("gravity", offset_of!(playerState_t, gravity), 16),
        nf("weapon", offset_of!(playerState_t, weapon), 8),
        nf(
            "delta_angles[2]",
            offset_of!(playerState_t, delta_angles) + 2 * 4,
            16,
        ),
        nf(
            "viewangles[2]",
            offset_of!(playerState_t, viewangles) + 2 * 4,
            0,
        ),
        nf(
            "externalEvent",
            offset_of!(playerState_t, externalEvent),
            10,
        ),
        nf(
            "eventParms[1]",
            offset_of!(playerState_t, eventParms) + 1 * 4,
            8,
        ),
        nf("pm_type", offset_of!(playerState_t, pm_type), 8),
        nf(
            "externalEventParm",
            offset_of!(playerState_t, externalEventParm),
            8,
        ),
        nf(
            "eventParms[0]",
            offset_of!(playerState_t, eventParms) + 0 * 4,
            -16,
        ),
        nf(
            "vehOrientation[0]",
            offset_of!(playerState_t, vehOrientation) + 0 * 4,
            0,
        ),
        nf(
            "vehOrientation[1]",
            offset_of!(playerState_t, vehOrientation) + 1 * 4,
            0,
        ),
        nf("moveDir[1]", offset_of!(playerState_t, moveDir) + 1 * 4, 0),
        nf("moveDir[0]", offset_of!(playerState_t, moveDir) + 0 * 4, 0),
        nf(
            "vehOrientation[2]",
            offset_of!(playerState_t, vehOrientation) + 2 * 4,
            0,
        ),
        nf("moveDir[2]", offset_of!(playerState_t, moveDir) + 2 * 4, 0),
        nf(
            "rocketTargetTime",
            offset_of!(playerState_t, rocketTargetTime),
            32,
        ),
        nf(
            "electrifyTime",
            offset_of!(playerState_t, electrifyTime),
            32,
        ),
        nf("loopSound", offset_of!(playerState_t, loopSound), 16),
        nf(
            "rocketLockTime",
            offset_of!(playerState_t, rocketLockTime),
            32,
        ),
        nf(
            "m_iVehicleNum",
            offset_of!(playerState_t, m_iVehicleNum),
            GENTITYNUM_BITS,
        ),
        nf(
            "vehTurnaroundTime",
            offset_of!(playerState_t, vehTurnaroundTime),
            32,
        ),
        nf("hackingTime", offset_of!(playerState_t, hackingTime), 32),
        nf("brokenLimbs", offset_of!(playerState_t, brokenLimbs), 8),
        nf(
            "vehWeaponsLinked",
            offset_of!(playerState_t, vehWeaponsLinked),
            1,
        ),
        nf(
            "hyperSpaceTime",
            offset_of!(playerState_t, hyperSpaceTime),
            32,
        ),
        nf("eFlags2", offset_of!(playerState_t, eFlags2), 10),
        nf(
            "hyperSpaceAngles[1]",
            offset_of!(playerState_t, hyperSpaceAngles) + 1 * 4,
            0,
        ),
        nf("vehBoarding", offset_of!(playerState_t, vehBoarding), 1),
        nf(
            "vehTurnaroundIndex",
            offset_of!(playerState_t, vehTurnaroundIndex),
            GENTITYNUM_BITS,
        ),
        nf("vehSurfaces", offset_of!(playerState_t, vehSurfaces), 16),
        nf(
            "hyperSpaceAngles[0]",
            offset_of!(playerState_t, hyperSpaceAngles) + 0 * 4,
            0,
        ),
        nf(
            "hyperSpaceAngles[2]",
            offset_of!(playerState_t, hyperSpaceAngles) + 2 * 4,
            0,
        ),
        nf("userInt1", offset_of!(playerState_t, userInt1), 1),
        nf("userInt2", offset_of!(playerState_t, userInt2), 1),
        nf("userInt3", offset_of!(playerState_t, userInt3), 1),
        nf("userFloat1", offset_of!(playerState_t, userFloat1), 1),
        nf("userFloat2", offset_of!(playerState_t, userFloat2), 1),
        nf("userFloat3", offset_of!(playerState_t, userFloat3), 1),
        nf(
            "userVec1[0]",
            offset_of!(playerState_t, userVec1) + 0 * 4,
            1,
        ),
        nf(
            "userVec1[1]",
            offset_of!(playerState_t, userVec1) + 1 * 4,
            1,
        ),
        nf(
            "userVec1[2]",
            offset_of!(playerState_t, userVec1) + 2 * 4,
            1,
        ),
        nf(
            "userVec2[0]",
            offset_of!(playerState_t, userVec2) + 0 * 4,
            1,
        ),
        nf(
            "userVec2[1]",
            offset_of!(playerState_t, userVec2) + 1 * 4,
            1,
        ),
        nf(
            "userVec2[2]",
            offset_of!(playerState_t, userVec2) + 2 * 4,
            1,
        ),
    ]
}

/// Raven's `netField_t *PSFields` cursor — which playerstate delta table
/// `MSG_{Write,Read}DeltaPlayerstate` selected under the always-on
/// `_OPTIMIZED_VEHICLE_NETWORKING` build (`q_shared.h:2154`).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:2225,2241-2261,2492-2511`
#[derive(Clone, Copy)]
enum PsfTable {
    Normal,
    Pilot,
    Veh,
}

/// The selected table's fields (Raven's `PSFields[i]` reads).
fn psf_fields(common: &Common, table: PsfTable) -> &[netField_t] {
    match table {
        PsfTable::Normal => &common.player_state_fields,
        PsfTable::Pilot => &common.pilot_player_state_fields,
        PsfTable::Veh => &common.veh_player_state_fields,
    }
}

/// Mutable view for the write path's `field->mCount++` profiling bump.
fn psf_fields_mut(common: &mut Common, table: PsfTable) -> &mut [netField_t] {
    match table {
        PsfTable::Normal => &mut common.player_state_fields,
        PsfTable::Pilot => &mut common.pilot_player_state_fields,
        PsfTable::Veh => &mut common.veh_player_state_fields,
    }
}

/// The delta-coder field tables are file-scope statics in Raven; here they live
/// on `Common` and are populated lazily on first `MSG_Init`/`MSG_InitOOB` so the
/// override check sees them filled.
fn ensure_field_tables(common: &mut Common) {
    if common.entity_state_fields.is_empty() {
        common.entity_state_fields = build_entity_state_fields();
    }
    if common.player_state_fields.is_empty() {
        common.player_state_fields = build_player_state_fields();
    }
    if common.pilot_player_state_fields.is_empty() {
        common.pilot_player_state_fields = build_pilot_player_state_fields();
    }
    if common.veh_player_state_fields.is_empty() {
        common.veh_player_state_fields = build_veh_player_state_fields();
    }
}

/// Raven `MSG_Init` — one-time netf/psf override check, lazy Huffman init, then
/// zero the `msg_t` and wire its buffer. (`_XBOX` is not defined, so the
/// override-check block is live.)
///
/// Source: `oracle/codemp/qcommon/msg.cpp:46-68`
pub fn MSG_Init(view: &mut EngineHostView, buf: *mut msg_t, data: *mut byte, length: c_int) {
    ensure_field_tables(view.common);

    if !view.common.g_nOverrideChecked {
        // Check for netf overrides, then for psf overrides.
        MSG_CheckNETFPSFOverrides(view, qfalse);
        MSG_CheckNETFPSFOverrides(view, qtrue);

        view.common.g_nOverrideChecked = true;
    }

    if view.common.msg_init == qfalse {
        MSG_initHuffman(view.common);
    }

    unsafe {
        crate::common_fns::Com_Memset(buf as *mut (), 0, core::mem::size_of::<msg_t>());
        (*buf).data = data;
        (*buf).maxsize = length;
    }
}

/// Raven `MSG_InitOOB` — like [`MSG_Init`] but flags the buffer out-of-band
/// (`oob = qtrue`) for raw byte framing (netchan headers, connectionless
/// datagrams). Runs the same one-time override check and lazy Huffman init.
/// (`_XBOX` is not defined, so the override-check block is live.)
///
/// Source: `oracle/codemp/qcommon/msg.cpp:70-92`
pub fn MSG_InitOOB(view: &mut EngineHostView, buf: *mut msg_t, data: *mut byte, length: c_int) {
    ensure_field_tables(view.common);

    if !view.common.g_nOverrideChecked {
        // Check for netf overrides, then for psf overrides.
        MSG_CheckNETFPSFOverrides(view, qfalse);
        MSG_CheckNETFPSFOverrides(view, qtrue);

        view.common.g_nOverrideChecked = true;
    }

    if view.common.msg_init == qfalse {
        MSG_initHuffman(view.common);
    }

    unsafe {
        crate::common_fns::Com_Memset(buf as *mut (), 0, core::mem::size_of::<msg_t>());
        (*buf).data = data;
        (*buf).maxsize = length;
        (*buf).oob = qtrue;
    }
}

/// `strcmp(cbuf, s) == 0` for a NUL-terminated `c_char` buffer against a Rust
/// `&str` (no embedded NUL): equal bytes and the C string ends exactly where
/// `s` does.
fn msg_override_cstr_eq(cbuf: &[c_char], s: &str) -> bool {
    let sb = s.as_bytes();
    if sb.len() >= cbuf.len() {
        return false;
    }
    for (k, &b) in sb.iter().enumerate() {
        if cbuf[k] as u8 != b {
            return false;
        }
    }
    cbuf[sb.len()] as u8 == 0
}

/// Render a NUL-terminated `c_char` buffer as a `String` for a `%s` warning.
fn msg_override_cstr_str(cbuf: &[c_char]) -> String {
    let nul = cbuf.iter().position(|&c| c == 0).unwrap_or(cbuf.len());
    let bytes = unsafe { core::slice::from_raw_parts(cbuf.as_ptr() as *const u8, nul) };
    String::from_utf8_lossy(bytes).into_owned()
}

/// Raven `MSG_CheckNETFPSFOverrides` — rww's mod hook: reload
/// `ext_data/MP/{netf,psf}_overrides.txt` and stomp the delta-coder field
/// `bits`. On the first call it stashes each field's default `bits` into a
/// `bitStorage_t` list; subsequent calls restore those defaults before
/// re-applying the file's `name, bits` lines. Live only under `!_XBOX`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:2005-2197`
pub fn MSG_CheckNETFPSFOverrides(view: &mut EngineHostView, psfOverrides: qboolean) {
    let mut overrideFile: [c_char; 4096] = [0; 4096];
    let mut entryName: [c_char; 4096] = [0; 4096];
    let mut bits: [c_char; 4096] = [0; 4096];
    let fileName: &str;
    let mut i: c_int = 0;
    let mut j: c_int;
    let len: c_int;
    let numFields: c_int;
    let mut f: fileHandle_t = 0;
    // Raven's `bitStorage_t **bitStorage` walking cursor: it starts at the
    // address of the head field on `Common` and advances through `->next`.
    let mut bitStorage: *mut *mut bitStorage_t;

    if psfOverrides != qfalse {
        //do PSF overrides instead of NETF
        fileName = "psf_overrides.txt";
        bitStorage = &mut view.common.g_psfBitStorage;
        numFields = view.common.player_state_fields.len() as c_int;
    } else {
        fileName = "netf_overrides.txt";
        bitStorage = &mut view.common.g_netfBitStorage;
        numFields = view.common.entity_state_fields.len() as c_int;
    }

    if !unsafe { *bitStorage }.is_null() {
        //if we have saved off the defaults before we want to stuff them all back in now
        let mut restore: *mut bitStorage_t = unsafe { *bitStorage };

        while i < numFields {
            // Raven's `assert(restore)` is debug-only (NDEBUG here) and omitted;
            // a defaults list shorter than `numFields` would null-deref, matching C.
            unsafe {
                if psfOverrides != qfalse {
                    view.common.player_state_fields[i as usize].bits = (*restore).bits;
                } else {
                    view.common.entity_state_fields[i as usize].bits = (*restore).bits;
                }
                i += 1;
                restore = (*restore).next;
            }
        }
    }

    let path = format!("ext_data/MP/{fileName}");
    len = FS_FOpenFileRead(view, &path, &mut f, false);

    if f == 0 {
        //silently exit since this file is not needed to proceed.
        return;
    }

    if len >= 4096 {
        com_printf(
            view.common,
            &format!(
                "WARNING: {} is >= 4096 bytes and is being ignored\n",
                fileName
            ),
        );
        FS_FCloseFile(view.common, f);
        return;
    }

    //Get contents of the file
    FS_Read(view.common, overrideFile.as_mut_ptr() as *mut (), len, f);
    FS_FCloseFile(view.common, f);

    //because FS_Read does not do this for us.
    overrideFile[len as usize] = 0;

    //If we haven't saved off the initial stuff yet then stuff it all into
    //a list.
    if unsafe { *bitStorage }.is_null() {
        i = 0;

        while i < numFields {
            //Alloc memory for this new ptr
            let node = Z_Malloc(
                view,
                core::mem::size_of::<bitStorage_t>() as c_int,
                memtag_t::TAG_GENERAL,
                qtrue,
                4,
            ) as *mut bitStorage_t;

            unsafe {
                *bitStorage = node;

                if psfOverrides != qfalse {
                    (*node).bits = view.common.player_state_fields[i as usize].bits;
                } else {
                    (*node).bits = view.common.entity_state_fields[i as usize].bits;
                }

                //Point to the ->next of the existing current ptr
                bitStorage = &mut (*node).next;
            }
            i += 1;
        }
    }

    i = 0;
    //Now parse through. Lines beginning with ; are disabled.
    // Faithful to C's unchecked buffer walk: malformed input (any final line —
    // comment or value — lacking its trailing newline) runs past `len` and
    // panics on the Rust bounds check where C would read adjacent stack (UB).
    while overrideFile[i as usize] != 0 {
        if overrideFile[i as usize] == b';' as c_char {
            //parse to end of the line
            while overrideFile[i as usize] != b'\n' as c_char {
                i += 1;
            }
        }

        if overrideFile[i as usize] != b';' as c_char
            && overrideFile[i as usize] != b'\n' as c_char
            && overrideFile[i as usize] != b'\r' as c_char
        {
            //on a valid char I guess, parse it
            j = 0;

            while overrideFile[i as usize] != 0 && overrideFile[i as usize] != b',' as c_char {
                entryName[j as usize] = overrideFile[i as usize];
                j += 1;
                i += 1;
            }
            entryName[j as usize] = 0;

            if overrideFile[i as usize] == 0 {
                //just give up, this shouldn't happen
                com_printf(
                    view.common,
                    &format!("WARNING: Parsing error for {}\n", fileName),
                );
                return;
            }

            while overrideFile[i as usize] == b',' as c_char
                || overrideFile[i as usize] == b' ' as c_char
            {
                //parse to the start of the value
                i += 1;
            }

            j = 0;
            while overrideFile[i as usize] != b'\n' as c_char
                && overrideFile[i as usize] != b'\r' as c_char
            {
                //now read the value in
                bits[j as usize] = overrideFile[i as usize];
                j += 1;
                i += 1;
            }
            bits[j as usize] = 0;

            if bits[0] != 0 {
                let ibits: c_int;
                if msg_override_cstr_eq(&bits, "GENTITYNUM_BITS") {
                    //special case
                    ibits = GENTITYNUM_BITS;
                } else {
                    ibits = unsafe { libc::atoi(bits.as_ptr()) };
                }

                j = 0;

                //Now go through all the fields and see if we can find a match
                while j < numFields {
                    if psfOverrides != qfalse {
                        //check psf fields
                        if msg_override_cstr_eq(
                            &entryName,
                            view.common.player_state_fields[j as usize].name,
                        ) {
                            //found it, set the bits
                            view.common.player_state_fields[j as usize].bits = ibits;
                            break;
                        }
                    } else {
                        //otherwise check netf fields
                        if msg_override_cstr_eq(
                            &entryName,
                            view.common.entity_state_fields[j as usize].name,
                        ) {
                            //found it, set the bits
                            view.common.entity_state_fields[j as usize].bits = ibits;
                            break;
                        }
                    }
                    j += 1;
                }

                if j == numFields {
                    //failed to find the value
                    com_printf(
                        view.common,
                        &format!(
                            "WARNING: Value '{}' from {} is not valid\n",
                            msg_override_cstr_str(&entryName),
                            fileName
                        ),
                    );
                }
            } else {
                //also should not happen
                com_printf(
                    view.common,
                    &format!("WARNING: Parsing error for {}\n", fileName),
                );
                return;
            }
        }

        i += 1;
    }
}

/// Raven `MSG_initHuffman`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:3219-3234`
pub fn MSG_initHuffman(common: &mut Common) {
    // Raven's `_NEWHUFFTABLE_` debug-log fopen is compiled out (undefined).
    common.msg_init = qtrue;
    unsafe {
        crate::qcommon::huff::Huff_Init(&mut common.msg_huff);
        for i in 0..256usize {
            for _j in 0..MSG_H_DATA[i] {
                crate::qcommon::huff::Huff_addRef(&mut common.msg_huff.compressor, i as u8); // Do update
                crate::qcommon::huff::Huff_addRef(&mut common.msg_huff.decompressor, i as u8);
                // Do update
            }
        }
    }
}

/// Raven `MSG_WriteBits`. The module-static write-only `overflows` diagnostic
/// counter (its `Com_Printf` is compiled out and the counter is never read
/// anywhere in the tree) is dropped as dead surface.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:129-207`
pub fn MSG_WriteBits(common: &mut Common, msg: *mut msg_t, mut value: c_int, mut bits: c_int) {
    common.oldsize += bits;
    unsafe {
        // this isn't an exact overflow check, but close enough
        if (*msg).maxsize - (*msg).cursize < 4 {
            (*msg).overflowed = qtrue;
            return;
        }
        if bits == 0 || bits < -31 || bits > 32 {
            com_error(
                errorParm_t::ERR_DROP,
                format!("MSG_WriteBits: bad bits {bits}"),
            );
        }
        // check for overflows: the oracle only bumps the dead `overflows` counter.
        if bits < 0 {
            bits = -bits;
        }
        if (*msg).oob != qfalse {
            if bits == 8 {
                *(*msg).data.add((*msg).cursize as usize) = value as u8;
                (*msg).cursize += 1;
                (*msg).bit += 8;
            } else if bits == 16 {
                let sp = (*msg).data.add((*msg).cursize as usize) as *mut u16;
                *sp = (value as u16).to_le();
                (*msg).cursize += 2;
                (*msg).bit += 16;
            } else if bits == 32 {
                let ip = (*msg).data.add((*msg).cursize as usize) as *mut u32;
                *ip = (value as u32).to_le();
                (*msg).cursize += 4;
                (*msg).bit += 8;
            } else {
                com_error(errorParm_t::ERR_DROP, format!("can't read {bits} bits\n"));
            }
        } else {
            value &= (0xffffffffu32 >> (32 - bits)) as c_int;
            if bits & 7 != 0 {
                let nbits = bits & 7;
                for _ in 0..nbits {
                    crate::qcommon::huff::Huff_putBit(value & 1, (*msg).data, &mut (*msg).bit);
                    value >>= 1;
                }
                bits -= nbits;
            }
            if bits != 0 {
                let mut i = 0;
                while i < bits {
                    crate::qcommon::huff::Huff_offsetTransmit(
                        &mut common.msg_huff.compressor,
                        value & 0xff,
                        (*msg).data,
                        &mut (*msg).bit,
                    );
                    value = ((value as u32) >> 8) as c_int;
                    i += 8;
                }
            }
            (*msg).cursize = ((*msg).bit >> 3) + 1;
        }
    }
}

/// Raven `MSG_ReadBits`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:211-283`
pub fn MSG_ReadBits(common: &mut Common, msg: *mut msg_t, mut bits: c_int) -> c_int {
    let mut value: c_int = 0;
    let sgn;
    unsafe {
        if bits < 0 {
            bits = -bits;
            sgn = true;
        } else {
            sgn = false;
        }

        if (*msg).oob != qfalse {
            if bits == 8 {
                value = *(*msg).data.add((*msg).readcount as usize) as c_int;
                (*msg).readcount += 1;
                (*msg).bit += 8;
            } else if bits == 16 {
                let sp = (*msg).data.add((*msg).readcount as usize) as *const u16;
                value = u16::from_le(*sp) as c_int;
                (*msg).readcount += 2;
                (*msg).bit += 16;
            } else if bits == 32 {
                let ip = (*msg).data.add((*msg).readcount as usize) as *const u32;
                value = u32::from_le(*ip) as c_int;
                (*msg).readcount += 4;
                (*msg).bit += 32;
            } else {
                com_error(errorParm_t::ERR_DROP, format!("can't read {bits} bits\n"));
            }
        } else {
            let mut nbits = 0;
            if bits & 7 != 0 {
                nbits = bits & 7;
                for i in 0..nbits {
                    value |= crate::qcommon::huff::Huff_getBit((*msg).data, &mut (*msg).bit) << i;
                }
                bits -= nbits;
            }
            if bits != 0 {
                let mut get: c_int = 0;
                let mut i = 0;
                while i < bits {
                    crate::qcommon::huff::Huff_offsetReceive(
                        common.msg_huff.decompressor.tree,
                        &mut get,
                        (*msg).data,
                        &mut (*msg).bit,
                    );
                    value |= get << (i + nbits);
                    i += 8;
                }
            }
            (*msg).readcount = ((*msg).bit >> 3) + 1;
        }
        if sgn && value & (1 << (bits - 1)) != 0 {
            value |= -1 ^ ((1 << bits) - 1);
        }
    }
    value
}

/// Raven `MSG_WriteByte`. `PARANOID` range-check guard compiles out.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:289-296`
pub fn MSG_WriteByte(common: &mut Common, sb: *mut msg_t, c: c_int) {
    MSG_WriteBits(common, sb, c, 8);
}

/// Raven `MSG_WriteData`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:298-303`
pub fn MSG_WriteData(common: &mut Common, buf: *mut msg_t, data: *const (), length: c_int) {
    unsafe {
        for i in 0..length {
            MSG_WriteByte(common, buf, *(data as *const u8).add(i as usize) as c_int);
        }
    }
}

/// Raven `MSG_WriteShort`. `PARANOID` range-check guard compiles out.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:305-312`
pub fn MSG_WriteShort(common: &mut Common, sb: *mut msg_t, c: c_int) {
    MSG_WriteBits(common, sb, c, 16);
}

/// Raven `MSG_WriteLong`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:314-316`
pub fn MSG_WriteLong(common: &mut Common, sb: *mut msg_t, c: c_int) {
    MSG_WriteBits(common, sb, c, 32);
}

/// Raven `MSG_WriteString`. The eurofix 0xff-strip loop is left commented in
/// the oracle and not ported. Raven's `!s` NULL arm — write a bare NUL — is the
/// empty `&str` here: `""` emits zero body bytes plus the trailing NUL, the same
/// single byte Raven's `MSG_WriteData(sb, "", 1)` writes, so callers that
/// previously passed a possibly-null pointer map their null case to `""`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:328-354`
pub fn MSG_WriteString(common: &mut Common, sb: *mut msg_t, s: &str) {
    // Wire bytes are the bijective Latin-1 encoding of `s` (one byte per char),
    // matching retail's byte-transparent chat/userinfo strings; `l` is the WIRE
    // byte count (one per char), not `s.len()`'s UTF-8 width which would over-count
    // non-ASCII. Pure-ASCII content is byte-identical to the prior `s.as_ptr()` path.
    let bytes = string_to_latin1(s);
    let l = bytes.len();
    if l >= MAX_STRING_CHARS {
        com_printf(common, "MSG_WriteString: MAX_STRING_CHARS");
        MSG_WriteData(common, sb, b"\0".as_ptr() as *const (), 1);
        return;
    }
    // Raven copies into a scratch buffer then `MSG_WriteData(string, l+1)` — the
    // `l` body bytes followed by the terminating NUL. `MSG_WriteData` is a plain
    // per-byte `MSG_WriteByte` loop, so writing the body then one NUL byte emits
    // the identical wire sequence with no scratch copy.
    MSG_WriteData(common, sb, bytes.as_ptr() as *const (), l as c_int);
    MSG_WriteByte(common, sb, 0);
}

/// Raven `MSG_ReadLong`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:433-441`
pub fn MSG_ReadLong(common: &mut Common, msg: *mut msg_t) -> c_int {
    let mut c = MSG_ReadBits(common, msg, 32);
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            c = -1;
        }
    }
    c
}

/// Raven `MSG_WriteChar`. `PARANOID` range-check guard compiles out (no
/// `PARANOID` define in this build).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:280-287`
pub fn MSG_WriteChar(common: &mut Common, sb: *mut msg_t, c: c_int) {
    let _ = errorParm_t::ERR_FATAL; // PARANOID range-check compiles out; cited for parity reference
    MSG_WriteBits(common, sb, c, 8);
}

/// Raven `MSG_WriteFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:318-326`
pub fn MSG_WriteFloat(common: &mut Common, sb: *mut msg_t, f: f32) {
    let l: i32 = f.to_bits() as i32;
    MSG_WriteBits(common, sb, l, 32);
}

/// Raven `MSG_WriteBigString`. Same NUL-arm and body+NUL wire shape as
/// `MSG_WriteString`; the empty `&str` is Raven's `!s` case. The
/// `Com_Printf` guard keeps Raven's literal `"MSG_WriteString:"` prefix.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:356-383`
pub fn MSG_WriteBigString(common: &mut Common, sb: *mut msg_t, s: &str) {
    // Latin-1 wire bytes (one per char); `l` is the WIRE byte count, not the
    // UTF-8 width. See `MSG_WriteString`.
    let bytes = string_to_latin1(s);
    let l = bytes.len();
    if l >= BIG_INFO_STRING {
        com_printf(common, "MSG_WriteString: BIG_INFO_STRING");
        MSG_WriteData(common, sb, b"\0".as_ptr() as *const (), 1);
        return;
    }

    // eurofix: remove this so we can chat in european languages...	-ste
    // (0xff-strip loop left commented in the oracle; not ported)

    MSG_WriteData(common, sb, bytes.as_ptr() as *const (), l as c_int);
    MSG_WriteByte(common, sb, 0);
}

/// Raven `MSG_WriteAngle`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:385-387`
pub fn MSG_WriteAngle(common: &mut Common, sb: *mut msg_t, f: f32) {
    MSG_WriteByte(common, sb, (f * 256.0 / 360.0) as c_int & 255);
}

/// Raven `MSG_WriteAngle16`. `ANGLE2SHORT(x)` = `(int)((x)*65536/360) & 65535`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:389-391`
pub fn MSG_WriteAngle16(common: &mut Common, sb: *mut msg_t, f: f32) {
    MSG_WriteShort(common, sb, ((f * 65536.0 / 360.0) as c_int) & 65535);
}

/// Raven `MSG_ReadChar`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:401-410`
pub fn MSG_ReadChar(common: &mut Common, msg: *mut msg_t) -> c_int {
    let mut c = MSG_ReadBits(common, msg, 8) as i8 as c_int;
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            c = -1;
        }
    }
    c
}

/// Raven `MSG_ReadByte`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:412-420`
pub fn MSG_ReadByte(common: &mut Common, msg: *mut msg_t) -> c_int {
    let mut c = MSG_ReadBits(common, msg, 8) as u8 as c_int;
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            c = -1;
        }
    }
    c
}

/// Raven `MSG_ReadShort`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:422-431`
pub fn MSG_ReadShort(common: &mut Common, msg: *mut msg_t) -> c_int {
    let mut c = MSG_ReadBits(common, msg, 16) as i16 as c_int;
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            c = -1;
        }
    }
    c
}

/// Raven `MSG_ReadFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:444-457`
pub fn MSG_ReadFloat(common: &mut Common, msg: *mut msg_t) -> f32 {
    let l = MSG_ReadBits(common, msg, 32);
    let mut f = f32::from_bits(l as u32);
    unsafe {
        if (*msg).readcount > (*msg).cursize {
            f = -1.0;
        }
    }
    f
}

/// Raven `MSG_WriteDelta`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:567-574`
pub fn MSG_WriteDelta(common: &mut Common, msg: *mut msg_t, oldV: c_int, newV: c_int, bits: c_int) {
    if oldV == newV {
        MSG_WriteBits(common, msg, 0, 1);
        return;
    }
    MSG_WriteBits(common, msg, 1, 1);
    MSG_WriteBits(common, msg, newV, bits);
}

/// Raven `MSG_ReadDelta`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:576-581`
pub fn MSG_ReadDelta(common: &mut Common, msg: *mut msg_t, oldV: c_int, bits: c_int) -> c_int {
    if MSG_ReadBits(common, msg, 1) != 0 {
        return MSG_ReadBits(common, msg, bits);
    }
    oldV
}

/// Raven `MSG_WriteDeltaFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:583-590`
pub fn MSG_WriteDeltaFloat(common: &mut Common, msg: *mut msg_t, oldV: f32, newV: f32) {
    if oldV == newV {
        MSG_WriteBits(common, msg, 0, 1);
        return;
    }
    MSG_WriteBits(common, msg, 1, 1);
    MSG_WriteBits(common, msg, newV.to_bits() as i32, 32);
}

/// Raven `MSG_ReadDeltaFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:592-600`
pub fn MSG_ReadDeltaFloat(common: &mut Common, msg: *mut msg_t, oldV: f32) -> f32 {
    if MSG_ReadBits(common, msg, 1) != 0 {
        let newV = f32::from_bits(MSG_ReadBits(common, msg, 32) as u32);
        return newV;
    }
    oldV
}

/// Raven `kbitmask` — precomputed `(1 << bits) - 1` masks for
/// `MSG_ReadDeltaKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:610-619`
pub const KBITMASK: [i32; 32] = [
    0x00000001, 0x00000003, 0x00000007, 0x0000000F, 0x0000001F, 0x0000003F, 0x0000007F, 0x000000FF,
    0x000001FF, 0x000003FF, 0x000007FF, 0x00000FFF, 0x00001FFF, 0x00003FFF, 0x00007FFF, 0x0000FFFF,
    0x0001FFFF, 0x0003FFFF, 0x0007FFFF, 0x000FFFFF, 0x001FFFFF, 0x003FFFFF, 0x007FFFFF, 0x00FFFFFF,
    0x01FFFFFF, 0x03FFFFFF, 0x07FFFFFF, 0x0FFFFFFF, 0x1FFFFFFF, 0x3FFFFFFF, 0x7FFFFFFF, -1i32,
];

/// Raven `MSG_WriteDeltaKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:621-630`
pub fn MSG_WriteDeltaKey(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    oldV: c_int,
    newV: c_int,
    bits: c_int,
) {
    if oldV == newV {
        MSG_WriteBits(common, msg, 0, 1);
        return;
    }
    MSG_WriteBits(common, msg, 1, 1);
    MSG_WriteBits(common, msg, (newV ^ key) & ((1 << bits) - 1), bits);
}

/// Raven `MSG_ReadDeltaKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:632-637`
pub fn MSG_ReadDeltaKey(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    oldV: c_int,
    bits: c_int,
) -> c_int {
    if MSG_ReadBits(common, msg, 1) != 0 {
        return MSG_ReadBits(common, msg, bits) ^ (key & KBITMASK[bits as usize]);
    }
    oldV
}

/// Raven `MSG_WriteDeltaKeyFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:639-646`
pub fn MSG_WriteDeltaKeyFloat(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    oldV: f32,
    newV: f32,
) {
    if oldV == newV {
        MSG_WriteBits(common, msg, 0, 1);
        return;
    }
    MSG_WriteBits(common, msg, 1, 1);
    MSG_WriteBits(common, msg, (newV.to_bits() as i32) ^ key, 32);
}

/// Raven `MSG_ReadDeltaKeyFloat`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:648-656`
pub fn MSG_ReadDeltaKeyFloat(common: &mut Common, msg: *mut msg_t, key: c_int, oldV: f32) -> f32 {
    if MSG_ReadBits(common, msg, 1) != 0 {
        let newV = f32::from_bits((MSG_ReadBits(common, msg, 32) ^ key) as u32);
        return newV;
    }
    oldV
}

/// Raven `MSG_ReportChangeVectors_f`. Debug tooling gated by
/// `!_XBOX && !FINAL_BUILD`; `FINAL_BUILD` is undefined for this build
/// (porting-rules ledger), so the body is live.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:3291-3316`
pub fn MSG_ReportChangeVectors_f(common: &mut Common) {
    let num_entity_fields = common.entity_state_fields.len();
    crate::common::com_printf(common, "Entity State Fields:\n");
    for i in 0..common.entity_state_fields.len() {
        let name = common.entity_state_fields[i].name;
        let mCount = common.entity_state_fields[i].mCount;
        crate::common::com_printf(common, &format!("{}\t\t{}\n", name, mCount));
        common.entity_state_fields[i].mCount = 0;
    }
    let _ = num_entity_fields;

    crate::common::com_printf(common, "\nPlayer State Fields:\n");
    for i in 0..common.player_state_fields.len() {
        let name = common.player_state_fields[i].name;
        let mCount = common.player_state_fields[i].mCount;
        crate::common::com_printf(common, &format!("{}\t\t{}\n", name, mCount));
        common.player_state_fields[i].mCount = 0;
    }
}

/// Raven `MSG_ReadString`. Returns the decoded bytes as an owned `String`; the
/// static/`Common`-field scratch buffer is gone. '%' is translated to '.' (defang
/// format specifiers) and the loop stops on NUL or an out-of-bounds read (-1).
///
/// §19: Raven's `while (l <= sizeof-1)` lets `l` reach `MAX_STRING_CHARS` (one
/// past the last stored index), then the "bonus protection" clamp NUL-caps at
/// `sizeof-1`, so the last over-limit byte is dropped from the returned C string.
/// `truncate(cap-1)` reproduces that: it is a no-op on the common (NUL/-1) exit
/// and drops the one extra byte on the overflow exit. High bytes surviving into
/// a non-UTF-8 result are lossily replaced, matching the prior consumer-side
/// `to_string_lossy`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:459-495`
pub fn MSG_ReadString(common: &mut Common, msg: *mut msg_t) -> String {
    let cap = mp_qshared::shared::limits::MAX_STRING_CHARS;
    let mut string: Vec<u8> = Vec::new();
    loop {
        let c = MSG_ReadByte(common, msg);
        if c == -1 || c == 0 {
            break;
        }
        let mut c = c;
        // translate all fmt spec to avoid crash bugs
        if c == '%' as c_int {
            c = '.' as c_int;
        }
        string.push(c as u8);
        if string.len() > cap - 1 {
            break;
        }
    }
    // some bonus protection, shouldn't occur cause server doesn't write such things
    string.truncate(cap - 1);
    latin1_to_string(&string)
}

/// Raven `MSG_ReadBigString`. As `MSG_ReadString` but bounded by
/// `BIG_INFO_STRING`; Raven's `while (l < sizeof-1)` never over-runs, so no
/// bonus-protection truncate is needed.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:497-519`
pub fn MSG_ReadBigString(common: &mut Common, msg: *mut msg_t) -> String {
    let cap = mp_qshared::shared::limits::BIG_INFO_STRING;
    let mut string: Vec<u8> = Vec::new();
    loop {
        let c = MSG_ReadByte(common, msg);
        if c == -1 || c == 0 {
            break;
        }
        let mut c = c;
        // translate all fmt spec to avoid crash bugs
        if c == '%' as c_int {
            c = '.' as c_int;
        }
        string.push(c as u8);
        if string.len() >= cap - 1 {
            break;
        }
    }
    latin1_to_string(&string)
}

/// Raven `MSG_ReadStringLine`. As `MSG_ReadBigString` (bounded by
/// `MAX_STRING_CHARS`) but the loop also stops on a newline.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:521-542`
pub fn MSG_ReadStringLine(common: &mut Common, msg: *mut msg_t) -> String {
    let cap = mp_qshared::shared::limits::MAX_STRING_CHARS;
    let mut string: Vec<u8> = Vec::new();
    loop {
        let c = MSG_ReadByte(common, msg);
        if c == -1 || c == 0 || c == '\n' as c_int {
            break;
        }
        let mut c = c;
        // translate all fmt spec to avoid crash bugs
        if c == '%' as c_int {
            c = '.' as c_int;
        }
        string.push(c as u8);
        if string.len() >= cap - 1 {
            break;
        }
    }
    latin1_to_string(&string)
}

/// Raven `MSG_ReadAngle16`. `SHORT2ANGLE(x)` = `(x)*(360.0/65536)`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:544-546`
pub fn MSG_ReadAngle16(common: &mut Common, msg: *mut msg_t) -> f32 {
    (MSG_ReadShort(common, msg) as f32) * (360.0 / 65536.0)
}

/// Raven `MSG_ReadData`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:548-554`
pub fn MSG_ReadData(common: &mut Common, msg: *mut msg_t, data: *mut (), len: c_int) {
    unsafe {
        let out = data as *mut u8;
        for i in 0..len {
            *out.offset(i as isize) = MSG_ReadByte(common, msg) as u8;
        }
    }
}

/// Raven `MSG_WriteDeltaUsercmd`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:685-706`
pub fn MSG_WriteDeltaUsercmd(
    common: &mut Common,
    msg: *mut msg_t,
    from: *mut usercmd_t,
    to: *mut usercmd_t,
) {
    unsafe {
        if (*to).serverTime - (*from).serverTime < 256 {
            MSG_WriteBits(common, msg, 1, 1);
            MSG_WriteBits(common, msg, (*to).serverTime - (*from).serverTime, 8);
        } else {
            MSG_WriteBits(common, msg, 0, 1);
            MSG_WriteBits(common, msg, (*to).serverTime, 32);
        }
        MSG_WriteDelta(common, msg, (*from).angles[0], (*to).angles[0], 16);
        MSG_WriteDelta(common, msg, (*from).angles[1], (*to).angles[1], 16);
        MSG_WriteDelta(common, msg, (*from).angles[2], (*to).angles[2], 16);
        MSG_WriteDelta(
            common,
            msg,
            (*from).forwardmove as c_int,
            (*to).forwardmove as c_int,
            8,
        );
        MSG_WriteDelta(
            common,
            msg,
            (*from).rightmove as c_int,
            (*to).rightmove as c_int,
            8,
        );
        MSG_WriteDelta(
            common,
            msg,
            (*from).upmove as c_int,
            (*to).upmove as c_int,
            8,
        );
        MSG_WriteDelta(common, msg, (*from).buttons, (*to).buttons, 16);
        MSG_WriteDelta(
            common,
            msg,
            (*from).weapon as c_int,
            (*to).weapon as c_int,
            8,
        );

        MSG_WriteDelta(
            common,
            msg,
            (*from).forcesel as c_int,
            (*to).forcesel as c_int,
            8,
        );
        MSG_WriteDelta(
            common,
            msg,
            (*from).invensel as c_int,
            (*to).invensel as c_int,
            8,
        );

        MSG_WriteDelta(
            common,
            msg,
            (*from).generic_cmd as c_int,
            (*to).generic_cmd as c_int,
            8,
        );
    }
}

/// Raven `MSG_ReadDeltaUsercmd`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:714-733`
pub fn MSG_ReadDeltaUsercmd(
    common: &mut Common,
    msg: *mut msg_t,
    from: *mut usercmd_t,
    to: *mut usercmd_t,
) {
    unsafe {
        if MSG_ReadBits(common, msg, 1) != 0 {
            (*to).serverTime = (*from).serverTime + MSG_ReadBits(common, msg, 8);
        } else {
            (*to).serverTime = MSG_ReadBits(common, msg, 32);
        }
        (*to).angles[0] = MSG_ReadDelta(common, msg, (*from).angles[0], 16);
        (*to).angles[1] = MSG_ReadDelta(common, msg, (*from).angles[1], 16);
        (*to).angles[2] = MSG_ReadDelta(common, msg, (*from).angles[2], 16);
        (*to).forwardmove = MSG_ReadDelta(common, msg, (*from).forwardmove as c_int, 8) as _;
        (*to).rightmove = MSG_ReadDelta(common, msg, (*from).rightmove as c_int, 8) as _;
        (*to).upmove = MSG_ReadDelta(common, msg, (*from).upmove as c_int, 8) as _;
        (*to).buttons = MSG_ReadDelta(common, msg, (*from).buttons, 16);
        (*to).weapon = MSG_ReadDelta(common, msg, (*from).weapon as c_int, 8) as _;

        (*to).forcesel = MSG_ReadDelta(common, msg, (*from).forcesel as c_int, 8) as _;
        (*to).invensel = MSG_ReadDelta(common, msg, (*from).invensel as c_int, 8) as _;

        (*to).generic_cmd = MSG_ReadDelta(common, msg, (*from).generic_cmd as c_int, 8) as _;
    }
}

/// Raven `MSG_WriteDeltaUsercmdKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:740-778`
pub fn MSG_WriteDeltaUsercmdKey(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    from: *mut usercmd_t,
    to: *mut usercmd_t,
) {
    unsafe {
        if (*to).serverTime - (*from).serverTime < 256 {
            MSG_WriteBits(common, msg, 1, 1);
            MSG_WriteBits(common, msg, (*to).serverTime - (*from).serverTime, 8);
        } else {
            MSG_WriteBits(common, msg, 0, 1);
            MSG_WriteBits(common, msg, (*to).serverTime, 32);
        }
        if (*from).angles[0] == (*to).angles[0]
            && (*from).angles[1] == (*to).angles[1]
            && (*from).angles[2] == (*to).angles[2]
            && (*from).forwardmove == (*to).forwardmove
            && (*from).rightmove == (*to).rightmove
            && (*from).upmove == (*to).upmove
            && (*from).buttons == (*to).buttons
            && (*from).weapon == (*to).weapon
            && (*from).forcesel == (*to).forcesel
            && (*from).invensel == (*to).invensel
            && (*from).generic_cmd == (*to).generic_cmd
        {
            MSG_WriteBits(common, msg, 0, 1); // no change
            common.oldsize += 7;
            return;
        }
        let key = key ^ (*to).serverTime;
        MSG_WriteBits(common, msg, 1, 1);
        MSG_WriteDeltaKey(common, msg, key, (*from).angles[0], (*to).angles[0], 16);
        MSG_WriteDeltaKey(common, msg, key, (*from).angles[1], (*to).angles[1], 16);
        MSG_WriteDeltaKey(common, msg, key, (*from).angles[2], (*to).angles[2], 16);
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).forwardmove as c_int,
            (*to).forwardmove as c_int,
            8,
        );
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).rightmove as c_int,
            (*to).rightmove as c_int,
            8,
        );
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).upmove as c_int,
            (*to).upmove as c_int,
            8,
        );
        MSG_WriteDeltaKey(common, msg, key, (*from).buttons, (*to).buttons, 16);
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).weapon as c_int,
            (*to).weapon as c_int,
            8,
        );

        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).forcesel as c_int,
            (*to).forcesel as c_int,
            8,
        );
        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).invensel as c_int,
            (*to).invensel as c_int,
            8,
        );

        MSG_WriteDeltaKey(
            common,
            msg,
            key,
            (*from).generic_cmd as c_int,
            (*to).generic_cmd as c_int,
            8,
        );
    }
}

/// Raven `MSG_ReadDeltaUsercmdKey`.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:786-822`
pub fn MSG_ReadDeltaUsercmdKey(
    common: &mut Common,
    msg: *mut msg_t,
    key: c_int,
    from: *mut usercmd_t,
    to: *mut usercmd_t,
) {
    unsafe {
        if MSG_ReadBits(common, msg, 1) != 0 {
            (*to).serverTime = (*from).serverTime + MSG_ReadBits(common, msg, 8);
        } else {
            (*to).serverTime = MSG_ReadBits(common, msg, 32);
        }
        if MSG_ReadBits(common, msg, 1) != 0 {
            let key = key ^ (*to).serverTime;
            (*to).angles[0] = MSG_ReadDeltaKey(common, msg, key, (*from).angles[0], 16);
            (*to).angles[1] = MSG_ReadDeltaKey(common, msg, key, (*from).angles[1], 16);
            (*to).angles[2] = MSG_ReadDeltaKey(common, msg, key, (*from).angles[2], 16);
            (*to).forwardmove =
                MSG_ReadDeltaKey(common, msg, key, (*from).forwardmove as c_int, 8) as _;
            (*to).rightmove =
                MSG_ReadDeltaKey(common, msg, key, (*from).rightmove as c_int, 8) as _;
            (*to).upmove = MSG_ReadDeltaKey(common, msg, key, (*from).upmove as c_int, 8) as _;
            (*to).buttons = MSG_ReadDeltaKey(common, msg, key, (*from).buttons, 16);
            (*to).weapon = MSG_ReadDeltaKey(common, msg, key, (*from).weapon as c_int, 8) as _;

            (*to).forcesel = MSG_ReadDeltaKey(common, msg, key, (*from).forcesel as c_int, 8) as _;
            (*to).invensel = MSG_ReadDeltaKey(common, msg, key, (*from).invensel as c_int, 8) as _;

            (*to).generic_cmd =
                MSG_ReadDeltaKey(common, msg, key, (*from).generic_cmd as c_int, 8) as _;
        } else {
            (*to).angles[0] = (*from).angles[0];
            (*to).angles[1] = (*from).angles[1];
            (*to).angles[2] = (*from).angles[2];
            (*to).forwardmove = (*from).forwardmove;
            (*to).rightmove = (*from).rightmove;
            (*to).upmove = (*from).upmove;
            (*to).buttons = (*from).buttons;
            (*to).weapon = (*from).weapon;

            (*to).forcesel = (*from).forcesel;
            (*to).invensel = (*from).invensel;

            (*to).generic_cmd = (*from).generic_cmd;
        }
    }
}

/// Raven `MSG_WriteDeltaEntity` — writes part of a packetentities message,
/// including the entity number. Deltas from a baseline or a previous packet;
/// a NULL `to` emits a remove update. With `force` clear, an identical entity
/// emits nothing (the in-order delta code catches it). The `_XBOX` `realSize`
/// paths and the compiled-out `assert(numFields + 1 == sizeof(*from)/4)` are
/// not present in this build.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:1069-1211`
pub fn MSG_WriteDeltaEntity(
    common: &mut Common,
    msg: *mut msg_t,
    from: *mut entityState_t,
    to: *mut entityState_t,
    force: qboolean,
) {
    unsafe {
        let num_fields = common.entity_state_fields.len();

        // a NULL to is a delta remove message
        if to.is_null() {
            if from.is_null() {
                return;
            }
            MSG_WriteBits(common, msg, (*from).number, GENTITYNUM_BITS);
            MSG_WriteBits(common, msg, 1, 1);
            return;
        }

        if (*to).number < 0 || (*to).number >= mp_qshared::shared::limits::MAX_GENTITIES as c_int {
            com_error(
                errorParm_t::ERR_FATAL,
                format!("MSG_WriteDeltaEntity: Bad entity number: {}", (*to).number),
            );
        }

        // build the change vector as bytes so it is endian independent
        let mut lc = 0usize;
        for i in 0..num_fields {
            let offset = common.entity_state_fields[i].offset;
            let fromF = (from as *const u8).add(offset as usize) as *const c_int;
            let toF = (to as *const u8).add(offset as usize) as *const c_int;
            if *fromF != *toF {
                lc = i + 1;
                common.entity_state_fields[i].mCount += 1;
            }
        }

        if lc == 0 {
            // nothing at all changed
            if force == qfalse {
                return; // nothing at all
            }
            // write two bits for no change
            MSG_WriteBits(common, msg, (*to).number, GENTITYNUM_BITS);
            MSG_WriteBits(common, msg, 0, 1); // not removed
            MSG_WriteBits(common, msg, 0, 1); // no delta
            return;
        }

        MSG_WriteBits(common, msg, (*to).number, GENTITYNUM_BITS);
        MSG_WriteBits(common, msg, 0, 1); // not removed
        MSG_WriteBits(common, msg, 1, 1); // we have a delta

        MSG_WriteByte(common, msg, lc as c_int); // # of changes

        common.oldsize += num_fields as i32;

        for i in 0..lc {
            let offset = common.entity_state_fields[i].offset;
            let bits = common.entity_state_fields[i].bits;
            let fromF = (from as *const u8).add(offset as usize) as *const c_int;
            let toF = (to as *const u8).add(offset as usize) as *const c_int;

            if *fromF == *toF {
                MSG_WriteBits(common, msg, 0, 1); // no change
                continue;
            }

            MSG_WriteBits(common, msg, 1, 1); // changed

            if bits == 0 {
                // float
                let fullFloat = *(toF as *const f32);
                let trunc = fullFloat as c_int;

                if fullFloat == 0.0f32 {
                    MSG_WriteBits(common, msg, 0, 1);
                    common.oldsize += FLOAT_INT_BITS;
                } else {
                    MSG_WriteBits(common, msg, 1, 1);
                    if trunc as f32 == fullFloat
                        && trunc + FLOAT_INT_BIAS >= 0
                        && trunc + FLOAT_INT_BIAS < (1 << FLOAT_INT_BITS)
                    {
                        // send as small integer
                        MSG_WriteBits(common, msg, 0, 1);
                        MSG_WriteBits(common, msg, trunc + FLOAT_INT_BIAS, FLOAT_INT_BITS);
                    } else {
                        // send as full floating point value
                        MSG_WriteBits(common, msg, 1, 1);
                        MSG_WriteBits(common, msg, *toF, 32);
                    }
                }
            } else if *toF == 0 {
                MSG_WriteBits(common, msg, 0, 1);
            } else {
                MSG_WriteBits(common, msg, 1, 1);
                // integer
                MSG_WriteBits(common, msg, *toF, bits);
            }
        }
    }
}

/// Raven `MSG_ReadDeltaEntity`.
///
/// STATE: `cl_shownet` (cvar read), `entityStateFields`. The `sv`/
/// `SV_GentityNum` classname probe (msg.cpp:1268-1270) reaches the server spine
/// through the sanctioned host edge ([`EngineHost::sv_shownet_entity_classname`],
/// ruling 56c) — qcommon cannot depend on `mp_engine_server` (cycle).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:1228-1383`
pub fn MSG_ReadDeltaEntity(
    view: &mut EngineHostView,
    msg: *mut msg_t,
    from: *mut entityState_t,
    to: *mut entityState_t,
    number: c_int,
) {
    unsafe {
        if !(0..mp_qshared::shared::limits::MAX_GENTITIES as c_int).contains(&number) {
            //TODO: Port Com_Error
            // Source: oracle/codemp/qcommon/msg.cpp:1239 (ruling 1: receiverless panic)
            com_error(
                errorParm_t::ERR_DROP,
                format!("Bad delta entity number: {number}"),
            );
        }

        let startBit = (*msg).bit;

        // check for a remove
        if MSG_ReadBits(view.common, msg, 1) == 1 {
            crate::common_fns::Com_Memset(to as *mut (), 0, core::mem::size_of::<entityState_t>());
            (*to).number = mp_qshared::shared::limits::MAX_GENTITIES as c_int - 1;
            //TODO: Port cl_shownet
            // Source: oracle/codemp/qcommon/msg.cpp:12
            if view.common.cl_shownet >= 2 || view.common.cl_shownet == -1 {
                crate::common::com_printf(
                    view.common,
                    &format!("{:3}: #{:<3} remove\n", (*msg).readcount, number),
                );
            }
            return;
        }

        // check for no delta
        if MSG_ReadBits(view.common, msg, 1) == 0 {
            *to = *from;
            (*to).number = number;
            return;
        }

        let num_fields = view.common.entity_state_fields.len();
        let lc = MSG_ReadByte(view.common, msg);

        // shownet 2/3 will interleave with other printed info, -1 will
        // just print the delta records`
        let print = if view.common.cl_shownet >= 2 || view.common.cl_shownet == -1 {
            // Bind the host probe to an owned local first: the returned
            // `Option<String>` holds no borrow of `view`, so `view.common` is
            // free inside the branches (an `if let` scrutinee would otherwise
            // extend the `&mut view` receiver borrow across the block).
            let classname = view.sv_shownet_entity_classname(number);
            if let Some(classname) = classname {
                crate::common::com_printf(
                    view.common,
                    &format!("{:3}: #{:<3} ({}) ", (*msg).readcount, number, classname),
                );
            } else {
                crate::common::com_printf(
                    view.common,
                    &format!("{:3}: #{:<3} ", (*msg).readcount, number),
                );
            }
            true
        } else {
            false
        };

        (*to).number = number;

        for i in 0..lc {
            // Copy the field's data out (all `Copy`/`'static`) so no borrow of
            // `common.entity_state_fields` is held across the `MSG_Read*`/
            // `com_printf` calls below, which need `&mut common`.
            let field_offset = view.common.entity_state_fields[i as usize].offset;
            let field_bits = view.common.entity_state_fields[i as usize].bits;
            let field_name = view.common.entity_state_fields[i as usize].name;
            let fromF = (from as *const u8).add(field_offset as usize) as *const c_int;
            let toF = (to as *mut u8).add(field_offset as usize) as *mut c_int;

            if MSG_ReadBits(view.common, msg, 1) == 0 {
                // no change
                *toF = *fromF;
            } else if field_bits == 0 {
                // float
                if MSG_ReadBits(view.common, msg, 1) == 0 {
                    *(toF as *mut f32) = 0.0f32;
                } else if MSG_ReadBits(view.common, msg, 1) == 0 {
                    // integral float
                    let mut trunc =
                        MSG_ReadBits(view.common, msg, crate::qcommon::msg_consts::FLOAT_INT_BITS);
                    // bias to allow equal parts positive and negative
                    trunc -= crate::qcommon::msg_consts::FLOAT_INT_BIAS;
                    *(toF as *mut f32) = trunc as f32;
                    if print {
                        crate::common::com_printf(
                            view.common,
                            &format!("{}:{} ", field_name, trunc),
                        );
                    }
                } else {
                    // full floating point value
                    *toF = MSG_ReadBits(view.common, msg, 32);
                    if print {
                        crate::common::com_printf(
                            view.common,
                            &format!("{}:{} ", field_name, *(toF as *mut f32)),
                        );
                    }
                }
            } else if MSG_ReadBits(view.common, msg, 1) == 0 {
                *toF = 0;
            } else {
                // integer
                *toF = MSG_ReadBits(view.common, msg, field_bits);
                if print {
                    crate::common::com_printf(view.common, &format!("{}:{} ", field_name, *toF));
                }
            }
        }
        for i in lc..(num_fields as c_int) {
            let field = &view.common.entity_state_fields[i as usize];
            let fromF = (from as *const u8).add(field.offset as usize) as *const c_int;
            let toF = (to as *mut u8).add(field.offset as usize) as *mut c_int;
            // no change
            *toF = *fromF;
        }

        if print {
            let endBit = (*msg).bit;
            crate::common::com_printf(view.common, &format!(" ({} bits)\n", endBit - startBit));
        }
    }
}

/// Raven `MSG_WriteDeltaPlayerstate`. `_OPTIMIZED_VEHICLE_NETWORKING` is
/// unconditionally defined (`q_shared.h:2154`), so the table-selection branch
/// is live: a vehicle ps uses `vehPlayerStateFields` with no selector bit;
/// otherwise a mandatory 1-bit selector distinguishes a pilot ps
/// (`m_iVehicleNum && (eFlags & EF_NODRAW)`, `pilotPlayerStateFields`
/// truncated to `len - 82`) from a normal ps (`playerStateFields`).
/// `_ONEBIT_COMBO` is undefined; its bit-combo-mask tail is dead. Raven's
/// write-only `c = msg->cursize` / `gLastBitIndex = lc` diagnostics are
/// dropped as dead surface.
///
/// Source: `oracle/codemp/qcommon/msg.cpp:2211-2444`
pub fn MSG_WriteDeltaPlayerstate(
    common: &mut Common,
    msg: *mut msg_t,
    from: *mut playerState_t,
    to: *mut playerState_t,
    isVehiclePS: qboolean,
) {
    unsafe {
        let mut dummy: playerState_t;
        let from = if from.is_null() {
            dummy = core::mem::zeroed();
            crate::common_fns::Com_Memset(
                &mut dummy as *mut playerState_t as *mut (),
                0,
                core::mem::size_of::<playerState_t>(),
            );
            &mut dummy as *mut playerState_t
        } else {
            from
        };

        // `EF_NODRAW` — bg_public.h entity flag; mirrored here since mp_bg
        // (game tier) is above this crate's dependency graph.
        // Source: oracle/codemp/game/bg_public.h:581
        const EF_NODRAW: c_int = 1 << 8;

        let table: PsfTable;
        let num_fields: usize;
        if isVehiclePS != qfalse {
            //a vehicle playerstate
            table = PsfTable::Veh;
            num_fields = common.veh_player_state_fields.len();
        } else {
            //regular client playerstate
            if (*to).m_iVehicleNum != 0 && ((*to).eFlags & EF_NODRAW) != 0 {
                //pilot riding *inside* a vehicle!
                MSG_WriteBits(common, msg, 1, 1); // Pilot player state
                table = PsfTable::Pilot;
                num_fields = common.pilot_player_state_fields.len() - 82;
            } else {
                //normal client
                MSG_WriteBits(common, msg, 0, 1); // Normal player state
                table = PsfTable::Normal;
                num_fields = common.player_state_fields.len();
            }
        }

        let mut lc = 0usize;
        for i in 0..num_fields {
            let offset = psf_fields(common, table)[i].offset;
            let fromF = (from as *const u8).add(offset as usize) as *const c_int;
            let toF = (to as *const u8).add(offset as usize) as *const c_int;
            if *fromF != *toF {
                lc = i + 1;
                psf_fields_mut(common, table)[i].mCount += 1;
            }
        }

        MSG_WriteByte(common, msg, lc as c_int); // # of changes

        common.oldsize += (num_fields - lc) as i32;

        for i in 0..lc {
            let offset = psf_fields(common, table)[i].offset;
            let bits = psf_fields(common, table)[i].bits;
            let fromF = (from as *const u8).add(offset as usize) as *const c_int;
            let toF = (to as *const u8).add(offset as usize) as *const c_int;

            if *fromF == *toF {
                MSG_WriteBits(common, msg, 0, 1); // no change
                continue;
            }

            MSG_WriteBits(common, msg, 1, 1); // changed

            if bits == 0 {
                // float
                let fullFloat = *(toF as *const f32);
                let trunc = fullFloat as c_int;

                if trunc as f32 == fullFloat
                    && trunc + FLOAT_INT_BIAS >= 0
                    && trunc + FLOAT_INT_BIAS < (1 << FLOAT_INT_BITS)
                {
                    // send as small integer
                    MSG_WriteBits(common, msg, 0, 1);
                    MSG_WriteBits(common, msg, trunc + FLOAT_INT_BIAS, FLOAT_INT_BITS);
                } else {
                    // send as full floating point value
                    MSG_WriteBits(common, msg, 1, 1);
                    MSG_WriteBits(common, msg, *toF, 32);
                }
            } else {
                // integer
                MSG_WriteBits(common, msg, *toF, bits);
            }
        }

        //
        // send the arrays
        //
        let mut statsbits = 0;
        for i in 0..16 {
            if (*to).stats[i] != (*from).stats[i] {
                statsbits |= 1 << i;
            }
        }
        let mut persistantbits = 0;
        for i in 0..16 {
            if (*to).persistant[i] != (*from).persistant[i] {
                persistantbits |= 1 << i;
            }
        }
        let mut ammobits = 0;
        for i in 0..16 {
            if (*to).ammo[i] != (*from).ammo[i] {
                ammobits |= 1 << i;
            }
        }
        let mut powerupbits = 0;
        for i in 0..16 {
            if (*to).powerups[i] != (*from).powerups[i] {
                powerupbits |= 1 << i;
            }
        }

        if statsbits == 0 && persistantbits == 0 && ammobits == 0 && powerupbits == 0 {
            MSG_WriteBits(common, msg, 0, 1); // no change
            common.oldsize += 4;
            return;
        }
        MSG_WriteBits(common, msg, 1, 1); // changed

        if statsbits != 0 {
            // `STAT_WEAPONS` — bg_public.h statIndex_t; mirrored here since mp_bg
            // (game tier) is above this crate's dependency graph.
            // Source: oracle/codemp/game/bg_public.h:520-532
            const STAT_WEAPONS: usize = 4;
            MSG_WriteBits(common, msg, 1, 1); // changed
            MSG_WriteShort(common, msg, statsbits);
            for i in 0..16usize {
                if statsbits & (1 << i) != 0 {
                    if i == STAT_WEAPONS {
                        // just send this one in MAX_WEAPONS bits, so we can add up
                        // to MAX_WEAPONS weaps without hassle -rww
                        MSG_WriteBits(
                            common,
                            msg,
                            (*to).stats[i],
                            mp_qshared::common::mp::qcommon::player_state::MAX_WEAPONS as c_int,
                        );
                    } else {
                        MSG_WriteShort(common, msg, (*to).stats[i]);
                    }
                }
            }
        } else {
            MSG_WriteBits(common, msg, 0, 1); // no change
        }

        if persistantbits != 0 {
            MSG_WriteBits(common, msg, 1, 1); // changed
            MSG_WriteShort(common, msg, persistantbits);
            for i in 0..16usize {
                if persistantbits & (1 << i) != 0 {
                    MSG_WriteShort(common, msg, (*to).persistant[i]);
                }
            }
        } else {
            MSG_WriteBits(common, msg, 0, 1); // no change
        }

        if ammobits != 0 {
            MSG_WriteBits(common, msg, 1, 1); // changed
            MSG_WriteShort(common, msg, ammobits);
            for i in 0..16usize {
                if ammobits & (1 << i) != 0 {
                    MSG_WriteShort(common, msg, (*to).ammo[i]);
                }
            }
        } else {
            MSG_WriteBits(common, msg, 0, 1); // no change
        }

        if powerupbits != 0 {
            MSG_WriteBits(common, msg, 1, 1); // changed
            MSG_WriteShort(common, msg, powerupbits);
            for i in 0..16usize {
                if powerupbits & (1 << i) != 0 {
                    MSG_WriteLong(common, msg, (*to).powerups[i]);
                }
            }
        } else {
            MSG_WriteBits(common, msg, 0, 1); // no change
        }
    }
}

/// Raven `MSG_ReadDeltaPlayerstate`. Mirrors the write side's live
/// `_OPTIMIZED_VEHICLE_NETWORKING` table selection (`q_shared.h:2154`): a
/// vehicle ps reads `vehPlayerStateFields` with no selector bit; otherwise a
/// mandatory 1-bit selector picks `pilotPlayerStateFields` truncated to
/// `len - 82` (bit 1) or `playerStateFields` (bit 0).
///
/// Source: `oracle/codemp/qcommon/msg.cpp:2455-2692`
pub fn MSG_ReadDeltaPlayerstate(
    common: &mut Common,
    msg: *mut msg_t,
    from: *mut playerState_t,
    to: *mut playerState_t,
    isVehiclePS: qboolean,
) {
    unsafe {
        let mut dummy: playerState_t;
        let from = if from.is_null() {
            dummy = core::mem::zeroed();
            crate::common_fns::Com_Memset(
                &mut dummy as *mut playerState_t as *mut (),
                0,
                core::mem::size_of::<playerState_t>(),
            );
            &mut dummy as *mut playerState_t
        } else {
            from
        };
        *to = *from;

        let startBit = if (*msg).bit == 0 {
            (*msg).readcount * 8 - mp_qshared::shared::limits::GENTITYNUM_BITS
        } else {
            ((*msg).readcount - 1) * 8 + (*msg).bit - mp_qshared::shared::limits::GENTITYNUM_BITS
        };

        // shownet 2/3 will interleave with other printed info, -2 will
        // just print the delta records
        let print = if common.cl_shownet >= 2 || common.cl_shownet == -2 {
            crate::common::com_printf(common, &format!("{:3}: playerstate ", (*msg).readcount));
            true
        } else {
            false
        };

        let table: PsfTable;
        let num_fields: usize;
        if isVehiclePS != qfalse {
            //a vehicle playerstate
            table = PsfTable::Veh;
            num_fields = common.veh_player_state_fields.len();
        } else {
            let isPilot = MSG_ReadBits(common, msg, 1);
            if isPilot != 0 {
                //pilot riding *inside* a vehicle!
                table = PsfTable::Pilot;
                num_fields = common.pilot_player_state_fields.len() - 82;
            } else {
                //normal client
                table = PsfTable::Normal;
                num_fields = common.player_state_fields.len();
            }
        }

        let lc = MSG_ReadByte(common, msg);

        for i in 0..lc {
            let offset = psf_fields(common, table)[i as usize].offset;
            let bits = psf_fields(common, table)[i as usize].bits;
            let name = psf_fields(common, table)[i as usize].name;
            let fromF = (from as *const u8).add(offset as usize) as *const c_int;
            let toF = (to as *mut u8).add(offset as usize) as *mut c_int;

            if MSG_ReadBits(common, msg, 1) == 0 {
                // no change
                *toF = *fromF;
            } else if bits == 0 {
                // float
                if MSG_ReadBits(common, msg, 1) == 0 {
                    // integral float
                    let mut trunc =
                        MSG_ReadBits(common, msg, crate::qcommon::msg_consts::FLOAT_INT_BITS);
                    // bias to allow equal parts positive and negative
                    trunc -= crate::qcommon::msg_consts::FLOAT_INT_BIAS;
                    *(toF as *mut f32) = trunc as f32;
                    if print {
                        crate::common::com_printf(common, &format!("{}:{} ", name, trunc));
                    }
                } else {
                    // full floating point value
                    *toF = MSG_ReadBits(common, msg, 32);
                    if print {
                        crate::common::com_printf(
                            common,
                            &format!("{}:{} ", name, *(toF as *mut f32)),
                        );
                    }
                }
            } else {
                // integer
                *toF = MSG_ReadBits(common, msg, bits);
                if print {
                    crate::common::com_printf(common, &format!("{}:{} ", name, *toF));
                }
            }
        }
        for i in lc..(num_fields as c_int) {
            let field = &psf_fields(common, table)[i as usize];
            let fromF = (from as *const u8).add(field.offset as usize) as *const c_int;
            let toF = (to as *mut u8).add(field.offset as usize) as *mut c_int;
            // no change
            *toF = *fromF;
        }

        // read the arrays
        if MSG_ReadBits(common, msg, 1) != 0 {
            // parse stats
            if MSG_ReadBits(common, msg, 1) != 0 {
                //TODO: Port LOG
                // Source: oracle/codemp/qcommon/msg.cpp:2588 ("PS_STATS")
                let bits = MSG_ReadShort(common, msg);
                for i in 0..16 {
                    if bits & (1 << i) != 0 {
                        // `STAT_WEAPONS` — bg_public.h statIndex_t; mirrored here since
                        // mp_bg (game tier) is above this crate's dependency graph.
                        // Source: oracle/codemp/game/bg_public.h:520-532
                        const STAT_WEAPONS: c_int = 4;
                        if i == STAT_WEAPONS {
                            (*to).stats[i as usize] = MSG_ReadBits(
                                common,
                                msg,
                                mp_qshared::common::mp::qcommon::player_state::MAX_WEAPONS as c_int,
                            );
                        } else {
                            (*to).stats[i as usize] = MSG_ReadShort(common, msg);
                        }
                    }
                }
            }

            // parse persistant stats
            if MSG_ReadBits(common, msg, 1) != 0 {
                //TODO: Port LOG
                // Source: oracle/codemp/qcommon/msg.cpp:2614 ("PS_PERSISTANT")
                let bits = MSG_ReadShort(common, msg);
                for i in 0..16 {
                    if bits & (1 << i) != 0 {
                        (*to).persistant[i as usize] = MSG_ReadShort(common, msg);
                    }
                }
            }

            // parse ammo
            if MSG_ReadBits(common, msg, 1) != 0 {
                //TODO: Port LOG
                // Source: oracle/codemp/qcommon/msg.cpp:2632 ("PS_AMMO")
                let bits = MSG_ReadShort(common, msg);
                for i in 0..16 {
                    if bits & (1 << i) != 0 {
                        (*to).ammo[i as usize] = MSG_ReadShort(common, msg);
                    }
                }
            }

            // parse powerups
            if MSG_ReadBits(common, msg, 1) != 0 {
                //TODO: Port LOG
                // Source: oracle/codemp/qcommon/msg.cpp:2650 ("PS_POWERUPS")
                let bits = MSG_ReadShort(common, msg);
                for i in 0..16 {
                    if bits & (1 << i) != 0 {
                        (*to).powerups[i as usize] = MSG_ReadLong(common, msg);
                    }
                }
            }
        }

        if print {
            let endBit = if (*msg).bit == 0 {
                (*msg).readcount * 8 - mp_qshared::shared::limits::GENTITYNUM_BITS
            } else {
                ((*msg).readcount - 1) * 8 + (*msg).bit
                    - mp_qshared::shared::limits::GENTITYNUM_BITS
            };
            crate::common::com_printf(common, &format!(" ({} bits)\n", endBit - startBit));
        }

        // _ONEBIT_COMBO not defined for this build; the mask-replay tail is dead.
    }
}
