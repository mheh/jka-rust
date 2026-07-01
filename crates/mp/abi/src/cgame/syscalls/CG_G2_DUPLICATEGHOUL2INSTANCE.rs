use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_DUPLICATEGHOUL2INSTANCE`.
///
/// Raven wrapper: `syscall(CG_G2_DUPLICATEGHOUL2INSTANCE, g2From, g2To);`
/// Raven transport: `G2API_DuplicateGhoul2Instance(*((CGhoul2Info_v *)args[1]), (CGhoul2Info_v **)VMA(2)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:900-902`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2531`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1426-1431`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2Duplicateghoul2instanceArgs {
    g2_from: *mut c_void,
    g2_to: *mut *mut c_void,
}

impl CgG2Duplicateghoul2instanceArgs {
    pub const fn new(g2_from: *mut c_void, g2_to: *mut *mut c_void) -> Self {
        Self { g2_from, g2_to }
    }
}

/// `CG_G2_DUPLICATEGHOUL2INSTANCE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:275`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:900-902`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1426-1431`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1426-1431`
pub struct CgG2Duplicateghoul2instance;

impl OutboundSysCall for CgG2Duplicateghoul2instance {
    type Import = MpCgameImport;
    type Args = CgG2Duplicateghoul2instanceArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_DUPLICATEGHOUL2INSTANCE;
}

impl EncodeSysCall for CgG2Duplicateghoul2instance {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.g2_from),
            ptr_to_word(args.g2_to as *const _),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Duplicateghoul2instance {
    fn decode_return(_word: isize) -> Self::Output {}
}
