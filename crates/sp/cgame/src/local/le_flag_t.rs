#![allow(non_camel_case_types, non_snake_case)]

/// Raven `leFlag_t` — local entity flags.
///
/// Type definition source: `oracle/oracle/code/cgame/cg_local.h:208-213`
#[repr(i32)]
pub enum leFlag_t {
    LEF_PUFF_DONT_SCALE = 0x0001,
    LEF_TUMBLE = 0x0002,
    LEF_FADE_RGB = 0x0004,
    LEF_NO_RANDOM_ROTATE = 0x0008,
}
