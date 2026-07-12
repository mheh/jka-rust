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

const _: () = assert!(core::mem::size_of::<aas_reversedlink_t>() == 16);
const _: () = assert!(core::mem::offset_of!(aas_reversedlink_t, linknum) == 0);
const _: () = assert!(core::mem::offset_of!(aas_reversedlink_t, areanum) == 4);
const _: () = assert!(core::mem::offset_of!(aas_reversedlink_t, next) == 8);
