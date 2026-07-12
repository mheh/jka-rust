#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_reversedlink_t` — reverse link from an area to the reachabilities that reach it.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:163-168`
#[repr(C)]
pub struct aas_reversedlink_t {
    pub linknum: i32,                  //the aas_areareachability_t
    pub areanum: i32,                  //reachable from this area
    pub next: *mut aas_reversedlink_t, //next link
}

pub type aas_reversedlink_s = aas_reversedlink_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<aas_reversedlink_t>() == 16);
    assert!(core::mem::offset_of!(aas_reversedlink_t, linknum) == 0);
    assert!(core::mem::offset_of!(aas_reversedlink_t, areanum) == 4);
    assert!(core::mem::offset_of!(aas_reversedlink_t, next) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<aas_reversedlink_t>() == 12);
    assert!(core::mem::offset_of!(aas_reversedlink_t, linknum) == 0);
    assert!(core::mem::offset_of!(aas_reversedlink_t, areanum) == 4);
    assert!(core::mem::offset_of!(aas_reversedlink_t, next) == 8);
};
