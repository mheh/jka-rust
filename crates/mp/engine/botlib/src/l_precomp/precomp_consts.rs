//! `l_precomp.cpp`-local preprocessor constants.
//!
//! Source: `oracle/codemp/botlib/l_precomp.cpp:81-94`

/// Raven `MAX_DEFINEPARMS` — max parameters a `#define` macro may take.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:81`
pub const MAX_DEFINEPARMS: usize = 128;

/// Raven `MAX_PATH` — botlib redefines the Win32 macro to `MAX_QPATH` for the
/// preprocessor path buffers.
/// Source: `oracle/codemp/botlib/l_precomp.h:16` (`#define MAX_PATH MAX_QPATH`).
pub const MAX_PATH: usize = mp_qshared::shared::MAX_QPATH;

/// Raven `DEFINEHASHSIZE` — size of the `define_t` hash chain table.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:92`
pub const DEFINEHASHSIZE: usize = 1024;

/// Raven `TOKEN_HEAP_SIZE` — size of the static `token_t` heap.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:94`
pub const TOKEN_HEAP_SIZE: usize = 4096;

/// Raven `DEFINEHASHING` — enables the `define_t` hash-chain lookup path;
/// tested with `#if` (not just `#ifdef`) against its real value `1` at
/// dozens of sites in `l_precomp.cpp`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:83`
pub const DEFINEHASHING: i32 = 1;

/// Raven `MAX_VALUES` — max `value_t` entries live at once.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1708`
pub const MAX_VALUES: usize = 64;

/// Raven `MAX_OPERATORS` — max `operator_t` entries live at once.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1709`
pub const MAX_OPERATORS: usize = 64;

/// Raven `MAX_SOURCEFILES` — size of the `sourceFiles` table.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3185`
pub const MAX_SOURCEFILES: usize = 64;
