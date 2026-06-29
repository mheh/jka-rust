use core::ffi::c_int;

use crate::ffi::types::qboolean;
use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_NAV_ENTISDOOR, entityNum, ...)--> jampgame
//   jampgame   --G_EntIsDoor(entityNum)---------------------> query entity navigation state
//   jampgame   --return qboolean---------------------------> executable
//
// `GAME_NAV_ENTISDOOR` is an inbound executable-to-game call raised when the
// engine asks whether an entity is a door for navigation purposes.

/// Arguments for `GAME_NAV_ENTISDOOR`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameNavEntIsDoorArgs {
    entity_num: c_int,
}

impl GameNavEntIsDoorArgs {
    pub const fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }

    pub const fn entity_num(self) -> c_int {
        self.entity_num
    }
}

/// `GAME_NAV_ENTISDOOR` MP game exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:793`
pub struct GameNavEntIsDoor;

impl InboundVmCall for GameNavEntIsDoor {
    type Command = GameExport;
    type Args = GameNavEntIsDoorArgs;
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_NAV_ENTISDOOR;
}
