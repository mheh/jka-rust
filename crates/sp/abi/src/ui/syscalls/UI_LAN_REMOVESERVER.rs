use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_REMOVESERVER`.
///
/// Raven wrapper: `LAN_RemoveServer(source, addr);`
/// Raven transport: `LAN_RemoveServer(args[1], (const char *)VMA(2));`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:226`
/// Args source (SP fallback): `oracle/oracle/code/client/cl_ui.cpp` does not implement `UI_LAN_REMOVESERVER`.
/// Args source (fallback): `oracle/oracle/codemp/ui/ui_local.h:978`
/// Transport/switch source (fallback): `oracle/oracle/codemp/client/cl_ui.cpp:1066-1067`
/// Output source fallback: `oracle/oracle/codemp/ui/ui_local.h:978`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLanRemoveserverArgs {
    source: c_int,
    addr: *const c_char,
}

impl UiLanRemoveserverArgs {
    pub const fn new(source: c_int, addr: *const c_char) -> Self {
        Self { source, addr }
    }

    pub const fn source(&self) -> c_int {
        self.source
    }

    pub const fn addr(&self) -> *const c_char {
        self.addr
    }
}

/// `UI_LAN_REMOVESERVER` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:226`
pub struct UiLanRemoveserver;

impl OutboundSysCall for UiLanRemoveserver {
    type Import = SpUiImport;
    type Args = UiLanRemoveserverArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_REMOVESERVER;
}

impl EncodeSysCall for UiLanRemoveserver {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.source() as isize, ptr_to_word(args.addr())])
    }
}

impl DecodeSysCallReturn for UiLanRemoveserver {
    fn decode_return(_word: isize) -> Self::Output {}
}
