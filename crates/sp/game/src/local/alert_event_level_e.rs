#![allow(non_camel_case_types, non_snake_case)]

/// Raven `alertEventLevel_e` — alert severity level for AI awareness.
///
/// Type definition source: `oracle/code/game/g_local.h:115-122`
#[repr(i32)]
pub enum alertEventLevel_e {
    /// Enemy responds to the sound, but only by looking
    AEL_MINOR,
    /// Enemy looks at the sound, and will also investigate it
    AEL_SUSPICIOUS,
    /// Enemy knows the player is around, and will actively hunt
    AEL_DISCOVERED,
    /// Enemy should try to find cover
    AEL_DANGER,
    /// Enemy should run like hell!
    AEL_DANGER_GREAT,
}
