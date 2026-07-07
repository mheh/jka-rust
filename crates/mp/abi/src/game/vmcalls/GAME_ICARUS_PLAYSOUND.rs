use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::{DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport};

/// `GAME_ICARUS_PLAYSOUND` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:771`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:558`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:560`
/// Source (call site): `oracle/oracle/codemp/icarus/Q3_Interface.cpp:322`
pub struct GameIcarusPlaysound;

impl InboundVmCall for GameIcarusPlaysound {
    type Command = MpGameExport;
    type Args = ();
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::GAME_ICARUS_PLAYSOUND;
}

impl DecodeVmMain for GameIcarusPlaysound {
    // Payload arrives out-of-band in `gSharedBuffer`, not via vmMain arg words —
    // Source: `oracle/oracle/codemp/game/g_main.c:558`.
    fn decode_vm_main(_t: VmMainTransport) -> Self::Args {}
}

impl EncodeVmMainReturn for GameIcarusPlaysound {
    fn encode_return(output: Self::Output) -> isize {
        // Returns the handler result. Source: `oracle/oracle/codemp/game/g_main.c:560`.
        output as isize
    }
}
