#![allow(non_camel_case_types, non_snake_case)]

/// Raven `taskID_t` — enumeration of task IDs for tracking NPC task completion.
///
/// Type definition source: `oracle/codemp/game/g_public.h:625-639`
#[repr(i32)]
pub enum taskID_t {
    TID_CHAN_VOICE = 0, // Waiting for a voice sound to complete
    TID_ANIM_UPPER,     // Waiting to finish a lower anim holdtime
    TID_ANIM_LOWER,     // Waiting to finish a lower anim holdtime
    TID_ANIM_BOTH,      // Waiting to finish lower and upper anim holdtimes or normal md3 animating
    TID_MOVE_NAV,       // Trying to get to a navgoal or For ET_MOVERS
    TID_ANGLE_FACE,     // Turning to an angle or facing
    TID_BSTATE,         // Waiting for a certain bState to finish
    TID_LOCATION,       // Waiting for ent to enter a specific trigger_location
    // TID_MISSIONSTATUS,  // Waiting for player to finish reading MISSION STATUS SCREEN
    TID_RESIZE, // Waiting for clear bbox to inflate size
    TID_SHOOT,  // Waiting for fire event
    NUM_TIDS,   // for def of taskID array
}
