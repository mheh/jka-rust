use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_GETSTRING` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:785`
/// Source (args): `oracle/codemp/game/g_main.c:644`
/// Source (output): `oracle/codemp/game/g_main.c:647`
/// Source (call site): `oracle/codemp/icarus/Q3_Interface.cpp:941`
pub struct GameIcarusGetstring;

impl InboundVmCall for GameIcarusGetstring {
    type Command = MpGameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_GETSTRING;
}

impl DecodeVmMain for GameIcarusGetstring {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/codemp/game/g_main.c:644`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusGetstring {
    fn encode_return(output: Self::Output) -> isize {
        // Returns the handler result. Source: `oracle/codemp/game/g_main.c:647`.
        output as isize
    }
}
