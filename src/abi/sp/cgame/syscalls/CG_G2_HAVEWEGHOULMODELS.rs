use core::ffi::c_void;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_G2_HAVEWEGHOULMODELS`.
///
/// Raven wrapper: `return (qboolean)(syscall(CG_G2_HAVEWEGHOULMODELS, ghoul2));`
/// Raven transport: `return G2API_HaveWeGhoul2Models( *((CGhoul2Info_v *)VMA(1)) );`
///
/// Args source: `oracle/oracle/code/client/cl_cgame.cpp:791-792`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:791-792`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2HaveweghoulmodelsArgs {
    ghoul2: *const c_void,
}

impl CgG2HaveweghoulmodelsArgs {
    pub const fn new(ghoul2: *const c_void) -> Self {
        Self { ghoul2 }
    }
}

/// `CG_G2_HAVEWEGHOULMODELS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:174`
/// Args source: `oracle/oracle/code/client/cl_cgame.cpp:791-792`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:791-792`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:791-792`
pub struct CgG2Haveweghoulmodels;

impl OutboundSysCall for CgG2Haveweghoulmodels {
    type Import = SpCgameImport;
    type Args = CgG2HaveweghoulmodelsArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_G2_HAVEWEGHOULMODELS;
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
