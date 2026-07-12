#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven `MAX_TRAVELTYPES` and travel types (`TRAVEL_*`) — how a reachability link is traversed.
///
/// Source: `oracle/codemp/botlib/aasfile.h:16-35`
pub const MAX_TRAVELTYPES: c_int = 32;
/// temporary not possible
pub const TRAVEL_INVALID: c_int = 1;
/// walking
pub const TRAVEL_WALK: c_int = 2;
/// crouching
pub const TRAVEL_CROUCH: c_int = 3;
/// jumping onto a barrier
pub const TRAVEL_BARRIERJUMP: c_int = 4;
/// jumping
pub const TRAVEL_JUMP: c_int = 5;
/// climbing a ladder
pub const TRAVEL_LADDER: c_int = 6;
/// walking of a ledge
pub const TRAVEL_WALKOFFLEDGE: c_int = 7;
/// swimming
pub const TRAVEL_SWIM: c_int = 8;
/// jump out of the water
pub const TRAVEL_WATERJUMP: c_int = 9;
/// teleportation
pub const TRAVEL_TELEPORT: c_int = 10;
/// travel by elevator
pub const TRAVEL_ELEVATOR: c_int = 11;
/// rocket jumping required for travel
pub const TRAVEL_ROCKETJUMP: c_int = 12;
/// bfg jumping required for travel
pub const TRAVEL_BFGJUMP: c_int = 13;
/// grappling hook required for travel
pub const TRAVEL_GRAPPLEHOOK: c_int = 14;
/// double jump
pub const TRAVEL_DOUBLEJUMP: c_int = 15;
/// ramp jump
pub const TRAVEL_RAMPJUMP: c_int = 16;
/// strafe jump
pub const TRAVEL_STRAFEJUMP: c_int = 17;
/// jump pad
pub const TRAVEL_JUMPPAD: c_int = 18;
/// func bob
pub const TRAVEL_FUNCBOB: c_int = 19;

/// Raven additional travel flags packed into a `reachability_t.traveltype`.
///
/// Source: `oracle/codemp/botlib/aasfile.h:38-40`
pub const TRAVELTYPE_MASK: c_int = 0xFFFFFF;
pub const TRAVELFLAG_NOTTEAM1: c_int = 1 << 24;
pub const TRAVELFLAG_NOTTEAM2: c_int = 2 << 24;
