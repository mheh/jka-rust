#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use sp_qshared::common::sp::qcommon::player_state::playerState_t;
use sp_qshared::common::sp::renderer::refdef_t::MAX_MAP_AREA_BYTES;

/// Raven `clientSnapshot_t` — per-client server-side snapshot bookkeeping.
///
/// Type definition source: `oracle/code/server/server.h:76-87`
#[repr(C)]
pub struct clientSnapshot_t {
    pub areabytes: c_int,
    /// portalarea visibility bits
    pub areabits: [u8; MAX_MAP_AREA_BYTES],
    pub ps: playerState_t,
    pub num_entities: c_int,
    /// into the circular sv_packet_entities[]
    /// the entities MUST be in increasing state number
    /// order, otherwise the delta compression will fail
    pub first_entity: c_int,
    /// time the message was transmitted
    pub messageSent: c_int,
    /// time the message was acked
    pub messageAcked: c_int,
    /// used to rate drop packets
    pub messageSize: c_int,
}

const _: () = assert!(core::mem::size_of::<clientSnapshot_t>() == 5056);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, areabytes) == 0);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, areabits) == 4);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, ps) == 40);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, num_entities) == 5032);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, first_entity) == 5036);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, messageSent) == 5040);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, messageAcked) == 5044);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, messageSize) == 5048);
