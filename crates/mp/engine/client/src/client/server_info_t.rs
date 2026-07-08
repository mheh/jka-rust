#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::{c_char, c_int};

use mp_engine_qcommon::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::limits::MAX_NAME_LENGTH;
use mp_qshared::shared::qboolean;

/// Raven `serverInfo_t` — a server's info as shown in the server browser.
///
/// Type definition source: `oracle/oracle/codemp/client/client.h:257-288`
#[repr(C)]
pub struct serverInfo_t {
	pub adr: netadr_t,
	pub hostName: [c_char; MAX_NAME_LENGTH],
	pub mapName: [c_char; MAX_NAME_LENGTH],
	pub game: [c_char; MAX_NAME_LENGTH],

	pub netType: c_int,

	pub gameType: c_int,
	pub clients: c_int,
	pub maxClients: c_int,

	pub minPing: c_int,
	pub maxPing: c_int,

	pub ping: c_int,
	pub visible: qboolean,
	// int allowAnonymous;

	pub needPassword: qboolean,
	pub trueJedi: c_int,
	pub weaponDisable: c_int,
	pub forceDisable: c_int,
	// qboolean pure;
}

const _: () = assert!(core::mem::size_of::<serverInfo_t>() == 164);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, adr) == 0);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, hostName) == 20);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, mapName) == 52);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, game) == 84);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, netType) == 116);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, gameType) == 120);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, clients) == 124);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, maxClients) == 128);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, minPing) == 132);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, maxPing) == 136);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, ping) == 140);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, visible) == 144);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, needPassword) == 148);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, trueJedi) == 152);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, weaponDisable) == 156);
const _: () = assert!(core::mem::offset_of!(serverInfo_t, forceDisable) == 160);
