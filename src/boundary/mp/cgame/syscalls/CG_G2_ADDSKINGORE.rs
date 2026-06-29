use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_ADDSKINGORE`.
///
/// Raven wrapper: `void trap_G2API_AddSkinGore(void *ghlInfo, SSkinGoreData *gore)`.
/// Raven transport: `syscall(CG_G2_ADDSKINGORE, ghlInfo, gore);`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:280`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:925-927`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1459-1463`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1459-1463`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2AddskingoreArgs {
    /// Raw Ghoul2 handle word, decoded by Raven as `args[1]`.
    ghl_info: *mut c_void,
    /// Opaque `SSkinGoreData*` pointer, read by Raven as raw `args[2]`.
    gore: *mut c_void,
}

impl CgG2AddskingoreArgs {
    pub const fn new(ghl_info: *mut c_void, gore: *mut c_void) -> Self {
        Self { ghl_info, gore }
    }
}

/// `CG_G2_ADDSKINGORE` MP cgame imports syscall boundary token.
///
/// Raven transport: both pointer words are passed raw through the syscall and
/// the `_G2_GORE` switch arm returns `0` after mutating gore state.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:280`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:925-927`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1459-1463`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1459-1463`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1491-1494`
pub struct CgG2Addskingore;

impl OutboundSysCall for CgG2Addskingore {
    type Import = MpCgameImport;
    type Args = CgG2AddskingoreArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_ADDSKINGORE;
}

impl EncodeSysCall for CgG2Addskingore {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info), ptr_to_word(args.gore)])
    }
}

impl DecodeSysCallReturn for CgG2Addskingore {
    fn decode_return(_word: isize) -> Self::Output {}
}
