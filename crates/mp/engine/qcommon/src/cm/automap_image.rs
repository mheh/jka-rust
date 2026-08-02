#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::cm::cpixel32::CPixel32;

/// One RGBA source image `CTerrainMap` draws from: Raven's `byte* mSym…` plus
/// its `mSym…Width`/`mSym…Height` pair, gathered as one owned record.
///
/// Raven fills the triple from `R_LoadImage`, which leaves the width and height
/// untouched when the load fails, so a failed load makes the following
/// `BlitColor` read uninitialized sizes. An empty image here reports `0` for
/// both, and `BlitColor` then returns before it reads a pixel (porting-rules
/// §F19).
///
/// Type definition source: `oracle/codemp/qcommon/cm_terrainmap.h:23-37`
#[derive(Clone, Default, Debug)]
pub struct AutomapImage {
    /// `width * height` pixels, row-major, stride equal to `width`.
    pub pixels: Vec<CPixel32>,
    pub width: c_int,
    pub height: c_int,
}

impl AutomapImage {
    /// Build an image from a renderer RGBA buffer. `pic` must hold
    /// `width * height * 4` bytes.
    pub fn from_rgba(pic: &[u8], width: c_int, height: c_int) -> Self {
        let pixels = pic
            .chunks_exact(4)
            .map(|p| CPixel32::new(p[0], p[1], p[2], p[3]))
            .collect();
        AutomapImage {
            pixels,
            width,
            height,
        }
    }

    /// Raven's failed-load state: a null `byte*`.
    pub fn none() -> Self {
        AutomapImage::default()
    }

    /// Whether Raven's pointer would be non-null.
    pub fn is_loaded(&self) -> bool {
        !self.pixels.is_empty()
    }
}
