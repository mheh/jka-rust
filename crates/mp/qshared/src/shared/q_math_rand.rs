//! MP `q_math.c` VC-libc-compatible LCG random helpers.
//!
//! //TODO: Port holdrand, Rand_Init, flrand, Q_flrand, irand
//! // Source: `oracle/oracle/codemp/game/q_math.c:1432-1469`
//! The 32-bit LCG (`holdrand = holdrand * 214013 + 2531011`) and its
//! `flrand`/`Q_flrand`/`irand` wrappers are not ported yet; this constant is
//! placed here so it's beside the LCG when that lands.

use core::ffi::c_int;

/// Raven's `flrand`/`irand` divide the LCG's top 15 bits (`holdrand >> 17`)
/// by `32768.0F` (`0x8000`), i.e. treat the output range as `0..0x7fff`
/// inclusive — the VC libc `RAND_MAX` convention. No `#define RAND_MAX`
/// appears in `q_math.c` itself; this names that implicit constant.
/// Source: `oracle/oracle/codemp/game/q_math.c:1445-1446,1465-1466`
pub const RAND_MAX: c_int = 0x7fff;
