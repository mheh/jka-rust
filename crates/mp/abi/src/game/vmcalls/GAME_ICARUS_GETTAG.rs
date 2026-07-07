use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_GETTAG` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:776`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:593`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:596`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:833`
pub struct GameIcarusGettag;

impl InboundVmCall for GameIcarusGettag {
    type Command = MpGameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_GETTAG;
}

impl DecodeVmMain for GameIcarusGettag {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/oracle/codemp/game/g_main.c:593`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusGettag {
    fn encode_return(output: Self::Output) -> isize {
        // Returns the handler result. Source: `oracle/oracle/codemp/game/g_main.c:596`.
        output as isize
    }
}
