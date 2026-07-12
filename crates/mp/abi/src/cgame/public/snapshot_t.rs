#![allow(non_camel_case_types, non_snake_case)]

//! MP `cg_public.h` snapshot definition.

use mp_qshared::common::mp::cgame::refdef_t::MAX_MAP_AREA_BYTES;
use mp_qshared::common::mp::qcommon::{entityState_t, playerState_t};

/// Raven `MAX_ENTITIES_IN_SNAPSHOT`.
///
/// Source: `oracle/codemp/cgame/cg_public.h:13`
pub const MAX_ENTITIES_IN_SNAPSHOT: usize = 256;

/// Raven `snapshot_t` — a complete snapshot of game state at a serverTime.
///
/// Type definition source: `oracle/codemp/cgame/cg_public.h:20-36`
#[repr(C)]
pub struct snapshot_t {
    /// SNAPFLAG_RATE_DELAYED, etc
    pub snapFlags: i32,
    pub ping: i32,

    /// server time the message is valid for (in msec)
    pub serverTime: i32,

    /// portalarea visibility bits
    pub areamask: [u8; MAX_MAP_AREA_BYTES],

    /// complete information about the current player at this time
    pub ps: playerState_t,
    /// vehicle I'm riding's playerstate (if applicable) -rww
    pub vps: playerState_t,

    /// all of the entities that need to be presented
    pub numEntities: i32,
    /// at the time of this snapshot
    pub entities: [entityState_t; MAX_ENTITIES_IN_SNAPSHOT],

    /// text based server commands to execute when this
    pub numServerCommands: i32,
    /// snapshot becomes current
    pub serverCommandSequence: i32,
}

const _: () = assert!(core::mem::size_of::<snapshot_t>() == 139352);
const _: () = assert!(core::mem::offset_of!(snapshot_t, snapFlags) == 0);
const _: () = assert!(core::mem::offset_of!(snapshot_t, ping) == 4);
const _: () = assert!(core::mem::offset_of!(snapshot_t, serverTime) == 8);
const _: () = assert!(core::mem::offset_of!(snapshot_t, areamask) == 12);
const _: () = assert!(core::mem::offset_of!(snapshot_t, ps) == 44);
const _: () = assert!(core::mem::offset_of!(snapshot_t, vps) == 1596);
const _: () = assert!(core::mem::offset_of!(snapshot_t, numEntities) == 3148);
const _: () = assert!(core::mem::offset_of!(snapshot_t, entities) == 3152);
const _: () = assert!(core::mem::offset_of!(snapshot_t, numServerCommands) == 139344);
const _: () = assert!(core::mem::offset_of!(snapshot_t, serverCommandSequence) == 139348);
