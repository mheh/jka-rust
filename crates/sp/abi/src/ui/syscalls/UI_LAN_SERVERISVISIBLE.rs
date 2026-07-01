use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_LAN_SERVERISVISIBLE`.
///
/// Raven wrapper: `return syscall( UI_LAN_SERVERISVISIBLE, source, n );`
/// Raven transport: `return LAN_ServerIsVisible( args[1], args[2] );`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:236`
/// Args source (SP fallback): `oracle/oracle/code/client/cl_ui.cpp` does not implement `UI_LAN_SERVERISVISIBLE`.
/// Args source (fallback): `oracle/oracle/codemp/ui/ui_local.h:975`
/// Transport/switch source (fallback): `oracle/oracle/codemp/client/cl_ui.cpp:1103-1104`
/// Output source fallback: `oracle/oracle/codemp/ui/ui_local.h:975`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLanServerisvisibleArgs {
    source: c_int,
    n: c_int,
}

impl UiLanServerisvisibleArgs {
    pub const fn new(source: c_int, n: c_int) -> Self {
        Self { source, n }
    }

    pub const fn source(&self) -> c_int {
        self.source
    }

    pub const fn n(&self) -> c_int {
        self.n
    }
}

/// `UI_LAN_SERVERISVISIBLE` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:236`
pub struct UiLanServerisvisible;

impl OutboundSysCall for UiLanServerisvisible {
    type Import = SpUiImport;
    type Args = UiLanServerisvisibleArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_SERVERISVISIBLE;
}

impl EncodeSysCall for UiLanServerisvisible {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.source() as isize, args.n() as isize])
    }
}

impl DecodeSysCallReturn for UiLanServerisvisible {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
