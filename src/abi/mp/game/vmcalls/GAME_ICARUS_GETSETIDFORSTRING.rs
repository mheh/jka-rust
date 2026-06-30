use core::ffi::c_int;

use super::super::MpGameExport;

use crate::abi::generic::InboundVmCall;

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
