//! `native_math` — the canonical home of Raven's quake math.
//!
//! `qmath` holds the SP/MP-behaviorally-identical `q_math.c` functions (one
//! definition, Raven names); `deviations` holds the per-mode divergent pair
//! variants (`<Name>MP`/`<Name>SP`), re-exported per-mode under the plain
//! name. Membership is decided by `tools/qmath-census.py`, never by hand.
#![allow(non_camel_case_types)]

pub mod deviations;
pub mod eorientations;
pub mod orientation;
pub mod qmath;
pub mod vector;
