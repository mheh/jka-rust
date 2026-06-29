use core::ffi::{c_int, c_void};

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `UI_G2_ATTACHG2MODEL`.
///
/// Raven wrapper: `qboolean trap_G2API_AttachG2Model(void *ghoul2From, int modelIndexFrom,
/// void *ghoul2To, int toBoltIndex, int toModel)`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:661-663`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1421-1427`
#[derive(Debug)]
pub struct UiG2Attachg2modelArgs {
    /// Source Ghoul2 pointer transported as raw `args[1]`.
    ghoul2_from: *mut c_void,
    /// Source model index read directly from `args[2]`.
    model_index_from: c_int,
    /// Target Ghoul2 pointer transported as raw `args[3]`.
    ghoul2_to: *mut c_void,
    /// Target bolt index read directly from `args[4]`.
    to_bolt_index: c_int,
    /// Target model index read directly from `args[5]`.
    to_model: c_int,
}

impl UiG2Attachg2modelArgs {
    pub fn new(
        ghoul2_from: *mut c_void,
        model_index_from: c_int,
        ghoul2_to: *mut c_void,
        to_bolt_index: c_int,
        to_model: c_int,
    ) -> Self {
        Self {
            ghoul2_from,
            model_index_from,
            ghoul2_to,
            to_bolt_index,
            to_model,
        }
    }

    pub fn ghoul2_from(&self) -> *mut c_void {
        self.ghoul2_from
    }
    pub fn model_index_from(&self) -> c_int {
        self.model_index_from
    }
    pub fn ghoul2_to(&self) -> *mut c_void {
        self.ghoul2_to
    }
    pub fn to_bolt_index(&self) -> c_int {
        self.to_bolt_index
    }
    pub fn to_model(&self) -> c_int {
        self.to_model
    }
}

/// `UI_G2_ATTACHG2MODEL` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:188`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:188`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:661-663`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1421-1427`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1421-1427`
pub struct UiG2Attachg2model;

impl OutboundSysCall for UiG2Attachg2model {
    type Import = MpUiImport;
    type Args = UiG2Attachg2modelArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_G2_ATTACHG2MODEL;
}

impl EncodeSysCall for UiG2Attachg2model {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2_from),
            a.model_index_from as isize,
            ptr_to_word(a.ghoul2_to),
            a.to_bolt_index as isize,
            a.to_model as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiG2Attachg2model {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
