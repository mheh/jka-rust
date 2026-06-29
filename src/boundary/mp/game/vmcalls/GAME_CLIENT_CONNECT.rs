use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_CLIENT_CONNECT` MP game exports vmMain boundary token.
///
/// Raven: ( int clientNum, qboolean firstTime, qboolean isBot );
/// Raven: return NULL if the client is allowed to connect, otherwise return
/// Raven: a text string with the reason for denial
/// Source: `oracle/oracle/codemp/game/g_public.h:742`
pub struct GameClientConnect;

impl InboundVmCall for GameClientConnect {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_CLIENT_CONNECT;
}
