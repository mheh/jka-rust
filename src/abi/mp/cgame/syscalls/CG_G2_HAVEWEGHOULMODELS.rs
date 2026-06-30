use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_G2_HAVEWEGHOULMODELS`.
///
/// Raven wrapper: `return (qboolean)(syscall(CG_G2_HAVEWEGHOULMODELS, ghoul2));`
/// Raven transport: `return G2API_HaveWeGhoul2Models( *((CGhoul2Info_v *)args[1]) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:786-788`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1304-1305`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2HaveweghoulmodelsArgs {
    ghoul2: *mut c_void,
}

impl CgG2HaveweghoulmodelsArgs {
    pub const fn new(ghoul2: *mut c_void) -> Self {
        Self { ghoul2 }
    }
}

/// `CG_G2_HAVEWEGHOULMODELS` MP cgame imports syscall ABI token.
///
/// Raven transport: `ghoul2` is passed as a raw `args[1]` pointer word, not VMA.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:259`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:786-788`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:786-788`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1304-1305`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1304-1305`
pub struct CgG2Haveweghoulmodels;

impl OutboundSysCall for CgG2Haveweghoulmodels {
    type Import = MpCgameImport;
    type Args = CgG2HaveweghoulmodelsArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_HAVEWEGHOULMODELS;
}

impl EncodeSysCall for CgG2Haveweghoulmodels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghoul2)])
    }
}

impl DecodeSysCallReturn for CgG2Haveweghoulmodels {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
