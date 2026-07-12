use super::super::SpCgameImport;
use abi_transport::generic::OutboundSysCall;

/// `CG_UI_PARSE_STRING` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:197`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:583-585`
/// Output source: `oracle/code/client/cl_cgame.cpp:861-863`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:861-863`
///
/// Raven wrapper: `cgi_UI_Parse_String(buf);`
/// Raven transport: `PC_ParseString((const char **) VMA(1));`
///
/// Raven wrapper source: `oracle/code/cgame/cg_syscalls.cpp:583-585`
/// Raven wrapper prototype source: `oracle/code/cgame/cg_local.h:1214`
/// Raven transport source: `oracle/code/client/cl_cgame.cpp:861-863`
/// Raven VMA source: `oracle/code/client/cl_cgame.cpp:430-433`
/// Raven disabled callsite source: `oracle/code/cgame/cg_main.cpp:2738-2744`
///
/// NOTE: does not appear to be used anywhere.
pub struct CgUiParseString;

impl OutboundSysCall for CgUiParseString {
    type Import = SpCgameImport;
    type Args = ();
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_PARSE_STRING;
}
