use super::super::MpCgameImport;
use core::ffi::c_int;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::shared::qboolean;

/// Arguments for `CG_GETUSERCMD`.
///
/// Raven wrapper: `qboolean trap_GetUserCmd(int cmdNumber, usercmd_t *ucmd)`.
/// Raven transport: `return CL_GetUserCmd(args[1], (struct usercmd_s *)VMA(2));`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:490-491`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:971-972`
#[derive(Debug)]
pub struct CgGetusercmdArgs {
    /// Usercmd sequence number, read by Raven as `args[1]`.
    cmd_number: c_int,
    /// Caller-owned `usercmd_t` output buffer, decoded by Raven as `VMA(2)`.
    ucmd: *mut usercmd_t,
}

impl CgGetusercmdArgs {
    /// Construct raw `trap_GetUserCmd` syscall args.
    ///
    /// # Safety
    /// `ucmd` must point to a writable `usercmd_t` slot for the duration of the
    /// syscall.
    pub const unsafe fn new(cmd_number: c_int, ucmd: *mut usercmd_t) -> Self {
        Self { cmd_number, ucmd }
    }

    pub const fn cmd_number(&self) -> c_int {
        self.cmd_number
    }

    pub const fn ucmd(&self) -> *mut usercmd_t {
        self.ucmd
    }
}

/// `CG_GETUSERCMD` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall(CG_GETUSERCMD, cmdNumber, ucmd);`
/// Raven transport: `return CL_GetUserCmd(args[1], (struct usercmd_s *)VMA(2));`
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:186`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:490-491`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:490-491`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:971-972`
pub struct CgGetusercmd;

impl OutboundSysCall for CgGetusercmd {
    type Import = MpCgameImport;
    type Args = CgGetusercmdArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETUSERCMD;
}

impl EncodeSysCall for CgGetusercmd {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.cmd_number() as isize, ptr_to_word(args.ucmd())])
    }
}

impl DecodeSysCallReturn for CgGetusercmd {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
