#![allow(non_camel_case_types)]

use crate::common::engine_host_view::EngineHostView;

/// Console-command handler slot. Receiver-threaded in place of Raven's
/// global-reaching `void (*xcommand_t)(void)` (user ruling 2026-07-11); the
/// receiver list collapsed to the single `EngineHostView` world bundle in the
/// host-seam restructure (user ruling 2026-07-11 pt. 2, amending the pinned
/// receiver order): the dispatch site (`Cmd_ExecuteString`) passes the view in
/// scope there, and a handler that needs its island's real state casts the
/// view's type-erased slot at its boundary.
pub type CmdFunction = fn(&mut EngineHostView);

/// Raven `cmd_function_t` — one registered console command. Raven chains
/// `S_Malloc`'d nodes with `CopyString`'d names into the intrusive
/// `cmd_functions` list; the port owns them in
/// `Common::cmd_functions: Vec<cmd_function_t>` with `String` names
/// (index 0 = Raven's list head; head-insert and move-to-front preserved).
/// A `None` `function` marks a completion-only command handled by the
/// cgame/game.
///
/// Type definition source: `oracle/codemp/qcommon/cmd_pc.cpp:3-8`
pub struct cmd_function_t {
    pub name: String,
    pub function: Option<CmdFunction>,
}
