//! `Client` (the `Engine.cl` island host) + `SoundSystem` (`Engine.snd`).

/// The client-island state owned by `Engine.cl: Option<Client>` (state-ownership
/// § Client). Reuses the existing ported `clientActive_t`/`clientConnection_t`/
/// `clientStatic_t` types plus `KeyState`/console/screen — placeheld here so the
/// frozen `Engine` struct can name it. `None` on dedicated.
///
/// Source: `oracle/oracle/codemp/client/cl_main.cpp:105-107`
pub struct Client {
    //TODO: Port Client fields (cl, clc, cls, keys, console, screen)
    // Source: oracle/oracle/codemp/client/cl_main.cpp:105-107
    _private: (),
}

/// The `Engine.snd` faithful mixer (DEC-03; EAX/force-feedback dropped). `None`
/// on dedicated (`S_Init` gated `!com_dedicated`). Placeheld so `Engine` names it.
///
/// Source: `oracle/oracle/codemp/client/snd_dma.cpp:127-268`
pub struct SoundSystem {
    //TODO: Port SoundSystem fields (channels, dma, listener, knownSfx)
    // Source: oracle/oracle/codemp/client/snd_dma.cpp:127-268
    _private: (),
}
