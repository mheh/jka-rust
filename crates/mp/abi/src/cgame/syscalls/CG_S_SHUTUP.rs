use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_S_SHUTUP`.
///
/// Raven wrapper: `syscall(CG_S_SHUTUP, shutUpFactor);`
/// Raven transport: `s_shutUp = (qboolean)args[1]; return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:224-226`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2234`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:837-839`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSShutupArgs {
    shut_up_factor: qboolean,
}

impl CgSShutupArgs {
    pub const fn new(shut_up_factor: qboolean) -> Self {
        Self { shut_up_factor }
    }
}

/// `CG_S_SHUTUP` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:105`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:224-226`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:837-839`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:837-839`
pub struct CgSShutup;

impl OutboundSysCall for CgSShutup {
    type Import = MpCgameImport;
    type Args = CgSShutupArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_SHUTUP;
}

impl EncodeSysCall for CgSShutup {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.shut_up_factor as isize])
    }
}

impl DecodeSysCallReturn for CgSShutup {
    fn decode_return(_word: isize) -> Self::Output {}
}
