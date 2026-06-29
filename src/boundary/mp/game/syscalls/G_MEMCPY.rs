use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_MEMCPY` outbound game-to-engine syscall.
///
/// ABI: `Com_Memcpy(VMA(1), VMA(2), args[3])` → returns 0.
/// Maps to `TRAP_MEMCPY` in sv_game.cpp.
#[derive(Debug)]
pub struct GMemcpyArgs {
    /// Destination buffer (VMA(1)).
    dest: *mut u8,
    /// Source buffer (VMA(2)).
    src: *const u8,
    /// Number of bytes to copy (args[3]).
    count: c_int,
}

impl GMemcpyArgs {
    pub fn new(dest: *mut u8, src: *const u8, count: c_int) -> Self {
        Self { dest, src, count }
    }

    pub fn dest(&self) -> *mut u8 {
        self.dest
    }

    pub fn src(&self) -> *const u8 {
        self.src
    }

    pub fn count(&self) -> c_int {
        self.count
    }
}

/// `G_MEMCPY` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:277`
pub struct GMemcpy;

impl OutboundSysCall for GMemcpy {
    type Import = GameImport;
    type Args = GMemcpyArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_MEMCPY;
}

impl EncodeSysCall for GMemcpy {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.dest as *const u8),
            ptr_to_word(a.src),
            a.count as isize,
        ])
    }
}

impl DecodeSysCallReturn for GMemcpy {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
