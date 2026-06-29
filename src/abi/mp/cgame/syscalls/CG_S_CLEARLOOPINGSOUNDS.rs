use super::super::MpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_CLEARLOOPINGSOUNDS`.
///
/// C ABI: `void trap_S_ClearLoopingSounds(void)`.
/// Raven's wrapper sends only the syscall token, and the client switch calls
/// `S_ClearLoopingSounds()` with no decoded argument words.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:200-201`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2226`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:818-820`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgSClearloopingsoundsArgs;

impl CgSClearloopingsoundsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_S_CLEARLOOPINGSOUNDS` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_S_CLEARLOOPINGSOUNDS );`
/// Raven transport: `S_ClearLoopingSounds(); return 0;`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:99`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:200-201`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:200-201`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:818-820`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:818-820`
pub struct CgSClearloopingsounds;

impl OutboundSysCall for CgSClearloopingsounds {
    type Import = MpCgameImport;
    type Args = CgSClearloopingsoundsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_CLEARLOOPINGSOUNDS;
}

impl EncodeSysCall for CgSClearloopingsounds {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgSClearloopingsounds {
    // Raven returns 0; the C wrapper is `void`.
    fn decode_return(_word: isize) -> Self::Output {}
}
