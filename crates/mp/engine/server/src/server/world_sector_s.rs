#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use super::sv_entity_s::svEntity_t;

/// Raven `AREA_DEPTH` — max recursion depth of the world area bsp tree.
///
/// Type definition source: `oracle/codemp/server/sv_world.cpp:55`
pub const AREA_DEPTH: c_int = 4;

/// Raven `AREA_NODES`.
///
/// Type definition source: `oracle/codemp/server/sv_world.cpp:56`
pub const AREA_NODES: usize = 64;

/// Raven `worldSector_t`.
///
/// Raven: to avoid linearly searching through lists of entities during
/// environment testing, the world is carved up with an evenly spaced, axially
/// aligned bsp tree. Entities are kept in chains either at the final leafs, or
/// at the first node that splits them, which prevents having to deal with
/// multiple fragments of a single entity.
/// Type definition source: `oracle/codemp/server/sv_world.cpp:48-53`
#[repr(C)]
pub struct worldSector_t {
    /// -1 = leaf node
    pub axis: c_int,
    pub dist: f32,
    pub children: [*mut worldSector_t; 2],
    pub entities: *mut svEntity_t,
}

const _: () = assert!(core::mem::size_of::<worldSector_t>() == 32);
const _: () = assert!(core::mem::offset_of!(worldSector_t, axis) == 0);
const _: () = assert!(core::mem::offset_of!(worldSector_t, dist) == 4);
const _: () = assert!(core::mem::offset_of!(worldSector_t, children) == 8);
const _: () = assert!(core::mem::offset_of!(worldSector_t, entities) == 24);

/// C tag `worldSector_s` is the same type as the `worldSector_t` typedef.
pub type worldSector_s = worldSector_t;

/// Raven `MAX_TOTAL_ENT_LEAFS` — max leafs an entity's bounding box can span
/// in `SV_LinkEntity`.
///
/// Type definition source: `oracle/codemp/server/sv_world.cpp:188`
pub const MAX_TOTAL_ENT_LEAFS: usize = 128;
