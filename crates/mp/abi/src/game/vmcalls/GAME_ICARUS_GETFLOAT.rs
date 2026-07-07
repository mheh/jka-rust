use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_GETFLOAT` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:783`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:634`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:637`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:912`
pub struct GameIcarusGetfloat;

impl InboundVmCall for GameIcarusGetfloat {
    type Command = MpGameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_GETFLOAT;
}

impl DecodeVmMain for GameIcarusGetfloat {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/oracle/codemp/game/g_main.c:634`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusGetfloat {
    fn encode_return(output: Self::Output) -> isize {
        // Returns the handler result. Source: `oracle/oracle/codemp/game/g_main.c:637`.
        output as isize
    }
}
