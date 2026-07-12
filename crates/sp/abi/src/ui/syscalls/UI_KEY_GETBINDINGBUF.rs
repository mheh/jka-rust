use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use core::ffi::{c_char, c_int};

/// `UI_KEY_GETBINDINGBUF` SP UI imports syscall ABI token.
///
/// Source: `oracle/code/ui/ui_public.h:186`
pub struct UiKeyGetbindingbuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyGetbindingbufArgs {
    keynum: c_int,
    buf: *mut c_char,
    buflen: c_int,
}

impl UiKeyGetbindingbufArgs {
    pub const fn new(keynum: c_int, buf: *mut c_char, buflen: c_int) -> Self {
        Self {
            keynum,
            buf,
            buflen,
        }
    }

    pub const fn keynum(&self) -> c_int {
        self.keynum
    }

    pub const fn buf(&self) -> *mut c_char {
        self.buf
    }

    pub const fn buflen(&self) -> c_int {
        self.buflen
    }
}

impl OutboundSysCall for UiKeyGetbindingbuf {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_KEY_GETBINDINGBUF, keynum, buf, buflen );`
    ///
    /// Args source: `oracle/code/ui/ui_syscalls.cpp:91-92` and
    /// `oracle/code/client/cl_ui.cpp:493`
    /// Output source: `oracle/code/ui/ui_syscalls.cpp:91-92`
    /// Transport/switch source: `oracle/code/client/cl_ui.cpp:492-494`
    type Args = UiKeyGetbindingbufArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_GETBINDINGBUF;
}

impl EncodeSysCall for UiKeyGetbindingbuf {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.keynum() as isize,
            ptr_to_word(args.buf()),
            args.buflen() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiKeyGetbindingbuf {
    fn decode_return(_word: isize) -> Self::Output {}
}
