#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `vehicleType_t` — vehicle type enumeration.
///
/// Raven: Vehicle type for different vehicle behaviors.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_vehicles.h:9-18`
#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum vehicleType_t {
    /// No vehicle.
    VH_NONE = 0,
    /// Something you ride inside of; it walks like you, like an AT-ST.
    VH_WALKER,
    /// Something you fly inside of; like an X-Wing or TIE fighter.
    VH_FIGHTER,
    /// Something you ride on that hovers; like a speeder or swoop.
    VH_SPEEDER,
    /// Animal you ride on top of that walks; like a tauntaun.
    VH_ANIMAL,
    /// Animal you ride on top of that flies; like a giant mynoc.
    VH_FLIER,
    /// Number of vehicle types (sentinel value).
    VH_NUM_VEHICLES,
}
