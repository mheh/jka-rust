use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_CLEARSKINGORE`.
///
/// Raven wrapper: `void trap_G2API_ClearSkinGore(void* ghlInfo)`.
/// Raven transport: `syscall(CG_G2_CLEARSKINGORE, ghlInfo);`
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:281`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:930-932`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1465-1469`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1465-1469`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2ClearskingoreArgs {
    /// Raw Ghoul2 handle word, decoded by Raven as `args[1]`.
    ghl_info: *mut c_void,
}

impl CgG2ClearskingoreArgs {
    pub const fn new(ghl_info: *mut c_void) -> Self {
        Self { ghl_info }
    }
}

/// `CG_G2_CLEARSKINGORE` MP cgame imports syscall ABI token.
///
/// Raven transport: the pointer word is passed raw as `args[1]` and the
/// `_G2_GORE` switch arm returns `0` after clearing all skin gore state.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:281`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:930-932`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1465-1469`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1465-1469`
pub struct CgG2Clearskingore;

impl OutboundSysCall for CgG2Clearskingore {
    type Import = MpCgameImport;
    type Args = CgG2ClearskingoreArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_CLEARSKINGORE;
}

impl EncodeSysCall for CgG2Clearskingore {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info)])
    }
}

impl DecodeSysCallReturn for CgG2Clearskingore {
    fn decode_return(_word: isize) -> Self::Output {}
}
