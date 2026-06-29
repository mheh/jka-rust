use core::ffi::c_int;

use crate::ffi::types::qboolean;
use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_CHECKNODEFAILEDFORENT` MP game exports vmMain boundary token.
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
    type Command = GameExport;
    type Args = GameNavChecknodefailedforentArgs;
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_NAV_CHECKNODEFAILEDFORENT;
}
