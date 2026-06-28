/// A typed game-to-engine syscall.
///
/// These are the callbacks jampgame makes through the syscall pointer the
/// executable provided via `dllEntry`.
pub trait OutboundSysCall {
    type Import;
    type Args;
    type Output;

    const IMPORT: Self::Import;
}

/// Route an outbound game-to-engine syscall.
pub trait OutboundSysCallExecutor {
    fn call_outbound<C>(&self, args: C::Args) -> C::Output
    where
        C: OutboundSysCall;
}
