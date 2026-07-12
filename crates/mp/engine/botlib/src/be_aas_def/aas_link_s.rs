#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_link_t` — a link of an entity into an AAS area / BSP leaf list.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:50-56`
#[repr(C)]
pub struct aas_link_t {
    pub entnum: i32,
    pub areanum: i32,
    pub next_ent: *mut aas_link_t,
    pub prev_ent: *mut aas_link_t,
    pub next_area: *mut aas_link_t,
    pub prev_area: *mut aas_link_t,
}

pub type aas_link_s = aas_link_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<aas_link_t>() == 40);
    assert!(core::mem::offset_of!(aas_link_t, entnum) == 0);
    assert!(core::mem::offset_of!(aas_link_t, areanum) == 4);
    assert!(core::mem::offset_of!(aas_link_t, next_ent) == 8);
    assert!(core::mem::offset_of!(aas_link_t, prev_ent) == 16);
    assert!(core::mem::offset_of!(aas_link_t, next_area) == 24);
    assert!(core::mem::offset_of!(aas_link_t, prev_area) == 32);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<aas_link_t>() == 24);
    assert!(core::mem::offset_of!(aas_link_t, entnum) == 0);
    assert!(core::mem::offset_of!(aas_link_t, areanum) == 4);
    assert!(core::mem::offset_of!(aas_link_t, next_ent) == 8);
    assert!(core::mem::offset_of!(aas_link_t, prev_ent) == 12);
    assert!(core::mem::offset_of!(aas_link_t, next_area) == 16);
    assert!(core::mem::offset_of!(aas_link_t, prev_area) == 20);
};
