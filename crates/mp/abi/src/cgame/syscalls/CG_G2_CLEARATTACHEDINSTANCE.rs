use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_G2_CLEARATTACHEDINSTANCE`.
///
/// Raven wrapper: `syscall(CG_G2_CLEARATTACHEDINSTANCE, entityNum);`
/// Raven transport: `G2API_ClearAttachedInstance(args[1]); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1065-1067`
/// Args source: `oracle/codemp/cgame/cg_local.h:2588`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1625-1627`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2ClearattachedinstanceArgs {
    entity_num: c_int,
}

impl CgG2ClearattachedinstanceArgs {
    pub const fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }
}

/// `CG_G2_CLEARATTACHEDINSTANCE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:322`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1065-1067`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1625-1627`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1625-1627`
pub struct CgG2Clearattachedinstance;

impl OutboundSysCall for CgG2Clearattachedinstance {
    type Import = MpCgameImport;
    type Args = CgG2ClearattachedinstanceArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_CLEARATTACHEDINSTANCE;
}

impl EncodeSysCall for CgG2Clearattachedinstance {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.entity_num as isize])
    }
}

impl DecodeSysCallReturn for CgG2Clearattachedinstance {
    fn decode_return(_word: isize) -> Self::Output {}
}
