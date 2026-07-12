use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_G2_ATTACHENT`.
///
/// Raven wrapper: `return syscall(CG_G2_ATTACHENT, boltInfo, ghlInfoTo, toBoltIndex, entNum, toModelNum);`
/// Raven transport: `return G2API_AttachEnt( (int*)VMA(1), &g2[0], args[3], args[4], args[5] );`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:945-947`
/// Args source: `oracle/codemp/cgame/cg_local.h:2546`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1479-1484`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2AttachentArgs {
    bolt_info: *mut c_int,
    ghl_info_to: *mut c_void,
    to_bolt_index: c_int,
    ent_num: c_int,
    to_model_num: c_int,
}

impl CgG2AttachentArgs {
    pub const fn new(
        bolt_info: *mut c_int,
        ghl_info_to: *mut c_void,
        to_bolt_index: c_int,
        ent_num: c_int,
        to_model_num: c_int,
    ) -> Self {
        Self {
            bolt_info,
            ghl_info_to,
            to_bolt_index,
            ent_num,
            to_model_num,
        }
    }
}

/// `CG_G2_ATTACHENT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:284`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:945-947`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1479-1484`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1479-1484`
pub struct CgG2Attachent;

impl OutboundSysCall for CgG2Attachent {
    type Import = MpCgameImport;
    type Args = CgG2AttachentArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_ATTACHENT;
}

impl EncodeSysCall for CgG2Attachent {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.bolt_info),
            ptr_to_word(args.ghl_info_to),
            args.to_bolt_index as isize,
            args.ent_num as isize,
            args.to_model_num as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Attachent {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
