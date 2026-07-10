#![allow(non_camel_case_types, non_snake_case)]

/// Raven `vmInterpret_t` — VM interpretation modes.
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:275-279`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum vmInterpret_t {
    VMI_NATIVE = 0,
    VMI_BYTECODE = 1,
    VMI_COMPILED = 2,
}
