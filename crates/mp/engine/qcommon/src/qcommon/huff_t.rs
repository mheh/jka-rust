#![allow(non_camel_case_types, non_snake_case)]

use super::nodetype::node_t;

/// Raven `huff_t` — adaptive Huffman coder state (tree, free-list bookkeeping,
/// and the fixed node/pointer pools it allocates from).
///
/// Type definition source: `oracle/oracle/codemp/qcommon/qcommon.h:1057-1069`
#[repr(C)]
pub struct huff_t {
	pub blocNode: i32,
	pub blocPtrs: i32,

	pub tree: *mut node_t,
	pub lhead: *mut node_t,
	pub ltail: *mut node_t,
	pub loc: [*mut node_t; 257],
	pub freelist: *mut *mut node_t,

	pub nodeList: [node_t; 768],
	pub nodePtrs: [*mut node_t; 768],
}

const _: () = assert!(core::mem::size_of::<huff_t>() == 51248);
const _: () = assert!(core::mem::offset_of!(huff_t, blocNode) == 0);
const _: () = assert!(core::mem::offset_of!(huff_t, blocPtrs) == 4);
const _: () = assert!(core::mem::offset_of!(huff_t, tree) == 8);
const _: () = assert!(core::mem::offset_of!(huff_t, lhead) == 16);
const _: () = assert!(core::mem::offset_of!(huff_t, ltail) == 24);
const _: () = assert!(core::mem::offset_of!(huff_t, loc) == 32);
const _: () = assert!(core::mem::offset_of!(huff_t, freelist) == 2088);
const _: () = assert!(core::mem::offset_of!(huff_t, nodeList) == 2096);
const _: () = assert!(core::mem::offset_of!(huff_t, nodePtrs) == 45104);
