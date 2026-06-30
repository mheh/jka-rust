use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_R_GETLIGHTING`.
///
/// Raven wrapper: `syscall( CG_R_GETLIGHTING, origin, ambientLight, directedLight, ligthDir );`
/// Raven transport: `return re.GetLighting((const float *) VMA(1), (float *) VMA(2), (float *) VMA(3), (float *) VMA(4));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:376-377`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:696-697`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRGetlightingArgs {
    origin: *const vec3_t,
    ambient_light: *mut vec3_t,
    directed_light: *mut vec3_t,
    light_dir: *mut vec3_t,
}

impl CgRGetlightingArgs {
    pub const fn new(
        origin: *const vec3_t,
        ambient_light: *mut vec3_t,
        directed_light: *mut vec3_t,
        light_dir: *mut vec3_t,
    ) -> Self {
        Self {
            origin,
            ambient_light,
            directed_light,
            light_dir,
        }
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn ambient_light(&self) -> *mut vec3_t {
        self.ambient_light
    }

    pub const fn directed_light(&self) -> *mut vec3_t {
        self.directed_light
    }

    pub const fn light_dir(&self) -> *mut vec3_t {
        self.light_dir
    }
}

/// `CG_R_GETLIGHTING` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:136`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:376-377`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:696-697`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:696-697`
pub struct CgRGetlighting;

impl OutboundSysCall for CgRGetlighting {
    type Import = SpCgameImport;
    type Args = CgRGetlightingArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_GETLIGHTING;
}

impl EncodeSysCall for CgRGetlighting {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.origin()),
            ptr_to_word(args.ambient_light()),
            ptr_to_word(args.directed_light()),
            ptr_to_word(args.light_dir()),
        ])
    }
}

impl DecodeSysCallReturn for CgRGetlighting {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
