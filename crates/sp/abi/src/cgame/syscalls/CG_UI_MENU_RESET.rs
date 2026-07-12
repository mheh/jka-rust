use super::super::SpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_MENU_RESET`.
///
/// Raven wrapper: `syscall(CG_UI_MENU_RESET);`
/// Raven transport: `Menu_Reset(); return 0;`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:568-570`
/// Args source: `oracle/code/cgame/cg_local.h:1209`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:849-851`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiMenuResetArgs;

/// `CG_UI_MENU_RESET` SP cgame imports syscall ABI token.
///
/// Source: `oracle/code/cgame/cg_public.h:192`
/// Enum value source: `oracle/code/cgame/cg_public.h:192`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:568-570`
/// Args source: `oracle/code/cgame/cg_local.h:1209`
/// Output source: `oracle/code/client/cl_cgame.cpp:849-851`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:849-851`
pub struct CgUiMenuReset;

impl OutboundSysCall for CgUiMenuReset {
    type Import = SpCgameImport;
    type Args = CgUiMenuResetArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_MENU_RESET;
}

impl EncodeSysCall for CgUiMenuReset {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgUiMenuReset {
    fn decode_return(_word: isize) -> Self::Output {}
}
