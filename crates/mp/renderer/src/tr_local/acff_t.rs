#![allow(non_camel_case_types, non_snake_case)]

/// Raven `acff_t` — alpha combine function format.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:272-277`
// Fieldless C enum stored by value in `ShaderStage::adjust_colors_for_fog`;
// the derives are layout-neutral.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum acff_t {
    ACFF_NONE,
    ACFF_MODULATE_RGB,
    ACFF_MODULATE_RGBA,
    ACFF_MODULATE_ALPHA,
}
