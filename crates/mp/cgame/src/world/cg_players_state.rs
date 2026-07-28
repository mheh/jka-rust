//! `CgPlayersState` — `cg_players.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

/// `cg_players.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Empty at C5 skeleton time by design: fields fold in as the waves transcribe
/// `cg_players.c`'s file-scope statics (DEC-46.1), so a wave transcriber only ever
/// touches its own TU's two files — the function file and this one — and never
/// `cg_world.rs`. Raven's read-only tables beside them are compiled-in data,
/// not state; they land as `const`s beside the functions that read them (§C8).
///
/// Source: `oracle/codemp/cgame/cg_players.c:1945,3362,7583,7887,7978,7980,8338`
#[derive(Debug, Clone, Default)]
pub struct CgPlayersState {}
