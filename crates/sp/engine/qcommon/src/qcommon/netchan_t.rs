#![allow(non_camel_case_types, non_snake_case)]

use super::netadr_t::netadr_t;
use super::netsrc_t::netsrc_t;

// Raven `MAX_MSGLEN` — max length of a message, which may be fragmented into
// multiple packets.
// Source: oracle/oracle/code/qcommon/qcommon.h:155
const MAX_MSGLEN: usize = 1 * 17408;

/// Raven `netchan_t`.
///
/// Type definition source: `oracle/oracle/code/qcommon/qcommon.h:164-182`
#[repr(C)]
pub struct netchan_t {
	pub sock: netsrc_t,

	// between last packet and previous
	pub dropped: i32,

	pub remoteAddress: netadr_t,
	// qport value to write when transmitting
	pub qport: i32,

	// sequencing variables
	pub incomingSequence: i32,
	pub incomingAcknowledged: i32,

	pub outgoingSequence: i32,

	// incoming fragment assembly buffer
	pub fragmentSequence: i32,
	pub fragmentLength: i32,
	pub fragmentBuffer: [u8; MAX_MSGLEN],
}

const _: () = assert!(core::mem::size_of::<netchan_t>() == 17448);
const _: () = assert!(core::mem::offset_of!(netchan_t, sock) == 0);
const _: () = assert!(core::mem::offset_of!(netchan_t, dropped) == 4);
const _: () = assert!(core::mem::offset_of!(netchan_t, remoteAddress) == 8);
const _: () = assert!(core::mem::offset_of!(netchan_t, qport) == 16);
const _: () = assert!(core::mem::offset_of!(netchan_t, incomingSequence) == 20);
const _: () = assert!(core::mem::offset_of!(netchan_t, incomingAcknowledged) == 24);
const _: () = assert!(core::mem::offset_of!(netchan_t, outgoingSequence) == 28);
const _: () = assert!(core::mem::offset_of!(netchan_t, fragmentSequence) == 32);
const _: () = assert!(core::mem::offset_of!(netchan_t, fragmentLength) == 36);
const _: () = assert!(core::mem::offset_of!(netchan_t, fragmentBuffer) == 40);
