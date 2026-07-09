use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_USE` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:779`
/// Source (args): `oracle/codemp/game/g_main.c:610`
/// Source (output): `oracle/codemp/game/g_main.c:613`
/// Source (call site): `oracle/codemp/icarus/Q3_Interface.cpp:867`
pub struct GameIcarusUse;

impl InboundVmCall for GameIcarusUse {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_USE;
}

impl DecodeVmMain for GameIcarusUse {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/codemp/game/g_main.c:610`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusUse {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/codemp/game/g_main.c:613`.
        0
    }
}
