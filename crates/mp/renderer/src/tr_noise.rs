//! Raven `tr_noise.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_noise.cpp`

// Wave-0 ports of Raven `static` helpers: private by fidelity, with their
// callers landing in later R3 waves.
#![allow(dead_code)]
// Raven-named functions keep their original casing across this transcription.
#![allow(non_snake_case)]

use native_math::rng::{Rng, RAND_MAX};

/// Size of the noise permutation/value tables.
///
/// Raven `#define NOISE_SIZE 256`.
/// Source: `oracle/codemp/renderer/tr_noise.cpp:7`
const NOISE_SIZE: usize = 256;

/// Raven `#define NOISE_MASK (NOISE_SIZE-1)`.
/// Source: `oracle/codemp/renderer/tr_noise.cpp:8`
const NOISE_MASK: i32 = (NOISE_SIZE as i32) - 1;

/// Per-subsystem owned state for `tr_noise.cpp` (DEC-37 A13.3 — named by this wave).
///
/// Homes Raven's `s_noise_perm`/`s_noise_table` file-scope statics: init-once
/// tables filled once by `R_NoiseInit`, read by `GetNoiseValue`/`GetNoiseTime`.
/// Source: `oracle/codemp/renderer/tr_noise.cpp:13-14`
pub struct NoiseState {
    /// Raven `static int s_noise_perm[NOISE_SIZE]`.
    pub perm: [i32; NOISE_SIZE],
    /// Raven `static float s_noise_table[NOISE_SIZE]`.
    pub table: [f32; NOISE_SIZE],
}

impl Default for NoiseState {
    fn default() -> Self {
        Self {
            perm: [0; NOISE_SIZE],
            table: [0.0; NOISE_SIZE],
        }
    }
}

/// Raven `VAL(a)` macro: `s_noise_perm[(a) & (NOISE_MASK)]`.
///
// PORT-NOTE: not a named oracle function — the shared `VAL` macro body
// factored into a helper since `GetNoiseValue`/`GetNoiseTime` both use it.
/// Source: `oracle/codemp/renderer/tr_noise.cpp:10`
fn noise_val(noise: &NoiseState, a: i32) -> i32 {
    noise.perm[(a & NOISE_MASK) as usize]
}

/// Raven `INDEX(x, y, z, t)` macro: `VAL(x + VAL(y + VAL(z + VAL(t))))`.
///
/// Source: `oracle/codemp/renderer/tr_noise.cpp:11`
fn noise_index(noise: &NoiseState, x: i32, y: i32, z: i32, t: i32) -> i32 {
    noise_val(
        noise,
        x + noise_val(noise, y + noise_val(noise, z + noise_val(noise, t))),
    )
}

/// Raven `GetNoiseValue`.
///
/// Source: `oracle/codemp/renderer/tr_noise.cpp:18-23`
fn get_noise_value(noise: &NoiseState, x: i32, y: i32, z: i32, t: i32) -> f32 {
    let index = noise_index(noise, x, y, z, t);
    noise.table[index as usize]
}

/// Raven `GetNoiseTime`.
///
/// Source: `oracle/codemp/renderer/tr_noise.cpp:25-30`
pub fn get_noise_time(noise: &NoiseState, t: i32) -> f32 {
    let index = noise_val(noise, t);
    1.0 + noise.table[index as usize]
}

/// Raven `R_NoiseInit`.
///
/// Raven's `srand`/`rand`/`RAND_MAX` are the C runtime's (msvcrt on retail),
/// not `q_math.c`'s `holdrand` generator; `Rng::srand`/`Rng::rand` are the
/// exact msvcrt replica and `Rng` is threaded rather than reached
/// (porting-rules §B4, `tr_WorldEffects`'s `WE_flrand` precedent).
///
/// Source: `oracle/codemp/renderer/tr_noise.cpp:32-43`
pub fn R_NoiseInit(noise: &mut NoiseState, rng: &mut Rng) {
    rng.srand(1001);

    for i in 0..NOISE_SIZE {
        // C promotes to double; f64 intermediate per wave-0 ruling 12.
        noise.table[i] = ((rng.rand() as f32 / RAND_MAX as f32) as f64 * 2.0 - 1.0) as f32;
        noise.perm[i] = ((rng.rand() as f32 / RAND_MAX as f32 * 255.0) as u8) as i32;
    }
}
