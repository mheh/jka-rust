#![allow(non_camel_case_types, non_snake_case)]

/// Raven `EWeaponPose` — weapon pose state for vehicles.
///
/// Type definition source: `oracle/code/game/G_Vehicles.h:18-24`
#[repr(i32)]
pub enum EWeaponPose {
    WPOSE_NONE = 0,
    WPOSE_BLASTER = 1,
    WPOSE_SABERLEFT = 2,
    WPOSE_SABERRIGHT = 3,
}
