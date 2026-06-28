use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ARGV` outbound game-to-engine syscall.
///
/// Copies token `n` of the current command into a caller-supplied buffer.
/// Mirrors: `syscall!(G_ARGV, n, buf.as_mut_ptr(), buf.len() as i32)`
#[derive(Debug)]
pub struct GArgvArgs {
    /// Index of the command token to retrieve.
    pub n: c_int,
    /// Caller-supplied output buffer (engine writes the token string here).
    pub buffer: *mut c_char,
    /// Capacity of `buffer` in bytes.
    pub buffer_len: c_int,
}

impl GArgvArgs {
    pub fn new(n: c_int, buffer: *mut c_char, buffer_len: c_int) -> Self {
        Self { n, buffer, buffer_len }
    }

    pub fn n(&self) -> c_int { self.n }
    pub fn buffer(&self) -> *mut c_char { self.buffer }
    pub fn buffer_len(&self) -> c_int { self.buffer_len }
}

pub struct GArgv;

impl OutboundSysCall for GArgv {
    type Import = GameImport;
    type Args = GArgvArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ARGV;
}

impl EncodeSysCall for GArgv {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.n as isize,
            ptr_to_word(a.buffer),
            a.buffer_len as isize,
        ])
    }
}

impl DecodeSysCallReturn for GArgv {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
