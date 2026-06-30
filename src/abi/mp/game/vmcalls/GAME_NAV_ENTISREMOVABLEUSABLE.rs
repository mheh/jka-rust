use core::ffi::c_int;

use crate::ffi::GameExport;
use crate::shared::qboolean;

use crate::abi::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_NAV_ENTISREMOVABLEUSABLE, entityNum, ...)--> jampgame
//   jampgame   --G_EntIsRemovableUsable(entityNum)-------------------> query entity navigation state
//   jampgame   --return qboolean------------------------------------> executable
//
// `GAME_NAV_ENTISREMOVABLEUSABLE` is an inbound executable-to-game call raised
// when the engine asks whether an entity is a removable usable navigation object.

/// Arguments for `GAME_NAV_ENTISREMOVABLEUSABLE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameNavEntIsRemovableUsableArgs {
    entity_num: c_int,
}

impl GameNavEntIsRemovableUsableArgs {
    pub const fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }

    pub const fn entity_num(self) -> c_int {
        self.entity_num
    }
}

/// `GAME_NAV_ENTISREMOVABLEUSABLE` MP game exports vmMain ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:795`
pub struct GameNavEntIsRemovableUsable;

impl InboundVmCall for GameNavEntIsRemovableUsable {
    type Command = GameExport;
    type Args = GameNavEntIsRemovableUsableArgs;
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_NAV_ENTISREMOVABLEUSABLE;
}
