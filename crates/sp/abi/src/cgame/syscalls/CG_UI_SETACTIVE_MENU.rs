use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_SETACTIVE_MENU`.
///
/// Raven wrapper: `syscall(CG_UI_SETACTIVE_MENU,name);`
/// Raven transport: `UI_SetActiveMenu((const char *) VMA(1),NULL); return 0;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:558-560`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1212`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:841-843`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiSetactiveMenuArgs {
    name: *const c_char,
}

impl CgUiSetactiveMenuArgs {
    /// # Safety
    /// `name` must point to a valid NUL-terminated C string.
    pub const unsafe fn new(name: *const c_char) -> Self {
        Self { name }
    }
}

/// `CG_UI_SETACTIVE_MENU` SP cgame imports syscall ABI token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:194`
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:194`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:558-560`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1212`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:841-843`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:841-843`
pub struct CgUiSetactiveMenu;

impl OutboundSysCall for CgUiSetactiveMenu {
    type Import = SpCgameImport;
    type Args = CgUiSetactiveMenuArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_SETACTIVE_MENU;
}

impl EncodeSysCall for CgUiSetactiveMenu {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name)])
    }
}

impl DecodeSysCallReturn for CgUiSetactiveMenu {
    fn decode_return(_word: isize) -> Self::Output {}
}
