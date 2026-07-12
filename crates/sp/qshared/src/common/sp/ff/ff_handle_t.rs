#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `ffHandle_t` — a force-feedback effect handle, `#ifdef _IMMERSION`
/// only. SP-only: the force-feedback subsystem (`code/ff/`) has no MP
/// counterpart.
///
/// Type definition source: `oracle/code/ff/ff_public.h:8`
pub type ffHandle_t = c_int;
