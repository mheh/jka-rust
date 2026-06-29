use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_MEMSET` outbound game-to-engine syscall.
///
/// ABI: `TRAP_MEMSET` → `Com_Memset(VMA(1), args[2], args[3])`
/// C equivalent: `void *memset(void *dest, int val, int size)` (handler returns 0).
#[derive(Debug)]
pub struct GMemsetArgs {
    /// Destination buffer pointer (VMA(1)).
    dest: *mut c_void,
    /// Fill value (args[2]).
    val: c_int,
    /// Number of bytes to fill (args[3]).
    size: c_int,
}

impl GMemsetArgs {
    pub fn new(dest: *mut c_void, val: c_int, size: c_int) -> Self {
        Self { dest, val, size }
    }

    pub fn dest(&self) -> *mut c_void {
        self.dest
    }

    pub fn val(&self) -> c_int {
        self.val
    }

    pub fn size(&self) -> c_int {
        self.size
    }
}

/// `G_MEMSET` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:276`
pub struct GMemset;

impl OutboundSysCall for GMemset {
    type Import = GameImport;
    type Args = GMemsetArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_MEMSET;
}

impl EncodeSysCall for GMemset {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.dest as *const _),
            a.val as isize,
            a.size as isize,
        ])
    }
}

impl DecodeSysCallReturn for GMemset {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
