use core::ffi::{c_float, c_int};

use super::super::MpGameExport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    word_to_c_int, word_to_const_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

/// `GAME_NAV_CLEARLOS` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:789`
/// Source (args): `oracle/codemp/game/g_main.c:674`
/// Source (output): `oracle/codemp/game/g_main.c:674`
/// Source (call site): `oracle/codemp/server/NPCNav/gameCallbacks.cpp:13`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameNavClearlosArgs {
    entity_num: c_int,
    end: *const c_float,
}

impl GameNavClearlosArgs {
    pub const fn new(entity_num: c_int, end: *const c_float) -> Self {
        Self { entity_num, end }
    }

    pub const fn entity_num(self) -> c_int {
        self.entity_num
    }

    pub const fn end(self) -> *const c_float {
        self.end
    }
}

pub struct GameNavClearlos;

impl InboundVmCall for GameNavClearlos {
    type Command = MpGameExport;
    type Args = GameNavClearlosArgs;
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_CLEARLOS;
}

impl DecodeVmMain for GameNavClearlos {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `NPC_ClearLOS2(&g_entities[arg0], (const float *)arg1)` — g_main.c:675.
        GameNavClearlosArgs::new(word_to_c_int(t.arg(0)), word_to_const_ptr(t.arg(1)))
    }
}

impl EncodeVmMainReturn for GameNavClearlos {
    fn encode_return(output: Self::Output) -> isize {
        // `return NPC_ClearLOS2(...);` — g_main.c:675. `qboolean`.
        output as isize
    }
}
