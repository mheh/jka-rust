//! Shared Raven-derived implementation strata.
//!
//! `qcommon` is for code proven shared or intentionally unified across SP and MP.
//! `mp` and `sp` hold product-line-local common code until comparison proves a
//! narrower piece can move upward.

pub mod mp;
pub mod qcommon;
pub mod sp;
