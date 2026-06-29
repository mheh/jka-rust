use core::ffi::c_int;

use super::super::inbound::InboundVmCall;

/// Raw executable-to-game `vmMain` transport words.
///
/// `command` is decoded separately into `GameExport`; `args` are the twelve
/// `intptr_t`-width payload slots received by the C ABI entry point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmMainTransport {
    args: [isize; 12],
}

impl VmMainTransport {
    pub const fn new(args: [isize; 12]) -> Self {
        Self { args }
    }

    pub const fn args(self) -> [isize; 12] {
        self.args
    }

    pub const fn arg(self, index: usize) -> isize {
        self.args[index]
    }
}

/// Decode raw `vmMain` ABI words into a typed inbound call payload.
pub trait DecodeVmMain: InboundVmCall {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args;
}

/// Encode a handler's typed [`Output`](InboundVmCall::Output) into the single
/// `intptr_t` word `vmMain` returns to the engine. The return-side counterpart to
/// [`DecodeVmMain`]: that turns ABI words into typed args; this turns the typed
/// result back into the one word the C entry point hands back. Handlers whose C
/// return is `void`/`0` encode `()` to `0`.
pub trait EncodeVmMainReturn: InboundVmCall {
    fn encode_return(output: Self::Output) -> isize;
}

/// Decode an engine `intptr_t` word as a C `int`.
pub const fn word_to_c_int(word: isize) -> c_int {
    word as c_int
}

/// Decode an engine `intptr_t` word as a mutable raw pointer.
pub fn word_to_mut_ptr<T>(word: isize) -> *mut T {
    word as *mut T
}

/// Decode an engine `intptr_t` word as a const raw pointer.
pub fn word_to_const_ptr<T>(word: isize) -> *const T {
    word as *const T
}
