use core::ffi::{c_char, c_int};
use std::ffi::CString;

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_CM_REGISTER_TERRAIN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GCmRegisterTerrainArgs {
    config: CString,
}

impl GCmRegisterTerrainArgs {
    pub fn new(config: CString) -> Self {
        Self { config }
    }

    pub fn config(&self) -> *const c_char {
        self.config.as_ptr()
    }
}

/// `G_CM_REGISTER_TERRAIN` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:572`
pub struct GCmRegisterTerrain;

impl OutboundSysCall for GCmRegisterTerrain {
    type Import = MpGameImport;
    type Args = GCmRegisterTerrainArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_CM_REGISTER_TERRAIN;
}

impl EncodeSysCall for GCmRegisterTerrain {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.config())])
    }
}

impl DecodeSysCallReturn for GCmRegisterTerrain {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
