#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use sp_engine_qcommon::qcommon::netadr_t::netadr_t;
use sp_engine_qcommon::qcommon::netchan_t::netchan_t;

// Raven `#define MAX_OSPATH 260` (max length of a filesystem pathname).
// Source: oracle/oracle/code/game/q_shared.h (see crates/sp/engine/qcommon/src/files/directory_t.rs)
const MAX_OSPATH: usize = 260;

/// Raven `MAX_RELIABLE_COMMANDS` — max string commands buffered for retransmit.
///
/// Source: `oracle/oracle/code/qcommon/qcommon.h`
const MAX_RELIABLE_COMMANDS: usize = 64;

/// Raven `clientConnection_t`.
///
/// Type definition source: `oracle/oracle/code/client/client.h:127-147`
#[repr(C)]
pub struct clientConnection_t {
	pub lastPacketSentTime: i32, // for retransmits
	pub lastPacketTime: i32,
	pub servername: [c_char; MAX_OSPATH], // name of server from original connect
	pub serverAddress: netadr_t,
	pub connectTime: i32,        // for connection retransmits
	pub connectPacketCount: i32, // for display on connection dialog

	pub challenge: i32, // from the server to use for connecting

	pub reliableSequence: i32,
	pub reliableAcknowledge: i32,
	pub reliableCommands: [*mut c_char; MAX_RELIABLE_COMMANDS],

	// reliable messages received from server
	pub serverCommandSequence: i32,
	pub serverCommands: [*mut c_char; MAX_RELIABLE_COMMANDS],

	// big stuff at end of structure so most offsets are 15 bits or less
	pub netchan: netchan_t,
}

const _: () = assert!(core::mem::size_of::<clientConnection_t>() == 18776);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, lastPacketSentTime) == 0);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, lastPacketTime) == 4);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, servername) == 8);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, serverAddress) == 268);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, connectTime) == 276);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, connectPacketCount) == 280);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, challenge) == 284);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, reliableSequence) == 288);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, reliableAcknowledge) == 292);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, reliableCommands) == 296);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, serverCommandSequence) == 808);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, serverCommands) == 816);
const _: () = assert!(core::mem::offset_of!(clientConnection_t, netchan) == 1328);
