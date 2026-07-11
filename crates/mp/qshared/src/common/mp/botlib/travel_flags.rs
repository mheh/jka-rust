#![allow(non_upper_case_globals)]

use std::os::raw::c_int;

/// Raven travel flags (`TFL_*`) — bot AAS travel-type/terrain bitmask.
///
/// Source: `oracle/codemp/game/be_aas.h:23-50`
pub const TFL_INVALID: c_int = 0x00000001;
/// Walking.
pub const TFL_WALK: c_int = 0x00000002;
/// Crouching.
pub const TFL_CROUCH: c_int = 0x00000004;
/// Jumping onto a barrier.
pub const TFL_BARRIERJUMP: c_int = 0x00000008;
/// Jumping.
pub const TFL_JUMP: c_int = 0x00000010;
/// Climbing a ladder.
pub const TFL_LADDER: c_int = 0x00000020;
/// Walking off a ledge.
pub const TFL_WALKOFFLEDGE: c_int = 0x00000080;
/// Swimming.
pub const TFL_SWIM: c_int = 0x00000100;
/// Jumping out of the water.
pub const TFL_WATERJUMP: c_int = 0x00000200;
/// Teleporting.
pub const TFL_TELEPORT: c_int = 0x00000400;
/// Elevator.
pub const TFL_ELEVATOR: c_int = 0x00000800;
/// Rocket jumping.
pub const TFL_ROCKETJUMP: c_int = 0x00001000;
/// BFG jumping.
pub const TFL_BFGJUMP: c_int = 0x00002000;
/// Grappling hook.
pub const TFL_GRAPPLEHOOK: c_int = 0x00004000;
/// Double jump.
pub const TFL_DOUBLEJUMP: c_int = 0x00008000;
/// Ramp jump.
pub const TFL_RAMPJUMP: c_int = 0x00010000;
/// Strafe jump.
pub const TFL_STRAFEJUMP: c_int = 0x00020000;
/// Jump pad.
pub const TFL_JUMPPAD: c_int = 0x00040000;
/// Travel through air.
pub const TFL_AIR: c_int = 0x00080000;
/// Travel through water.
pub const TFL_WATER: c_int = 0x00100000;
/// Travel through slime.
pub const TFL_SLIME: c_int = 0x00200000;
/// Travel through lava.
pub const TFL_LAVA: c_int = 0x00400000;
/// Travel through donotenter area.
pub const TFL_DONOTENTER: c_int = 0x00800000;
/// Func bobbing.
pub const TFL_FUNCBOB: c_int = 0x01000000;
/// Flight.
pub const TFL_FLIGHT: c_int = 0x02000000;
/// Move over a bridge.
pub const TFL_BRIDGE: c_int = 0x04000000;
/// Not team 1.
pub const TFL_NOTTEAM1: c_int = 0x08000000;
/// Not team 2.
pub const TFL_NOTTEAM2: c_int = 0x10000000;

/// Raven `TFL_DEFAULT` — default travel flags mask.
///
/// Source: `oracle/codemp/game/be_aas.h:53-57`
pub const TFL_DEFAULT: c_int = TFL_WALK
    | TFL_CROUCH
    | TFL_BARRIERJUMP
    | TFL_JUMP
    | TFL_LADDER
    | TFL_WALKOFFLEDGE
    | TFL_SWIM
    | TFL_WATERJUMP
    | TFL_TELEPORT
    | TFL_ELEVATOR
    | TFL_AIR
    | TFL_WATER
    | TFL_JUMPPAD
    | TFL_FUNCBOB;
