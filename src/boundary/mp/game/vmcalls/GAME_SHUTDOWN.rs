use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

// Flow:
//
//   executable --vmMain(GAME_SHUTDOWN, restart, ...)--> jampgame
//   jampgame   --G_ShutdownGame(restart)-------------> shutdown game module state
//   jampgame   --return 0----------------------------> executable
//
// `GAME_SHUTDOWN` is an inbound executable-to-game call raised when the engine
// asks jampgame to release per-level module state.

/// Arguments for `GAME_SHUTDOWN`.
///
/// The live dispatcher passes arg0 through as `restart`, even though the older
/// enum comment still describes this command as `(void)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameShutdownArgs {
    restart: c_int,
}

impl GameShutdownArgs {
    pub const fn new(restart: c_int) -> Self {
        Self { restart }
    }

    pub const fn restart(self) -> c_int {
        self.restart
    }
}

/// `GAME_SHUTDOWN` releases per-level module state.
pub struct GameShutdown;

impl InboundVmCall for GameShutdown {
    type Command = GameExport;
    type Args = GameShutdownArgs;
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_SHUTDOWN;
}
