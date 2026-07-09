use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::OutboundSysCall;

/// `UI_STRNCPY` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:241`
pub struct UiStrncpy;

impl OutboundSysCall for UiStrncpy {
    type Import = SpUiImport;
    /// Args source: `oracle/codemp/client/cl_ui.cpp:824` (`TRAP_STRNCPY`).
    /// Fallback source for SP-facing `UI_STRNCPY` is shared with MP transport ABI.
    /// Output source: `oracle/codemp/client/cl_ui.cpp:824`
    /// TODO: SP-side `UI_STRNCPY` transport is known only via fallback (`TRAP_STRNCPY`) evidence.
    /// Verify direct SP `UI_STRNCPY` evidence in `oracle/code/client/cl_ui.cpp` when available.
    type Args = (*mut c_char, *const c_char, c_int);
    /// `strncpy` returns destination pointer.
    type Output = *mut c_char;

    const IMPORT: SpUiImport = SpUiImport::UI_STRNCPY;
}
