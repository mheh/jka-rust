//! `q_shared.c` byte-order family — the shared-tier home for engine-island
//! callers (`mp_game` carries its own module-island copies in `q_shared.rs`).
//!
//! The `Short/Long/Float(No)Swap` primitives are the endianness mechanism; the
//! `Little*`/`Big*` wrappers select per target. This build targets little-endian
//! (WIN32 `q_shared.h:169-176`: `#define LittleShort/LittleLong/LittleFloat`
//! erase to identity; `BigShort/BigLong/BigFloat` are real swaps).

use core::ffi::{c_int, c_short};

use crate::shared::int64::qint64;

/// Raven `ShortSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:162-170`
pub fn ShortSwap(l: c_short) -> c_short {
    let b1 = (l & 255) as u16;
    let b2 = ((l >> 8) & 255) as u16;
    ((b1 << 8) + b2) as c_short
}

/// Raven `ShortNoSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:172-175`
pub fn ShortNoSwap(l: c_short) -> c_short {
    l
}

/// Raven `LongSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:177-187`
pub fn LongSwap(l: c_int) -> c_int {
    let b1 = (l & 255) as u32;
    let b2 = ((l >> 8) & 255) as u32;
    let b3 = ((l >> 16) & 255) as u32;
    let b4 = ((l >> 24) & 255) as u32;
    ((b1 << 24) + (b2 << 16) + (b3 << 8) + b4) as c_int
}

/// Raven `LongNoSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:189-192`
pub fn LongNoSwap(l: c_int) -> c_int {
    l
}

/// Raven `Long64Swap`.
///
/// Source: `oracle/codemp/game/q_shared.c:194-208`
pub fn Long64Swap(ll: qint64) -> qint64 {
    qint64 {
        b0: ll.b7,
        b1: ll.b6,
        b2: ll.b5,
        b3: ll.b4,
        b4: ll.b3,
        b5: ll.b2,
        b6: ll.b1,
        b7: ll.b0,
    }
}

/// Raven `Long64NoSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:210-213`
pub fn Long64NoSwap(ll: qint64) -> qint64 {
    ll
}

/// Raven `FloatSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:220-228`
pub fn FloatSwap(f: *const f32) -> f32 {
    unsafe {
        let i = (*f).to_bits() as c_int;
        f32::from_bits(LongSwap(i) as u32)
    }
}

/// Raven `FloatNoSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:230-233`
pub fn FloatNoSwap(f: *const f32) -> f32 {
    unsafe { *f }
}

// This build is little-endian (WIN32): `Little*` erase to identity, `Big*` swap.
// Source: `oracle/codemp/game/q_shared.h:169-176`

/// Raven `LittleShort` (WIN32 identity macro).
///
/// Source: `oracle/codemp/game/q_shared.h:172`
pub fn LittleShort(l: c_short) -> c_short {
    l
}

/// Raven `LittleLong` (WIN32 identity macro).
///
/// Source: `oracle/codemp/game/q_shared.h:174`
pub fn LittleLong(l: c_int) -> c_int {
    l
}

/// Raven `LittleFloat` (WIN32 identity macro).
///
/// Source: `oracle/codemp/game/q_shared.h:176`
pub fn LittleFloat(l: f32) -> f32 {
    l
}

/// Raven `BigShort` (WIN32: `ShortSwap`).
///
/// Source: `oracle/codemp/game/q_shared.h:171`
pub fn BigShort(l: c_short) -> c_short {
    ShortSwap(l)
}

/// Raven `BigLong` (WIN32: `LongSwap`).
///
/// Source: `oracle/codemp/game/q_shared.h:173`
pub fn BigLong(l: c_int) -> c_int {
    LongSwap(l)
}

/// Raven `BigFloat` (WIN32: `FloatSwap`).
///
/// Source: `oracle/codemp/game/q_shared.h:175`
// Raven's WIN32 inline lacks a return statement (UB); returns the swapped value.
pub fn BigFloat(l: *const f32) -> f32 {
    FloatSwap(l)
}
