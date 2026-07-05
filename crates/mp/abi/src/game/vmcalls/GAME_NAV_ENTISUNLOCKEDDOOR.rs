use core::ffi::c_int;

use super::super::MpGameExport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

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

/// `GAME_NAV_ENTISUNLOCKEDDOOR` MP game exports vmMain ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:792`
pub struct GameNavEntIsUnlockedDoor;

impl InboundVmCall for GameNavEntIsUnlockedDoor {
    type Command = MpGameExport;
    type Args = GameNavEntIsUnlockedDoorArgs;
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_ENTISUNLOCKEDDOOR;
}

impl DecodeVmMain for GameNavEntIsUnlockedDoor {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `G_EntIsUnlockedDoor(arg0)` — g_main.c:681.
        GameNavEntIsUnlockedDoorArgs::new(word_to_c_int(t.arg(0)))
    }
}

impl EncodeVmMainReturn for GameNavEntIsUnlockedDoor {
    fn encode_return(output: Self::Output) -> isize {
        // `return G_EntIsUnlockedDoor(arg0);` — g_main.c:681. `qboolean`.
        output as isize
    }
}
