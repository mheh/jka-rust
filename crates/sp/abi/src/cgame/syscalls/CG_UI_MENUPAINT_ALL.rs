use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_UI_MENUPAINT_ALL`.
///
/// Raven wrapper: `syscall(CG_UI_MENUPAINT_ALL);`
/// Raven transport: `Menu_PaintAll(); return 0;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:613-615`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:887-889`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:887-889`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiMenupaintAllArgs;

/// `CG_UI_MENUPAINT_ALL` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:202`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:613-615`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:887-889`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:887-889`
pub struct CgUiMenupaintAll;

impl OutboundSysCall for CgUiMenupaintAll {
    type Import = SpCgameImport;
    type Args = CgUiMenupaintAllArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_MENUPAINT_ALL;
}

impl EncodeSysCall for CgUiMenupaintAll {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgUiMenupaintAll {
    fn decode_return(_word: isize) -> Self::Output {}
}
