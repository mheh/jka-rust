use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_MENU_NEW`.
///
/// Raven wrapper: `syscall(CG_UI_MENU_NEW,buf);`
/// Raven transport: `Menu_New((char *) VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:573-575`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1210`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:853-855`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiMenuNewArgs {
    buffer: *mut c_char,
}

impl CgUiMenuNewArgs {
    /// # Safety
    /// `buffer` must be valid for the required menu initialization use.
    pub const unsafe fn new(buffer: *mut c_char) -> Self {
        Self { buffer }
    }
}

/// `CG_UI_MENU_NEW` SP cgame imports syscall ABI token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:193`
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:193`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:573-575`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1210`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:853-855`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:853-855`
pub struct CgUiMenuNew;

impl OutboundSysCall for CgUiMenuNew {
    type Import = SpCgameImport;
    type Args = CgUiMenuNewArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_MENU_NEW;
}

impl EncodeSysCall for CgUiMenuNew {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buffer)])
    }
}

impl DecodeSysCallReturn for CgUiMenuNew {
    fn decode_return(_word: isize) -> Self::Output {}
}
