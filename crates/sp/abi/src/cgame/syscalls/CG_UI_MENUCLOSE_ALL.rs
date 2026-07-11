use super::super::SpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_MENUCLOSE_ALL`.
///
/// Raven wrapper: `syscall(CG_UI_MENUCLOSE_ALL);`
/// Raven transport: `Menus_CloseAll(); return 0;`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:608-610`
/// Output source: `oracle/code/client/cl_cgame.cpp:883-885`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:883-885`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiMenucloseAllArgs;

/// `CG_UI_MENUCLOSE_ALL` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:203`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:608-610`
/// Output source: `oracle/code/client/cl_cgame.cpp:883-885`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:883-885`
pub struct CgUiMenucloseAll;

impl OutboundSysCall for CgUiMenucloseAll {
    type Import = SpCgameImport;
    type Args = CgUiMenucloseAllArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_MENUCLOSE_ALL;
}

impl EncodeSysCall for CgUiMenucloseAll {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgUiMenucloseAll {
    fn decode_return(_word: isize) -> Self::Output {}
}
