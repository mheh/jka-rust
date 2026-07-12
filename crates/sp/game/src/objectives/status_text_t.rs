#![allow(non_camel_case_types, non_snake_case)]

/// Raven `statusText_t` — status message codes.
///
/// Type definition source: `oracle/code/game/objectives.h:141-157`
#[repr(i32)]
pub enum statusText_t {
    STAT_INSUBORDINATION = 0, // Starfleet will not tolerate such insubordination
    STAT_YOUCAUSEDDEATHOFTEAMMATE, // You caused the death of a teammate.
    STAT_DIDNTPROTECTTECH,    // You failed to protect Chell, your technician.
    STAT_DIDNTPROTECT7OF9,    // You failed to protect 7 of 9
    STAT_NOTSTEALTHYENOUGH,   // You weren't quite stealthy enough
    STAT_STEALTHTACTICSNECESSARY, // Starfleet will not tolerate such insubordination
    STAT_WATCHYOURSTEP,       // Watch your step
    STAT_JUDGEMENTMUCHDESIRED, // Your judgement leaves much to be desired
    MAX_STATUSTEXT,
}
