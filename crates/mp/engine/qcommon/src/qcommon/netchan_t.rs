#![allow(non_camel_case_types, non_snake_case)]

use super::netadr_t::netadr_t;
use super::netsrc_t::netsrc_t;
use mp_qshared::shared::qboolean;

// Raven `MAX_MSGLEN` — max length of a message, which may be fragmented into
// multiple packets.
// Source: oracle/codemp/qcommon/qcommon.h:150
const MAX_MSGLEN: usize = 49152;

/// Raven `netchan_t`.
///
/// Type definition source: `oracle/codemp/qcommon/qcommon.h:163-186`
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
	pub outgoingSequence: i32,

	// incoming fragment assembly buffer
	pub fragmentSequence: i32,
	pub fragmentLength: i32,
	pub fragmentBuffer: [u8; MAX_MSGLEN],

	// outgoing fragment buffer
	// we need to space out the sending of large fragmented messages
	pub unsentFragments: qboolean,
	pub unsentFragmentStart: i32,
	pub unsentLength: i32,
	pub unsentBuffer: [u8; MAX_MSGLEN],
}

const _: () = assert!(core::mem::size_of::<netchan_t>() == 98364);
const _: () = assert!(core::mem::offset_of!(netchan_t, sock) == 0);
const _: () = assert!(core::mem::offset_of!(netchan_t, dropped) == 4);
const _: () = assert!(core::mem::offset_of!(netchan_t, remoteAddress) == 8);
const _: () = assert!(core::mem::offset_of!(netchan_t, qport) == 28);
const _: () = assert!(core::mem::offset_of!(netchan_t, incomingSequence) == 32);
const _: () = assert!(core::mem::offset_of!(netchan_t, outgoingSequence) == 36);
const _: () = assert!(core::mem::offset_of!(netchan_t, fragmentSequence) == 40);
const _: () = assert!(core::mem::offset_of!(netchan_t, fragmentLength) == 44);
const _: () = assert!(core::mem::offset_of!(netchan_t, fragmentBuffer) == 48);
const _: () = assert!(core::mem::offset_of!(netchan_t, unsentFragments) == 49200);
const _: () = assert!(core::mem::offset_of!(netchan_t, unsentFragmentStart) == 49204);
const _: () = assert!(core::mem::offset_of!(netchan_t, unsentLength) == 49208);
const _: () = assert!(core::mem::offset_of!(netchan_t, unsentBuffer) == 49212);
