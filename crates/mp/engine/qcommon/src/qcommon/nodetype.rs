#![allow(non_camel_case_types, non_snake_case)]

/// Raven `node_t` (struct tag `nodetype`) — a node in the adaptive Huffman
/// compression tree, doubly-linked within its weight class and linked into
/// the tree/list structures used by the Huffman coder.
///
/// Raven: tree structure / doubly-linked list / highest ranked node in block.
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:1047-1053`
#[repr(C)]
pub struct node_t {
	pub left: *mut node_t,
	pub right: *mut node_t,
	pub parent: *mut node_t, /* tree structure */
	pub next: *mut node_t,
	pub prev: *mut node_t, /* doubly-linked list */
	pub head: *mut *mut node_t, /* highest ranked node in block */
	pub weight: i32,
	pub symbol: i32,
}

pub type nodetype = node_t;

const _: () = assert!(core::mem::size_of::<node_t>() == 56);
const _: () = assert!(core::mem::offset_of!(node_t, left) == 0);
const _: () = assert!(core::mem::offset_of!(node_t, right) == 8);
const _: () = assert!(core::mem::offset_of!(node_t, parent) == 16);
const _: () = assert!(core::mem::offset_of!(node_t, next) == 24);
const _: () = assert!(core::mem::offset_of!(node_t, prev) == 32);
const _: () = assert!(core::mem::offset_of!(node_t, head) == 40);
const _: () = assert!(core::mem::offset_of!(node_t, weight) == 48);
const _: () = assert!(core::mem::offset_of!(node_t, symbol) == 52);
