use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_GETGLCONFIG`.
///
/// Raven: The `glconfig_t` will not change during the life of a cgame.
/// If it needs to change, the entire cgame will be restarted, because all the
/// `qhandle_t` are then invalid.
/// Raven wrapper: `syscall( CG_GETGLCONFIG, glconfig );`
/// Raven transport: `CL_GetGlconfig( (glconfig_t *)VMA(1) ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:461-462`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2313-2316`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:954-956`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgGetglconfigArgs {
    glconfig: *mut c_void,
}

impl CgGetglconfigArgs {
    pub const fn new(glconfig: *mut c_void) -> Self {
        Self { glconfig }
    }
}

/// `CG_GETGLCONFIG` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:179`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:461-462`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:954-956`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:954-956`
pub struct CgGetglconfig;

impl OutboundSysCall for CgGetglconfig {
    type Import = MpCgameImport;
    type Args = CgGetglconfigArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETGLCONFIG;
}

impl EncodeSysCall for CgGetglconfig {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.glconfig)])
    }
}

impl DecodeSysCallReturn for CgGetglconfig {
    fn decode_return(_word: isize) -> Self::Output {}
}
