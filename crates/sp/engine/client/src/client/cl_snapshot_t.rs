#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::qcommon::player_state::playerState_t;
use sp_qshared::common::sp::renderer::refdef_t::MAX_MAP_AREA_BYTES;
use sp_qshared::shared::qboolean;

/// Raven `clSnapshot_t` — a locally saved snapshot of the game state.
///
/// Type definition source: `oracle/oracle/code/client/client.h:14-33`
#[repr(C)]
pub struct clSnapshot_t {
    /// cleared if delta parsing was invalid
    pub valid: qboolean,
    /// rate delayed and dropped commands
    pub snapFlags: i32,

    /// server time the message is valid for (in msec)
    pub serverTime: i32,

    /// copied from netchan->incoming_sequence
    pub messageNum: i32,
    /// messageNum the delta is from
    pub deltaNum: i32,
    /// time from when cmdNum-1 was sent to time packet was reeceived
    pub ping: i32,
    /// portalarea visibility bits
    pub areamask: [u8; MAX_MAP_AREA_BYTES],

    /// the next cmdNum the server is expecting
    pub cmdNum: i32,
    /// complete information about the current player at this time
    pub ps: playerState_t,

    /// all of the entities that need to be presented
    pub numEntities: i32,
    /// at the time of this snapshot
    pub parseEntitiesNum: i32,

    /// execute all commands up to this before
    /// making the snapshot current
    pub serverCommandNum: i32,
}

const _: () = assert!(core::mem::size_of::<clSnapshot_t>() == 5072);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, valid) == 0);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, snapFlags) == 4);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, serverTime) == 8);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, messageNum) == 12);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, deltaNum) == 16);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, ping) == 20);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, areamask) == 24);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, cmdNum) == 56);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, ps) == 64);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, numEntities) == 5056);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, parseEntitiesNum) == 5060);
const _: () = assert!(core::mem::offset_of!(clSnapshot_t, serverCommandNum) == 5064);
