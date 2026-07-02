#![allow(non_camel_case_types, non_snake_case)]

/// Raven `aas_cluster_t` — an AAS cluster.
///
/// Type definition source: `oracle/oracle/codemp/botlib/aasfile.h:144-150`
#[repr(C)]
pub struct aas_cluster_t {
	/// number of areas in the cluster
	pub numareas: i32,
	/// number of areas with reachabilities
	pub numreachabilityareas: i32,
	/// number of cluster portals
	pub numportals: i32,
	/// first cluster portal in the index
	pub firstportal: i32,
}

pub type aas_cluster_s = aas_cluster_t;

const _: () = assert!(core::mem::size_of::<aas_cluster_t>() == 16);
const _: () = assert!(core::mem::offset_of!(aas_cluster_t, numareas) == 0);
const _: () = assert!(core::mem::offset_of!(aas_cluster_t, numreachabilityareas) == 4);
const _: () = assert!(core::mem::offset_of!(aas_cluster_t, numportals) == 8);
const _: () = assert!(core::mem::offset_of!(aas_cluster_t, firstportal) == 12);
