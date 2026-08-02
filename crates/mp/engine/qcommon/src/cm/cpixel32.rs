#![allow(non_camel_case_types, non_snake_case)]

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

const _: () = {
    assert!(core::mem::size_of::<CPixel32>() == 4);
    assert!(core::mem::offset_of!(CPixel32, r) == 0);
    assert!(core::mem::offset_of!(CPixel32, g) == 1);
    assert!(core::mem::offset_of!(CPixel32, b) == 2);
    assert!(core::mem::offset_of!(CPixel32, a) == 3);
};
