//! `CgDrawState` — `cg_draw.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

/// `cg_draw.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Empty at C5 skeleton time by design: fields fold in as the waves transcribe
/// `cg_draw.c`'s file-scope statics (DEC-46.1), so a wave transcriber only ever
/// touches its own TU's two files — the function file and this one — and never
/// `cg_world.rs`. Raven's read-only tables beside them are compiled-in data,
/// not state; they land as `const`s beside the functions that read them (§C8).
///
/// Source: `oracle/codemp/cgame/cg_draw.c:23-40,1791-1792,1940-1941,2196,2425,3167,3173-3174,4152,4738-4740,4799-4803,4847,5325-5326,7317-7338,7351-7354,7481`
#[derive(Debug, Clone, Default)]
pub struct CgDrawState {}
