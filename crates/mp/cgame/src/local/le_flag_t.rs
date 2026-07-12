#![allow(non_camel_case_types, non_snake_case)]

/// Raven `leFlag_t` — local entity flags.
///
/// Raven: do not scale size over time; tumble over time, used for ejecting shells;
/// explicitly fade; MakeExplosion adds random rotate which could be bad in some cases.
/// Type definition source: `oracle/codemp/cgame/cg_local.h:498-503`
#[repr(i32)]
pub enum leFlag_t {
    LEF_PUFF_DONT_SCALE = 0x0001,
    LEF_TUMBLE = 0x0002,
    LEF_FADE_RGB = 0x0004,
    LEF_NO_RANDOM_ROTATE = 0x0008,
}
