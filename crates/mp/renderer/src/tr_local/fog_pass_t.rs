#![allow(non_camel_case_types, non_snake_case)]

/// Raven `fogPass_t` — fog rendering passes.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:442-447`
#[repr(i32)]
pub enum fogPass_t {
    FP_NONE,  // surface is translucent and will just be adjusted properly
    FP_EQUAL, // surface is opaque but possibly alpha tested
    FP_LE,    // surface is trnaslucent, but still needs a fog pass (fog surface)
    FP_GLFOG,
}
