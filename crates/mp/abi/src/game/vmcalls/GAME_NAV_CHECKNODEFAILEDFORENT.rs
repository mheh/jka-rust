use core::ffi::c_int;

use super::super::MpGameExport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// `GAME_NAV_CHECKNODEFAILEDFORENT` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:791`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:678`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:678`
/// Source (call site): `oracle/oracle/codemp/server/NPCNav/gameCallbacks.cpp:23`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameNavChecknodefailedforentArgs {
    entity_num: c_int,
    node_num: c_int,
}

impl GameNavChecknodefailedforentArgs {
    pub const fn new(entity_num: c_int, node_num: c_int) -> Self {
        Self {
            entity_num,
            node_num,
        }
    }

    pub const fn entity_num(self) -> c_int {
        self.entity_num
    }

    pub const fn node_num(self) -> c_int {
        self.node_num
    }
}

pub struct GameNavChecknodefailedforent;

impl InboundVmCall for GameNavChecknodefailedforent {
    type Command = MpGameExport;
    type Args = GameNavChecknodefailedforentArgs;
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_CHECKNODEFAILEDFORENT;
}

impl DecodeVmMain for GameNavChecknodefailedforent {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `NAV_CheckNodeFailedForEnt(&g_entities[arg0], arg1)` — g_main.c:679.
        GameNavChecknodefailedforentArgs::new(word_to_c_int(t.arg(0)), word_to_c_int(t.arg(1)))
    }
}

impl EncodeVmMainReturn for GameNavChecknodefailedforent {
    fn encode_return(output: Self::Output) -> isize {
        // `return NAV_CheckNodeFailedForEnt(...);` — g_main.c:679. `qboolean`.
        output as isize
    }
}
