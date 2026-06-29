use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::abi::generic::InboundVmCall;

/// `GAME_GETITEMINDEXBYTAG` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:798`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:691`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:691`
/// Source (call site): no VM_Call site currently found for `GAME_GETITEMINDEXBYTAG` in tracked sources; dispatch is defined in `oracle/oracle/codemp/game/g_main.c:691`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameGetitemindexbytagArgs {
    tag: c_int,
    type_: c_int,
}

impl GameGetitemindexbytagArgs {
    pub const fn new(tag: c_int, type_: c_int) -> Self {
        Self { tag, type_ }
    }

    pub const fn tag(self) -> c_int {
        self.tag
    }

    pub const fn type_(self) -> c_int {
        self.type_
    }
}

pub struct GameGetitemindexbytag;

impl InboundVmCall for GameGetitemindexbytag {
    type Command = GameExport;
    type Args = GameGetitemindexbytagArgs;
    type Output = c_int;

    const COMMAND: GameExport = GameExport::GAME_GETITEMINDEXBYTAG;
}
