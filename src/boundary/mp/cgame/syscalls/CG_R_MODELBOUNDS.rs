use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_R_MODELBOUNDS`.
///
/// Raven wrapper: `syscall( CG_R_MODELBOUNDS, model, mins, maxs );`
/// Raven transport: `re.ModelBounds( args[1], (float *)VMA(2), (float *)VMA(3) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:373-374`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2280`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:931-933`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRModelboundsArgs {
    model: c_int,
    mins: *mut vec3_t,
    maxs: *mut vec3_t,
}

impl CgRModelboundsArgs {
    pub const fn new(model: c_int, mins: *mut vec3_t, maxs: *mut vec3_t) -> Self {
        Self { model, mins, maxs }
    }
}

/// `CG_R_MODELBOUNDS` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:161`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:373-374`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:931-933`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:931-933`
pub struct CgRModelbounds;

impl OutboundSysCall for CgRModelbounds {
    type Import = MpCgameImport;
    type Args = CgRModelboundsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_MODELBOUNDS;
}

impl EncodeSysCall for CgRModelbounds {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.model as isize,
            ptr_to_word(args.mins),
            ptr_to_word(args.maxs),
        ])
    }
}

impl DecodeSysCallReturn for CgRModelbounds {
    fn decode_return(_word: isize) -> Self::Output {}
}
