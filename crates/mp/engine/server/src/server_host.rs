//! `Server` (the `Engine.sv` island host) + `ServerGame` (the game dispatcher's
//! reborrowed host state) + `sv_game_system_calls` (the MP game dispatcher).

/// The server-island state owned by `Engine.sv: Option<Server>` (state-ownership
/// § Server). Reuses the existing ported `server_t`/`serverStatic_t` types as
/// fields (`server/server_t.rs`, `server/server_static_t.rs`) plus bot/master/
/// savegame state — placeheld here so the frozen `Engine` struct can name it.
///
/// Source: `oracle/oracle/codemp/server/sv_main.cpp:10-11`
pub struct Server {
    //TODO: Port Server fields (sv: server_t, svs: serverStatic_t, bot, …)
    // Source: oracle/oracle/codemp/server/sv_main.cpp:10-11
    _private: (),
}

/// engine-seam's name for the game dispatcher's `&mut ServerGame` argument — the
/// server-island reborrow (`&mut Engine.sv`'s `Server`) carrying its
/// `SharedGameData` registration. Its concrete shape (a `type ServerGame =
/// Server` alias vs. a wrapper struct) is a seam-executor mechanic left
/// *forward-declared* / not pinned (state-ownership STATE-Q7); placeheld as a
/// minimal wrapper here so `sv_game_system_calls` can name it.
///
/// Source: `docs/architecture/engine-seam.md` § Engine-side dispatchers.
//TODO: Port ServerGame concrete shape (alias vs wrapper — STATE-Q7)
pub struct ServerGame {
    _private: (),
}

/// The MP game outbound dispatcher — our `SV_GameSystemCalls` equivalent
/// (SEAM-D3). A hand-written exhaustive `match` over `MpGameImport`; `args[0]` =
/// syscall number decoded via `TryFrom<i32>`; return is the C `intptr_t` word.
/// An unknown trap number reproduces Raven's `Com_Error(ERR_DROP, "Bad game
/// system trap: %i")` faithfully (`sv_game.cpp:1654`).
///
/// Source: `oracle/oracle/codemp/server/sv_game.cpp:458`
pub fn sv_game_system_calls(engine: &mut ServerGame, args: &[isize]) -> isize {
    let _ = (engine, args);
    // Bad-number fallback lives in the isize→i32→MpGameImport::try_from conversion,
    // reproducing `Com_Error(ERR_DROP, "Bad game system trap")` (sv_game.cpp:1654).
    todo!("Port SV_GameSystemCalls — oracle/oracle/codemp/server/sv_game.cpp:458")
}
