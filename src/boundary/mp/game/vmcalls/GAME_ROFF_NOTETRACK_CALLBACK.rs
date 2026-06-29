use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_ROFF_NOTETRACK_CALLBACK` MP game exports vmMain boundary token.
///
/// Raven: int entnum, char *notetrack
/// Source: `oracle/oracle/codemp/game/g_public.h:766`
pub struct GameRoffNotetrackCallback;

impl InboundVmCall for GameRoffNotetrackCallback {
    type Command = GameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: GameExport = GameExport::GAME_ROFF_NOTETRACK_CALLBACK;
}
