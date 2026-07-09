use core::ffi::{c_float, c_int};

use super::super::MpGameExport;

use abi_transport::generic::{
    word_to_c_int, word_to_const_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};

/// `GAME_NAV_CLEARPATHBETWEENPOINTS` MP game exports vmMain ABI token.
///
/// Source (enum): `oracle/codemp/game/g_public.h:790`
/// Source (args): `oracle/codemp/game/g_main.c:676`
/// Source (output): `oracle/codemp/game/g_main.c:676`
/// Source (call site): `oracle/codemp/server/NPCNav/gameCallbacks.cpp:18`
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
    type Command = MpGameExport;
    type Args = GameNavClearpathbetweenpointsArgs;
    type Output = c_int;

    const COMMAND: MpGameExport = MpGameExport::GAME_NAV_CLEARPATHBETWEENPOINTS;
}

impl DecodeVmMain for GameNavClearpathbetweenpoints {
    fn decode_vm_main(t: VmMainTransport) -> Self::Args {
        // `NAVNEW_ClearPathBetweenPoints((float *)arg0, (float *)arg1,
        //  (float *)arg2, (float *)arg3, arg4, arg5)` — g_main.c:677.
        GameNavClearpathbetweenpointsArgs::new(
            word_to_const_ptr(t.arg(0)),
            word_to_const_ptr(t.arg(1)),
            word_to_const_ptr(t.arg(2)),
            word_to_const_ptr(t.arg(3)),
            word_to_c_int(t.arg(4)),
            word_to_c_int(t.arg(5)),
        )
    }
}

impl EncodeVmMainReturn for GameNavClearpathbetweenpoints {
    fn encode_return(output: Self::Output) -> isize {
        // `return NAVNEW_ClearPathBetweenPoints(...);` — g_main.c:677.
        output as isize
    }
}
