use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_LERP2ORIGIN` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:774`
/// Source (args): `oracle/codemp/game/g_main.c:581`
/// Source (output): `oracle/codemp/game/g_main.c:584`
/// Source (call site): `oracle/codemp/icarus/Q3_Interface.cpp:806`
pub struct GameIcarusLerp2Origin;

impl InboundVmCall for GameIcarusLerp2Origin {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_LERP2ORIGIN;
}

impl DecodeVmMain for GameIcarusLerp2Origin {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/codemp/game/g_main.c:581`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusLerp2Origin {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/codemp/game/g_main.c:584`.
        0
    }
}
