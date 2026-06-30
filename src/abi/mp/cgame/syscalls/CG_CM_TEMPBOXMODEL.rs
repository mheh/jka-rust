use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_CM_TEMPBOXMODEL`.
///
/// C ABI: `clipHandle_t trap_CM_TempBoxModel(const vec3_t mins, const vec3_t maxs)`.
/// Raven's wrapper forwards the raw `vec3_t` pointers, and the client switch
/// reads both transport words with `VMA` as `const float *` before calling
/// `CM_TempBoxModel(..., qfalse)`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:135-136`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:785-786`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmTempboxmodelArgs {
    mins: *const vec3_t,
    maxs: *const vec3_t,
}

impl CgCmTempboxmodelArgs {
    pub const fn new(mins: *const vec3_t, maxs: *const vec3_t) -> Self {
        Self { mins, maxs }
    }

    pub const fn mins(&self) -> *const vec3_t {
        self.mins
    }

    pub const fn maxs(&self) -> *const vec3_t {
        self.maxs
    }
}

/// `CG_CM_TEMPBOXMODEL` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall( CG_CM_TEMPBOXMODEL, mins, maxs );`
/// Raven transport: `return CM_TempBoxModel((const float *)VMA(1), (const float *)VMA(2), qfalse);`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:86`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:135-136`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:135-136`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:785-786`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:785-786`
pub struct CgCmTempboxmodel;

impl OutboundSysCall for CgCmTempboxmodel {
    type Import = MpCgameImport;
    type Args = CgCmTempboxmodelArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_TEMPBOXMODEL;
}

impl EncodeSysCall for CgCmTempboxmodel {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.mins()), ptr_to_word(args.maxs())])
    }
}

impl DecodeSysCallReturn for CgCmTempboxmodel {
    // `clipHandle_t` is an int-compatible Raven handle returned in the syscall word.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
