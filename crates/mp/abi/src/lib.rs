//! `mp_abi` — Multiplayer module ABI surfaces
//! (`codemp` scope): typed syscall/vmMain tokens.

#![allow(non_camel_case_types, non_snake_case)]

pub mod cgame;
pub mod game;
pub mod ui;

// The four seam traits, re-exported at the seam crate (STATE-Q12 resolution,
// round-7 item 24): module logic crates import them from `mp_abi`, keeping
// their frozen dep sets — no direct `abi_transport` edge is added to them.
pub use abi_transport::generic::{Dispatch, Execute, InboundVmCall, OutboundSysCall};
