#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_portal_t` — AAS portal connecting two clusters.
///
/// Type definition source: `oracle/oracle/codemp/botlib/aasfile.h:132-138`
#[repr(C)]
pub struct aas_portal_t {
	/// area that is the actual portal
	pub areanum: i32,
	/// cluster at front of portal
	pub frontcluster: i32,
	/// cluster at back of portal
	pub backcluster: i32,
	/// number of the area in the front and back cluster
	pub clusterareanum: [i32; 2],
}

pub type aas_portal_s = aas_portal_t;

const _: () = assert!(core::mem::size_of::<aas_portal_t>() == 20);
const _: () = assert!(core::mem::offset_of!(aas_portal_t, areanum) == 0);
const _: () = assert!(core::mem::offset_of!(aas_portal_t, frontcluster) == 4);
const _: () = assert!(core::mem::offset_of!(aas_portal_t, backcluster) == 8);
const _: () = assert!(core::mem::offset_of!(aas_portal_t, clusterareanum) == 12);
