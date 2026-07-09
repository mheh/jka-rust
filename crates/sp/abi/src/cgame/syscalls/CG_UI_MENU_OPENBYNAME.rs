use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_MENU_OPENBYNAME`.
///
/// Raven wrapper: `syscall(CG_UI_MENU_OPENBYNAME,buf);`
/// Raven transport: `Menus_OpenByName((const char *) VMA(1));`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:563-565`
/// Args source: `oracle/code/cgame/cg_local.h:1211`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:845-847`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiMenuOpenbynameArgs {
    buffer: *const c_char,
}

impl CgUiMenuOpenbynameArgs {
    /// # Safety
    /// `buffer` must point to a valid NUL-terminated C string.
    pub const unsafe fn new(buffer: *const c_char) -> Self {
        Self { buffer }
    }
}

/// `CG_UI_MENU_OPENBYNAME` SP cgame imports syscall ABI token.
///
/// Source: `oracle/code/cgame/cg_public.h:195`
/// Enum value source: `oracle/code/cgame/cg_public.h:195`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:563-565`
/// Args source: `oracle/code/cgame/cg_local.h:1211`
/// Output source: `oracle/code/client/cl_cgame.cpp:845-847`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:845-847`
pub struct CgUiMenuOpenbyname;

impl OutboundSysCall for CgUiMenuOpenbyname {
    type Import = SpCgameImport;
    type Args = CgUiMenuOpenbynameArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_MENU_OPENBYNAME;
}

impl EncodeSysCall for CgUiMenuOpenbyname {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buffer)])
    }
}

impl DecodeSysCallReturn for CgUiMenuOpenbyname {
    fn decode_return(_word: isize) -> Self::Output {}
}
