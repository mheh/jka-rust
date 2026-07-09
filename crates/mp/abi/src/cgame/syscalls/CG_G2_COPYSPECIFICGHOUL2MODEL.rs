use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_COPYSPECIFICGHOUL2MODEL`.
///
/// Raven wrapper: `syscall(CG_G2_COPYSPECIFICGHOUL2MODEL, g2From, modelFrom, g2To, modelTo);`
/// Raven transport: `G2API_CopySpecificG2Model(*((CGhoul2Info_v *)args[1]), args[2], *((CGhoul2Info_v *)args[3]), args[4]); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:895-897`
/// Args source: `oracle/codemp/cgame/cg_local.h:2530`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1422-1424`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2Copyspecificghoul2modelArgs {
    g2_from: *mut c_void,
    model_from: c_int,
    g2_to: *mut c_void,
    model_to: c_int,
}

impl CgG2Copyspecificghoul2modelArgs {
    pub const fn new(
        g2_from: *mut c_void,
        model_from: c_int,
        g2_to: *mut c_void,
        model_to: c_int,
    ) -> Self {
        Self {
            g2_from,
            model_from,
            g2_to,
            model_to,
        }
    }
}

/// `CG_G2_COPYSPECIFICGHOUL2MODEL` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:274`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:895-897`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1422-1424`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1422-1424`
pub struct CgG2Copyspecificghoul2model;

impl OutboundSysCall for CgG2Copyspecificghoul2model {
    type Import = MpCgameImport;
    type Args = CgG2Copyspecificghoul2modelArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_COPYSPECIFICGHOUL2MODEL;
}

impl EncodeSysCall for CgG2Copyspecificghoul2model {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.g2_from),
            args.model_from as isize,
            ptr_to_word(args.g2_to),
            args.model_to as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Copyspecificghoul2model {
    fn decode_return(_word: isize) -> Self::Output {}
}
