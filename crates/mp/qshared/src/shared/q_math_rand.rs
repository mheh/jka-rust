//! MP `q_math.c` VC-libc-compatible LCG random helpers.
//!
//! The 32-bit LCG itself (`holdrand`, `Rand_Init`, `flrand`, `Q_flrand`,
//! `irand`, `Q_irand`) lives on the game tier as `bg_channel::rng::Rng` in
//! `crates/mp/game/src/bg_channel/rng.rs` (the generator is stateful, so it is
//! an owned threaded object in `BgState`, not a qshared free function). Only
//! this stateless `RAND_MAX` constant stays here.
//! Source: `oracle/oracle/codemp/game/q_math.c:1432-1469`

use core::ffi::c_int;

/// Raven's `flrand`/`irand` divide the LCG's top 15 bits (`holdrand >> 17`)
/// by `32768.0F` (`0x8000`), i.e. treat the output range as `0..0x7fff`
/// inclusive — the VC libc `RAND_MAX` convention. No `#define RAND_MAX`
/// appears in `q_math.c` itself; this names that implicit constant.
/// Source: `oracle/oracle/codemp/game/q_math.c:1445-1446,1465-1466`
pub const RAND_MAX: c_int = 0x7fff;
