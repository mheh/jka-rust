#![allow(non_camel_case_types, non_snake_case)]

/// Raven `cullType_t` — face culling modes.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:436-440`
#[repr(i32)]
pub enum cullType_t {
	CT_FRONT_SIDED,
	CT_BACK_SIDED,
	CT_TWO_SIDED,
}
