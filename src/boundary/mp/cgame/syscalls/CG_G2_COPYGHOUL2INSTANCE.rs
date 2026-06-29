use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_COPYGHOUL2INSTANCE`.
///
/// Raven wrapper: `return syscall(CG_G2_COPYGHOUL2INSTANCE, g2From, g2To, modelIndex);`
/// Raven transport: `return (int)G2API_CopyGhoul2Instance(*((CGhoul2Info_v *)args[1]), *((CGhoul2Info_v *)args[2]), args[3]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:890-892`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2529`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1419-1420`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2Copyghoul2instanceArgs {
    g2_from: *mut c_void,
    g2_to: *mut c_void,
    model_index: c_int,
}

impl CgG2Copyghoul2instanceArgs {
    pub const fn new(g2_from: *mut c_void, g2_to: *mut c_void, model_index: c_int) -> Self {
        Self {
            g2_from,
            g2_to,
            model_index,
        }
    }
}

/// `CG_G2_COPYGHOUL2INSTANCE` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:273`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:890-892`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1419-1420`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1419-1420`
pub struct CgG2Copyghoul2instance;

impl OutboundSysCall for CgG2Copyghoul2instance {
    type Import = MpCgameImport;
    type Args = CgG2Copyghoul2instanceArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_COPYGHOUL2INSTANCE;
}

impl EncodeSysCall for CgG2Copyghoul2instance {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.g2_from),
            ptr_to_word(args.g2_to),
            args.model_index as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Copyghoul2instance {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
