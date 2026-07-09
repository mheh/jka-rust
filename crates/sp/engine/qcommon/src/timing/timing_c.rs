#![allow(non_camel_case_types, non_snake_case)]

/// Raven `timing_c` — RDTSC-based CPU-cycle stopwatch used to profile hot code
/// paths.
///
/// Raven: `Start()`/`End()` read the CPU timestamp counter via a hand-written
/// `rdtsc` instruction (`__asm { rdtsc; mov [ebx], eax; mov [ebx+4], edx }`,
/// Windows-only), then `End()` returns `(int)(end - start)`, clamped to zero
/// if negative.
///
/// Type definition source: `oracle/code/qcommon/timing.h:2-60`
//TODO: Port timing_c::start, timing_c::end, timing_c::reset
// Source: oracle/code/qcommon/timing.h:5-8
// Raven's `start`/`end` (__int64) and `reset` (int) are stamped by inline
// x86 `rdtsc` assembly, which has no portable Rust equivalent; the class is a
// host-side profiling helper never crossing the ABI seam, so only its
// (parser-verified) 1-byte footprint is kept here.
#[repr(C, align(1))]
pub struct timing_c(pub [u8; 1]);
const _: () = assert!(core::mem::size_of::<timing_c>() == 1);
