use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_LERP2START` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:777`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:598`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:601`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:846`
pub struct GameIcarusLerp2Start;

impl InboundVmCall for GameIcarusLerp2Start {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_LERP2START;
}

impl DecodeVmMain for GameIcarusLerp2Start {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/oracle/codemp/game/g_main.c:598`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusLerp2Start {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/oracle/codemp/game/g_main.c:601`.
        0
    }
}
