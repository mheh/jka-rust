use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_LERP2END` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:778`
/// Source (args): `oracle/codemp/game/g_main.c:604`
/// Source (output): `oracle/codemp/game/g_main.c:607`
/// Source (call site): `oracle/codemp/icarus/Q3_Interface.cpp:857`
pub struct GameIcarusLerp2End;

impl InboundVmCall for GameIcarusLerp2End {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_LERP2END;
}

impl DecodeVmMain for GameIcarusLerp2End {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/codemp/game/g_main.c:604`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusLerp2End {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/codemp/game/g_main.c:607`.
        0
    }
}
