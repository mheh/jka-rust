use super::super::SpCgameImport;
use crate::abi::generic::OutboundSysCall;

/// `CG_UI_PARSE_STRING` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:197`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:583-585`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:861-863`
/// Transport/switch source: `oracle/oracle/code/cgame/cg_syscalls.cpp:583-585`, `oracle/oracle/code/client/cl_cgame.cpp:861-863`
///
/// TODO: Port args — ambiguous ABI shape between Raven sources:
/// - Prototype/source wrapper: `void cgi_UI_Parse_String(char *buf)` forwards
///   `buf` directly (`oracle/oracle/code/cgame/cg_local.h:1214`,
///   `oracle/oracle/code/cgame/cg_syscalls.cpp:583-585`).
/// - Engine switch: the same word is decoded as `(const char **) VMA(1)` for
///   `PC_ParseString` (`oracle/oracle/code/client/cl_cgame.cpp:861-863`).
/// - VM transport in this SP path defines `VMA(x)` as `((void*)args[x])`, so no
///   pointer translation resolves the mismatch
///   (`oracle/oracle/code/client/cl_cgame.cpp:430-433`).
/// - The only visible cgame callsite is inside a disabled block and passes a
///   `char *tempStr`, not `char **`
///   (`oracle/oracle/code/cgame/cg_main.cpp:2738-2744`).
///
/// This should be resolved as either `*mut c_char` (wrapper shape) or
/// `*mut *const c_char` (transport shape) depending on intended ownership.
///
/// Keep this unresolved: Raven evidence proves the wrapper and switch disagree,
/// and the dormant callsite does not prove a working ABI shape.
pub struct CgUiParseString;

impl OutboundSysCall for CgUiParseString {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args - Raven wrapper `char *buf` conflicts with switch `const char **VMA(1)`.
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_PARSE_STRING;
}
