use core::ffi::c_uchar;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vec3_t;

/// Arguments for `CG_CM_SNAPPVS`.
///
/// Raven wrapper: `void cgi_CM_SnapPVS(vec3_t origin,byte *buffer)`.
/// Raven transport: `CM_SnapPVS((float(*))VMA(1),(byte *) VMA(2));`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:175-177`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:547-549`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgCmSnappvsArgs {
    origin: *const vec3_t,
    buffer: *mut c_uchar,
}

impl CgCmSnappvsArgs {
    pub const fn new(origin: *const vec3_t, buffer: *mut c_uchar) -> Self {
        Self { origin, buffer }
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn buffer(&self) -> *mut c_uchar {
        self.buffer
    }
}

/// `CG_CM_SNAPPVS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:90`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:175-177`
/// Output source: `oracle/code/client/cl_cgame.cpp:547-549`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:547-549`
pub struct CgCmSnappvs;

impl OutboundSysCall for CgCmSnappvs {
    type Import = SpCgameImport;
    type Args = CgCmSnappvsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_SNAPPVS;
}

impl EncodeSysCall for CgCmSnappvs {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.origin()), ptr_to_word(args.buffer())])
    }
}

impl DecodeSysCallReturn for CgCmSnappvs {
    fn decode_return(_word: isize) -> Self::Output {}
}
