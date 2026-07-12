//! `timing` types.

pub mod timing_c;

use crate::common::Common;

/// Raven `Sys_Milliseconds` — the engine time base. Timing is NOT a host
/// service: `Common` owns the `Instant` base (`time_base`), so the
/// base-relative clock read lives here as the one implementation
/// (`mp_engine_core`'s `sys_milliseconds` delegates to it): `now − base` as
/// `u64` ms truncated `as i32`, reproducing `timeGetTime`'s practical 49.7-day
/// wrap. The `base_time=true` absolute variant (Rand_Init seed) is a separate
/// SystemTime read kept in `mp_engine_core::lifecycle`.
///
/// Source: `oracle/codemp/win32/win_shared.cpp:22-34`
pub fn sys_milliseconds(common: &Common) -> i32 {
    common.time_base.elapsed().as_millis() as u64 as i32
}
