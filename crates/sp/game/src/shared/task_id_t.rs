#![allow(non_camel_case_types, non_snake_case)]

/// Raven `taskID_t` — Task identifiers for timed events.
///
/// Raven: (comments preserved inline per variant below).
/// Type definition source: `oracle/code/game/g_shared.h:20-34`
#[repr(i32)]
pub enum taskID_t {
    /// Waiting for a voice sound to complete.
    TID_CHAN_VOICE = 0,
    /// Waiting to finish a lower anim holdtime.
    TID_ANIM_UPPER,
    /// Waiting to finish a lower anim holdtime.
    TID_ANIM_LOWER,
    /// Waiting to finish lower and upper anim holdtimes or normal md3 animating.
    TID_ANIM_BOTH,
    /// Trying to get to a navgoal or For ET_MOVERS.
    TID_MOVE_NAV,
    /// Turning to an angle or facing.
    TID_ANGLE_FACE,
    /// Waiting for a certain bState to finish.
    TID_BSTATE,
    /// Waiting for ent to enter a specific trigger_location.
    TID_LOCATION,
    /// Waiting for clear bbox to inflate size.
    TID_RESIZE,
    /// Waiting for fire event.
    TID_SHOOT,
    /// For def of taskID array.
    NUM_TIDS,
}
