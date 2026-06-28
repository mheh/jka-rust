use core::ffi::c_char;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_SET_SHARED_BUFFER` outbound game-to-engine syscall.
///
/// Passes the address of the game module's shared-memory buffer to the engine
/// (`trap_SV_RegisterSharedMemory` / `gSharedBuffer`).  The engine writes into
/// that buffer for certain callbacks (e.g. the ICARUS bridge); the caller must
/// keep the buffer alive for as long as the engine holds the pointer.
#[derive(Debug)]
pub struct GSetSharedBufferArgs {
    /// Raw pointer to the shared-memory buffer the engine will write into.
    memory: *mut c_char,
}

impl GSetSharedBufferArgs {
    pub fn new(memory: *mut c_char) -> Self {
        Self { memory }
    }

    pub fn memory(&self) -> *mut c_char {
        self.memory
    }
}

pub struct GSetSharedBuffer;

impl OutboundSysCall for GSetSharedBuffer {
    type Import = GameImport;
    type Args = GSetSharedBufferArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_SET_SHARED_BUFFER;
}

impl EncodeSysCall for GSetSharedBuffer {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.memory)])
    }
}

impl DecodeSysCallReturn for GSetSharedBuffer {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
