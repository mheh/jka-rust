use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vec3_t;

/// Arguments for `CG_S_ADDLOCALSET`.
///
/// Raven wrapper: `syscall( CG_S_ADDLOCALSET, name, listener_origin, origin, entID, time );`
/// Raven transport: `S_AddLocalSet((const char *) VMA(1), (float *) VMA(2), (float *) VMA(3), args[4], args[5]);`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:201-202`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:570-571`
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

    pub const fn name(&self) -> *const c_char {
        self.name
    }

    pub const fn listener_origin(&self) -> *const vec3_t {
        self.listener_origin
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn ent_id(&self) -> c_int {
        self.ent_id
    }

    pub const fn time(&self) -> c_int {
        self.time
    }
}

/// `CG_S_ADDLOCALSET` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:163`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:201-202`
/// Output source: `oracle/code/client/cl_cgame.cpp:570-571`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:570-571`
pub struct CgSAddlocalset;

impl OutboundSysCall for CgSAddlocalset {
    type Import = SpCgameImport;
    type Args = CgSAddlocalsetArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_ADDLOCALSET;
}

impl EncodeSysCall for CgSAddlocalset {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.name()),
            ptr_to_word(args.listener_origin()),
            ptr_to_word(args.origin()),
            args.ent_id() as isize,
            args.time() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSAddlocalset {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
