use core::ffi::c_int;

use crate::ffi::GameExport;
use crate::shared::qboolean;

use crate::abi::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_NAV_ENTISBREAKABLE, entityNum, ...)--> jampgame
//   jampgame   --G_EntIsBreakable(entityNum)--------------------> query entity navigation state
//   jampgame   --return qboolean-------------------------------> executable
//
// `GAME_NAV_ENTISBREAKABLE` is an inbound executable-to-game call raised when
// the engine asks whether an entity is a breakable navigation obstacle.

/// Arguments for `GAME_NAV_ENTISBREAKABLE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameNavEntIsBreakableArgs {
    entity_num: c_int,
}

impl GameNavEntIsBreakableArgs {
    pub const fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }

    pub const fn entity_num(self) -> c_int {
        self.entity_num
    }
}

/// `GAME_NAV_ENTISBREAKABLE` MP game exports vmMain ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:794`
pub struct GameNavEntIsBreakable;

impl InboundVmCall for GameNavEntIsBreakable {
    type Command = GameExport;
    type Args = GameNavEntIsBreakableArgs;
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_NAV_ENTISBREAKABLE;
}
