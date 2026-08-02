#![allow(non_camel_case_types, non_snake_case)]

//! `CDraw32` — the 32-bit-per-pixel raster.
//!
//! Two deliberate divergences from Raven, both forced by porting-rules §B3
//! (no globals):
//!
//! - Raven keeps the whole drawing context in class statics. The port makes it
//!   the fields of this struct, and a caller threads one `CDraw32` value.
//! - Raven keeps the pixel buffer in the `buffer` static, so `SetBuffer` picks
//!   the target once for many draw calls. The port passes the target slice to
//!   each drawing method, and `SetBuffer` dissolves.
//!
//! Raven's `assert(buffer != NULL)` guards compile out of a release build, so
//! they do not port. Raven's no-clip (`*NC`) methods write out of bounds when
//! the caller gives unclipped coordinates; the port indexes a slice, so that
//! case panics instead of corrupting memory (porting-rules §F19).
//!
//! Source: `oracle/codemp/qcommon/cm_draw.cpp`, `oracle/codemp/qcommon/cm_draw.h`

use core::ffi::c_long;

use native_types::byte;

use crate::cm::cm_draw_cpp_consts::{imgKernel, BOTTOM, INT_SHIFT, KWIDTH, LEFT, RIGHT, TOP};
use crate::cm::cpixel32::{CPixel32, ALPHA_PIX, AVE_PIX, LIGHT_PIX};
use crate::cm::point::POINT;
use crate::cm::poly_scan::PolyScan;
use crate::cm_draw::{code, compare_active, compare_ind, del_edge, ins_edge, shell_sort};

/// Raven `PIXPOS` — the buffer offset of the pixel at `(x, y)`.
///
/// Source: `oracle/codemp/qcommon/cm_draw.h:16`
fn PIXPOS(x: c_long, y: c_long, stride: c_long) -> c_long {
    (y * stride) + x
}

/// Raven `SIGN` — `-1`, `0`, or `1`.
///
/// Source: `oracle/codemp/qcommon/cm_draw.h:23`
fn SIGN(x: c_long) -> c_long {
    if x < 0 {
        -1
    } else if x > 0 {
        1
    } else {
        0
    }
}

/// Raven `CLAMP`.
///
/// Source: `oracle/codemp/qcommon/cm_draw.h:29`
fn CLAMP(v: c_long, l: c_long, h: c_long) -> c_long {
    if v < l {
        l
    } else if v > h {
        h
    } else {
        v
    }
}

/// Raven `CDraw32` — the 32-bit-per-pixel drawing class.
///
/// Raven keeps the drawing context in class statics so that a caller sets it
/// once for many draw calls.
/// This port carries that context as fields, because the codebase allows no
/// globals.
///
/// Type definition source: `oracle/codemp/qcommon/cm_draw.h:86-247`
/// Static definitions source: `oracle/codemp/qcommon/cm_draw.cpp:16-23`
#[derive(Clone, Default, Debug)]
pub struct CDraw32 {
    /// size of buffer
    pub buf_width: c_long,
    /// size of buffer
    pub buf_height: c_long,
    /// stride of buffer in pixels
    pub stride: c_long,
    /// clip bounds
    pub clip_min_x: c_long,
    /// clip bounds
    pub clip_min_y: c_long,
    /// clip bounds
    pub clip_max_x: c_long,
    /// clip bounds
    pub clip_max_y: c_long,
    /// Table for quick Y calculations
    pub row_off: Vec<c_long>,
}

impl CDraw32 {
    /// Raven `CDraw32::CDraw32` — an unset context. Raven's constructor is
    /// empty because the context lives in statics.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:26-29`
    pub fn new() -> Self {
        CDraw32::default()
    }

    /// Raven `CDraw32::SetClip` — set the rect to clip drawing functions to.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:110-112`
    pub fn SetClip(&mut self, min_x: c_long, min_y: c_long, max_x: c_long, max_y: c_long) {
        self.clip_min_x = min_x.max(0);
        self.clip_max_x = max_x.min(self.buf_width - 1);
        self.clip_min_y = min_y.max(0);
        self.clip_max_y = max_y.min(self.buf_height - 1);
    }

    /// Raven `CDraw32::GetClip` — the clip rect, as `(min_x, min_y, max_x,
    /// max_y)`. Raven's four out-params collapse to a tuple (§C7).
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:114-116`
    pub fn GetClip(&self) -> (c_long, c_long, c_long, c_long) {
        (
            self.clip_min_x,
            self.clip_min_y,
            self.clip_max_x,
            self.clip_max_y,
        )
    }

    /// Raven `CDraw32::SetBufferSize` — set up for a buffer of this size, and
    /// rebuild the row-offset table when the size changed. Always resets the
    /// clip rect to the whole buffer.
    ///
    /// Raven returns `false` only when `new long[height]` returns null, which
    /// an allocating Rust `Vec` never does; the port always returns `true`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:93-131`
    pub fn SetBufferSize(&mut self, width: c_long, height: c_long, stride_len: c_long) -> bool {
        if self.buf_width != width || self.buf_height != height || stride_len != self.stride {
            // need to re-create row_off table
            self.buf_width = width;
            self.buf_height = height;
            self.stride = stride_len;

            // row offsets used for quick pixel address calcs
            // table for quick pixel lookups
            self.row_off = (0..height).map(|i| i * self.stride).collect();
        }
        // set default clip bounds
        self.SetClip(0, 0, width - 1, height - 1);

        true
    }

    /// Raven `CDraw32::CleanUp` — drop the row-offset table before the program
    /// ends. Raven leaves `stride` at its last value; the port keeps that.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:125-126`
    pub fn CleanUp(&mut self) {
        self.row_off = Vec::new();
        self.buf_width = 0;
        self.buf_height = 0;
    }

    /// Raven `CDraw32::PutPixNC` — set a pixel at (x,y) to color (no clipping).
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:129-130`
    pub fn PutPixNC(&self, buf: &mut [CPixel32], x: c_long, y: c_long, color: CPixel32) {
        buf[(self.row_off[y as usize] + x) as usize] = color;
    }

    /// Raven `CDraw32::PutPix` — set a pixel at (x,y) to color.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:133-139`
    pub fn PutPix(&self, buf: &mut [CPixel32], x: c_long, y: c_long, color: CPixel32) {
        // clipping check
        if x < self.clip_min_x
            || x > self.clip_max_x
            || y < self.clip_min_y
            || y > self.clip_max_y
        {
            return;
        }
        self.PutPixNC(buf, x, y, color);
    }

    /// Raven `CDraw32::GetPix` — get the color of a pixel at (x,y).
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:142-143`
    pub fn GetPix(&self, buf: &[CPixel32], x: c_long, y: c_long) -> CPixel32 {
        buf[(self.row_off[y as usize] + x) as usize]
    }

    /// Raven `CDraw32::PutPixAveNC` — set a pixel at (x,y) with 50 percent
    /// translucency (no clip).
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:146-147`
    pub fn PutPixAveNC(&self, buf: &mut [CPixel32], x: c_long, y: c_long, color: CPixel32) {
        let blended = AVE_PIX(self.GetPix(buf, x, y), color);
        self.PutPixNC(buf, x, y, blended);
    }

    /// Raven `CDraw32::PutPixAve` — set a pixel at (x,y) with 50 percent
    /// translucency.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:150-156`
    pub fn PutPixAve(&self, buf: &mut [CPixel32], x: c_long, y: c_long, color: CPixel32) {
        // clipping check
        if x < self.clip_min_x
            || x > self.clip_max_x
            || y < self.clip_min_y
            || y > self.clip_max_y
        {
            return;
        }
        self.PutPixAveNC(buf, x, y, color);
    }

    /// Raven `CDraw32::PutPixAlphaNC` — set a pixel at (x,y) with translucency
    /// level (no clip).
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:159-160`
    pub fn PutPixAlphaNC(&self, buf: &mut [CPixel32], x: c_long, y: c_long, color: CPixel32) {
        let dst = self.GetPix(buf, x, y);
        let blended = ALPHA_PIX(color, dst, color.a as c_long, 256 - color.a as c_long);
        self.PutPixNC(buf, x, y, blended);
    }

    /// Raven `CDraw32::PutPixAlpha` — set a pixel at (x,y) with translucency
    /// level.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:163-168`
    pub fn PutPixAlpha(&self, buf: &mut [CPixel32], x: c_long, y: c_long, color: CPixel32) {
        // clipping check
        if x < self.clip_min_x
            || x > self.clip_max_x
            || y < self.clip_min_y
            || y > self.clip_max_y
        {
            return;
        }
        self.PutPixAlphaNC(buf, x, y, color);
    }

    /// Raven `CDraw32::ClearLines` — clear lines `start` through `end` to
    /// `color`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:133-158`
    pub fn ClearLines(&self, buf: &mut [CPixel32], color: CPixel32, start: c_long, end: c_long) {
        let mut dest = self.row_off[start as usize];

        let next_line = self.stride - self.buf_width;
        let mut line = end - start + 1;

        while line != 0 {
            line -= 1;
            // very simple-minded fill loop
            let mut i = self.buf_width;
            while i != 0 {
                i -= 1;
                buf[dest as usize] = color;
                dest += 1;
            }
            dest += next_line;
        }
    }

    /// Raven `CDraw32::ClearBuffer` — clear the whole buffer to `color`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:174-175`
    pub fn ClearBuffer(&self, buf: &mut [CPixel32], color: CPixel32) {
        self.ClearLines(buf, color, 0, self.buf_height - 1);
    }

    /// Raven `CDraw32::SetAlphaLines` — set the alpha value only, on lines
    /// `start` through `end`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:160-188`
    pub fn SetAlphaLines(&self, buf: &mut [CPixel32], alpha: byte, start: c_long, end: c_long) {
        let mut dest = self.row_off[start as usize];

        let next_line = self.stride - self.buf_width;
        let mut line = end - start + 1;

        while line != 0 {
            line -= 1;
            // very simple-minded fill loop
            let mut i = self.buf_width;
            while i != 0 {
                i -= 1;
                buf[dest as usize].a = alpha;
                dest += 1;
            }
            dest += next_line;
        }
    }

    /// Raven `CDraw32::SetAlphaBuffer` — set the alpha of the whole buffer.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:181-182`
    pub fn SetAlphaBuffer(&self, buf: &mut [CPixel32], alpha: byte) {
        self.SetAlphaLines(buf, alpha, 0, self.buf_height - 1);
    }

    /// Raven `CDraw32::ClipLine` — clip a line to the clip rect. Returns `true`
    /// when something is left to draw.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:211-277`
    pub fn ClipLine(
        &self,
        x1: &mut c_long,
        y1: &mut c_long,
        x2: &mut c_long,
        y2: &mut c_long,
    ) -> bool {
        let mut x = *x1;
        let mut y = *y1;

        let mut c1 = code(self, *x1, *y1); // find where first pt. is
        let mut c2 = code(self, *x2, *y2); // find where second pt. is

        if (c1 & c2) == 0 {
            // the line may be visible
            while (c1 | c2) != 0 {
                // where there is 2D clipping to be done
                if (c1 & c2) != 0 {
                    return false; // if both on same side, quit
                }

                let mut c = c1;
                if c == 0 {
                    c = c2; // pick a point
                }

                let f: c_long;
                if (c & TOP) != 0 {
                    f = ((self.clip_max_y - *y1) << 15) / (*y2 - *y1);
                    x = *x1 + (((*x2 - *x1) * f + 16384) >> 15);
                    y = self.clip_max_y;
                } else if (c & BOTTOM) != 0 {
                    f = ((self.clip_min_y - *y1) << 15) / (*y2 - *y1);
                    x = *x1 + (((*x2 - *x1) * f + 16384) >> 15);
                    y = self.clip_min_y;
                } else if (c & LEFT) != 0 {
                    f = ((self.clip_min_x - *x1) << 15) / (*x2 - *x1);
                    y = *y1 + (((*y2 - *y1) * f + 16384) >> 15);
                    x = self.clip_min_x;
                } else if (c & RIGHT) != 0 {
                    f = ((self.clip_max_x - *x1) << 15) / (*x2 - *x1);
                    y = *y1 + (((*y2 - *y1) * f + 16384) >> 15);
                    x = self.clip_max_x;
                }

                if c == c1 {
                    *x1 = x;
                    *y1 = y;
                    c1 = code(self, *x1, *y1);
                } else {
                    *x2 = x;
                    *y2 = y;
                    c2 = code(self, *x2, *y2);
                }
            } // while still needs clipping
        } else {
            // line not visible
            return false;
        }
        true
    }

    /// Raven `CDraw32::DrawLineNC` — draw a solid colored line, no clipping.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:279-374`
    pub fn DrawLineNC(
        &self,
        buf: &mut [CPixel32],
        x1: c_long,
        y1: c_long,
        x2: c_long,
        y2: c_long,
        color: CPixel32,
    ) {
        let mut x1 = x1;
        let mut y1 = y1;

        let dx = x2 - x1;
        let ax = dx.abs() << 1;
        let sx = SIGN(dx);
        let mut dy = y2 - y1;
        let ay = dy.abs() << 1;
        let sy = SIGN(dy);

        if 255 == color.a {
            if dy == 0 {
                // horz line
                let (mut dest, mut i) = if dx >= 0 {
                    (self.row_off[y1 as usize] + x1, dx + 1)
                } else {
                    (self.row_off[y1 as usize] + x1 + dx, -dx + 1)
                };
                while i != 0 {
                    i -= 1;
                    buf[dest as usize] = color;
                    dest += 1;
                }
                return;
            }

            if dx == 0 {
                // vert line
                let mut dest;
                if dy >= 0 {
                    dest = self.row_off[y1 as usize] + x1;
                    dy += 1;
                } else {
                    dest = self.row_off[y2 as usize] + x1;
                    dy = -dy + 1;
                }

                while dy != 0 {
                    dy -= 1;
                    buf[dest as usize] = color;
                    dest += self.stride;
                }
                return;
            }
        }

        // bressenham's algorithm
        if ax > ay {
            let mut d = ay - (ax >> 1);
            while x1 != x2 {
                self.PutPixAlphaNC(buf, x1, y1, color);

                if d >= 0 {
                    y1 += sy;
                    d -= ax;
                }
                x1 += sx;
                d += ay;
            }
        } else {
            let mut d = ax - (ay >> 1);
            while y1 != y2 {
                self.PutPixAlphaNC(buf, x1, y1, color);
                if d >= 0 {
                    x1 += sx;
                    d -= ay;
                }
                y1 += sy;
                d += ax;
            }
        }
        self.PutPixAlphaNC(buf, x1, y1, color);
    }

    /// Raven `CDraw32::DrawLine` — draw a solid color line.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:191-192`
    pub fn DrawLine(
        &self,
        buf: &mut [CPixel32],
        x1: c_long,
        y1: c_long,
        x2: c_long,
        y2: c_long,
        color: CPixel32,
    ) {
        let (mut x1, mut y1, mut x2, mut y2) = (x1, y1, x2, y2);
        if self.ClipLine(&mut x1, &mut y1, &mut x2, &mut y2) {
            self.DrawLineNC(buf, x1, y1, x2, y2, color);
        }
    }

    /// Raven `CDraw32::DrawLineAveNC` — draw a translucent line, no clipping.
    ///
    /// Raven's horizontal and vertical fast paths write `*dest++ =
    /// AVE_PIX(*dest, color)`, where the read and the pointer bump are
    /// unsequenced (UB). The port blends against the pixel it writes, which is
    /// the intent the surrounding code states (porting-rules §F19).
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:376-468`
    pub fn DrawLineAveNC(
        &self,
        buf: &mut [CPixel32],
        x1: c_long,
        y1: c_long,
        x2: c_long,
        y2: c_long,
        color: CPixel32,
    ) {
        let mut x1 = x1;
        let mut y1 = y1;

        let dx = x2 - x1;
        let ax = dx.abs() << 1;
        let sx = SIGN(dx);
        let mut dy = y2 - y1;
        let ay = dy.abs() << 1;
        let sy = SIGN(dy);

        if dy == 0 {
            // horz line
            let (mut dest, mut i) = if dx >= 0 {
                (self.row_off[y1 as usize] + x1, dx + 1)
            } else {
                (self.row_off[y1 as usize] + x1 + dx, -dx + 1)
            };
            while i != 0 {
                i -= 1;
                buf[dest as usize] = AVE_PIX(buf[dest as usize], color);
                dest += 1;
            }
            return;
        }

        if dx == 0 {
            // vert line
            let mut dest;
            if dy >= 0 {
                dest = self.row_off[y1 as usize] + x1;
                dy += 1;
            } else {
                dest = self.row_off[y2 as usize] + x1;
                dy = -dy + 1;
            }

            while dy != 0 {
                dy -= 1;
                buf[dest as usize] = AVE_PIX(buf[dest as usize], color);
                dest += self.stride;
            }
            return;
        }

        // bressenham's algorithm
        if ax > ay {
            let mut d = ay - (ax >> 1);
            while x1 != x2 {
                self.PutPixAveNC(buf, x1, y1, color);

                if d >= 0 {
                    y1 += sy;
                    d -= ax;
                }
                x1 += sx;
                d += ay;
            }
        } else {
            let mut d = ax - (ay >> 1);
            while y1 != y2 {
                self.PutPixAveNC(buf, x1, y1, color);
                if d >= 0 {
                    x1 += sx;
                    d -= ay;
                }
                y1 += sy;
                d += ax;
            }
        }
        self.PutPixAveNC(buf, x1, y1, color);
    }

    /// Raven `CDraw32::DrawLineAve` — draw a translucent solid color line.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:197-198`
    pub fn DrawLineAve(
        &self,
        buf: &mut [CPixel32],
        x1: c_long,
        y1: c_long,
        x2: c_long,
        y2: c_long,
        color: CPixel32,
    ) {
        let (mut x1, mut y1, mut x2, mut y2) = (x1, y1, x2, y2);
        if self.ClipLine(&mut x1, &mut y1, &mut x2, &mut y2) {
            self.DrawLineAveNC(buf, x1, y1, x2, y2, color);
        }
    }

    /// Raven `CDraw32::DrawLineAANC` — Xiaolin Wu antialiased line, no
    /// clipping.
    ///
    /// The two paired writes go through `PutPixAlphaNC`, so each already-blended
    /// pixel is blended a second time against the destination. That is Raven's
    /// arithmetic and the port keeps it.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:470-610`
    pub fn DrawLineAANC(
        &self,
        buf: &mut [CPixel32],
        x0: c_long,
        y0: c_long,
        x1: c_long,
        y1: c_long,
        color: CPixel32,
    ) {
        let (mut x0, mut y0, mut x1, mut y1) = (x0, y0, x1, y1);

        // Make sure the line runs top to bottom
        if y0 > y1 {
            core::mem::swap(&mut y0, &mut y1);
            core::mem::swap(&mut x0, &mut x1);
        }

        let mut DeltaX = x1 - x0;
        let mut DeltaY = y1 - y0;
        let XDir: c_long;

        // Draw the initial pixel, which is always exactly intersected by
        // the line and so needs no Alpha
        self.PutPixAlphaNC(buf, x0, y0, color);

        if DeltaX >= 0 {
            XDir = 1;
        } else {
            XDir = -1;
            DeltaX = -DeltaX; // make DeltaX positive
        }

        // Special-case horizontal, vertical, and diagonal lines, which
        // require no Alpha because they go right through the center of
        // every pixel
        if DeltaY == 0 {
            // Horizontal line
            while DeltaX != 0 {
                DeltaX -= 1;
                x0 += XDir;
                self.PutPixAlphaNC(buf, x0, y0, color);
            }
            return;
        }
        if DeltaX == 0 {
            // Vertical line
            loop {
                y0 += 1;
                self.PutPixAlphaNC(buf, x0, y0, color);
                DeltaY -= 1;
                if DeltaY == 0 {
                    break;
                }
            }
            return;
        }
        if DeltaX == DeltaY {
            // Diagonal line
            loop {
                x0 += XDir;
                y0 += 1;
                self.PutPixAlphaNC(buf, x0, y0, color);
                DeltaY -= 1;
                if DeltaY == 0 {
                    break;
                }
            }
            return;
        }

        // Line is not horizontal, diagonal, or vertical
        let mut ErrorAcc: u16 = 0; // initialize the line error accumulator to 0

        // # of bits by which to shift ErrorAcc to get intensity level
        const IntensityShift: u32 = 16 - 8;

        // Is this an X-major or Y-major line?
        if DeltaY > DeltaX {
            // Y-major line; calculate 16-bit fixed-point fractional part of a
            // pixel that X advances each time Y advances 1 pixel, truncating the
            // result so that we won't overrun the endpoint along the X axis
            let ErrorAdj = (((DeltaX as u64) << 16) / DeltaY as u64) as u16;

            // Draw all pixels other than the first and last
            loop {
                DeltaY -= 1;
                if DeltaY == 0 {
                    break;
                }
                let ErrorAccTemp = ErrorAcc; // remember currrent accumulated error
                ErrorAcc = ErrorAcc.wrapping_add(ErrorAdj); // calculate error for next pixel
                if ErrorAcc <= ErrorAccTemp {
                    // The error accumulator turned over, so advance the X coord
                    x0 += XDir;
                }
                y0 += 1; // Y-major, so always advance Y
                         // The IntensityBits most significant bits of ErrorAcc give us the
                         // intensity Alpha for this pixel, and the complement of the
                         // Alpha for the paired pixel
                let Alpha = (ErrorAcc >> IntensityShift) as c_long;
                let InvAlpha = 256 - Alpha;

                let here = ALPHA_PIX(self.GetPix(buf, x0, y0), color, Alpha, InvAlpha);
                self.PutPixAlphaNC(buf, x0, y0, here);
                let paired = ALPHA_PIX(self.GetPix(buf, x0 + XDir, y0), color, InvAlpha, Alpha);
                self.PutPixAlphaNC(buf, x0 + XDir, y0, paired);
            }
            // Draw the final pixel, which is always exactly intersected by the line
            // and so needs no Alpha
            self.PutPixAlphaNC(buf, x1, y1, color);
            return;
        }
        // It's an X-major line; calculate 16-bit fixed-point fractional part of a
        // pixel that Y advances each time X advances 1 pixel, truncating the
        // result to avoid overrunning the endpoint along the X axis
        let ErrorAdj = (((DeltaY as u64) << 16) / DeltaX as u64) as u16;
        // Draw all pixels other than the first and last
        loop {
            DeltaX -= 1;
            if DeltaX == 0 {
                break;
            }
            let ErrorAccTemp = ErrorAcc; // remember currrent accumulated error
            ErrorAcc = ErrorAcc.wrapping_add(ErrorAdj); // calculate error for next pixel
            if ErrorAcc <= ErrorAccTemp {
                // The error accumulator turned over, so advance the Y coord
                y0 += 1;
            }
            x0 += XDir; // X-major, so always advance X
                        // The IntensityBits most significant bits of ErrorAcc give us the
                        // intensity Alpha for this pixel, and the complement of the
                        // Alpha for the paired pixel
            let Alpha = (ErrorAcc >> IntensityShift) as c_long;
            let InvAlpha = 256 - Alpha;
            let here = ALPHA_PIX(self.GetPix(buf, x0, y0), color, Alpha, InvAlpha);
            self.PutPixAlphaNC(buf, x0, y0, here);
            let paired = ALPHA_PIX(self.GetPix(buf, x0, y0 + 1), color, InvAlpha, Alpha);
            self.PutPixAlphaNC(buf, x0, y0 + 1, paired);
        }
        // Draw the final pixel, which is always exactly intersected by the line
        // and so needs no Alpha
        self.PutPixAlphaNC(buf, x1, y1, color);
    }

    /// Raven `CDraw32::DrawLineAA` — draw an anti-aliased line.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.h:204-205`
    pub fn DrawLineAA(
        &self,
        buf: &mut [CPixel32],
        x1: c_long,
        y1: c_long,
        x2: c_long,
        y2: c_long,
        color: CPixel32,
    ) {
        let (mut x1, mut y1, mut x2, mut y2) = (x1, y1, x2, y2);
        if self.ClipLine(&mut x1, &mut y1, &mut x2, &mut y2) {
            self.DrawLineAANC(buf, x1, y1, x2, y2, color);
        }
    }

    /// Raven `CDraw32::DrawRectNC` — draw a filled rectangle, no clipping.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:612-634`
    pub fn DrawRectNC(
        &self,
        buf: &mut [CPixel32],
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        let (mut uly, mut height) = (uly, height);
        if height < 1 || width < 1 {
            return;
        }

        while height != 0 {
            height -= 1;
            self.DrawLineNC(buf, ulx, uly, ulx + width - 1, uly, color);
            uly += 1;
        }
    }

    /// Raven `CDraw32::DrawRect` — draw a filled rectangle.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:636-653`
    pub fn DrawRect(
        &self,
        buf: &mut [CPixel32],
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        let (mut uly, mut height) = (uly, height);
        if height < 1 || width < 1 {
            return;
        }

        while height != 0 {
            height -= 1;
            self.DrawLine(buf, ulx, uly, ulx + width - 1, uly, color);
            uly += 1;
        }
    }

    /// Raven `CDraw32::DrawRectAve` — draw a translucent filled rectangle.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:655-672`
    pub fn DrawRectAve(
        &self,
        buf: &mut [CPixel32],
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        let (mut uly, mut height) = (uly, height);
        if height < 1 || width < 1 {
            return;
        }

        while height != 0 {
            height -= 1;
            self.DrawLineAve(buf, ulx, uly, ulx + width - 1, uly, color);
            uly += 1;
        }
    }

    /// Raven `CDraw32::DrawBoxNC` — draw an unfilled rectangle, no clipping.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:674-690`
    pub fn DrawBoxNC(
        &self,
        buf: &mut [CPixel32],
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        if height < 1 || width < 1 {
            return;
        }

        self.DrawLineNC(buf, ulx, uly, ulx + width - 1, uly, color);
        self.DrawLineNC(
            buf,
            ulx,
            uly + height - 1,
            ulx + width - 1,
            uly + height - 1,
            color,
        );
        self.DrawLineNC(buf, ulx, uly, ulx, uly + height - 1, color);
        self.DrawLineNC(
            buf,
            ulx + width - 1,
            uly,
            ulx + width - 1,
            uly + height - 1,
            color,
        );
    }

    /// Raven `CDraw32::DrawBox` — draw an unfilled rectangle.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:692-708`
    pub fn DrawBox(
        &self,
        buf: &mut [CPixel32],
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        if height < 1 || width < 1 {
            return;
        }

        self.DrawLine(buf, ulx, uly, ulx + width - 1, uly, color);
        self.DrawLine(
            buf,
            ulx,
            uly + height - 1,
            ulx + width - 1,
            uly + height - 1,
            color,
        );
        self.DrawLine(buf, ulx, uly, ulx, uly + height - 1, color);
        self.DrawLine(
            buf,
            ulx + width - 1,
            uly,
            ulx + width - 1,
            uly + height - 1,
            color,
        );
    }

    /// Raven `CDraw32::DrawBoxAve` — draw a translucent unfilled rectangle.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:710-726`
    pub fn DrawBoxAve(
        &self,
        buf: &mut [CPixel32],
        ulx: c_long,
        uly: c_long,
        width: c_long,
        height: c_long,
        color: CPixel32,
    ) {
        if height < 1 || width < 1 {
            return;
        }

        self.DrawLineAve(buf, ulx, uly, ulx + width - 1, uly, color);
        self.DrawLineAve(
            buf,
            ulx,
            uly + height - 1,
            ulx + width - 1,
            uly + height - 1,
            color,
        );
        self.DrawLineAve(buf, ulx, uly, ulx, uly + height - 1, color);
        self.DrawLineAve(
            buf,
            ulx + width - 1,
            uly,
            ulx + width - 1,
            uly + height - 1,
            color,
        );
    }

    /// Raven `CDraw32::DrawCircle` — Bresenham circle with fill and edge
    /// colors. An alpha of zero on either color skips that pass.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:728-882`
    pub fn DrawCircle(
        &self,
        buf: &mut [CPixel32],
        xc: c_long,
        yc: c_long,
        r: c_long,
        edge: CPixel32,
        fill: CPixel32,
    ) {
        if r < 1 {
            return;
        }

        // draw fill
        if fill.a != 0 {
            let mut x: c_long = 0;
            let mut last_x = x;
            let mut y: c_long = r;
            let mut last_y = y;
            let mut di = 2 * (1 - r);
            let limit: c_long = 0;

            loop {
                if y >= limit {
                    if di < 0 {
                        let delta = 2 * di + 2 * y - 1;
                        if delta <= 0 {
                            // move horizontal
                            last_x = x;
                            x += 1;
                            di += 2 * x + 1;
                        } else {
                            // move diagonal
                            last_x = x;
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        }
                    } else if di > 0 {
                        let delta = 2 * di - 2 * x - 1;
                        if delta <= 0 {
                            // move diagonal
                            last_x = x;
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        } else {
                            // move vertical
                            y -= 1;
                            di += 1 - 2 * y;
                        }
                    } else {
                        // di = 0, move diagonal
                        last_x = x;
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                }

                if y != last_y {
                    // circle fill
                    self.DrawLine(buf, xc - last_x, yc + last_y, xc + last_x, yc + last_y, fill);
                    if last_y > limit {
                        self.DrawLine(buf, xc - last_x, yc - last_y, xc + last_x, yc - last_y, fill);
                    }
                    last_y = y;
                }

                if y < limit {
                    break;
                }
            }
        }

        // draw edge
        if edge.a != 0 {
            let mut x: c_long = 0;
            let mut y: c_long = r;
            let limit: c_long = 0;
            let mut di = 2 * (1 - r);

            loop {
                // circle edge
                self.PutPix(buf, xc + x, yc + y, edge);
                self.PutPix(buf, xc - x, yc + y, edge);
                if y > limit {
                    self.PutPix(buf, xc + x, yc - y, edge);
                    self.PutPix(buf, xc - x, yc - y, edge);
                }

                if y >= limit {
                    if di < 0 {
                        let delta = 2 * di + 2 * y - 1;
                        if delta <= 0 {
                            // move horizontal
                            x += 1;
                            di += 2 * x + 1;
                        } else {
                            // move diagonal
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        }
                    } else if di > 0 {
                        let delta = 2 * di - 2 * x - 1;
                        if delta <= 0 {
                            // move diagonal
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        } else {
                            // move vertical
                            y -= 1;
                            di += 1 - 2 * y;
                        }
                    } else {
                        // di = 0, move diagonal
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                }

                if y < limit {
                    break;
                }
            }
        }
    }

    /// Raven `CDraw32::DrawCircleAve` — the `DrawCircle` walk with every write
    /// averaged against the destination.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:884-1041`
    pub fn DrawCircleAve(
        &self,
        buf: &mut [CPixel32],
        xc: c_long,
        yc: c_long,
        r: c_long,
        edge: CPixel32,
        fill: CPixel32,
    ) {
        if r < 1 {
            return;
        }

        // draw fill
        if fill.a != 0 {
            let mut x: c_long = 0;
            let mut last_x = x;
            let mut y: c_long = r;
            let mut last_y = y;
            let mut di = 2 * (1 - r);
            let limit: c_long = 0;

            loop {
                if y >= limit {
                    if di < 0 {
                        let delta = 2 * di + 2 * y - 1;
                        if delta <= 0 {
                            // move horizontal
                            last_x = x;
                            x += 1;
                            di += 2 * x + 1;
                        } else {
                            // move diagonal
                            last_x = x;
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        }
                    } else if di > 0 {
                        let delta = 2 * di - 2 * x - 1;
                        if delta <= 0 {
                            // move diagonal
                            last_x = x;
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        } else {
                            // move vertical
                            y -= 1;
                            di += 1 - 2 * y;
                        }
                    } else {
                        // di = 0, move diagonal
                        last_x = x;
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                }

                if y != last_y {
                    // circle fill
                    let mut f = xc - last_x;
                    while f <= xc + last_x {
                        self.PutPixAve(buf, f, yc + last_y, fill);
                        f += 1;
                    }
                    if last_y > limit {
                        let mut f = xc - last_x;
                        while f <= xc + last_x {
                            self.PutPixAve(buf, f, yc - last_y, fill);
                            f += 1;
                        }
                    }
                    last_y = y;
                }

                if y < limit {
                    break;
                }
            }
        }

        // draw edge
        if edge.a != 0 {
            let mut x: c_long = 0;
            let mut y: c_long = r;
            let limit: c_long = 0;
            let mut di = 2 * (1 - r);

            loop {
                // circle edge
                self.PutPixAve(buf, xc + x, yc + y, edge);
                self.PutPixAve(buf, xc - x, yc + y, edge);
                if y > limit {
                    self.PutPixAve(buf, xc + x, yc - y, edge);
                    self.PutPixAve(buf, xc - x, yc - y, edge);
                }
                if y >= limit {
                    if di < 0 {
                        let delta = 2 * di + 2 * y - 1;
                        if delta <= 0 {
                            // move horizontal
                            x += 1;
                            di += 2 * x + 1;
                        } else {
                            // move diagonal
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        }
                    } else if di > 0 {
                        let delta = 2 * di - 2 * x - 1;
                        if delta <= 0 {
                            // move diagonal
                            x += 1;
                            y -= 1;
                            di += 2 * x - 2 * y + 2;
                        } else {
                            // move vertical
                            y -= 1;
                            di += 1 - 2 * y;
                        }
                    } else {
                        // di = 0, move diagonal
                        x += 1;
                        y -= 1;
                        di += 2 * x - 2 * y + 2;
                    }
                }

                if y < limit {
                    break;
                }
            }
        }
    }

    /// Raven `CDraw32::DrawPolygon` — even-odd scan conversion of a concave
    /// polygon, then an antialiased edge pass.
    ///
    /// Raven's brace placement closes the scanline loop one line early
    /// (`cm_draw.cpp:1316`), so the span sort and the span fill run one extra
    /// time after the loop, at `y == y1`, against the final active edge list.
    /// The port keeps that pass: it is the shipped output.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:1191-1366`
    pub fn DrawPolygon(
        &self,
        buf: &mut [CPixel32],
        nvert: c_long,
        point: &[POINT],
        edge: CPixel32,
        fill: CPixel32,
    ) {
        let n = nvert;

        if n <= 0 {
            // nothing to do
            return;
        }

        let mut scan = PolyScan::new(n, point);

        if fill.a != 0 {
            // draw fill

            // create y-sorted array of indices ind[k] into vertex list
            let mut ind = [0 as c_long; 256];
            for (k, slot) in ind.iter_mut().enumerate().take(n as usize) {
                *slot = k as c_long;
            }

            // sort ind by pt[ind[k]].y
            shell_sort(&mut ind, n, |u, v| compare_ind(point, u, v));

            scan.nact = 0; // start with empty active list
            let mut k: c_long = 0; // ind[k] is next vertex to process

            // ymin of polygon
            let y0 = (self.clip_min_y - 1).max(point[ind[0] as usize].y);

            // ymax of polygon
            let y1 = (self.clip_max_y + 1).min(point[ind[(n - 1) as usize] as usize].y);

            // step through scanlines
            let mut y = y0;
            while y < y1 {
                // Check vertices between previous scanline
                // and current one, if any
                while k < n && point[ind[k as usize] as usize].y <= y {
                    let i = ind[k as usize];
                    //  insert or delete edges before and after vertex i
                    //  (i-1 to i, and i to i+1)
                    //  from active list if they cross scanline y
                    let mut j = if i > 0 { i - 1 } else { n - 1 }; // vertex previous to i
                    if point[j as usize].y < y {
                        // old edge, remove from active list
                        del_edge(&mut scan, j);
                    } else if point[j as usize].y > y {
                        // new edge, add to active list
                        ins_edge(&mut scan, j, y);
                    }
                    j = if i < n - 1 { i + 1 } else { 0 }; // vertex next after i
                    if point[j as usize].y < y {
                        // old edge, remove from active list
                        del_edge(&mut scan, i);
                    } else if point[j as usize].y > y {
                        // new edge, add to active list
                        ins_edge(&mut scan, i, y);
                    }
                    k += 1;
                }

                self.fill_spans(buf, &mut scan, y, fill);
                y += 1;
            }

            // Raven's early brace: one more sort and span fill, at y == y1.
            self.fill_spans(buf, &mut scan, y, fill);
        }

        if edge.a != 0 {
            // draw edges
            for k in 0..(n - 1) {
                self.DrawLineAA(
                    buf,
                    point[k as usize].x,
                    point[k as usize].y,
                    point[(k + 1) as usize].x,
                    point[(k + 1) as usize].y,
                    edge,
                );
            }

            self.DrawLineAA(
                buf,
                point[(n - 1) as usize].x,
                point[(n - 1) as usize].y,
                point[0].x,
                point[0].y,
                edge,
            );
        }
    }

    /// One scanline of `DrawPolygon`'s fill: sort the active edges by x, draw
    /// the inside spans, and step every edge to the next scanline.
    ///
    /// Raven writes this block twice, verbatim, at `cm_draw.cpp:1318-1352` and
    /// `:1281-1315`.
    ///
    /// An odd `nact` makes the last iteration read `active[nact]`, which Raven
    /// leaves at whatever the previous polygon wrote. The port reads its own
    /// zeroed slot instead (porting-rules §F19); a closed polygon always has an
    /// even active count.
    fn fill_spans(
        &self,
        buf: &mut [CPixel32],
        scan: &mut PolyScan,
        y: c_long,
        fill: CPixel32,
    ) {
        // sort active edge list by active[j].x
        let nact = scan.nact;
        shell_sort(&mut scan.active, nact, compare_active);

        // draw horizontal segments for scanline y
        let mut j: c_long = 0;
        while j < nact {
            // span 'tween j & j+1 is inside, span tween
            // j+1 & j+2 is outside

            // left end of span
            // convert back from fixed point - round down
            let mut xl = scan.active[j as usize].x >> INT_SHIFT;
            if xl < self.clip_min_x - 1 {
                xl = self.clip_min_x - 1;
            }

            // right end of span
            // convert back from fixed point - round down
            let mut xr = scan.active[(j + 1) as usize].x >> INT_SHIFT;
            if xr > self.clip_max_x {
                xr = self.clip_max_x;
            }

            if xl < xr {
                // draw pixels in span
                self.DrawLine(buf, xl + 1, y, xr, y, fill);
            }

            // increment edge coords
            scan.active[j as usize].x += scan.active[j as usize].dx;
            scan.active[(j + 1) as usize].x += scan.active[(j + 1) as usize].dx;

            j += 2;
        }
    }

    /// Raven `CDraw32::BlitClip` — trim a blit rectangle to the clip rect.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:1409-1445`
    fn BlitClip(
        &self,
        dstX: &mut c_long,
        dstY: &mut c_long,
        width: &mut c_long,
        height: &mut c_long,
        srcX: &mut c_long,
        srcY: &mut c_long,
    ) {
        // clip to our buffer size
        if *dstX < self.clip_min_x {
            let dif = self.clip_min_x - *dstX;
            *dstX += dif;
            *srcX += dif;
            *width -= dif;
        }

        if *dstY < self.clip_min_y {
            let dif = self.clip_min_y - *dstY;
            *dstY += dif;
            *srcY += dif;
            *height -= dif;
        }

        if *dstX + *width - 1 > self.clip_max_x {
            *width -= *dstX + *width - 1 - self.clip_max_x;
        }

        if *dstY + *height - 1 > self.clip_max_y {
            *height -= *dstY + *height - 1 - self.clip_max_y;
        }
    }

    /// Raven `CDraw32::Blit` — alpha-composite `srcImage` into the buffer,
    /// keeping the destination alpha.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:1368-1407`
    #[allow(clippy::too_many_arguments)]
    pub fn Blit(
        &self,
        buf: &mut [CPixel32],
        dstX: c_long,
        dstY: c_long,
        width: c_long,
        height: c_long,
        srcImage: &[CPixel32],
        srcX: c_long,
        srcY: c_long,
        srcStride: c_long,
    ) {
        let (mut dstX, mut dstY, mut width, mut height, mut srcX, mut srcY) =
            (dstX, dstY, width, height, srcX, srcY);

        self.BlitClip(
            &mut dstX,
            &mut dstY,
            &mut width,
            &mut height,
            &mut srcX,
            &mut srcY,
        );

        if width < 1 || height < 1 {
            return;
        }

        let mut dst = PIXPOS(dstX, dstY, self.stride);
        let mut src = PIXPOS(srcX, srcY, srcStride);

        for _y in 0..height {
            for _x in 0..width {
                let s = srcImage[src as usize];
                let d = buf[dst as usize];
                let alpha = s.a as c_long;
                let dst_alpha = d.a;
                let mut blended = ALPHA_PIX(s, d, alpha, 256 - alpha);
                blended.a = dst_alpha;
                buf[dst as usize] = blended;
                dst += 1;
                src += 1;
            }
            dst += self.stride - width;
            src += srcStride - width;
        }
    }

    /// Raven `CDraw32::BlitColor` — blit `color` through `srcImage`'s alpha as
    /// a mask.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:1447-1489`
    #[allow(clippy::too_many_arguments)]
    pub fn BlitColor(
        &self,
        buf: &mut [CPixel32],
        dstX: c_long,
        dstY: c_long,
        width: c_long,
        height: c_long,
        srcImage: &[CPixel32],
        srcX: c_long,
        srcY: c_long,
        srcStride: c_long,
        color: CPixel32,
    ) {
        let (mut dstX, mut dstY, mut width, mut height, mut srcX, mut srcY) =
            (dstX, dstY, width, height, srcX, srcY);

        self.BlitClip(
            &mut dstX,
            &mut dstY,
            &mut width,
            &mut height,
            &mut srcX,
            &mut srcY,
        );

        if width < 1 || height < 1 {
            return;
        }

        let mut dst = PIXPOS(dstX, dstY, self.stride);
        let mut src = PIXPOS(srcX, srcY, srcStride);

        let dstNextLine = self.stride - width;
        let srcNextLine = srcStride - width;

        for _y in 0..height {
            for _x in 0..width {
                let alpha = srcImage[src as usize].a as c_long;
                buf[dst as usize] =
                    ALPHA_PIX(color, buf[dst as usize], alpha, 256 - alpha);
                dst += 1;
                src += 1;
            }
            dst += dstNextLine;
            src += srcNextLine;
        }
    }

    /// Raven `CDraw32::Emboss` — light `clrImage` by the diagonal gradient of
    /// its own alpha channel.
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:47-91`
    #[allow(clippy::too_many_arguments)]
    pub fn Emboss(
        &self,
        buf: &mut [CPixel32],
        dstX: c_long,
        dstY: c_long,
        width: c_long,
        height: c_long,
        clrImage: &[CPixel32],
        clrX: c_long,
        clrY: c_long,
        clrStride: c_long,
    ) {
        let (mut dstX, mut dstY, mut width, mut height, mut clrX, mut clrY) =
            (dstX, dstY, width, height, clrX, clrY);

        self.BlitClip(
            &mut dstX,
            &mut dstY,
            &mut width,
            &mut height,
            &mut clrX,
            &mut clrY,
        );

        if width < 1 || height < 1 {
            return;
        }

        let mut dst = PIXPOS(dstX, dstY, self.stride);
        let mut clr = PIXPOS(clrX, clrY, clrStride);

        let dstNextLine = self.stride - width;
        let clrNextLine = clrStride - width;

        for y in 0..height {
            for x in 0..width {
                let mut accum: c_long = 0;
                for j in -(KWIDTH as c_long)..=(KWIDTH as c_long) {
                    for i in -(KWIDTH as c_long)..=(KWIDTH as c_long) {
                        let xk = CLAMP(x + i, clrX, clrX + width - 1);
                        let yk = CLAMP(y + j, clrY, clrY + height - 1);
                        accum += clrImage[PIXPOS(xk, yk, clrStride) as usize].a as c_long
                            * imgKernel[(j + KWIDTH as c_long) as usize]
                                [(i + KWIDTH as c_long) as usize]
                                as c_long;
                    }
                }
                let mut lit = LIGHT_PIX(clrImage[clr as usize], accum);
                lit.a = 255;
                buf[dst as usize] = lit;
                dst += 1;
                clr += 1;
            }
            dst += dstNextLine;
            clr += clrNextLine;
        }
    }
}

// Raven declares `CDraw32::BlitNC` (`cm_draw.h:235-236`) but never defines it,
// and no caller in either tree names it. Dropped (porting-rules §20).
