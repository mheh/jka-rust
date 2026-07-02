#![allow(non_camel_case_types, non_snake_case)]

/// Raven `vehicleType_t` — vehicle type enumeration.
///
/// Raven: Types of vehicles.
/// Type definition source: `oracle/oracle/code/game/G_Vehicles.h:7-16`
#[repr(i32)]
pub enum vehicleType_t {
    VH_NONE = 0,
    /// Something you ride inside of, it walks like you, like an AT-ST
    VH_WALKER = 1,
    /// Something you fly inside of, like an X-Wing or TIE fighter
    VH_FIGHTER = 2,
    /// Something you ride on that hovers, like a speeder or swoop
    VH_SPEEDER = 3,
    /// Animal you ride on top of that walks, like a tauntaun
    VH_ANIMAL = 4,
    /// Animal you ride on top of that flies, like a giant mynoc?
    VH_FLIER = 5,
    VH_NUM_VEHICLES = 6,
}
