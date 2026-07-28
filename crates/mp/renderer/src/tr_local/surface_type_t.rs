#![allow(non_camel_case_types, non_snake_case)]

/// Raven `surfaceType_t` — surface type enumeration.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:656-678`
// `Clone, Copy` added by DEC-43.4 — a fieldless `#[repr(i32)]` enum, so the
// derives are layout-neutral; `GridMesh`/`srfFlare_t` carry one as their
// leading tag and need it to be `Clone` for the surface carrier.
#[derive(Clone, Copy)]
#[repr(i32)]
pub enum surfaceType_t {
    SF_BAD = 0,
    SF_SKIP = 1, // ignore
    SF_FACE = 2,
    SF_GRID = 3,
    SF_TRIANGLES = 4,
    SF_POLY = 5,
    SF_TERRAIN = 6, // rwwRMG - added
    SF_MD3 = 7,
    SF_MDX = 8,
    SF_FLARE = 9,
    SF_ENTITY = 10, // beams, rails, lightning, etc that can be determined by entity
    SF_DISPLAY_LIST = 11,
    SF_NUM_SURFACE_TYPES = 12,
}
