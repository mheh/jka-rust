#![allow(non_camel_case_types, non_snake_case)]

/// Raven `EGLFogOverride` — fog override modes.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:279-285`
// Fieldless C enum stored by value in `ShaderStage::gl_fog_color_override`;
// the derives are layout-neutral.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum EGLFogOverride {
    GLFOGOVERRIDE_NONE = 0,
    GLFOGOVERRIDE_BLACK,
    GLFOGOVERRIDE_WHITE,
    GLFOGOVERRIDE_MAX,
}
