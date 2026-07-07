use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_SOUNDINDEX` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:786`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:659`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:662`
/// Source (call site): `oracle/oracle/codemp/icarus/GameInterface.cpp:406`
pub struct GameIcarusSoundindex;

impl InboundVmCall for GameIcarusSoundindex {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_SOUNDINDEX;
}

impl DecodeVmMain for GameIcarusSoundindex {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/oracle/codemp/game/g_main.c:659`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusSoundindex {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/oracle/codemp/game/g_main.c:662`.
        0
    }
}
