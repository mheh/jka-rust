use core::ffi::{c_int, c_void};
use std::ffi::CString;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_GETSURFACERENDERSTATUS` outbound game-to-engine syscall.
///
/// C signature:
/// ```c
/// int trap_G2API_GetSurfaceRenderStatus(void *ghoul2, const int modelIndex, const char *surfaceName);
/// ```
#[derive(Debug)]
pub struct GG2GetsurfacerenderstatusArgs {
    ghoul2: *mut c_void,
    model_index: c_int,
    surface_name: CString,
}

impl GG2GetsurfacerenderstatusArgs {
    pub fn new(ghoul2: *mut c_void, model_index: c_int, surface_name: CString) -> Self {
        Self {
            ghoul2,
            model_index,
            surface_name,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn model_index(&self) -> c_int {
        self.model_index
    }

    pub fn surface_name(&self) -> &CString {
        &self.surface_name
    }
}

/// `G_G2_GETSURFACERENDERSTATUS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:537`
pub struct GG2Getsurfacerenderstatus;

impl OutboundSysCall for GG2Getsurfacerenderstatus {
    type Import = MpGameImport;
    type Args = GG2GetsurfacerenderstatusArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_G2_GETSURFACERENDERSTATUS;
}

impl EncodeSysCall for GG2Getsurfacerenderstatus {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            a.model_index as isize,
            ptr_to_word(a.surface_name.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for GG2Getsurfacerenderstatus {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
