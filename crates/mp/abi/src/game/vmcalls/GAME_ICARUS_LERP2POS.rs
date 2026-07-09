use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_LERP2POS` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:773`
/// Source (args): `oracle/codemp/game/g_main.c:568`
/// Source (output): `oracle/codemp/game/g_main.c:576`
/// Source (call site): `oracle/codemp/icarus/Q3_Interface.cpp:786`
pub struct GameIcarusLerp2Pos;

impl InboundVmCall for GameIcarusLerp2Pos {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_LERP2POS;
}

impl DecodeVmMain for GameIcarusLerp2Pos {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/codemp/game/g_main.c:568`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusLerp2Pos {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/codemp/game/g_main.c:576`.
        0
    }
}
