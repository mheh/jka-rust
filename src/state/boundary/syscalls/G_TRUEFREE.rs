use core::ffi::c_void;

use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for the `G_TRUEFREE` game→engine syscall.
///
/// Releases a block allocated by `G_TRUEMALLOC` and NULLs the slot.
/// `ptr` is a `void **` — the engine writes through it to clear the pointer.
#[derive(Debug)]
pub struct GTruefreeArgs {
    /// Pointer to the `void *` slot to free (a `void **`).
    ptr: *mut *mut c_void,
}

impl GTruefreeArgs {
    pub fn new(ptr: *mut *mut c_void) -> Self {
        Self { ptr }
    }

    pub fn ptr(&self) -> *mut *mut c_void {
        self.ptr
    }
}

pub struct GTruefree;

impl OutboundSysCall for GTruefree {
    type Args = GTruefreeArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_TRUEFREE;
}

impl EncodeSysCall for GTruefree {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ptr as *const _)])
    }
}

impl DecodeSysCallReturn for GTruefree {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
