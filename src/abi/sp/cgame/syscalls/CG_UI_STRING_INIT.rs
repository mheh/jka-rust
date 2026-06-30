use super::super::SpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_UI_STRING_INIT`.
///
/// Raven wrapper: `syscall(CG_UI_STRING_INIT);`
/// Raven transport: `String_Init(); return 0;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:618-620`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:891-893`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:891-893`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiStringInitArgs;

/// `CG_UI_STRING_INIT` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:204`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:618-620`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:891-893`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:891-893`
pub struct CgUiStringInit;

impl OutboundSysCall for CgUiStringInit {
    type Import = SpCgameImport;
    type Args = CgUiStringInitArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_STRING_INIT;
}

impl EncodeSysCall for CgUiStringInit {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgUiStringInit {
    fn decode_return(_word: isize) -> Self::Output {}
}
