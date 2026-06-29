use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;

/// Arguments for `CG_S_RESPATIALIZE`.
///
/// Raven wrapper: `syscall( CG_S_RESPATIALIZE, entityNum, origin, axis, inwater );`
/// Raven transport: `S_Respatialize( args[1], (const float *) VMA(2), (float(*)[3]) VMA(3), args[4] ); return 0;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:225-226`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:602-604`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSRespatializeArgs {
    entity_num: c_int,
    origin: *const vec3_t,
    axis: *const vec3_t,
    inwater: qboolean,
}

impl CgSRespatializeArgs {
    pub const fn new(
        entity_num: c_int,
        origin: *const vec3_t,
        axis: *const vec3_t,
        inwater: qboolean,
    ) -> Self {
        Self {
            entity_num,
            origin,
            axis,
            inwater,
        }
    }

    pub const fn entity_num(&self) -> c_int {
        self.entity_num
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn axis(&self) -> *const vec3_t {
        self.axis
    }

    pub const fn inwater(&self) -> qboolean {
        self.inwater
    }
}

/// `CG_S_RESPATIALIZE` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:97`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:225-226`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:602-604`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:602-604`
pub struct CgSRespatialize;

impl OutboundSysCall for CgSRespatialize {
    type Import = SpCgameImport;
    type Args = CgSRespatializeArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_RESPATIALIZE;
}

impl EncodeSysCall for CgSRespatialize {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.entity_num() as isize,
            ptr_to_word(args.origin()),
            ptr_to_word(args.axis()),
            args.inwater() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSRespatialize {
    fn decode_return(_word: isize) -> Self::Output {}
}
