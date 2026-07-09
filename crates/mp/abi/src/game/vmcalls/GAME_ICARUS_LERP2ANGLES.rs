use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_LERP2ANGLES` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:775`
/// Source (args): `oracle/codemp/game/g_main.c:587`
/// Source (output): `oracle/codemp/game/g_main.c:590`
/// Source (call site): `oracle/codemp/icarus/Q3_Interface.cpp:819`
pub struct GameIcarusLerp2Angles;

impl InboundVmCall for GameIcarusLerp2Angles {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_LERP2ANGLES;
}

impl DecodeVmMain for GameIcarusLerp2Angles {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/codemp/game/g_main.c:587`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusLerp2Angles {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/codemp/game/g_main.c:590`.
        0
    }
}
