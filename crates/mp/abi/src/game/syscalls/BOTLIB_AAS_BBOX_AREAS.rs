use core::ffi::c_int;

use super::super::MpGameImport;
use mp_qshared::shared::vec3_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `BOTLIB_AAS_BBOX_AREAS`.
///
/// Queries AAS (Area Awareness System) for all areas that overlap the given
/// bounding box. `areas` is a caller-allocated out-param buffer; the engine
/// writes up to `maxareas` area numbers into it through the pointer.
#[derive(Debug)]
pub struct BotlibAasBboxAreasArgs {
    absmins: *const vec3_t,
    absmaxs: *const vec3_t,
    areas: *mut c_int,
    maxareas: c_int,
}

impl BotlibAasBboxAreasArgs {
    pub fn new(
        absmins: *const vec3_t,
        absmaxs: *const vec3_t,
        areas: *mut c_int,
        maxareas: c_int,
    ) -> Self {
        Self {
            absmins,
            absmaxs,
            areas,
            maxareas,
        }
    }

    pub const fn absmins(&self) -> *const vec3_t {
        self.absmins
    }

    pub const fn absmaxs(&self) -> *const vec3_t {
        self.absmaxs
    }

    pub const fn areas(&self) -> *mut c_int {
        self.areas
    }

    pub const fn maxareas(&self) -> c_int {
        self.maxareas
    }
}

/// `BOTLIB_AAS_BBOX_AREAS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:357`
pub struct BotlibAasBboxAreas;

impl OutboundSysCall for BotlibAasBboxAreas {
    type Import = MpGameImport;
    type Args = BotlibAasBboxAreasArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AAS_BBOX_AREAS;
}

impl EncodeSysCall for BotlibAasBboxAreas {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.absmins()),
            ptr_to_word(a.absmaxs()),
            ptr_to_word(a.areas()),
            a.maxareas() as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasBboxAreas {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
