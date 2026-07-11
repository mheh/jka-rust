#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_reachabilityareas_t` — first/count of areas reachable via a reachability.
///
/// Type definition source: `oracle/codemp/botlib/be_aas_def.h:178-181`
#[derive(Clone, Copy)]
#[repr(C)]
pub struct aas_reachabilityareas_t {
    pub firstarea: i32,
    pub numareas: i32,
}

pub type aas_reachabilityareas_s = aas_reachabilityareas_t;

const _: () = assert!(core::mem::size_of::<aas_reachabilityareas_t>() == 8);
const _: () = assert!(core::mem::offset_of!(aas_reachabilityareas_t, firstarea) == 0);
const _: () = assert!(core::mem::offset_of!(aas_reachabilityareas_t, numareas) == 4);
