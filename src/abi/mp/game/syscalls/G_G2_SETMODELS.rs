use core::ffi::c_void;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;
use crate::shared::qhandle_t;

/// `G_G2_SETMODELS` outbound game-to-engine syscall.
///
/// Sets the model and skin handle lists on a Ghoul2 instance.
/// Mirrors `trap_G2_SetGhoul2ModelIndexes` / `syscall!(G_G2_SETMODELS, ghoul2, model_list, skin_list)`.
#[derive(Debug)]
pub struct GG2SetmodelsArgs {
    /// Ghoul2 instance handle (opaque pointer).
    ghoul2: *mut c_void,
    /// Pointer to the array of model handles.
    model_list: *mut qhandle_t,
    /// Pointer to the array of skin handles.
    skin_list: *mut qhandle_t,
}

impl GG2SetmodelsArgs {
    pub fn new(ghoul2: *mut c_void, model_list: *mut qhandle_t, skin_list: *mut qhandle_t) -> Self {
        Self {
            ghoul2,
            model_list,
            skin_list,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn model_list(&self) -> *mut qhandle_t {
        self.model_list
    }
    pub fn skin_list(&self) -> *mut qhandle_t {
        self.skin_list
    }
}

/// `G_G2_SETMODELS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:510`
pub struct GG2Setmodels;

impl OutboundSysCall for GG2Setmodels {
    type Import = GameImport;
    type Args = GG2SetmodelsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_SETMODELS;
}

impl EncodeSysCall for GG2Setmodels {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2()),
            ptr_to_word(a.model_list()),
            ptr_to_word(a.skin_list()),
        ])
    }
}

impl DecodeSysCallReturn for GG2Setmodels {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
