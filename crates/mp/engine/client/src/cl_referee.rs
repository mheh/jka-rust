//! Client-side referee state: the headless demo mode the seam referee runs in.
//!
//! This is NEW engine tooling, not a Raven port, and it is the client twin of
//! `mp_engine_server::sv_referee`. DEC-58.1 drives the full client engine from a
//! committed `.dm_26` demo and compares the engine-to-module trap journal
//! against an oracle recording. One lane the demo path crosses is not ported
//! yet: the platform shell that builds and seats `Engine.re` (gh#22, DEC-56,
//! which leaves the renderer slot NULL). The mixer landed with gh#24 and the
//! music and ambient half with gh#25.
//!
//! Headless mode is the documented seam around that lane. Every gated
//! call site names this module, names its ticket, and runs unchanged in the
//! default `Off` mode, so retail behavior is untouched. Each gate disappears
//! when its lane lands.
//!
//! The mode is set by the rig before playback, not by a cvar: the rig is the
//! only caller, and a cvar would put a live console switch on a mode that
//! deliberately skips engine work.

use crate::client_host::Client;

/// The client referee operating mode.
///
/// - `Off`: every gate is inactive and the client runs the retail path.
/// - `Headless`: the demo seam referee drives the client, so the gated sound
///   and renderer call sites are skipped.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ClientRefMode {
    #[default]
    Off,
    Headless,
}

/// The client referee state, owned as `Client.referee`.
/// `Default` is [`ClientRefMode::Off`], which is what every non-test boot gets.
#[derive(Default)]
pub struct ClientReferee {
    pub mode: ClientRefMode,
}

/// Reports whether the headless demo referee owns this client.
/// A gated call site skips its unported lane when this returns true.
pub fn ref_headless(cl: &Client) -> bool {
    cl.referee.mode == ClientRefMode::Headless
}
