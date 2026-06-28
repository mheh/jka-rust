use crate::ffi::GameExport;

/// A typed executable-to-game call.
///
/// These are the commands the executable sends through `vmMain`.
pub trait InboundVmCall {
    type Args;
    type Output;

    const COMMAND: GameExport;
}

/// Route an inbound executable-to-game `vmMain` command.
pub trait InboundVmCallExecutor {
    fn call_inbound<C>(&self, args: C::Args) -> C::Output
    where
        C: InboundVmCall;
}
