#![allow(non_camel_case_types, non_snake_case)]

/// Raven `bState_t` — behavior state type for NPC autonomous behavior.
///
/// Raven: These take over only if script allows them to be autonomous.
/// Type definition source: `oracle/codemp/game/g_public.h:584-606`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum bState_t {
    BS_DEFAULT = 0,   // default behavior for that NPC
    BS_ADVANCE_FIGHT, // Advance to captureGoal and shoot enemies if you can
    BS_SLEEP,         // Play awake script when startled by sound
    BS_FOLLOW_LEADER, // Follow your leader and shoot any enemies you come across
    BS_JUMP,          // Face navgoal and jump to it.
    BS_SEARCH, // Using current waypoint as a base, search the immediate branches of waypoints for enemies
    BS_WANDER, // Wander down random waypoint paths
    BS_NOCLIP, // Moves through walls, etc.
    BS_REMOVE, // Waits for player to leave PVS then removes itself
    BS_CINEMATIC, // Does nothing but face it's angles and move to a goal if it has one
    // internal bStates only
    BS_WAIT, // Does nothing but face it's angles
    BS_STAND_GUARD,
    BS_PATROL,
    BS_INVESTIGATE, // head towards temp goal and look for enemies and listen for sounds
    BS_STAND_AND_SHOOT,
    BS_HUNT_AND_KILL,
    BS_FLEE, // Run away!
    NUM_BSTATES,
}
