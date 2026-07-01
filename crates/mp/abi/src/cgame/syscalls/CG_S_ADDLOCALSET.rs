use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_S_ADDLOCALSET`.
///
/// Raven wrapper:
/// `syscall(CG_S_ADDLOCALSET, name, listener_origin, origin, entID, time)`.
/// Raven transport:
/// `S_AddLocalSet((const char *)VMA(1), (float *)VMA(2), (float *)VMA(3), args[4], args[5])`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:252-254`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2242`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:855-856`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSAddlocalsetArgs {
    name: *const c_char,
    listener_origin: *const vec3_t,
    origin: *const vec3_t,
    ent_id: c_int,
    time: c_int,
}

impl CgSAddlocalsetArgs {
    pub const fn new(
        name: *const c_char,
        listener_origin: *const vec3_t,
        origin: *const vec3_t,
        ent_id: c_int,
        time: c_int,
    ) -> Self {
        Self {
            name,
            listener_origin,
            origin,
            ent_id,
            time,
        }
    }
}

/// `CG_S_ADDLOCALSET` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:113`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:252-254`
/// Output source: `oracle/oracle/codemp/cgame/cg_local.h:2242`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:855-856`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:855-856`
pub struct CgSAddlocalset;

impl OutboundSysCall for CgSAddlocalset {
    type Import = MpCgameImport;
    type Args = CgSAddlocalsetArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_ADDLOCALSET;
}

impl EncodeSysCall for CgSAddlocalset {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.name),
            ptr_to_word(args.listener_origin),
            ptr_to_word(args.origin),
            args.ent_id as isize,
            args.time as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSAddlocalset {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
