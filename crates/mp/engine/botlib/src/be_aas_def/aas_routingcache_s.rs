#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `CACHETYPE_PORTAL` — routing cache is for a portal.
/// Source: `oracle/codemp/botlib/be_aas_def.h:129`
pub const CACHETYPE_PORTAL: u8 = 0;

/// Raven `CACHETYPE_AREA` — routing cache is for an area.
/// Source: `oracle/codemp/botlib/be_aas_def.h:130`
pub const CACHETYPE_AREA: u8 = 1;

/// Raven `aas_routingcache_t` — a cached routing table for a portal or area.
///
/// Raven: portal or area cache.
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:133-147`
#[repr(C)]
pub struct aas_routingcache_t {
    /// portal or area cache
    pub r#type: u8,
    /// last time accessed or updated
    pub time: f32,
    /// size of the routing cache
    pub size: i32,
    /// cluster the cache is for
    pub cluster: i32,
    /// area the cache is created for
    pub areanum: i32,
    /// origin within the area
    pub origin: vec3_t,
    /// travel time to start with
    pub starttraveltime: f32,
    /// combinations of the travel flags
    pub travelflags: i32,
    pub prev: *mut aas_routingcache_t,
    pub next: *mut aas_routingcache_t,
    pub time_prev: *mut aas_routingcache_t,
    pub time_next: *mut aas_routingcache_t,
    /// reachabilities used for routing
    pub reachabilities: *mut u8,
    /// travel time for every area (variable sized)
    pub traveltimes: [u16; 1],
}

pub type aas_routingcache_s = aas_routingcache_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<aas_routingcache_t>() == 88);
    assert!(core::mem::offset_of!(aas_routingcache_t, r#type) == 0);
    assert!(core::mem::offset_of!(aas_routingcache_t, time) == 4);
    assert!(core::mem::offset_of!(aas_routingcache_t, size) == 8);
    assert!(core::mem::offset_of!(aas_routingcache_t, cluster) == 12);
    assert!(core::mem::offset_of!(aas_routingcache_t, areanum) == 16);
    assert!(core::mem::offset_of!(aas_routingcache_t, origin) == 20);
    assert!(core::mem::offset_of!(aas_routingcache_t, starttraveltime) == 32);
    assert!(core::mem::offset_of!(aas_routingcache_t, travelflags) == 36);
    assert!(core::mem::offset_of!(aas_routingcache_t, prev) == 40);
    assert!(core::mem::offset_of!(aas_routingcache_t, next) == 48);
    assert!(core::mem::offset_of!(aas_routingcache_t, time_prev) == 56);
    assert!(core::mem::offset_of!(aas_routingcache_t, time_next) == 64);
    assert!(core::mem::offset_of!(aas_routingcache_t, reachabilities) == 72);
    assert!(core::mem::offset_of!(aas_routingcache_t, traveltimes) == 80);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<aas_routingcache_t>() == 64);
    assert!(core::mem::offset_of!(aas_routingcache_t, r#type) == 0);
    assert!(core::mem::offset_of!(aas_routingcache_t, time) == 4);
    assert!(core::mem::offset_of!(aas_routingcache_t, size) == 8);
    assert!(core::mem::offset_of!(aas_routingcache_t, cluster) == 12);
    assert!(core::mem::offset_of!(aas_routingcache_t, areanum) == 16);
    assert!(core::mem::offset_of!(aas_routingcache_t, origin) == 20);
    assert!(core::mem::offset_of!(aas_routingcache_t, starttraveltime) == 32);
    assert!(core::mem::offset_of!(aas_routingcache_t, travelflags) == 36);
    assert!(core::mem::offset_of!(aas_routingcache_t, prev) == 40);
    assert!(core::mem::offset_of!(aas_routingcache_t, next) == 44);
    assert!(core::mem::offset_of!(aas_routingcache_t, time_prev) == 48);
    assert!(core::mem::offset_of!(aas_routingcache_t, time_next) == 52);
    assert!(core::mem::offset_of!(aas_routingcache_t, reachabilities) == 56);
    assert!(core::mem::offset_of!(aas_routingcache_t, traveltimes) == 60);
};
