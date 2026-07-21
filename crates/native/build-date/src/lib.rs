//! Build-script date helpers shared by the engine and game `build.rs` scripts.
//! Both need C's `__DATE__` format for their `BUILD_DATE` env emit (stable Rust
//! has no `env!`-visible builtin), so the two identical copies live here once.

use std::time::{SystemTime, UNIX_EPOCH};

/// Renders "today" (build-script run time) in C's `__DATE__` format:
/// `Mmm dd yyyy`, day space-padded for single digits (e.g. `"Jul  2 2026"`).
pub fn build_date() -> String {
    const MONTH_NAMES: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs();
    let days = (secs / 86400) as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{} {:2} {}", MONTH_NAMES[(month - 1) as usize], day, year)
}

/// Days-since-Unix-epoch -> (year, month, day). Howard Hinnant's
/// `civil_from_days` algorithm (public-domain, `chrono`-independent — no new
/// build-dependency needed for one date computation).
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
