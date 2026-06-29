use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::boundary::generic::OutboundSysCall;

/// `UI_PC_SOURCE_FILE_AND_LINE` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:213`
pub struct UiPcSourceFileAndLine;

impl OutboundSysCall for UiPcSourceFileAndLine {
    type Import = SpUiImport;
    /// SP `client/cl_ui.cpp` only declares/forwards a `UI_PC_SOURCE_FILE_AND_LINE` prototype in comments
    /// and has no active switch case for it (`oracle/oracle/code/client/cl_ui.cpp:352-353`, `443-447`).
    /// Fallback transport shape from MP parity:
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:378-379` and
    /// `oracle/oracle/codemp/client/cl_ui.cpp:1165-1166`.
    ///
    /// Args source: `(int handle, char *filename, int *line)`.
    type Args = (c_int, *mut c_char, *mut c_int);
    /// Returns `int` from botlib parser helper; fallback source:
    /// `oracle/oracle/codemp/ui/ui_syscalls.c:378-379`.
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_PC_SOURCE_FILE_AND_LINE;
}
