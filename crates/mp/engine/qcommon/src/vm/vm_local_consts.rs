//! `vm_local.h`-local constants.
//!
//! Source: `oracle/codemp/qcommon/vm_local.h:1-4`

/// Raven `CRAZY_SYMBOL_MAP` — enables the `std::map`-backed VM debug symbol
/// map ("so that I may utilize vm debugging features WITHOUT DROPPING TO
/// 0.1FPS" — rww). Ported as `bool` since Raven never gives it a value, only
/// tests it with `#ifdef`; guarded by `#ifndef _XBOX`, which this project's
/// Linux target satisfies.
///
/// Source: `oracle/codemp/qcommon/vm_local.h:2-3`
pub const CRAZY_SYMBOL_MAP: bool = true;
