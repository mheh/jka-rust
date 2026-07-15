//! Referee diagnostic probes (`REF_PROBES`).
//!
//! Env-gated taps that dump float-bit-exact state at hand-picked sites so the
//! Rust module and the oracle build's matching C taps
//! (`tools/referee-oracle/build.sh`) emit byte-identical lines the lockstep
//! referee diffs. Enable by setting the `REF_PROBES` environment variable.

use std::sync::LazyLock;

/// Cached `REF_PROBES` env flag; the referee probes emit only when it is set.
pub static REF_PROBES_ENABLED: LazyLock<bool> =
    LazyLock::new(|| std::env::var_os("REF_PROBES").is_some());

/// `probe!("TAG", "fmt", args…)` — emit one `TAG …` referee diagnostic line on
/// stderr (matching the oracle taps' `fprintf(stderr, …)`) when `REF_PROBES` is set.
#[macro_export]
macro_rules! probe {
    ($tag:literal, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if *$crate::probes::REF_PROBES_ENABLED {
            eprintln!(concat!($tag, " ", $fmt) $(, $arg)*);
        }
    };
}
