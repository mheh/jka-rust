use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_UI_PARSE_STRING` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:197`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:583-585`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:861-863`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:861-863`
///
/// TODO: Port args — ambiguous ABI shape between
/// `cgi_UI_Parse_String(char *buf)` (`cg_syscalls.cpp:583-585`) and
/// `(const char **) VMA(1)` (`cl_cgame.cpp:861-863`).
///
/// This should be resolved as either `*mut c_char` (wrapper shape) or
/// `*mut *const c_char` (transport shape) depending on intended ownership.
///
/// Raven note: `const char **` is likely required by `PC_ParseString(const char **string)`.
pub struct CgUiParseString;

impl OutboundSysCall for CgUiParseString {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_PARSE_STRING;
}
