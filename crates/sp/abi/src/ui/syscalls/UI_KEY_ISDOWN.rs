use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::qboolean;

/// Arguments for `UI_KEY_ISDOWN`.
///
/// Raven wrapper: `return syscall( UI_KEY_ISDOWN, keynum );`
/// SP enum source: `oracle/code/ui/ui_public.h:188`
/// SP transport/source evidence: no direct `UI_KEY_ISDOWN` case in `oracle/code/client/cl_ui.cpp`.
/// Fallback transport evidence (MP): `oracle/codemp/ui/ui_syscalls.c:231`
/// Transport/switch source (MP): `oracle/codemp/client/cl_ui.cpp:1019-1020`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyIsdownArgs {
    /// Key number, read by Raven as `args[1]`.
    keynum: c_int,
}

impl UiKeyIsdownArgs {
    pub const fn new(keynum: c_int) -> Self {
        Self { keynum }
    }

    pub const fn keynum(&self) -> c_int {
        self.keynum
    }
}

/// `UI_KEY_ISDOWN` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/code/ui/ui_public.h:188`
/// Output source (fallback): `oracle/codemp/ui/ui_syscalls.c:231`
/// Transport/source (fallback): `oracle/codemp/client/cl_ui.cpp:1019-1020`
/// Key result type source (SP/Multi): `oracle/code/client/cl_keys.cpp:882`
pub struct UiKeyIsdown;

impl OutboundSysCall for UiKeyIsdown {
    type Import = SpUiImport;
    type Args = UiKeyIsdownArgs;
    type Output = qboolean;

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_ISDOWN;
}

impl EncodeSysCall for UiKeyIsdown {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.keynum() as isize])
    }
}

impl DecodeSysCallReturn for UiKeyIsdown {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
