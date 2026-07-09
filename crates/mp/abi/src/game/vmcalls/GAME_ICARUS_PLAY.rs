use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_PLAY` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:782`
/// Source (args): `oracle/codemp/game/g_main.c:628`
/// Source (output): `oracle/codemp/game/g_main.c:631`
/// Source (call site): `oracle/codemp/icarus/Q3_Interface.cpp:899`
pub struct GameIcarusPlay;

impl InboundVmCall for GameIcarusPlay {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_PLAY;
}

impl DecodeVmMain for GameIcarusPlay {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/codemp/game/g_main.c:628`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusPlay {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/codemp/game/g_main.c:631`.
        0
    }
}
