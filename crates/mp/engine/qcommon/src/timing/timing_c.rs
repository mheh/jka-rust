#![allow(non_camel_case_types, non_snake_case)]

use std::time::Instant;

/// Raven `timing_c` — RDTSC-based CPU-cycle stopwatch used to profile hot code
/// paths (the `G_PRECISIONTIMER_START`/`END` game-syscall pair boxes one,
/// `Start`s it, then `End`s it for the elapsed count).
///
/// Raven: `Start()`/`End()` read the CPU timestamp counter via a hand-written
/// `rdtsc` instruction (`__asm { rdtsc; mov [ebx], eax; mov [ebx+4], edx }`,
/// Windows-only — `End()`'s read is skipped under `#ifndef __linux__`), then
/// `End()` returns `(int)(end - start)`, clamped to zero if negative.
///
/// Type definition source: `oracle/codemp/qcommon/timing.h:1-60`
// Divergence: `rdtsc` cycle counts have no portable Rust equivalent, so this
// debug-only, never-ABI-crossing profiler measures over `Instant` and returns
// elapsed nanoseconds (Raven's `end`/`start` __int64 and unused `reset` drop).
pub struct timing_c {
    start: Option<Instant>,
}

impl Default for timing_c {
    /// Raven `timing_c(void) {}` — leaves the counters unset.
    /// Source: `oracle/codemp/qcommon/timing.h:9-11`
    fn default() -> Self {
        timing_c { start: None }
    }
}

impl timing_c {
    /// Raven `void Start()` — stamp the start counter.
    /// Source: `oracle/codemp/qcommon/timing.h:12-27`
    pub fn Start(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Raven `int End()` — return `end - start`, clamped to zero if negative.
    /// Source: `oracle/codemp/qcommon/timing.h:28-58`
    pub fn End(&self) -> i32 {
        match self.start {
            Some(start) => start.elapsed().as_nanos().min(i32::MAX as u128) as i32,
            None => 0,
        }
    }
}
