use core::ffi::c_int;

use super::super::MpGameExport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

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

/// `GAME_NAV_ENTISDOOR` MP game exports vmMain ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:793`
pub struct GameNavEntIsDoor;

impl InboundVmCall for GameNavEntIsDoor {
    type Command = MpGameExport;
    type Args = GameNavEntIsDoorArgs;
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_ENTISDOOR;
}

impl DecodeVmMain for GameNavEntIsDoor {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `G_EntIsDoor(arg0)` — g_main.c:683.
        GameNavEntIsDoorArgs::new(word_to_c_int(t.arg(0)))
    }
}

impl EncodeVmMainReturn for GameNavEntIsDoor {
    fn encode_return(output: Self::Output) -> isize {
        // `return G_EntIsDoor(arg0);` — g_main.c:683. `qboolean`.
        output as isize
    }
}
