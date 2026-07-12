#![allow(non_camel_case_types, non_snake_case)]

/// Raven `shaderSort_t` — shader sort order enumeration.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:144-175`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum shaderSort_t {
    SS_BAD = 0,
    SS_PORTAL = 1,      // mirrors, portals, viewscreens
    SS_ENVIRONMENT = 2, // sky box
    SS_OPAQUE = 3,      // opaque
    SS_DECAL = 4,       // scorch marks, etc.
    SS_SEE_THROUGH = 5, // ladders, grates, grills that may have small blended edges
    SS_BANNER = 6,
    SS_INSIDE = 7, // inside body parts (i.e. heart)
    SS_MID_INSIDE = 8,
    SS_MIDDLE = 9,
    SS_MID_OUTSIDE = 10,
    SS_OUTSIDE = 11, // outside body parts (i.e. ribs)
    SS_FOG = 12,
    SS_UNDERWATER = 13, // for items that should be drawn in front of the water plane
    SS_BLEND0 = 14,     // regular transparency and filters
    SS_BLEND1 = 15,     // generally only used for additive type effects
    SS_BLEND2 = 16,
    SS_BLEND3 = 17,
    SS_BLEND6 = 18,
    SS_STENCIL_SHADOW = 19,
    SS_ALMOST_NEAREST = 20, // gun smoke puffs
    SS_NEAREST = 21,        // blood blobs
}
