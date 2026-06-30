use core::ffi::{c_int, c_void};

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for the [`GTruemalloc`] outbound syscall.
///
/// `ptr` is a `void **` out-parameter — the engine writes the allocated address
/// through it.  It is kept as a raw pointer here to mirror the C ABI exactly.
#[derive(Debug)]
pub struct GTruemallocArgs {
    /// Destination pointer-to-pointer; engine writes the allocation here.
    pub ptr: *mut *mut c_void,
    /// Requested allocation size in bytes.
    pub size: c_int,
}

impl GTruemallocArgs {
    pub fn new(ptr: *mut *mut c_void, size: c_int) -> Self {
        Self { ptr, size }
    }

    pub fn ptr(&self) -> *mut *mut c_void {
        self.ptr
    }

    pub fn size(&self) -> c_int {
        self.size
    }
}

/// `G_TRUEMALLOC` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:248`
pub struct GTruemalloc;

impl OutboundSysCall for GTruemalloc {
    type Import = MpGameImport;
    type Args = GTruemallocArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_TRUEMALLOC;
}

impl EncodeSysCall for GTruemalloc {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ptr as *const _), a.size as isize])
    }
}

impl DecodeSysCallReturn for GTruemalloc {
    fn decode_return(_word: isize) -> Self::Output {}
}
