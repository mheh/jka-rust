use core::ffi::c_int;

use super::super::MpGameExport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

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
    type Command = MpGameExport;
    type Args = GameNavEntIsRemovableUsableArgs;
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_ENTISREMOVABLEUSABLE;
}

impl DecodeVmMain for GameNavEntIsRemovableUsable {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `G_EntIsRemovableUsable(arg0)` — g_main.c:687.
        GameNavEntIsRemovableUsableArgs::new(word_to_c_int(t.arg(0)))
    }
}

impl EncodeVmMainReturn for GameNavEntIsRemovableUsable {
    fn encode_return(output: Self::Output) -> isize {
        // `return G_EntIsRemovableUsable(arg0);` — g_main.c:687. `qboolean`.
        output as isize
    }
}
