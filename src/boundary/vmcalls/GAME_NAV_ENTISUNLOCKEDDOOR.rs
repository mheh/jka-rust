use core::ffi::c_int;

use crate::ffi::types::qboolean;
use crate::ffi::GameExport;

use super::super::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_NAV_ENTISUNLOCKEDDOOR, entityNum, ...)--> jampgame
//   jampgame   --G_EntIsUnlockedDoor(entityNum)--------------------> query entity navigation state
//   jampgame   --return qboolean----------------------------------> executable
//
// `GAME_NAV_ENTISUNLOCKEDDOOR` is an inbound executable-to-game call raised when
// the engine asks whether an entity is an unlocked door for navigation purposes.

/// Arguments for `GAME_NAV_ENTISUNLOCKEDDOOR`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameNavEntIsUnlockedDoorArgs {
    entity_num: c_int,
}

impl GameNavEntIsUnlockedDoorArgs {
    pub const fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }

    pub const fn entity_num(self) -> c_int {
        self.entity_num
    }
}

/// `GAME_NAV_ENTISUNLOCKEDDOOR` checks whether an entity is an unlocked door.
pub struct GameNavEntIsUnlockedDoor;

impl InboundVmCall for GameNavEntIsUnlockedDoor {
    type Args = GameNavEntIsUnlockedDoorArgs;
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_NAV_ENTISUNLOCKEDDOOR;
}
