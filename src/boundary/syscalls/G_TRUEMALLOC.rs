use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use super::super::generic::{
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

/// `G_TRUEMALLOC` outbound game-to-engine syscall.
///
/// Asks the engine to allocate `size` bytes of dynamic VM memory, writing the
/// resulting pointer through the caller-supplied `void **` out-parameter.
/// Returns `void`; the allocation result arrives via `ptr`.
pub struct GTruemalloc;

impl OutboundSysCall for GTruemalloc {
    type Args = GTruemallocArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_TRUEMALLOC;
}

impl EncodeSysCall for GTruemalloc {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ptr as *const _), a.size as isize])
    }
}

impl DecodeSysCallReturn for GTruemalloc {
    fn decode_return(_word: isize) -> Self::Output {}
}
