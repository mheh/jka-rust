use core::ffi::{c_float, c_int};

use crate::ffi::GameExport;
use crate::shared::qboolean;

use crate::abi::generic::InboundVmCall;

/// `GAME_NAV_CLEARLOS` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:789`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:674`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:674`
/// Source (call site): `oracle/oracle/codemp/server/NPCNav/gameCallbacks.cpp:13`
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
    type Command = GameExport;
    type Args = GameNavClearlosArgs;
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARLOS;
}
