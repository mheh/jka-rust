use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_R_LIGHTFORPOINT`.
///
/// Raven wrapper: `int trap_R_LightForPoint( vec3_t point, vec3_t ambientLight,
/// vec3_t directedLight, vec3_t lightDir )`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRLightforpointArgs {
    point: *const vec3_t,
    ambient_light: *mut vec3_t,
    directed_light: *mut vec3_t,
    light_dir: *mut vec3_t,
}

impl CgRLightforpointArgs {
    /// # Safety
    /// `point` must be readable; the three light buffers must be writable.
    pub const unsafe fn new(
        point: *const vec3_t,
        ambient_light: *mut vec3_t,
        directed_light: *mut vec3_t,
        light_dir: *mut vec3_t,
    ) -> Self {
        Self {
            point,
            ambient_light,
            directed_light,
            light_dir,
        }
    }

    pub const fn point(&self) -> *const vec3_t {
        self.point
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

/// `CG_R_LIGHTFORPOINT` MP cgame imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:155`
pub struct CgRLightforpoint;

impl OutboundSysCall for CgRLightforpoint {
    type Import = MpCgameImport;
    type Args = CgRLightforpointArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_LIGHTFORPOINT;
}

impl EncodeSysCall for CgRLightforpoint {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.point()),
            ptr_to_word(args.ambient_light()),
            ptr_to_word(args.directed_light()),
            ptr_to_word(args.light_dir()),
        ])
    }
}

impl DecodeSysCallReturn for CgRLightforpoint {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
