//! MP `bg_public.h` broken limb type definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:172-178`

#![allow(non_camel_case_types)]

/// Raven `brokenLimb_t`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:172-178`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum brokenLimb_t {
    BROKENLIMB_NONE = 0,
    BROKENLIMB_LARM = 1,
    BROKENLIMB_RARM = 2,
    NUM_BROKENLIMBS = 3,
}