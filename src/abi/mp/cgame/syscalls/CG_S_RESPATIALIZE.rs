use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_S_RESPATIALIZE`.
///
/// Raven wrapper: `syscall( CG_S_RESPATIALIZE, entityNum, origin, axis, inwater );`
/// Raven transport: `S_Respatialize( args[1], (const float *)VMA(2), (vec3_t *)VMA(3), args[4] ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:220-221`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2230`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:834-836`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSRespatializeArgs {
    entity_num: c_int,
    origin: *const vec3_t,
    axis: *const vec3_t,
    inwater: c_int,
}

impl CgSRespatializeArgs {
    pub const fn new(
        entity_num: c_int,
        origin: *const vec3_t,
        axis: *const vec3_t,
        inwater: c_int,
    ) -> Self {
        Self {
            entity_num,
            origin,
            axis,
            inwater,
        }
    }
}

/// `CG_S_RESPATIALIZE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:104`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:220-221`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:834-836`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:834-836`
pub struct CgSRespatialize;

impl OutboundSysCall for CgSRespatialize {
    type Import = MpCgameImport;
    type Args = CgSRespatializeArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_RESPATIALIZE;
}

impl EncodeSysCall for CgSRespatialize {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.entity_num as isize,
            ptr_to_word(args.origin),
            ptr_to_word(args.axis),
            args.inwater as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSRespatialize {
    fn decode_return(_word: isize) -> Self::Output {}
}
