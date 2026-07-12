//! SP `tr_types.h` texture-compression mode.

#![allow(non_camel_case_types)]

/// Raven `textureCompression_t` — GL texture-compression capability reported in `glconfig_t`.
///
/// Type definition source: `oracle/code/renderer/tr_types.h:193-197`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum textureCompression_t {
    TC_NONE = 0,
    TC_S3TC = 1,
    TC_S3TC_DXT = 2,
}
