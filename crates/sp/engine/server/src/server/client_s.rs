#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_engine_qcommon::qcommon::netchan_t::netchan_t;
use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::qcommon::usercmd::usercmd_t;
use sp_qshared::shared::qboolean;

use super::client_snapshot_t::clientSnapshot_t;
use super::client_state_t::clientState_t;

/// Raven `MAX_INFO_STRING`.
///
/// Source: `oracle/oracle/code/game/q_shared.h:210`
const MAX_INFO_STRING: usize = 1024;

/// Raven `MAX_RELIABLE_COMMANDS` — max string commands buffered for retransmit.
///
/// Source: `oracle/oracle/code/qcommon/qcommon.h:125`
const MAX_RELIABLE_COMMANDS: usize = 64;

/// Raven `MAX_NAME_LENGTH` — max length of a client name.
///
/// Source: `oracle/oracle/code/game/q_shared.h:218`
const MAX_NAME_LENGTH: usize = 32;

/// Raven `PACKET_BACKUP` — number of old messages that must be kept on client
/// and server for delta compression and ping estimation.
///
/// Source: `oracle/oracle/code/qcommon/qcommon.h:117`
const PACKET_BACKUP: usize = 16;

/// Raven `client_t` — server-side per-client connection state.
///
/// Type definition source: `oracle/oracle/code/server/server.h:99-130`
#[repr(C)]
pub struct client_t {
	pub state: clientState_t,
	/// name, etc
	pub userinfo: [c_char; MAX_INFO_STRING],

	pub reliableCommands: [*mut c_char; MAX_RELIABLE_COMMANDS],
	/// last added reliable message, not necesarily sent or acknowledged yet
	pub reliableSequence: i32,
	/// last acknowledged reliable message
	pub reliableAcknowledge: i32,

	/// netchan->outgoingSequence of gamestate
	pub gamestateMessageNum: i32,

	pub lastUsercmd: usercmd_t,
	/// for delta compression
	pub lastMessageNum: i32,
	/// command number last executed
	pub cmdNum: i32,
	/// reliable client message sequence
	pub lastClientCommand: i32,
	/// SV_GentityNum(clientnum)
	pub gentity: *mut gentity_t,
	/// extracted from userinfo, high bits masked
	pub name: [c_char; MAX_NAME_LENGTH],
	/// file being downloaded
	pub download: *mut u8,
	/// total bytes (can't use EOF because of paks)
	pub downloadsize: i32,
	/// bytes sent
	pub downloadcount: i32,
	/// frame last client usercmd message
	pub deltaMessage: i32,
	/// sv.time when packet was last received
	pub lastPacketTime: i32,
	/// sv.time when connection started
	pub lastConnectTime: i32,
	/// send another snapshot when sv.time >= nextSnapshotTime
	pub nextSnapshotTime: i32,
	/// true if nextSnapshotTime was set based on rate instead of snapshotMsec
	pub rateDelayed: qboolean,
	/// true if enough pakets to pass the cl_packetdup were dropped
	pub droppedCommands: qboolean,
	/// must timeout a few frames in a row so debugging doesn't break
	pub timeoutCount: i32,
	/// updates can be delta'd from here
	pub frames: [clientSnapshot_t; PACKET_BACKUP],
	pub ping: i32,
	/// bytes / second
	pub rate: i32,
	/// requests a snapshot every snapshotMsec unless rate choked
	pub snapshotMsec: i32,
	pub netchan: netchan_t,
}

/// Manifest alias: siblings importing the oracle tag name `client_s` resolve
/// to the typedef `client_t`.
pub type client_s = client_t;

const _: () = assert!(core::mem::size_of::<client_t>() == 100048);
const _: () = assert!(core::mem::offset_of!(client_t, state) == 0);
const _: () = assert!(core::mem::offset_of!(client_t, userinfo) == 4);
const _: () = assert!(core::mem::offset_of!(client_t, reliableCommands) == 1032);
const _: () = assert!(core::mem::offset_of!(client_t, reliableSequence) == 1544);
const _: () = assert!(core::mem::offset_of!(client_t, reliableAcknowledge) == 1548);
const _: () = assert!(core::mem::offset_of!(client_t, gamestateMessageNum) == 1552);
const _: () = assert!(core::mem::offset_of!(client_t, lastUsercmd) == 1556);
const _: () = assert!(core::mem::offset_of!(client_t, lastMessageNum) == 1584);
const _: () = assert!(core::mem::offset_of!(client_t, cmdNum) == 1588);
const _: () = assert!(core::mem::offset_of!(client_t, lastClientCommand) == 1592);
const _: () = assert!(core::mem::offset_of!(client_t, gentity) == 1600);
const _: () = assert!(core::mem::offset_of!(client_t, name) == 1608);
const _: () = assert!(core::mem::offset_of!(client_t, download) == 1640);
const _: () = assert!(core::mem::offset_of!(client_t, downloadsize) == 1648);
const _: () = assert!(core::mem::offset_of!(client_t, downloadcount) == 1652);
const _: () = assert!(core::mem::offset_of!(client_t, deltaMessage) == 1656);
const _: () = assert!(core::mem::offset_of!(client_t, lastPacketTime) == 1660);
const _: () = assert!(core::mem::offset_of!(client_t, lastConnectTime) == 1664);
const _: () = assert!(core::mem::offset_of!(client_t, nextSnapshotTime) == 1668);
const _: () = assert!(core::mem::offset_of!(client_t, rateDelayed) == 1672);
const _: () = assert!(core::mem::offset_of!(client_t, droppedCommands) == 1676);
const _: () = assert!(core::mem::offset_of!(client_t, timeoutCount) == 1680);
const _: () = assert!(core::mem::offset_of!(client_t, frames) == 1688);
const _: () = assert!(core::mem::offset_of!(client_t, ping) == 82584);
const _: () = assert!(core::mem::offset_of!(client_t, rate) == 82588);
const _: () = assert!(core::mem::offset_of!(client_t, snapshotMsec) == 82592);
const _: () = assert!(core::mem::offset_of!(client_t, netchan) == 82596);
