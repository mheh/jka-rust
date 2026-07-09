use core::ffi::{c_char, c_int, c_void};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `UI_G2_GETBONEFRAME`.
///
/// Raven wrapper: `qboolean trap_G2API_GetBoneFrame(void *ghoul2, const char *boneName,
/// const int currentTime, float *currentFrame, int *modelList, const int modelIndex)`.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:564-566`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1302-1310`
#[derive(Debug)]
pub struct UiG2GetboneframeArgs {
    /// Ghoul2 instance pointer transported as raw `args[1]`.
    ghoul2: *mut c_void,
    /// Bone name decoded by Raven as `(const char*)VMA(2)`.
    bone_name: *const c_char,
    /// Current time read directly from `args[3]`.
    current_time: c_int,
    /// Out-param current frame decoded by Raven as `(float *)VMA(4)`.
    current_frame: *mut f32,
    /// Model list decoded by Raven as `(int *)VMA(5)`.
    model_list: *mut c_int,
    /// Model index used by the switch to select `g2[modelIndex]`.
    model_index: c_int,
}

impl UiG2GetboneframeArgs {
    pub fn new(
        ghoul2: *mut c_void,
        bone_name: *const c_char,
        current_time: c_int,
        current_frame: *mut f32,
        model_list: *mut c_int,
        model_index: c_int,
    ) -> Self {
        Self {
            ghoul2,
            bone_name,
            current_time,
            current_frame,
            model_list,
            model_index,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn bone_name(&self) -> *const c_char {
        self.bone_name
    }
    pub fn current_time(&self) -> c_int {
        self.current_time
    }
    pub fn current_frame(&self) -> *mut f32 {
        self.current_frame
    }
    pub fn model_list(&self) -> *mut c_int {
        self.model_list
    }
    pub fn model_index(&self) -> c_int {
        self.model_index
    }
}

/// `UI_G2_GETBONEFRAME` MP UI imports syscall ABI token.
///
/// Raven: trimmed down version of GBA, so I don't have to pass all those unused args across the VM-exe border
/// Raven switch: `//rwwFIXMEFIXME: Just make a G2API_GetBoneFrame func too. This is dirty.`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:156`
/// Enum comment source: `oracle/codemp/ui/ui_public.h:156`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:564-566`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1302-1310`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1302-1310`
pub struct UiG2Getboneframe;

impl OutboundSysCall for UiG2Getboneframe {
    type Import = MpUiImport;
    type Args = UiG2GetboneframeArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_GETBONEFRAME;
}

impl EncodeSysCall for UiG2Getboneframe {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            ptr_to_word(a.bone_name),
            a.current_time as isize,
            ptr_to_word(a.current_frame),
            ptr_to_word(a.model_list),
            a.model_index as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiG2Getboneframe {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
