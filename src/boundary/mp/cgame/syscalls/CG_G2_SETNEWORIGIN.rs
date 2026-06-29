use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_G2_SETNEWORIGIN`.
///
/// Raven wrapper: `return syscall(CG_G2_SETNEWORIGIN, ghoul2, boltIndex);`
/// Raven transport: `return G2API_SetNewOrigin(*((CGhoul2Info_v *)args[1]), args[2]);`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:965-967`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2561`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1504-1505`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetneworiginArgs {
    ghoul2: *mut c_void,
    bolt_index: c_int,
}

impl CgG2SetneworiginArgs {
    pub const fn new(ghoul2: *mut c_void, bolt_index: c_int) -> Self {
        Self { ghoul2, bolt_index }
    }
}

/// `CG_G2_SETNEWORIGIN` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:288`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:965-967`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1504-1505`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1504-1505`
pub struct CgG2Setneworigin;

impl OutboundSysCall for CgG2Setneworigin {
    type Import = MpCgameImport;
    type Args = CgG2SetneworiginArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETNEWORIGIN;
}

impl EncodeSysCall for CgG2Setneworigin {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghoul2), args.bolt_index as isize])
    }
}

impl DecodeSysCallReturn for CgG2Setneworigin {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
