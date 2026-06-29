use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_CM_TEMPCAPSULEMODEL`.
///
/// Raven wrapper: `clipHandle_t trap_CM_TempCapsuleModel(const vec3_t mins,
/// const vec3_t maxs)`. The client switch decodes both payload words through
/// `VMA` as read-only `const float *` vectors, then calls `CM_TempBoxModel`
/// with `qtrue` for the capsule flag.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:139-140`
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:787-788`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:139-140`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:787-788`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmTempcapsulemodelArgs {
    /// Minimum bounds vector, decoded by Raven as `(const float *)VMA(1)`.
    mins: *const vec3_t,
    /// Maximum bounds vector, decoded by Raven as `(const float *)VMA(2)`.
    maxs: *const vec3_t,
}

impl CgCmTempcapsulemodelArgs {
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

/// `CG_CM_TEMPCAPSULEMODEL` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall( CG_CM_TEMPCAPSULEMODEL, mins, maxs );`
/// Raven transport: `return CM_TempBoxModel( (const float *)VMA(1),
/// (const float *)VMA(2), /*int capsule*/ qtrue );`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:87`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:139-140`
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:787-788`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:139-140`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:787-788`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:787-788`
pub struct CgCmTempcapsulemodel;

impl OutboundSysCall for CgCmTempcapsulemodel {
    type Import = MpCgameImport;
    type Args = CgCmTempcapsulemodelArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_TEMPCAPSULEMODEL;
}

impl EncodeSysCall for CgCmTempcapsulemodel {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.mins()), ptr_to_word(args.maxs())])
    }
}

impl DecodeSysCallReturn for CgCmTempcapsulemodel {
    // `clipHandle_t` is an int-compatible Raven handle returned in the syscall word.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
