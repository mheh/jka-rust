#![allow(non_camel_case_types, non_snake_case)]

/// Raven `siegeClassFlags_t` — class feature flags.
///
/// Type definition source: `oracle/codemp/game/bg_saga.h:31-41`
#[repr(i32)]
pub enum siegeClassFlags_t {
    CFL_MORESABERDMG = 0,
    CFL_STRONGAGAINSTPHYSICAL,
    CFL_FASTFORCEREGEN,
    CFL_STATVIEWER,
    CFL_HEAVYMELEE,
    CFL_SINGLE_ROCKET,
    CFL_CUSTOMSKEL,
    CFL_EXTRA_AMMO,
}
