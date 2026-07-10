#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    clippy::too_many_arguments
)]

//! `cm_randomterrain.cpp` — the RMG random-terrain support math (Perlin-style
//! noise lookups, the Ken Shoemake `lincrv.c` spline family) and the
//! consonant/vowel piece-table name generator (`RMG_CreateSeed`).
//!
//! Source: `oracle/codemp/qcommon/cm_randomterrain.cpp`
//!
//! PORT-NOTE(cm-fields): `CollisionWorld` (`crate::collision_world`) is still
//! a `//TODO: Port CollisionWorld fields` placeholder (`_private: ()`), same
//! precedent as `cm_polylib.rs`/`cm_load.rs`. Bodies below reach
//! `cm.noise_table`/`cm.noise_perm` (the file-scope `noiseTable`/`noisePerm`
//! arrays) as missing symbols, reported for the finisher to add once the
//! struct lands.
//!
//! PORT-NOTE(qrand-field): the `common: &mut Common` receiver's `QRand` field
//! (ruling 21) has not landed on `Common` yet — the field name is pinned
//! when the type lands, per `_PREAMBLE.md`. Referenced here as
//! `common.qrand.irand(...)`, reported as a missing symbol.

use core::ffi::{c_char, c_int, c_uint};

use native_math::vector::{vec2_t, vec4_t, vec_t};

use crate::cm::ecptype::ECPType;
use crate::cm::cm_randomterrain_cpp_consts::NOISE_SIZE;
use crate::collision_world::CollisionWorld;
use crate::common::Common;

/// Raven `VAL`/`INDEX` function-like macros (`cm_randomterrain.cpp:29-31`),
/// expanded inline rather than re-declared as consts (§ macros are not
/// consts).
///
/// `NOISE_MASK` = `NOISE_SIZE - 1`.
const NOISE_MASK: c_int = NOISE_SIZE as c_int - 1;

/// `TCharacterPiece` (`oracle/codemp/qcommon/cm_randomterrain.cpp:840-845`).
/// Internal, file-local to the name-generator tables; not a rosetta-imported
/// type (absent from every packet's TYPE ROSETTA table).
struct TCharacterPiece {
    mPiece: &'static str,
    mCommonality: c_int,
}

/// `Consonants[]` — `oracle/codemp/qcommon/cm_randomterrain.cpp:847-869`.
const CONSONANTS: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: "b", mCommonality: 6 },
    TCharacterPiece { mPiece: "c", mCommonality: 8 },
    TCharacterPiece { mPiece: "d", mCommonality: 6 },
    TCharacterPiece { mPiece: "f", mCommonality: 5 },
    TCharacterPiece { mPiece: "g", mCommonality: 4 },
    TCharacterPiece { mPiece: "h", mCommonality: 5 },
    TCharacterPiece { mPiece: "j", mCommonality: 2 },
    TCharacterPiece { mPiece: "k", mCommonality: 4 },
    TCharacterPiece { mPiece: "l", mCommonality: 4 },
    TCharacterPiece { mPiece: "m", mCommonality: 7 },
    TCharacterPiece { mPiece: "n", mCommonality: 7 },
    TCharacterPiece { mPiece: "r", mCommonality: 6 },
    TCharacterPiece { mPiece: "s", mCommonality: 10 },
    TCharacterPiece { mPiece: "t", mCommonality: 10 },
    TCharacterPiece { mPiece: "v", mCommonality: 1 },
    TCharacterPiece { mPiece: "w", mCommonality: 2 },
    TCharacterPiece { mPiece: "x", mCommonality: 1 },
    TCharacterPiece { mPiece: "z", mCommonality: 1 },
];

/// `ComplexConsonants[]` — `oracle/codemp/qcommon/cm_randomterrain.cpp:871-896`.
const COMPLEX_CONSONANTS: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: "st", mCommonality: 10 },
    TCharacterPiece { mPiece: "ck", mCommonality: 10 },
    TCharacterPiece { mPiece: "ss", mCommonality: 10 },
    TCharacterPiece { mPiece: "tt", mCommonality: 7 },
    TCharacterPiece { mPiece: "ll", mCommonality: 8 },
    TCharacterPiece { mPiece: "nd", mCommonality: 10 },
    TCharacterPiece { mPiece: "rn", mCommonality: 6 },
    TCharacterPiece { mPiece: "nc", mCommonality: 6 },
    TCharacterPiece { mPiece: "mp", mCommonality: 4 },
    TCharacterPiece { mPiece: "sc", mCommonality: 10 },
    TCharacterPiece { mPiece: "sl", mCommonality: 10 },
    TCharacterPiece { mPiece: "tch", mCommonality: 6 },
    TCharacterPiece { mPiece: "th", mCommonality: 4 },
    TCharacterPiece { mPiece: "rn", mCommonality: 5 },
    TCharacterPiece { mPiece: "cl", mCommonality: 10 },
    TCharacterPiece { mPiece: "sp", mCommonality: 10 },
    TCharacterPiece { mPiece: "st", mCommonality: 10 },
    TCharacterPiece { mPiece: "fl", mCommonality: 4 },
    TCharacterPiece { mPiece: "sh", mCommonality: 7 },
    TCharacterPiece { mPiece: "ng", mCommonality: 4 },
];

/// `Vowels[]` — `oracle/codemp/qcommon/cm_randomterrain.cpp:898-908`.
const VOWELS: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: "a", mCommonality: 10 },
    TCharacterPiece { mPiece: "e", mCommonality: 10 },
    TCharacterPiece { mPiece: "i", mCommonality: 10 },
    TCharacterPiece { mPiece: "o", mCommonality: 10 },
    TCharacterPiece { mPiece: "u", mCommonality: 2 },
];

/// `ComplexVowels[]` — `oracle/codemp/qcommon/cm_randomterrain.cpp:910-927`.
const COMPLEX_VOWELS: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: "ea", mCommonality: 10 },
    TCharacterPiece { mPiece: "ue", mCommonality: 3 },
    TCharacterPiece { mPiece: "oi", mCommonality: 10 },
    TCharacterPiece { mPiece: "ai", mCommonality: 8 },
    TCharacterPiece { mPiece: "oo", mCommonality: 10 },
    TCharacterPiece { mPiece: "io", mCommonality: 10 },
    TCharacterPiece { mPiece: "oe", mCommonality: 10 },
    TCharacterPiece { mPiece: "au", mCommonality: 3 },
    TCharacterPiece { mPiece: "ee", mCommonality: 7 },
    TCharacterPiece { mPiece: "ei", mCommonality: 7 },
    TCharacterPiece { mPiece: "ou", mCommonality: 7 },
    TCharacterPiece { mPiece: "ia", mCommonality: 4 },
];

/// `Endings[]` — `oracle/codemp/qcommon/cm_randomterrain.cpp:929-958`.
const ENDINGS: &[TCharacterPiece] = &[
    TCharacterPiece { mPiece: "ing", mCommonality: 10 },
    TCharacterPiece { mPiece: "ed", mCommonality: 10 },
    TCharacterPiece { mPiece: "ute", mCommonality: 10 },
    TCharacterPiece { mPiece: "ance", mCommonality: 10 },
    TCharacterPiece { mPiece: "ey", mCommonality: 10 },
    TCharacterPiece { mPiece: "ation", mCommonality: 10 },
    TCharacterPiece { mPiece: "ous", mCommonality: 10 },
    TCharacterPiece { mPiece: "ent", mCommonality: 10 },
    TCharacterPiece { mPiece: "ate", mCommonality: 10 },
    TCharacterPiece { mPiece: "ible", mCommonality: 10 },
    TCharacterPiece { mPiece: "age", mCommonality: 10 },
    TCharacterPiece { mPiece: "ity", mCommonality: 10 },
    TCharacterPiece { mPiece: "ist", mCommonality: 10 },
    TCharacterPiece { mPiece: "ism", mCommonality: 10 },
    TCharacterPiece { mPiece: "ime", mCommonality: 10 },
    TCharacterPiece { mPiece: "ic", mCommonality: 10 },
    TCharacterPiece { mPiece: "ant", mCommonality: 10 },
    TCharacterPiece { mPiece: "etry", mCommonality: 10 },
    TCharacterPiece { mPiece: "ious", mCommonality: 10 },
    TCharacterPiece { mPiece: "ative", mCommonality: 10 },
    TCharacterPiece { mPiece: "er", mCommonality: 10 },
    TCharacterPiece { mPiece: "ize", mCommonality: 10 },
    TCharacterPiece { mPiece: "able", mCommonality: 10 },
    TCharacterPiece { mPiece: "itude", mCommonality: 10 },
];

/// Raven `GetNoiseValue`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:35-40`
pub fn GetNoiseValue(cm: &mut CollisionWorld, x: c_int, y: c_int, z: c_int, t: c_int) -> f32 {
    // `VAL(a) = noisePerm[a & NOISE_MASK]`; `INDEX(x,y,z,t) = VAL(x + VAL(y + VAL(z + VAL(t))))`.
    // PORT-NOTE(missing-field): `noiseTable`/`noisePerm` (`cm_randomterrain.cpp:14-15`)
    // are file-scope globals with no home field on `CollisionWorld` yet;
    // referenced as `cm.noise_perm`/`cm.noise_table`, reported as missing symbols.
    let val_t = cm.noise_perm[(t as usize) & (NOISE_MASK as usize)];
    let val_z = cm.noise_perm[((z + val_t) as usize) & (NOISE_MASK as usize)];
    let val_y = cm.noise_perm[((y + val_z) as usize) & (NOISE_MASK as usize)];
    let index = cm.noise_perm[((x + val_y) as usize) & (NOISE_MASK as usize)];

    cm.noise_table[index as usize]
}

/// Raven `CM_NoiseGet4f`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:51-89`
pub fn CM_NoiseGet4f(cm: &mut CollisionWorld, x: f32, y: f32, z: f32, t: f32) -> f32 {
    // `LERP(a, b, w) = a * (1.0f - w) + b * w` (function-like macro, expanded inline).
    let ix = x.floor() as c_int;
    let fx = x - ix as f32;
    let iy = y.floor() as c_int;
    let fy = y - iy as f32;
    let iz = z.floor() as c_int;
    let fz = z - iz as f32;
    let it = t.floor() as c_int;
    let ft = t - it as f32;

    let mut value = [0.0f32; 2];

    for i in 0..2 {
        let front = [
            GetNoiseValue(cm, ix, iy, iz, it + i),
            GetNoiseValue(cm, ix + 1, iy, iz, it + i),
            GetNoiseValue(cm, ix, iy + 1, iz, it + i),
            GetNoiseValue(cm, ix + 1, iy + 1, iz, it + i),
        ];
        let back = [
            GetNoiseValue(cm, ix, iy, iz + 1, it + i),
            GetNoiseValue(cm, ix + 1, iy, iz + 1, it + i),
            GetNoiseValue(cm, ix, iy + 1, iz + 1, it + i),
            GetNoiseValue(cm, ix + 1, iy + 1, iz + 1, it + i),
        ];

        let fvalue = lerp_macro(lerp_macro(front[0], front[1], fx), lerp_macro(front[2], front[3], fx), fy);
        let bvalue = lerp_macro(lerp_macro(back[0], back[1], fx), lerp_macro(back[2], back[3], fx), fy);

        value[i as usize] = lerp_macro(fvalue, bvalue, fz);
    }

    lerp_macro(value[0], value[1], ft)
}

/// Raven `LERP(a, b, w)` function-like macro helper (`cm_randomterrain.cpp:33`),
/// factored out so `CM_NoiseGet4f` reads 1:1 against the oracle's repeated
/// `LERP(...)` expansions.
fn lerp_macro(a: f32, b: f32, w: f32) -> f32 {
    a * (1.0f32 - w) + b * w
}

/// Raven `lerp` (the Ken Shoemake `lincrv.c` spline lerp — distinct from the
/// `LERP` macro above).
///
/// PORT-NOTE(shape): the resolved signature passes `p0`/`p1`/`p` by value
/// (`vec4_t` = `[f32; 4]`, `Copy`); Raven's `vec4_t p0/p1/p` parameters are
/// array-decayed pointers the body writes THROUGH into the caller's array.
/// Passing by value here means writes to `p` are NOT observed by
/// `DialASpline`'s call sites — reported in shape_mismatches; the resolved
/// signature is LAW for this pass.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:105-110`
pub fn lerp(t: f32, a0: f32, a1: f32, p0: vec4_t, p1: vec4_t, m: c_int, mut p: vec4_t) {
    let t0 = (a1 - t) / (a1 - a0);
    let t1 = 1.0 - t0;
    let mut i = m - 1;
    while i >= 0 {
        p[i as usize] = t0 * p0[i as usize] + t1 * p1[i as usize];
        i -= 1;
    }
}

/// Raven `DialASpline`.
///
/// PORT-NOTE(shape): see `lerp`'s PORT-NOTE — the `lerp(...)` calls below
/// transcribe the oracle 1:1 (LAW), but since `lerp`'s `p` param is
/// by-value, the `work[...]`/`val` writes those calls appear to perform are
/// NOT actually observed here; reported in shape_mismatches.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:119-157`
pub fn DialASpline(
    t: f32,
    a: *mut f32,
    p: *mut vec4_t,
    m: c_int,
    n: c_int,
    work: *mut vec4_t,
    mut Cn: c_uint,
    interp: bool,
    val: vec4_t,
) -> c_int {
    unsafe {
        if Cn as c_int > n - 1 {
            Cn = (n - 1) as c_uint;
        }
        let mut k: c_int = 0;
        while t > *a.offset(k as isize) {
            k += 1;
        }
        let mut h = k;
        while t == *a.offset(k as isize) {
            k += 1;
        }
        if k > n {
            k = n;
            if h > k {
                h = k;
            }
        }
        h = 1 + Cn as c_int - (k - h);
        k -= 1;
        let lo0 = k - Cn as c_int;
        let hi0 = k + 1 + Cn as c_int;
        let mut lo = lo0;
        let mut hi = hi0;

        if interp {
            let mut drop: c_int = 0;
            if lo < 0 {
                lo = 0;
                drop += Cn as c_int - k;
                if hi - lo < Cn as c_int {
                    drop += Cn as c_int - hi;
                    hi = Cn as c_int;
                }
            }
            if hi > n {
                hi = n;
                drop += k + 1 + Cn as c_int - n;
                if hi - lo < Cn as c_int {
                    drop += lo - (n - Cn as c_int);
                    lo = n - Cn as c_int;
                }
            }
            for i in lo..=hi {
                *work.offset(i as isize) = *p.offset(i as isize);
            }
            for j in 1..=(Cn as c_int) {
                for i in lo..=(hi - j) {
                    lerp(
                        t,
                        *a.offset(i as isize),
                        *a.offset((i + j) as isize),
                        *work.offset(i as isize),
                        *work.offset((i + 1) as isize),
                        m,
                        *work.offset(i as isize),
                    );
                }
            }
            h = 1 + Cn as c_int - drop;
        } else {
            if lo < 0 {
                h += lo;
                lo = 0;
            }
            for i in lo..=(lo + h) {
                *work.offset(i as isize) = *p.offset(i as isize);
            }
            if h < 0 {
                h = 0;
            }
        }
        for j in 0..h {
            let tmp = 1 + Cn as c_int - j;
            let mut i = h - 1;
            while i >= j {
                lerp(
                    t,
                    *a.offset((lo + i) as isize),
                    *a.offset((lo + i + tmp) as isize),
                    *work.offset((lo + i) as isize),
                    *work.offset((lo + i + 1) as isize),
                    m,
                    *work.offset((lo + i + 1) as isize),
                );
                i -= 1;
            }
        }
        // PORT-NOTE(shape): `V_Op(val,=,work[lo+h],m)` writes the result into
        // the caller's `val` array; `val` here is by-value (see this fn's
        // shape PORT-NOTE), so the copy below is not observed by callers.
        let _val_result: vec4_t = *work.offset((lo + h) as isize);
        let _ = val;

        k
    }
}

/// Raven `Vector2Normalize`.
///
/// PORT-NOTE(shape): the resolved signature takes `v: vec2_t` by value
/// (`[f32; 2]`, `Copy`) rather than a mutable reference/pointer — Raven's
/// `vec2_t v` parameter is an array-decayed pointer the body normalizes
/// in place. Writes to `v[0]`/`v[1]` here are local only; reported in
/// shape_mismatches.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:161-176`
pub fn Vector2Normalize(mut v: vec2_t) -> vec_t {
    let mut length = v[0] * v[0] + v[1] * v[1];
    length = length.sqrt();

    if length != 0.0 {
        let ilength = 1.0 / length;
        v[0] *= ilength;
        v[1] *= ilength;
    }

    length
}

/// Raven `FindPiece`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:960-1006`
pub fn FindPiece(common: &mut Common, cm: &mut CollisionWorld, r#type: ECPType, pos: &mut *mut c_char) {
    let start: &[TCharacterPiece] = match r#type {
        ECPType::CP_COMPLEX_CONSONANT => COMPLEX_CONSONANTS,
        ECPType::CP_VOWEL => VOWELS,
        ECPType::CP_COMPLEX_VOWEL => COMPLEX_VOWELS,
        ECPType::CP_ENDING => ENDINGS,
        // `CP_CONSONANT` and default.
        _ => CONSONANTS,
    };

    let mut count: c_int = 0;
    for piece in start {
        count += piece.mCommonality;
    }

    // PORT-NOTE(qrand-field): `irand` routes through the engine LCG on
    // `common`'s not-yet-landed `QRand` field (ruling 21) — reported as a
    // missing symbol; referenced exactly as the preamble resolves it.
    count = common.qrand.irand(0, count - 1);

    let mut search_idx = 0usize;
    while count > start[search_idx].mCommonality {
        count -= start[search_idx].mCommonality;
        search_idx += 1;
    }

    let piece = start[search_idx].mPiece;
    unsafe {
        for byte in piece.as_bytes() {
            **pos = *byte as c_char;
            (*pos) = (*pos).add(1);
        }
        **pos = 0;
        // Raven's `strcpy` NUL-terminates but does not itself advance `pos`
        // past the terminator; `pos += strlen(...)` leaves `pos` pointing AT
        // the (still-written) NUL, ready for the next piece's `strcpy` to
        // overwrite it — matches the loop above which stops before writing
        // the trailing NUL into the advanced position.
    }
}

/// Raven `RMG_CreateSeed`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:1008-1091`
pub fn RMG_CreateSeed(common: &mut Common, cm: &mut CollisionWorld, TextSeed: *mut c_char) -> c_uint {
    // PORT-NOTE(qrand-field): see `FindPiece` — `irand` via `common.qrand`,
    // reported as a missing symbol.
    let mut Length = common.qrand.irand(4, 9);

    let mut LookingFor = if common.qrand.irand(0, 100) < 20 {
        ECPType::CP_VOWEL
    } else {
        ECPType::CP_CONSONANT
    };

    // §19: `Ending` is a local scratch buffer Raven writes (`Ending[0] = 0`)
    // before any read; zero-init the whole buffer to keep it well-defined.
    let mut Ending: [c_char; 256] = [0; 256];
    let mut ending_len: usize = 0;

    if common.qrand.irand(0, 100) < 55 {
        let mut pos: *mut c_char = Ending.as_mut_ptr();
        FindPiece(common, cm, ECPType::CP_ENDING, &mut pos);
        ending_len = unsafe { pos.offset_from(Ending.as_mut_ptr()) as usize };
        Length -= ending_len as c_int;
    }

    unsafe {
        *TextSeed = 0;
    }
    let mut pos: *mut c_char = TextSeed;

    let mut ComplexVowelChance: c_int = -1;
    let mut ComplexConsonantChance: c_int = -1;

    while unsafe { pos.offset_from(TextSeed) } < Length as isize || matches!(LookingFor, ECPType::CP_CONSONANT) {
        if matches!(LookingFor, ECPType::CP_VOWEL) {
            if common.qrand.irand(0, 100) < ComplexVowelChance {
                ComplexVowelChance = -1;
                LookingFor = ECPType::CP_COMPLEX_VOWEL;
            } else {
                ComplexVowelChance += 10;
            }

            FindPiece(common, cm, LookingFor, &mut pos);
            LookingFor = ECPType::CP_CONSONANT;
        } else {
            if common.qrand.irand(0, 100) < ComplexConsonantChance {
                ComplexConsonantChance = -1;
                LookingFor = ECPType::CP_COMPLEX_CONSONANT;
            } else {
                ComplexConsonantChance += 45;
            }

            FindPiece(common, cm, LookingFor, &mut pos);
            LookingFor = ECPType::CP_VOWEL;
        }
    }

    if Ending[0] != 0 {
        unsafe {
            for i in 0..=ending_len {
                *pos.add(i) = Ending[i];
            }
        }
    }

    let mut pos: *const c_char = TextSeed;
    let mut SeedValue: c_uint = 0;
    unsafe {
        while *pos != 0 {
            let high = SeedValue >> 28;
            SeedValue ^= ((SeedValue << 4) as c_int + ((*pos as c_int) - ('a' as c_int))) as c_uint;
            SeedValue ^= high;
            pos = pos.add(1);
        }
    }

    SeedValue
}
