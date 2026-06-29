/// `GAME_INIT` MP game exports vmMain boundary token.
///
/// Raven: ( int levelTime, int randomSeed, int restart );
/// Raven: init and shutdown will be called every single level
/// Raven: The game should call G_GET_ENTITY_TOKEN to parse through all the
/// Raven: entity configuration text and spawn gentities.
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:735`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:517`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:518`
/// Source (call site): `oracle/oracle/codemp/server/sv_game.cpp:1690`
use core::ffi::c_int;

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameInitArgs {
    level_time: c_int,
    random_seed: c_int,
    restart: c_int,
}

impl GameInitArgs {
    pub const fn new(level_time: c_int, random_seed: c_int, restart: c_int) -> Self {
        Self {
            level_time,
            random_seed,
            restart,
        }
    }

    pub const fn level_time(self) -> c_int {
        self.level_time
    }

    pub const fn random_seed(self) -> c_int {
        self.random_seed
    }

    pub const fn restart(self) -> c_int {
        self.restart
    }
}

pub struct GameInit;

impl InboundVmCall for GameInit {
    type Command = GameExport;
    type Args = GameInitArgs;
    type Output = ();

    const COMMAND: GameExport = GameExport::GAME_INIT;
}
