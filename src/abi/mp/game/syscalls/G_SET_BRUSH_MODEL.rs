use core::ffi::c_char;
use std::ffi::CString;

use super::super::MpGameImport;
use crate::codemp::game::g_local::gentity_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for the `G_SET_BRUSH_MODEL` game→engine syscall.
///
/// Sets `ent`'s `mins`/`maxs` (and bmodel) from the named inline brush model
/// (e.g. `"*3"`).  Maps directly to the C ABI call:
/// `trap_SetBrushModel(gentity_t *ent, const char *name)`.
#[derive(Debug)]
pub struct GSetBrushModelArgs {
    /// Entity whose brush model is being set.
    pub ent: *mut gentity_t,
    /// Null-terminated name of the inline brush model (e.g. `"*3"`).
    pub name: CString,
}

impl GSetBrushModelArgs {
    pub fn new(ent: *mut gentity_t, name: CString) -> Self {
        Self { ent, name }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }

    pub fn name(&self) -> *const c_char {
        self.name.as_ptr()
    }
}

/// `G_SET_BRUSH_MODEL` MP game imports syscall ABI token.
///
/// Raven: ( gentity_t *ent, const char *name );
/// Raven: sets mins and maxs based on the brushmodel name
/// Source: `oracle/oracle/codemp/game/g_public.h:179`
pub struct GSetBrushModel;

impl OutboundSysCall for GSetBrushModel {
    type Import = MpGameImport;
    type Args = GSetBrushModelArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_SET_BRUSH_MODEL;
}

impl EncodeSysCall for GSetBrushModel {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent()), ptr_to_word(a.name())])
    }
}

impl DecodeSysCallReturn for GSetBrushModel {
    fn decode_return(_word: isize) -> Self::Output {}
}
