#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec4_t;

/// Raven `colorRangeDef_t` — a color range definition.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:148-152`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct colorRangeDef_t {
    pub color: vec4_t,
    pub low: f32,
    pub high: f32,
}

const _: () = assert!(core::mem::size_of::<colorRangeDef_t>() == 24);
const _: () = assert!(core::mem::offset_of!(colorRangeDef_t, color) == 0);
const _: () = assert!(core::mem::offset_of!(colorRangeDef_t, low) == 16);
const _: () = assert!(core::mem::offset_of!(colorRangeDef_t, high) == 20);
