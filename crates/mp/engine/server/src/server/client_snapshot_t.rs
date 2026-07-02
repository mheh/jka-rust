#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::refdef_t::MAX_MAP_AREA_BYTES;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;

/// Raven `clientSnapshot_t` — per-client server-side snapshot bookkeeping.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/../server/server.h:94-112`
#[repr(C)]
pub struct clientSnapshot_t {
    pub areabytes: c_int,
    /// portalarea visibility bits
    pub areabits: [u8; MAX_MAP_AREA_BYTES],
    pub ps: playerState_t,
    /// vehicle I'm riding's playerstate (if applicable) -rww
    pub vps: playerState_t,
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

const _: () = assert!(core::mem::size_of::<clientSnapshot_t>() == 3160);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, areabytes) == 0);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, areabits) == 4);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, ps) == 36);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, vps) == 1588);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, num_entities) == 3140);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, first_entity) == 3144);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, messageSent) == 3148);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, messageAcked) == 3152);
const _: () = assert!(core::mem::offset_of!(clientSnapshot_t, messageSize) == 3156);
