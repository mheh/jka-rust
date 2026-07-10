//! `q_shared.h` OS/compiler-identification `#define`s.
//!
//! These four names are also duplicated verbatim in
//! `oracle/codemp/Splines/q_shared.h` (a standalone vendored copy of this
//! header for the Splines math library); nothing in `crates/` references
//! `Splines/`, so it is dead/unreferenced and the real, compiled definitions
//! below cite the actual `game/q_shared.h` instead. This project only
//! targets Linux, so only that branch of each OS-conditional chain applies.
//!
//! Source: `oracle/codemp/game/q_shared.h:118-330`

/// Raven `CPUSTRING` — CPU/OS build-banner string. Raven's `__linux__`
/// branch only special-cases `__i386__` (`"linux-i386"`); anything else,
/// including `x86_64` (uncovered by this era's `#elif defined __axp__`
/// chain), falls through to `"linux-other"` — so the two lanes this
/// project's CI builds (`i686`/`x86_64-unknown-linux-gnu`) take different
/// literal values, matching what compiling this exact source would do.
///
/// Source: `oracle/codemp/game/q_shared.h:270-277`
#[cfg(target_arch = "x86")]
pub const CPUSTRING: &str = "linux-i386";
#[cfg(not(target_arch = "x86"))]
pub const CPUSTRING: &str = "linux-other";

/// Raven `PATH_SEP` — filesystem path separator. `'/'` on every non-Windows,
/// non-classic-Mac branch (Linux, Mac OS X, FreeBSD all agree).
///
/// Source: `oracle/codemp/game/q_shared.h:279`
pub const PATH_SEP: u8 = b'/';

/// Raven `QDECL` — calling-convention marker (`__cdecl` on Windows, empty
/// elsewhere). Rust's `extern "C"` ABI already fixes calling convention at
/// the FFI boundary, so this carries no runtime value; kept as a marker
/// const for grep/name parity with the C source.
///
/// Source: `oracle/codemp/game/q_shared.h:139` (non-Windows branch; the
/// Windows branch at `:152` is `__cdecl`, not applicable to this project)
pub const QDECL: () = ();

/// Raven `MAC_STATIC` — storage-class marker, empty on every branch except
/// classic single-address-space Mac OS builds we don't target. Same
/// no-runtime-value reasoning as [`QDECL`].
///
/// Source: `oracle/codemp/game/q_shared.h:268` (`__linux__` branch)
pub const MAC_STATIC: () = ();

/// Raven `Q3_VERSION` — build-banner version string, `"(internal)JAmp: v" +
/// VERSION_STRING_DOTTED` (neither `_DEBUG` nor `FINAL_BUILD` defined for
/// this build config).
///
/// Source: `oracle/codemp/qcommon/game_version.h:11`,
/// `oracle/codemp/win32/AutoVersion.h:10`
pub const Q3_VERSION: &str = "(internal)JAmp: v1.0.1.0";
