/// A typed executable-to-game call.
///
/// These are the commands the executable sends through `vmMain`.
pub trait InboundVmCall {
    type Command;
    type Args;
    type Output;

    const COMMAND: Self::Command;
}

/// The inbound seam every module-side handler implements (SEAM-D8), mirroring
/// `Execute<C>`'s shape with the bound moved to `InboundVmCall`. Used by the
/// module-side `vmMain` dispatch: each export-enum arm decodes via `DecodeVmMain`,
/// routes to that command's `Dispatch<C>` impl, and encodes via
/// `EncodeVmMainReturn`.
///
/// Source: `docs/architecture/engine-seam.md` § inbound dual (SEAM-D8).
pub trait Dispatch<C: InboundVmCall> {
    fn dispatch(&self, args: C::Args) -> C::Output;
}

/// Route an inbound executable-to-game `vmMain` command.
///
/// Placeholder marker trait with zero impls — superseded by `Dispatch<C>`
/// (SEAM-D8). Retained (not deleted) in this skeleton seed; see FINDINGS.
pub trait InboundVmCallExecutor {
    fn call_inbound<C>(&self, args: C::Args) -> C::Output
    where
        C: InboundVmCall;
}
