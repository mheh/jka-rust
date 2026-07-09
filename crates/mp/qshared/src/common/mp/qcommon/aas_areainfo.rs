//! MP `aas_areainfo_t` copied from Raven `codemp/game/be_aas.h`.
//!
//! Source: `oracle/codemp/game/be_aas.h:135-144`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::vec3_t;

/// Raven `aas_areainfo_t`.
///
/// Raven comment: `area info`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct aas_areainfo_t {
    pub contents: c_int,
    pub flags: c_int,
    pub presencetype: c_int,
    pub cluster: c_int,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub center: vec3_t,
}

const _: () = assert!(core::mem::size_of::<aas_areainfo_t>() == 52);
const _: () = assert!(core::mem::offset_of!(aas_areainfo_t, contents) == 0);
const _: () = assert!(core::mem::offset_of!(aas_areainfo_t, flags) == 4);
const _: () = assert!(core::mem::offset_of!(aas_areainfo_t, presencetype) == 8);
const _: () = assert!(core::mem::offset_of!(aas_areainfo_t, cluster) == 12);
const _: () = assert!(core::mem::offset_of!(aas_areainfo_t, mins) == 16);
const _: () = assert!(core::mem::offset_of!(aas_areainfo_t, maxs) == 28);
const _: () = assert!(core::mem::offset_of!(aas_areainfo_t, center) == 40);
