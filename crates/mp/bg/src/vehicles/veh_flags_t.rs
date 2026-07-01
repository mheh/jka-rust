#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `vehFlags_t` — vehicle behavior flags.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_vehicles.h:417-424`
#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum vehFlags_t {
    /// No flags set.
    VEH_NONE = 0,
    /// Vehicle is flying.
    VEH_FLYING = 0x00000001,
    /// Vehicle is crashing.
    VEH_CRASHING = 0x00000002,
    /// Vehicle is landing.
    VEH_LANDING = 0x00000004,
    /// Vehicle is bucking.
    VEH_BUCKING = 0x00000010,
    /// Vehicle wings are open.
    VEH_WINGSOPEN = 0x00000020,
    /// Vehicle gears are open.
    VEH_GEARSOPEN = 0x00000040,
    /// Vehicle is slide braking.
    VEH_SLIDEBREAKING = 0x00000080,
    /// Vehicle is spinning.
    VEH_SPINNING = 0x00000100,
    /// Vehicle is out of control.
    VEH_OUTOFCONTROL = 0x00000200,
    /// Saber in left hand.
    VEH_SABERINLEFTHAND = 0x00000400,
}
