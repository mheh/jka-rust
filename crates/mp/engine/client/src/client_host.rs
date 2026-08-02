//! `Client` (the `Engine.cl` island host) + `SoundSystem` (`Engine.snd`).

use native_platform::zeroed_box;

use crate::client::client_active_t::clientActive_t;
use crate::client::client_connection_t::clientConnection_t;
use crate::client::client_static_t::clientStatic_t;
use crate::client::console_t::console_t;
use crate::keys::key_globals_s::keyGlobals_t;

/// The client-island state owned by `Engine.cl: Option<Client>`, and `None` on dedicated.
/// The five fields are Raven's zero-filled client globals (state-ownership § Client).
/// Each field is a `Box` because the 2.6 MB mass must never transit the stack (STATE-D9 `zeroed_box`).
///
/// Source: `oracle/codemp/client/cl_main.cpp:105-107`
pub struct Client {
    /// Raven `cl` - the active game state that the engine parses from the server and wipes per gamestate.
    /// `cl.mSharedMemory` stays the raw module window, the same as `sv.mSharedMemory` on the server.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:105`
    pub cl: Box<clientActive_t>,
    /// Raven `clc` - the connection state that the engine wipes on every connect and every disconnect.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:106`
    pub clc: Box<clientConnection_t>,
    /// Raven `cls` - the client state that survives level loads, so the engine never wipes it.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:107`
    pub cls: Box<clientStatic_t>,
    /// Raven `kg` - the key bindings, the key-down table, and the console edit field with its history.
    ///
    /// Source: `oracle/codemp/client/cl_keys.cpp:17`
    pub kg: Box<keyGlobals_t>,
    /// Raven `con` - the console scrollback buffer and its display state.
    ///
    /// Source: `oracle/codemp/client/cl_console.cpp:13`
    pub con: Box<console_t>,
}

impl Default for Client {
    /// Returns the all-zero client island, the direct dual of Raven's zero-filled client globals.
    /// Every field is `ZeroValid`, so each box comes back zeroed and never builds on the stack.
    fn default() -> Self {
        Self {
            cl: zeroed_box(),
            clc: zeroed_box(),
            cls: zeroed_box(),
            kg: zeroed_box(),
            con: zeroed_box(),
        }
    }
}

/// The `Engine.snd` faithful mixer (DEC-03; EAX/force-feedback dropped). `None`
/// on dedicated (`S_Init` gated `!com_dedicated`). Placeheld so `Engine` names it.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:127-268`
pub struct SoundSystem {
    //TODO: Port SoundSystem fields (channels, dma, listener, knownSfx)
    // Source: oracle/codemp/client/snd_dma.cpp:127-268
    _private: (),
}
