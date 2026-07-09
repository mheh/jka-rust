use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `CG_G2_ATTACHINSTANCETOENTNUM`.
///
/// Raven wrapper: `syscall(CG_G2_ATTACHINSTANCETOENTNUM, ghoul2, entityNum, server);`
/// Raven transport: `G2API_AttachInstanceToEntNum(*((CGhoul2Info_v *)args[1]), args[2], (qboolean)args[3]); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1060-1062`
/// Args source: `oracle/codemp/cgame/cg_local.h:2587`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1620-1623`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2AttachinstancetoentnumArgs {
    ghoul2: *mut c_void,
    entity_num: c_int,
    server: qboolean,
}

impl CgG2AttachinstancetoentnumArgs {
    pub const fn new(ghoul2: *mut c_void, entity_num: c_int, server: qboolean) -> Self {
        Self {
            ghoul2,
            entity_num,
            server,
        }
    }
}

/// `CG_G2_ATTACHINSTANCETOENTNUM` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:321`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:1060-1062`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1620-1623`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1620-1623`
pub struct CgG2Attachinstancetoentnum;

impl OutboundSysCall for CgG2Attachinstancetoentnum {
    type Import = MpCgameImport;
    type Args = CgG2AttachinstancetoentnumArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_ATTACHINSTANCETOENTNUM;
}

impl EncodeSysCall for CgG2Attachinstancetoentnum {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            args.entity_num as isize,
            args.server as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgG2Attachinstancetoentnum {
    fn decode_return(_word: isize) -> Self::Output {}
}
