use core::ffi::c_int;

use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::q_shared_h::playerState_t;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_LOCATE_GAME_DATA` outbound game-to-engine syscall.
///
/// Mirrors: `syscall( G_LOCATE_GAME_DATA, gEnts, numGEntities, sizeofGEntity_t, clients, sizeofGClient )`
#[derive(Debug)]
pub struct GLocateGameDataArgs {
    g_ents: *mut gentity_t,
    num_g_entities: c_int,
    sizeof_g_entity_t: c_int,
    clients: *mut playerState_t,
    sizeof_g_client: c_int,
}

impl GLocateGameDataArgs {
    pub fn new(
        g_ents: *mut gentity_t,
        num_g_entities: c_int,
        sizeof_g_entity_t: c_int,
        clients: *mut playerState_t,
        sizeof_g_client: c_int,
    ) -> Self {
        Self {
            g_ents,
            num_g_entities,
            sizeof_g_entity_t,
            clients,
            sizeof_g_client,
        }
    }

    pub fn g_ents(&self) -> *mut gentity_t {
        self.g_ents
    }

    pub fn num_g_entities(&self) -> c_int {
        self.num_g_entities
    }

    pub fn sizeof_g_entity_t(&self) -> c_int {
        self.sizeof_g_entity_t
    }

    pub fn clients(&self) -> *mut playerState_t {
        self.clients
    }

    pub fn sizeof_g_client(&self) -> c_int {
        self.sizeof_g_client
    }
}

/// `G_LOCATE_GAME_DATA` MP game imports syscall boundary token.
///
/// Raven: ( gentity_t *gEnts, int numGEntities, int sizeofGEntity_t,
/// Raven: playerState_t *clients, int sizeofGameClient );
/// Raven: the game needs to let the server system know where and how big the gentities
/// Raven: are, so it can look at them directly without going through an interface
/// Source: `oracle/oracle/codemp/game/g_public.h:145`
pub struct GLocateGameData;

impl OutboundSysCall for GLocateGameData {
    type Import = GameImport;
    type Args = GLocateGameDataArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_LOCATE_GAME_DATA;
}

impl EncodeSysCall for GLocateGameData {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.g_ents),
            a.num_g_entities as isize,
            a.sizeof_g_entity_t as isize,
            ptr_to_word(a.clients),
            a.sizeof_g_client as isize,
        ])
    }
}

impl DecodeSysCallReturn for GLocateGameData {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
