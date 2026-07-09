use super::super::MpGameExport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_SET` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:772`
/// Source (args): `oracle/codemp/game/g_main.c:563`
/// Source (output): `oracle/codemp/game/g_main.c:566`
/// Source (call site): `oracle/codemp/icarus/Q3_Interface.cpp:397`
pub struct GameIcarusSet;

impl InboundVmCall for GameIcarusSet {
    type Command = MpGameExport;
    type Args = ();
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_SET;
}

impl DecodeVmMain for GameIcarusSet {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/codemp/game/g_main.c:563`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusSet {
    fn encode_return(output: Self::Output) -> isize {
        // Returns the handler result. Source: `oracle/codemp/game/g_main.c:566`.
        output as isize
    }
}
