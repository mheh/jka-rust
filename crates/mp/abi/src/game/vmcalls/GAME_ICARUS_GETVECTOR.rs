use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_GETVECTOR` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:784`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:639`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:642`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:927`
pub struct GameIcarusGetvector;

impl InboundVmCall for GameIcarusGetvector {
    type Command = MpGameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_GETVECTOR;
}

impl DecodeVmMain for GameIcarusGetvector {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/oracle/codemp/game/g_main.c:639`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusGetvector {
    fn encode_return(output: Self::Output) -> isize {
        // Returns the handler result. Source: `oracle/oracle/codemp/game/g_main.c:642`.
        output as isize
    }
}
