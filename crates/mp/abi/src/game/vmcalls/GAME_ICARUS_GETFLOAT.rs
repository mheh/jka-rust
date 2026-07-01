use core::ffi::c_int;

use super::super::MpGameExport;

use abi_transport::generic::InboundVmCall;

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
