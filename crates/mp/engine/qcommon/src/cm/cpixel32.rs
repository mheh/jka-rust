#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_long;

use native_types::byte;

/// Raven `CPixel32` — one RGBA pixel of a 32-bit-per-pixel image buffer.
///
/// Type definition source: `oracle/codemp/qcommon/cm_draw.h:42-55`
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct CPixel32 {
    pub r: byte,
    pub g: byte,
    pub b: byte,
    pub a: byte,
}

impl CPixel32 {
    /// Raven's constructor defaults the components to `R = 0`, `G = 0`, `B = 0`,
    /// and `A = 255`.
    /// Rust has no default arguments, so each call site states all four values.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:50`
    pub fn new(R: byte, G: byte, B: byte, A: byte) -> Self {
        CPixel32 {
            r: R,
            g: G,
            b: B,
            a: A,
        }
    }
}

// Raven's second constructor `CPixel32(long l)` (`cm_draw.h:51`) has no caller
// in either tree, so it is dropped (porting-rules §20).

/// Raven `AVE_PIX` — the per-component 50 percent average of two pixels.
///
/// Source: `oracle/codemp/qcommon/cm_draw.h:63-67`
pub fn AVE_PIX(x: CPixel32, y: CPixel32) -> CPixel32 {
    CPixel32 {
        r: ((x.r as c_long + y.r as c_long) >> 1) as byte,
        g: ((x.g as c_long + y.g as c_long) >> 1) as byte,
        b: ((x.b as c_long + y.b as c_long) >> 1) as byte,
        a: ((x.a as c_long + y.a as c_long) >> 1) as byte,
    }
}

/// Raven `ALPHA_PIX` — blend `x` over `y` at the given 8.8 fixed-point weights.
///
/// Raven keeps the destination alpha (`t.a = y.a`); the commented-out blended
/// alpha stays commented out.
/// Source: `oracle/codemp/qcommon/cm_draw.h:69-74`
pub fn ALPHA_PIX(x: CPixel32, y: CPixel32, alpha: c_long, inv_alpha: c_long) -> CPixel32 {
    CPixel32 {
        r: ((x.r as c_long * alpha + y.r as c_long * inv_alpha) >> 8) as byte,
        g: ((x.g as c_long * alpha + y.g as c_long * inv_alpha) >> 8) as byte,
        b: ((x.b as c_long * alpha + y.b as c_long * inv_alpha) >> 8) as byte,
        a: y.a,
    }
}

/// Raven `LIGHT_PIX` — brighten or darken a pixel by a signed 10-bit gain.
///
/// Source: `oracle/codemp/qcommon/cm_draw.h:76-81`
pub fn LIGHT_PIX(p: CPixel32, light: c_long) -> CPixel32 {
    CPixel32 {
        r: CLAMP_255((p.r as c_long * light >> 10) + p.r as c_long) as byte,
        g: CLAMP_255((p.g as c_long * light >> 10) + p.g as c_long) as byte,
        b: CLAMP_255((p.b as c_long * light >> 10) + p.b as c_long) as byte,
        a: p.a,
    }
}

/// Raven `CLAMP(v, 0, 255)` — the one instantiation the pixel helpers use.
///
/// Source: `oracle/codemp/qcommon/cm_draw.h:29`
fn CLAMP_255(v: c_long) -> c_long {
    if v < 0 {
        0
    } else if v > 255 {
        255
    } else {
        v
    }
}

const _: () = {
    assert!(core::mem::size_of::<CPixel32>() == 4);
    assert!(core::mem::offset_of!(CPixel32, r) == 0);
    assert!(core::mem::offset_of!(CPixel32, g) == 1);
    assert!(core::mem::offset_of!(CPixel32, b) == 2);
    assert!(core::mem::offset_of!(CPixel32, a) == 3);
};
