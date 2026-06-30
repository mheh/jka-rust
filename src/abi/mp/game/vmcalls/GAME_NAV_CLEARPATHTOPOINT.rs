use core::ffi::{c_float, c_int};

use crate::ffi::GameExport;
use crate::shared::qboolean;

use crate::abi::generic::InboundVmCall;

/// `GAME_NAV_CLEARPATHTOPOINT` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/oracle/codemp/game/g_public.h:788`
/// Source (args): `oracle/oracle/codemp/game/g_main.c:672`
/// Source (output): `oracle/oracle/codemp/game/g_main.c:672`
/// Source (call site): `oracle/oracle/codemp/server/NPCNav/gameCallbacks.cpp:8`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameNavClearpathtopointArgs {
    entity_num: c_int,
    pmins: *const c_float,
    pmaxs: *const c_float,
    point: *const c_float,
    clipmask: c_int,
    ok_to_hit_ent_num: c_int,
}

impl GameNavClearpathtopointArgs {
    pub const fn new(
        entity_num: c_int,
        pmins: *const c_float,
        pmaxs: *const c_float,
        point: *const c_float,
        clipmask: c_int,
        ok_to_hit_ent_num: c_int,
    ) -> Self {
        Self {
            entity_num,
            pmins,
            pmaxs,
            point,
            clipmask,
            ok_to_hit_ent_num,
        }
    }

    pub const fn entity_num(self) -> c_int {
        self.entity_num
    }

    pub const fn pmins(self) -> *const c_float {
        self.pmins
    }

    pub const fn pmaxs(self) -> *const c_float {
        self.pmaxs
    }

    pub const fn point(self) -> *const c_float {
        self.point
    }

    pub const fn clipmask(self) -> c_int {
        self.clipmask
    }

    pub const fn ok_to_hit_ent_num(self) -> c_int {
        self.ok_to_hit_ent_num
    }
}

pub struct GameNavClearpathtopoint;

impl InboundVmCall for GameNavClearpathtopoint {
    type Command = GameExport;
    type Args = GameNavClearpathtopointArgs;
    type Output = qboolean;

    const COMMAND: GameExport = GameExport::GAME_NAV_CLEARPATHTOPOINT;
}
