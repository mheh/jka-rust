use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_REMOVE` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:781`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:622`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:625`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:887`
pub struct GameIcarusRemove;

impl InboundVmCall for GameIcarusRemove {
    type Command = MpGameExport;
    type Args = ();
    type Output = ();

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_REMOVE;
}

impl DecodeVmMain for GameIcarusRemove {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/oracle/codemp/game/g_main.c:622`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusRemove {
    fn encode_return(_output: Self::Output) -> isize {
        // `... return 0;` — Source: `oracle/oracle/codemp/game/g_main.c:625`.
        0
    }
}
