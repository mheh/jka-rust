use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_GETVOICEVOLUME`.
///
/// Raven wrapper: `int trap_S_GetVoiceVolume(int entityNum)`.
/// Raven forwards `entityNum` as the only payload word, and the client switch
/// reads it from `args[1]` as the `s_entityWavVol` index.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2219`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:184-185`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:807-808`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgSGetvoicevolumeArgs {
    /// Entity number, read by Raven as `args[1]`.
    entity_num: c_int,
}

impl CgSGetvoicevolumeArgs {
    pub const fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }

    pub const fn entity_num(&self) -> c_int {
        self.entity_num
    }
}

/// `CG_S_GETVOICEVOLUME` MP cgame imports syscall ABI token.
///
/// Raven prototype: `int trap_S_GetVoiceVolume( int entityNum );`
/// Raven wrapper: `return syscall( CG_S_GETVOICEVOLUME, entityNum );`
/// Raven transport: `return s_entityWavVol[args[1]];`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:95`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2219`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:184-185`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2219`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:184-185`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:807-808`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:807-808`
pub struct CgSGetvoicevolume;

impl OutboundSysCall for CgSGetvoicevolume {
    type Import = MpCgameImport;
    type Args = CgSGetvoicevolumeArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_GETVOICEVOLUME;
}

impl EncodeSysCall for CgSGetvoicevolume {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.entity_num() as isize])
    }
}

impl DecodeSysCallReturn for CgSGetvoicevolume {
    // `trap_S_GetVoiceVolume` returns the int-compatible voice volume word.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
