#![allow(non_camel_case_types)]

//! Movement-AI flag/type constants.
//!
//! Source: `oracle/codemp/game/be_ai_move.h:17-56`

/// Raven `MOVE_WALK` — normal walking movement.
/// Source: `oracle/codemp/game/be_ai_move.h:18`
pub const MOVE_WALK: i32 = 1;
/// Raven `MOVE_CROUCH` — crouched movement.
/// Source: `oracle/codemp/game/be_ai_move.h:19`
pub const MOVE_CROUCH: i32 = 2;
/// Raven `MOVE_JUMP` — jumping movement.
/// Source: `oracle/codemp/game/be_ai_move.h:20`
pub const MOVE_JUMP: i32 = 4;
/// Raven `MOVE_GRAPPLE` — grapple-hook movement.
/// Source: `oracle/codemp/game/be_ai_move.h:21`
pub const MOVE_GRAPPLE: i32 = 8;
/// Raven `MOVE_ROCKETJUMP` — rocket-jump movement.
/// Source: `oracle/codemp/game/be_ai_move.h:22`
pub const MOVE_ROCKETJUMP: i32 = 16;
/// Raven `MOVE_BFGJUMP` — BFG-jump movement.
/// Source: `oracle/codemp/game/be_ai_move.h:23`
pub const MOVE_BFGJUMP: i32 = 32;

/// Raven `MFL_BARRIERJUMP` — bot is performing a barrier jump.
/// Source: `oracle/codemp/game/be_ai_move.h:25`
pub const MFL_BARRIERJUMP: i32 = 1;
/// Raven `MFL_ONGROUND` — bot is on the ground.
/// Source: `oracle/codemp/game/be_ai_move.h:26`
pub const MFL_ONGROUND: i32 = 2;
/// Raven `MFL_SWIMMING` — bot is swimming.
/// Source: `oracle/codemp/game/be_ai_move.h:27`
pub const MFL_SWIMMING: i32 = 4;
/// Raven `MFL_AGAINSTLADDER` — bot is against a ladder.
/// Source: `oracle/codemp/game/be_ai_move.h:28`
pub const MFL_AGAINSTLADDER: i32 = 8;
/// Raven `MFL_WATERJUMP` — bot is waterjumping.
/// Source: `oracle/codemp/game/be_ai_move.h:29`
pub const MFL_WATERJUMP: i32 = 16;
/// Raven `MFL_TELEPORTED` — bot is being teleported.
/// Source: `oracle/codemp/game/be_ai_move.h:30`
pub const MFL_TELEPORTED: i32 = 32;
/// Raven `MFL_GRAPPLEPULL` — bot is being pulled by the grapple.
/// Source: `oracle/codemp/game/be_ai_move.h:31`
pub const MFL_GRAPPLEPULL: i32 = 64;
/// Raven `MFL_ACTIVEGRAPPLE` — bot is using the grapple hook.
/// Source: `oracle/codemp/game/be_ai_move.h:32`
pub const MFL_ACTIVEGRAPPLE: i32 = 128;
/// Raven `MFL_GRAPPLERESET` — bot has reset the grapple.
/// Source: `oracle/codemp/game/be_ai_move.h:33`
pub const MFL_GRAPPLERESET: i32 = 256;
/// Raven `MFL_WALK` — bot should walk slowly.
/// Source: `oracle/codemp/game/be_ai_move.h:34`
pub const MFL_WALK: i32 = 512;

/// Raven `MOVERESULT_MOVEMENTVIEW` — bot uses view for movement.
/// Source: `oracle/codemp/game/be_ai_move.h:36`
pub const MOVERESULT_MOVEMENTVIEW: i32 = 1;
/// Raven `MOVERESULT_SWIMVIEW` — bot uses view for swimming.
/// Source: `oracle/codemp/game/be_ai_move.h:37`
pub const MOVERESULT_SWIMVIEW: i32 = 2;
/// Raven `MOVERESULT_WAITING` — bot is waiting for something.
/// Source: `oracle/codemp/game/be_ai_move.h:38`
pub const MOVERESULT_WAITING: i32 = 4;
/// Raven `MOVERESULT_MOVEMENTVIEWSET` — bot has set the view in movement code.
/// Source: `oracle/codemp/game/be_ai_move.h:39`
pub const MOVERESULT_MOVEMENTVIEWSET: i32 = 8;
/// Raven `MOVERESULT_MOVEMENTWEAPON` — bot uses weapon for movement.
/// Source: `oracle/codemp/game/be_ai_move.h:40`
pub const MOVERESULT_MOVEMENTWEAPON: i32 = 16;
/// Raven `MOVERESULT_ONTOPOFOBSTACLE` — bot is ontop of obstacle.
/// Source: `oracle/codemp/game/be_ai_move.h:41`
pub const MOVERESULT_ONTOPOFOBSTACLE: i32 = 32;
/// Raven `MOVERESULT_ONTOPOF_FUNCBOB` — bot is ontop of a func_bobbing.
/// Source: `oracle/codemp/game/be_ai_move.h:42`
pub const MOVERESULT_ONTOPOF_FUNCBOB: i32 = 64;
/// Raven `MOVERESULT_ONTOPOF_ELEVATOR` — bot is ontop of an elevator (func_plat).
/// Source: `oracle/codemp/game/be_ai_move.h:43`
pub const MOVERESULT_ONTOPOF_ELEVATOR: i32 = 128;
/// Raven `MOVERESULT_BLOCKEDBYAVOIDSPOT` — bot is blocked by an avoid spot.
/// Source: `oracle/codemp/game/be_ai_move.h:44`
pub const MOVERESULT_BLOCKEDBYAVOIDSPOT: i32 = 256;

/// Raven `AVOID_CLEAR` — clear all avoid spots.
/// Source: `oracle/codemp/game/be_ai_move.h:49`
pub const AVOID_CLEAR: i32 = 0;
/// Raven `AVOID_ALWAYS` — avoid always.
/// Source: `oracle/codemp/game/be_ai_move.h:50`
pub const AVOID_ALWAYS: i32 = 1;
/// Raven `AVOID_DONTBLOCK` — never totally block.
/// Source: `oracle/codemp/game/be_ai_move.h:51`
pub const AVOID_DONTBLOCK: i32 = 2;

/// Raven `RESULTTYPE_ELEVATORUP` — elevator is up.
/// Source: `oracle/codemp/game/be_ai_move.h:53`
pub const RESULTTYPE_ELEVATORUP: i32 = 1;
/// Raven `RESULTTYPE_WAITFORFUNCBOBBING` — waiting for func bobbing to arrive.
/// Source: `oracle/codemp/game/be_ai_move.h:54`
pub const RESULTTYPE_WAITFORFUNCBOBBING: i32 = 2;
/// Raven `RESULTTYPE_BADGRAPPLEPATH` — grapple path is obstructed.
/// Source: `oracle/codemp/game/be_ai_move.h:55`
pub const RESULTTYPE_BADGRAPPLEPATH: i32 = 4;
/// Raven `RESULTTYPE_INSOLIDAREA` — stuck in solid area, this is bad.
/// Source: `oracle/codemp/game/be_ai_move.h:56`
pub const RESULTTYPE_INSOLIDAREA: i32 = 8;
