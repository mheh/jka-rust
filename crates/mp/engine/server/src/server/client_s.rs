#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_engine_qcommon::qcommon::net_limits::{MAX_DOWNLOAD_WINDOW, MAX_RELIABLE_COMMANDS, PACKET_BACKUP};
use mp_engine_qcommon::qcommon::netchan_t::netchan_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::shared::limits::MAX_NAME_LENGTH;
use mp_qshared::shared::{fileHandle_t, qboolean, MAX_INFO_STRING, MAX_QPATH, MAX_STRING_CHARS};

use super::client_snapshot_t::clientSnapshot_t;
use super::client_state_t::clientState_t;

// `MAX_INFO_STRING` (`q_shared.h:384`) imported from its canonical home in
// `mp_qshared::shared`.

/// Raven `client_t` — server-side per-client connection state.
///
/// Type definition source: `oracle/codemp/qcommon/../server/server.h:124-182`
#[repr(C)]
pub struct client_t {
	pub state: clientState_t,
	/// name, etc
	pub userinfo: [c_char; MAX_INFO_STRING],

	/// see if he has been sent an svc_setgame
	pub sentGamedir: qboolean,

	pub reliableCommands: [[c_char; MAX_STRING_CHARS]; MAX_RELIABLE_COMMANDS],
	/// last added reliable message, not necesarily sent or acknowledged yet
	pub reliableSequence: i32,
	/// last acknowledged reliable message
	pub reliableAcknowledge: i32,
	/// last sent reliable message, not necesarily acknowledged yet
	pub reliableSent: i32,
	pub messageAcknowledge: i32,

	/// netchan->outgoingSequence of gamestate
	pub gamestateMessageNum: i32,
	pub challenge: i32,

	pub lastUsercmd: usercmd_t,
	/// for delta compression
	pub lastMessageNum: i32,
	/// reliable client message sequence
	pub lastClientCommand: i32,
	pub lastClientCommandString: [c_char; MAX_STRING_CHARS],
	/// SV_GentityNum(clientnum)
	pub gentity: *mut sharedEntity_t,
	/// extracted from userinfo, high bits masked
	pub name: [c_char; MAX_NAME_LENGTH],

	// downloading
	/// if not empty string, we are downloading
	pub downloadName: [c_char; MAX_QPATH],
	/// file being downloaded
	pub download: fileHandle_t,
	/// total bytes (can't use EOF because of paks)
	pub downloadSize: i32,
	/// bytes sent
	pub downloadCount: i32,
	/// last block we sent to the client, awaiting ack
	pub downloadClientBlock: i32,
	/// current block number
	pub downloadCurrentBlock: i32,
	/// last block we xmited
	pub downloadXmitBlock: i32,
	/// the buffers for the download blocks
	pub downloadBlocks: [*mut u8; MAX_DOWNLOAD_WINDOW],
	pub downloadBlockSize: [i32; MAX_DOWNLOAD_WINDOW],
	/// We have sent the EOF block
	pub downloadEOF: qboolean,
	/// time we last got an ack from the client
	pub downloadSendTime: i32,

	/// frame last client usercmd message
	pub deltaMessage: i32,
	/// svs.time when another reliable command will be allowed
	pub nextReliableTime: i32,
	/// svs.time when packet was last received
	pub lastPacketTime: i32,
	/// svs.time when connection started
	pub lastConnectTime: i32,
	/// send another snapshot when svs.time >= nextSnapshotTime
	pub nextSnapshotTime: i32,
	/// true if nextSnapshotTime was set based on rate instead of snapshotMsec
	pub rateDelayed: qboolean,
	/// must timeout a few frames in a row so debugging doesn't break
	pub timeoutCount: i32,
	/// updates can be delta'd from here
	pub frames: [clientSnapshot_t; PACKET_BACKUP],
	pub ping: i32,
	/// bytes / second
	pub rate: i32,
	/// requests a snapshot every snapshotMsec unless rate choked
	pub snapshotMsec: i32,
	pub pureAuthentic: i32,
	pub netchan: netchan_t,

	/// if > svs.time && count > x, deny change -rww
	pub lastUserInfoChange: i32,
	/// allow a certain number of changes within a certain time period -rww
	pub lastUserInfoCount: i32,
}

/// Manifest alias: siblings importing the oracle tag name `client_s` resolve
/// to the typedef `client_t`.
pub type client_s = client_t;

const _: () = assert!(core::mem::size_of::<client_t>() == 332960);
const _: () = assert!(core::mem::offset_of!(client_t, state) == 0);
const _: () = assert!(core::mem::offset_of!(client_t, userinfo) == 4);
const _: () = assert!(core::mem::offset_of!(client_t, sentGamedir) == 1028);
const _: () = assert!(core::mem::offset_of!(client_t, reliableCommands) == 1032);
const _: () = assert!(core::mem::offset_of!(client_t, reliableSequence) == 132104);
const _: () = assert!(core::mem::offset_of!(client_t, reliableAcknowledge) == 132108);
const _: () = assert!(core::mem::offset_of!(client_t, reliableSent) == 132112);
const _: () = assert!(core::mem::offset_of!(client_t, messageAcknowledge) == 132116);
const _: () = assert!(core::mem::offset_of!(client_t, gamestateMessageNum) == 132120);
const _: () = assert!(core::mem::offset_of!(client_t, challenge) == 132124);
const _: () = assert!(core::mem::offset_of!(client_t, lastUsercmd) == 132128);
const _: () = assert!(core::mem::offset_of!(client_t, lastMessageNum) == 132156);
const _: () = assert!(core::mem::offset_of!(client_t, lastClientCommand) == 132160);
const _: () = assert!(core::mem::offset_of!(client_t, lastClientCommandString) == 132164);
const _: () = assert!(core::mem::offset_of!(client_t, gentity) == 133192);
const _: () = assert!(core::mem::offset_of!(client_t, name) == 133200);
const _: () = assert!(core::mem::offset_of!(client_t, downloadName) == 133232);
const _: () = assert!(core::mem::offset_of!(client_t, download) == 133296);
const _: () = assert!(core::mem::offset_of!(client_t, downloadSize) == 133300);
const _: () = assert!(core::mem::offset_of!(client_t, downloadCount) == 133304);
const _: () = assert!(core::mem::offset_of!(client_t, downloadClientBlock) == 133308);
const _: () = assert!(core::mem::offset_of!(client_t, downloadCurrentBlock) == 133312);
const _: () = assert!(core::mem::offset_of!(client_t, downloadXmitBlock) == 133316);
const _: () = assert!(core::mem::offset_of!(client_t, downloadBlocks) == 133320);
const _: () = assert!(core::mem::offset_of!(client_t, downloadBlockSize) == 133384);
const _: () = assert!(core::mem::offset_of!(client_t, downloadEOF) == 133416);
const _: () = assert!(core::mem::offset_of!(client_t, downloadSendTime) == 133420);
const _: () = assert!(core::mem::offset_of!(client_t, deltaMessage) == 133424);
const _: () = assert!(core::mem::offset_of!(client_t, nextReliableTime) == 133428);
const _: () = assert!(core::mem::offset_of!(client_t, lastPacketTime) == 133432);
const _: () = assert!(core::mem::offset_of!(client_t, lastConnectTime) == 133436);
const _: () = assert!(core::mem::offset_of!(client_t, nextSnapshotTime) == 133440);
const _: () = assert!(core::mem::offset_of!(client_t, rateDelayed) == 133444);
const _: () = assert!(core::mem::offset_of!(client_t, timeoutCount) == 133448);
const _: () = assert!(core::mem::offset_of!(client_t, frames) == 133452);
const _: () = assert!(core::mem::offset_of!(client_t, ping) == 234572);
const _: () = assert!(core::mem::offset_of!(client_t, rate) == 234576);
const _: () = assert!(core::mem::offset_of!(client_t, snapshotMsec) == 234580);
const _: () = assert!(core::mem::offset_of!(client_t, pureAuthentic) == 234584);
const _: () = assert!(core::mem::offset_of!(client_t, netchan) == 234588);
const _: () = assert!(core::mem::offset_of!(client_t, lastUserInfoChange) == 332952);
const _: () = assert!(core::mem::offset_of!(client_t, lastUserInfoCount) == 332956);
