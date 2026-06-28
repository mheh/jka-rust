use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;
use crate::ffi::syscalls::pass_float;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Args for the `BOTLIB_EA_GET_INPUT` outbound game-to-engine syscall.
///
/// Mirrors `trap_EA_GetInput(int client, float thinktime, void *input)`.
/// `input` is a `bot_input_t *` in C (be_ea.h:48); passed opaquely as `*mut c_void`
/// because `bot_input_t` is not yet ported to Rust.
#[derive(Debug)]
pub struct BotlibEaGetInputArgs {
    client: c_int,
    thinktime: f32,
    input: *mut c_void,
}

impl BotlibEaGetInputArgs {
    pub fn new(client: c_int, thinktime: f32, input: *mut c_void) -> Self {
        Self { client, thinktime, input }
    }

    pub fn client(&self) -> c_int { self.client }
    pub fn thinktime(&self) -> f32 { self.thinktime }
    pub fn input(&self) -> *mut c_void { self.input }
}

/// `BOTLIB_EA_GET_INPUT` outbound game-to-engine syscall.
pub struct BotlibEaGetInput;

impl OutboundSysCall for BotlibEaGetInput {
    type Args = BotlibEaGetInputArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_GET_INPUT;
}

impl EncodeSysCall for BotlibEaGetInput {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.client as isize,
            pass_float(a.thinktime),
            ptr_to_word(a.input),
        ])
    }
}

impl DecodeSysCallReturn for BotlibEaGetInput {
    fn decode_return(_word: isize) -> Self::Output { () }
}
