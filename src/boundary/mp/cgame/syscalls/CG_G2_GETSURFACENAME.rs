use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_GETSURFACENAME`.
///
/// Raven wrapper: `syscall(CG_G2_GETSURFACENAME, ghoul2, surfNumber, modelIndex, fillBuf);`
/// Raven transport copies `G2API_GetSurfaceName` into the caller-provided buffer.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1080-1082`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2592`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1637-1655`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2GetsurfacenameArgs {
    ghoul2: *mut c_void,
    surf_number: c_int,
    model_index: c_int,
    fill_buf: *mut c_char,
}

impl CgG2GetsurfacenameArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        surf_number: c_int,
        model_index: c_int,
        fill_buf: *mut c_char,
    ) -> Self {
        Self {
            ghoul2,
            surf_number,
            model_index,
            fill_buf,
        }
    }
}

/// `CG_G2_GETSURFACENAME` MP cgame imports syscall boundary token.
///
/// Raven: returning a pointer across the VM caused failure, so Raven shoves data into caller storage.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:326`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1080-1082`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1637-1655`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1637-1655`
pub struct CgG2Getsurfacename;

impl OutboundSysCall for CgG2Getsurfacename {
    type Import = MpCgameImport;
    type Args = CgG2GetsurfacenameArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_GETSURFACENAME;
}

impl EncodeSysCall for CgG2Getsurfacename {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.surf_number as isize,
            args.model_index as isize,
            ptr_to_word(args.fill_buf),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Getsurfacename {
    fn decode_return(_word: isize) -> Self::Output {}
}
