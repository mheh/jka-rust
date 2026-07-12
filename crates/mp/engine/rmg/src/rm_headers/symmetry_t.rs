#![allow(non_camel_case_types)]

/// Raven `symmetry_t` — which corner holds the first node on a symmetric map.
///
/// Raven: on a symmetric map which corner is the first node.
/// Type definition source: `oracle/codemp/RMG/RM_Headers.h:29-35`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum symmetry_t {
    SYMMETRY_NONE,
    SYMMETRY_TOPLEFT,
    SYMMETRY_BOTTOMRIGHT,
}

const _: () = assert!(core::mem::size_of::<symmetry_t>() == 4);
