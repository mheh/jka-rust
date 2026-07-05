use core::ffi::{c_float, c_int};

use super::super::MpGameExport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{
    word_to_c_int, word_to_const_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

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
    type Command = MpGameExport;
    type Args = GameNavClearpathtopointArgs;
    type Output = qboolean;

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_CLEARPATHTOPOINT;
}

impl DecodeVmMain for GameNavClearpathtopoint {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `NAV_ClearPathToPoint(&g_entities[arg0], (float *)arg1, (float *)arg2,
        //  (float *)arg3, arg4, arg5)` — g_main.c:673. The `float *` vectors
        // cross as real `intptr_t`-width pointers.
        GameNavClearpathtopointArgs::new(
            word_to_c_int(t.arg(0)),
            word_to_const_ptr(t.arg(1)),
            word_to_const_ptr(t.arg(2)),
            word_to_const_ptr(t.arg(3)),
            word_to_c_int(t.arg(4)),
            word_to_c_int(t.arg(5)),
        )
    }
}

impl EncodeVmMainReturn for GameNavClearpathtopoint {
    fn encode_return(output: Self::Output) -> isize {
        // `return NAV_ClearPathToPoint(...);` — g_main.c:673. `qboolean`.
        output as isize
    }
}
