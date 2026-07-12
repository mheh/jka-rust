#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `aas_lreachability_t` — a temporary (loading) reachability link.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_reach.cpp:70-81`
#[repr(C)]
pub struct aas_lreachability_t {
    pub areanum: i32,
    pub facenum: i32,
    pub edgenum: i32,
    pub start: vec3_t,
    pub end: vec3_t,
    pub traveltype: i32,
    pub traveltime: u16,
    pub next: *mut aas_lreachability_t,
}

pub type aas_lreachability_s = aas_lreachability_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<aas_lreachability_t>() == 56);
    assert!(core::mem::offset_of!(aas_lreachability_t, areanum) == 0);
    assert!(core::mem::offset_of!(aas_lreachability_t, facenum) == 4);
    assert!(core::mem::offset_of!(aas_lreachability_t, edgenum) == 8);
    assert!(core::mem::offset_of!(aas_lreachability_t, start) == 12);
    assert!(core::mem::offset_of!(aas_lreachability_t, end) == 24);
    assert!(core::mem::offset_of!(aas_lreachability_t, traveltype) == 36);
    assert!(core::mem::offset_of!(aas_lreachability_t, traveltime) == 40);
    assert!(core::mem::offset_of!(aas_lreachability_t, next) == 48);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<aas_lreachability_t>() == 48);
    assert!(core::mem::offset_of!(aas_lreachability_t, areanum) == 0);
    assert!(core::mem::offset_of!(aas_lreachability_t, facenum) == 4);
    assert!(core::mem::offset_of!(aas_lreachability_t, edgenum) == 8);
    assert!(core::mem::offset_of!(aas_lreachability_t, start) == 12);
    assert!(core::mem::offset_of!(aas_lreachability_t, end) == 24);
    assert!(core::mem::offset_of!(aas_lreachability_t, traveltype) == 36);
    assert!(core::mem::offset_of!(aas_lreachability_t, traveltime) == 40);
    assert!(core::mem::offset_of!(aas_lreachability_t, next) == 44);
};
