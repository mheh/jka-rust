use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_INIT` MP game exports vmMain boundary token.
///
/// Raven: ( int levelTime, int randomSeed, int restart );
/// Raven: init and shutdown will be called every single level
/// Raven: The game should call G_GET_ENTITY_TOKEN to parse through all the
/// Raven: entity configuration text and spawn gentities.
/// Source: `oracle/oracle/codemp/game/g_public.h:735`
pub struct GameInit;

impl InboundVmCall for GameInit {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_INIT;
}
