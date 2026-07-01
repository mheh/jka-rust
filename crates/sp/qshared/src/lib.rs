//! `sp_qshared` — SP `q_shared.h`/`qcommon` substrate (Tier 0). Mirror of
//! `mp_qshared` for the SP tree; the shared structs are duplicated per-mode
//! (SP vs MP `q_shared.h` diverge) and will diverge as they are ported.

#![allow(non_camel_case_types, non_snake_case)]

pub mod common;
pub mod shared;
