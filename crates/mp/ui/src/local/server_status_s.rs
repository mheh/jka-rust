#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::{qboolean, qhandle_t, MAX_STRING_CHARS};

use super::pinglist_t::pinglist_t;

/// `MAX_PINGREQUESTS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:570`
const MAX_PINGREQUESTS: usize = 32;

/// `MAX_DISPLAY_SERVERS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:577`
const MAX_DISPLAY_SERVERS: usize = 2048;

/// Raven `serverStatus_t`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:660-687`
#[repr(C)]
pub struct serverStatus_t {
	pub pingList: [pinglist_t; MAX_PINGREQUESTS],
	pub numqueriedservers: i32,
	pub currentping: i32,
	pub nextpingtime: i32,
	pub maxservers: i32,
	pub refreshtime: i32,
	pub numServers: i32,
	pub sortKey: i32,
	pub sortDir: i32,
	pub lastCount: i32,
	pub refreshActive: qboolean,
	pub currentServer: i32,
	pub displayServers: [i32; MAX_DISPLAY_SERVERS],
	pub numDisplayServers: i32,
	pub numPlayersOnServers: i32,
	pub nextDisplayRefresh: i32,
	pub nextSortTime: i32,
	pub currentServerPreview: qhandle_t,
	pub currentServerCinematic: i32,
	pub motdLen: i32,
	pub motdWidth: i32,
	pub motdPaintX: i32,
	pub motdPaintX2: i32,
	pub motdOffset: i32,
	pub motdTime: i32,
	pub motd: [c_char; MAX_STRING_CHARS],
}

const _: () = assert!(core::mem::size_of::<serverStatus_t>() == 11484);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, pingList) == 0);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, numqueriedservers) == 2176);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, currentping) == 2180);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, nextpingtime) == 2184);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, maxservers) == 2188);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, refreshtime) == 2192);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, numServers) == 2196);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, sortKey) == 2200);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, sortDir) == 2204);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, lastCount) == 2208);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, refreshActive) == 2212);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, currentServer) == 2216);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, displayServers) == 2220);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, numDisplayServers) == 10412);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, numPlayersOnServers) == 10416);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, nextDisplayRefresh) == 10420);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, nextSortTime) == 10424);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, currentServerPreview) == 10428);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, currentServerCinematic) == 10432);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, motdLen) == 10436);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, motdWidth) == 10440);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, motdPaintX) == 10444);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, motdPaintX2) == 10448);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, motdOffset) == 10452);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, motdTime) == 10456);
const _: () = assert!(core::mem::offset_of!(serverStatus_t, motd) == 10460);
