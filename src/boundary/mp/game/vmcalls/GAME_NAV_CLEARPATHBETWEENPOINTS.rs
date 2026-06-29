use core::ffi::{c_float, c_int};

use crate::ffi::GameExport;

use crate::boundary::generic::InboundVmCall;

/// `GAME_NAV_CLEARPATHBETWEENPOINTS` MP game exports vmMain boundary token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:790`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:676`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:676`
/// Source (call site): `oracle/oracle/codemp/server/NPCNav/gameCallbacks.cpp:18`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameNavClearpathbetweenpointsArgs {
    start: *const c_float,
    end: *const c_float,
    mins: *const c_float,
    maxs: *const c_float,
    ignore: c_int,
    clipmask: c_int,
}

impl GameNavClearpathbetweenpointsArgs {
    pub const fn new(
        start: *const c_float,
        end: *const c_float,
        mins: *const c_float,
        maxs: *const c_float,
        ignore: c_int,
        clipmask: c_int,
    ) -> Self {
        Self {
            start,
            end,
            mins,
            maxs,
            ignore,
            clipmask,
        }
    }

    pub const fn start(self) -> *const c_float {
        self.start
    }

    pub const fn end(self) -> *const c_float {
        self.end
    }

    pub const fn mins(self) -> *const c_float {
        self.mins
    }

    pub const fn maxs(self) -> *const c_float {
        self.maxs
    }

    pub const fn ignore(self) -> c_int {
        self.ignore
    }

    pub const fn clipmask(self) -> c_int {
        self.clipmask
    }
}

pub struct GameNavClearpathbetweenpoints;

impl InboundVmCall for GameNavClearpathbetweenpoints {
    type Command = GameExport;
    type Args = GameNavClearpathbetweenpointsArgs;
    type Output = c_int;

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARPATHBETWEENPOINTS;
}
