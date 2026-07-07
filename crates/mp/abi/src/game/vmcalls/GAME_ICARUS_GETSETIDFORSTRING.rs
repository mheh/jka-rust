use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_GETSETIDFORSTRING` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:787`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:665`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:668`
/// Source (call site): `oracle/oracle/codemp/icarus/GameInterface.cpp:415`
pub struct GameIcarusGetsetidforstring;

impl InboundVmCall for GameIcarusGetsetidforstring {
    type Command = MpGameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_GETSETIDFORSTRING;
}

impl DecodeVmMain for GameIcarusGetsetidforstring {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/oracle/codemp/game/g_main.c:665`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusGetsetidforstring {
    fn encode_return(output: Self::Output) -> isize {
        // Returns the handler result. Source: `oracle/oracle/codemp/game/g_main.c:668`.
        output as isize
    }
}
