use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// `GAME_GETITEMINDEXBYTAG` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:798`
/// Source (args): `oracle/codemp/game/g_main.c:691`
/// Source (output): `oracle/codemp/game/g_main.c:691`
/// Source (call site): no VM_Call site currently found for `GAME_GETITEMINDEXBYTAG` in tracked sources; dispatch is defined in `oracle/codemp/game/g_main.c:691`
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
    type Command = MpGameExport;
    type Args = GameGetitemindexbytagArgs;
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::GAME_GETITEMINDEXBYTAG;
}

impl DecodeVmMain for GameGetitemindexbytag {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `BG_GetItemIndexByTag(arg0, arg1)` — g_main.c:691.
        GameGetitemindexbytagArgs::new(word_to_c_int(t.arg(0)), word_to_c_int(t.arg(1)))
    }
}

impl EncodeVmMainReturn for GameGetitemindexbytag {
    fn encode_return(output: Self::Output) -> isize {
        // `return BG_GetItemIndexByTag(arg0, arg1);` — g_main.c:691.
        output as isize
    }
}
