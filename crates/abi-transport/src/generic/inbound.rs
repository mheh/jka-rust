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
