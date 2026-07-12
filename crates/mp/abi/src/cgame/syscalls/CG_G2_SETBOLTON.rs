use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_SETBOLTON`.
///
/// Raven wrapper: `syscall(CG_G2_SETBOLTON, ghoul2, modelIndex, boltInfo);`
/// Raven transport: `G2API_SetBoltInfo(*((CGhoul2Info_v *)args[1]), args[2], args[3]);`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:950-952`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1486-1488`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetboltonArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    bolt_info: c_int,
}

impl CgG2SetboltonArgs {
    pub const fn new(ghoul2: *mut c_void, model_index: c_int, bolt_info: c_int) -> Self {
        Self {
            ghoul2,
            model_index,
            bolt_info,
        }
    }
}

/// `CG_G2_SETBOLTON` MP cgame imports syscall ABI token.
///
/// Raven transport: `ghoul2` is passed as a raw `args[1]` pointer word, not VMA.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:285`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:950-952`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1486-1488`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1486-1488`
pub struct CgG2Setbolton;

impl OutboundSysCall for CgG2Setbolton {
    type Import = MpCgameImport;
    type Args = CgG2SetboltonArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETBOLTON;
}

impl EncodeSysCall for CgG2Setbolton {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.model_index as isize,
            args.bolt_info as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Setbolton {
    fn decode_return(_word: isize) -> Self::Output {}
}
