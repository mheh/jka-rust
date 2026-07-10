//! Raven `pscript_t` — a cached compiled-script record.

/// Raven `pscript_t` → `Pscript` (§F idiomatic, ICARUS-D1 naming).
///
/// A cached `.IBI` script record. Raven's `char *buffer` becomes an owned
/// `Vec<u8>` (ICARUS-D3 / ruling 20 drops the ICARUS arena, so this
/// `TAG_ICARUS5` blob is owned here); `length` is `buffer.len()`.
/// Type definition source: `oracle/codemp/icarus/GameInterface.h:4-8`
#[derive(Default)]
pub struct Pscript {
    /// Raven `char *buffer` — the compiled block-instruction bytes (owned).
    pub buffer: Vec<u8>,
    /// Raven `long length`.
    pub length: i64,
}
