#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dnode_t` — BSP tree node.
///
/// Type definition source: `oracle/code/qcommon/../qcommon/qfiles.h:470-475`
#[repr(C)]
pub struct dnode_t {
	pub planeNum: i32,
	/// negative numbers are -(leafs+1), not nodes
	pub children: [i32; 2],
	/// for frustom culling
	pub mins: [i32; 3],
	pub maxs: [i32; 3],
}

const _: () = assert!(core::mem::size_of::<dnode_t>() == 36);
const _: () = assert!(core::mem::offset_of!(dnode_t, planeNum) == 0);
const _: () = assert!(core::mem::offset_of!(dnode_t, children) == 4);
const _: () = assert!(core::mem::offset_of!(dnode_t, mins) == 12);
const _: () = assert!(core::mem::offset_of!(dnode_t, maxs) == 24);
