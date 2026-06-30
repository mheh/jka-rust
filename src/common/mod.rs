//! Shared Raven-derived implementation strata.
//!
//! `mp` and `sp` hold product-line-local common code. Their `qcommon` modules
//! mirror Raven's separate executable common trees until comparison proves a
//! narrower piece can be intentionally unified.

pub mod mp;
pub mod sp;
