//! `native_sort` — the canonical home of the C sorting runtime (DEC-32
//! homogenization): one deterministic `qsort` implementation, safe and
//! generic, replacing the per-crate pointer-based copies. Like
//! `native_string`, consumers convert at their own seam sites.
#![forbid(unsafe_code)]

pub mod qsort;
