use core::ffi::c_void;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qhandle_t;

/// `UI_G2_SETMODELS` outbound game-to-engine syscall.
///
/// Sets the model and skin handle lists on a Ghoul2 instance.
/// Mirrors `trap_G2_SetGhoul2ModelIndexes` / `syscall!(UI_G2_SETMODELS, ghoul2, model_list, skin_list)`.
#[derive(Debug)]
pub struct UiG2SetmodelsArgs {
    /// Ghoul2 instance handle (opaque pointer).
    ghoul2: *mut c_void,
    /// Pointer to the array of model handles.
    model_list: *mut qhandle_t,
    /// Pointer to the array of skin handles.
    skin_list: *mut qhandle_t,
}

impl UiG2SetmodelsArgs {
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

/// `UI_G2_SETMODELS` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:510`
pub struct UiG2Setmodels;

impl OutboundSysCall for UiG2Setmodels {
    type Import = MpUiImport;
    type Args = UiG2SetmodelsArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_G2_SETMODELS;
}

impl EncodeSysCall for UiG2Setmodels {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2()),
            ptr_to_word(a.model_list()),
            ptr_to_word(a.skin_list()),
        ])
    }
}

impl DecodeSysCallReturn for UiG2Setmodels {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
