#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `EWeaponPose` — weapon pose enumeration for vehicles.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_vehicles.h:20-26`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EWeaponPose {
    /// No weapon pose.
    WPOSE_NONE = 0,
    /// Blaster weapon pose.
    WPOSE_BLASTER,
    /// Saber in left hand pose.
    WPOSE_SABERLEFT,
    /// Saber in right hand pose.
    WPOSE_SABERRIGHT,
}
