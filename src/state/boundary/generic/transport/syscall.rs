use core::ffi::c_int;

use super::super::outbound::OutboundSysCall;

/// Raw game-to-engine syscall transport words.
///
/// `import` is encoded separately from `OutboundSysCall::IMPORT`; `args` are the
/// variadic ABI words passed after the import number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SysCallTransport {
    args: Box<[isize]>,
}

impl SysCallTransport {
    pub fn new(args: impl Into<Box<[isize]>>) -> Self {
        Self { args: args.into() }
    }

    pub fn empty() -> Self {
        Self::new([])
    }

    pub fn args(&self) -> &[isize] {
        &self.args
    }
}

/// Encode a typed outbound syscall payload into raw engine syscall ABI words.
pub trait EncodeSysCall: OutboundSysCall {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport;
}

/// Decode the engine's raw `intptr_t` syscall return word into the call's typed
/// [`Output`](OutboundSysCall::Output). The return-side counterpart to
/// [`EncodeSysCall`]: that turns typed args into ABI words; this turns the single
/// word the engine hands back into `Output`. Calls whose C return is `void`
/// decode to `()` by ignoring the word.
pub trait DecodeSysCallReturn: OutboundSysCall {
    fn decode_return(word: isize) -> Self::Output;
}

/// Encode a C `int` as an engine `intptr_t` word.
pub const fn c_int_to_word(value: c_int) -> isize {
    value as isize
}

/// Encode a raw pointer as an engine `intptr_t` word.
pub fn ptr_to_word<T>(ptr: *const T) -> isize {
    ptr as isize
}
