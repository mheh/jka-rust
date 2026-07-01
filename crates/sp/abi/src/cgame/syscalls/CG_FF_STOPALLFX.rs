use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_FF_STOPALLFX`.
///
/// Raven wrapper: `syscall( CG_FF_STOPALLFX );`
/// Raven transport: `FFFX_STOPALL;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:281-282`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:640-642`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFfStopallfxArgs;

/// `CG_FF_STOPALLFX` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:111`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:281-282`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:640-642`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:640-642`
pub struct CgFfStopallfx;

impl OutboundSysCall for CgFfStopallfx {
    type Import = SpCgameImport;
    type Args = CgFfStopallfxArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_FF_STOPALLFX;
}

impl EncodeSysCall for CgFfStopallfx {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgFfStopallfx {
    fn decode_return(_word: isize) -> Self::Output {}
}
