#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_areasettings_t` — per-area settings.
///
/// Type definition source: `oracle/codemp/botlib/aasfile.h:119-129`
#[repr(C)]
pub struct aas_areasettings_t {
	//could also add all kind of statistic fields
	/// contents of the area
	pub contents: i32,
	/// several area flags
	pub areaflags: i32,
	/// how a bot can be present in this area
	pub presencetype: i32,
	/// cluster the area belongs to, if negative it's a portal
	pub cluster: i32,
	/// number of the area in the cluster
	pub clusterareanum: i32,
	/// number of reachable areas from this one
	pub numreachableareas: i32,
	/// first reachable area in the reachable area index
	pub firstreachablearea: i32,
}

pub type aas_areasettings_s = aas_areasettings_t;

const _: () = assert!(core::mem::size_of::<aas_areasettings_t>() == 28);
const _: () = assert!(core::mem::offset_of!(aas_areasettings_t, contents) == 0);
const _: () = assert!(core::mem::offset_of!(aas_areasettings_t, areaflags) == 4);
const _: () = assert!(core::mem::offset_of!(aas_areasettings_t, presencetype) == 8);
const _: () = assert!(core::mem::offset_of!(aas_areasettings_t, cluster) == 12);
const _: () = assert!(core::mem::offset_of!(aas_areasettings_t, clusterareanum) == 16);
const _: () = assert!(core::mem::offset_of!(aas_areasettings_t, numreachableareas) == 20);
const _: () = assert!(core::mem::offset_of!(aas_areasettings_t, firstreachablearea) == 24);
