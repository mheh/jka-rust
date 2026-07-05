//! `Server` (the `Engine.sv` island host) + `ServerGame` (the game dispatcher's
//! reborrowed host state) + `sv_game_system_calls` (the MP game dispatcher).

/// The server-island state owned by `Engine.sv: Server` — always present, NOT
/// an `Option` (LIFE-Q7 resolution, round-6): liveness is `sv.state == SS_DEAD`
/// (`serverState_t`, `SS_DEAD` = "no map loaded", `codemp/server/server.h:46-54`),
/// the direct dual of Raven's loader-zero-filled `sv`/`svs` statics. Reuses the
/// existing ported `server_t`/`serverStatic_t` types as fields
/// (`server/server_t.rs`, `server/server_static_t.rs`) plus bot/master/savegame
/// state — placeheld here so the frozen `Engine` struct can name it; the
/// `sv.state` liveness field arrives with the `server_t` field set.
///
/// Source: `oracle/oracle/codemp/server/sv_main.cpp:10-11`;
/// `oracle/oracle/codemp/server/server.h:46-54` (state/`SS_DEAD`).
pub struct Server {
    //TODO: Port Server fields (sv: server_t incl. the SS_DEAD liveness state,
    // svs: serverStatic_t, bot, …)
    // Source: oracle/oracle/codemp/server/sv_main.cpp:10-11; server.h:53-88
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
    use mp_abi::game::imports::MpGameImport;
    let _ = engine;

    // Slice-0 minimal dispatch: only the GAME_INIT-era traps the minimal
    // module emits. The full exhaustive TryFrom<i32>-decoded match (SEAM-D3)
    // and the ERR_DROP bad-number fallback (sv_game.cpp:1654) land with it:
    //TODO: Port SV_GameSystemCalls exhaustive dispatch
    // Source: oracle/oracle/codemp/server/sv_game.cpp:458-1654
    let trap = args[0] as i32;
    if trap == MpGameImport::G_PRINT as i32 {
        // `case G_PRINT: Com_Printf( "%s", VMA(1) );` (sv_game.cpp:503-505;
        // VMA is a native-DLL identity cast, vm.cpp:648-649). Slice-0 sink is
        // the com_printf minimal console write; routing through
        // `&mut Common` lands with the ServerGame reborrow wiring.
        let msg = unsafe { core::ffi::CStr::from_ptr(args[1] as *const core::ffi::c_char) };
        print!("{}", msg.to_string_lossy());
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
        return 0;
    }
    todo!("Port SV_GameSystemCalls trap {trap} — oracle/oracle/codemp/server/sv_game.cpp:458")
}

/// The injected `SlotSyscall` target (LOAD-D8 injection): unpacks the
/// trampoline's 16-word frame and enters the typed dispatcher above — the
/// inbound dual of `CEngine::raw_syscall_words`'s frame.
///
/// PROVISIONAL ctx handling (checkpoint-7 finding): `ServerGame`'s concrete
/// shape (alias vs wrapper over the `Engine.sv` reborrow) is not pinned by any
/// frozen doc, so `ctx` is carried but not yet dereferenced — the Slice-0
/// dispatch (G_PRINT) needs no host state; the real `ctx → &mut ServerGame`
/// reborrow lands when ServerGame's shape is pinned.
///
/// Source: `oracle/oracle/codemp/qcommon/vm.cpp:377` (`currentVM->systemCall( args )`).
pub extern "C-unwind" fn game_system_calls_shim(
    ctx: *mut core::ffi::c_void,
    args: *const isize,
) -> isize {
    let _ = ctx;
    // SAFETY: the trampoline shim always forwards its full 16-word frame.
    let frame = unsafe { core::slice::from_raw_parts(args, 16) };
    let mut server_game = ServerGame { _private: () };
    sv_game_system_calls(&mut server_game, frame)
}
