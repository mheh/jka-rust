//! Engine-side referee: a server input tap (record) + synthetic replay (inject).
//!
//! This is NEW engine tooling, not a Raven port. In RECORD mode the server
//! appends an ordered event stream of every client's `usercmd_t`, client
//! command, human connect, and disconnect, plus a per-frame msec input and a
//! post-frame state digest. In REPLAY mode the same engine boots on the same
//! map/cvars and drives its own captured session frame-for-frame: it forces the
//! per-frame msec from the tape, injects each frame's recorded events before the
//! game frame, and compares the resulting digest against the tape's — a single
//! engine reproducing its own bot session byte-identically. This machinery
//! becomes the secondary engine's operating mode in the later lockstep driver.
//!
//! Nondeterminism is pinned so record and replay agree: the module's parity-
//! critical `srand(randomSeed)` is fed a seed pinned via the `Com_Milliseconds`
//! referee gate (see `ref_seed_*` on `Common`), and the per-frame msec is
//! forced from the tape.
//!
//! FOLLOW mode (`ref_follow 1` alongside `ref_replay <file>`) is the lockstep
//! driver's transport: replay tail-follows a tape another engine is STILL
//! WRITING. The follower steps a frame only once its complete block
//! (`[events] F [events] S`) has landed; when starved it skips the `SV_Frame`
//! call and re-polls ~2ms later, so it paces itself purely by data
//! availability — a fraction of a second behind the live primary. The record
//! side flushes after every `S` (bounding follower lag at one game frame) and
//! emits a final `E` record at `SV_Shutdown` so a clean end of session is
//! distinguishable from a laggy write.
//!
//! Injection is PER-SLOT, not a global mode. Bot brains always RE-RUN
//! (`SV_BotFrame` untouched): under the pinned seed + forced msec the module
//! deterministically recreates and re-thinks the identical bot session
//! (verified byte-identical over a 2000+-frame bot-combat tape) — necessary
//! because in this engine bot CREATION (`G_CheckMinimumPlayers`) lives inside
//! `BotAIStartFrame`, so suppressing bot brains would also suppress bot
//! spawning. Only clients the TAPE created (`C` events — humans by
//! construction, which cannot re-run) are materialized as netchan-less
//! replicas and have their `T`/`X`/`D` events injected; taped events for
//! module-regenerated bots are verification data, never injected (an injected
//! think on top of the re-run brain would double-think the bot). This one rule
//! serves both the pure-bot round trip and the mixed human+bots session the
//! lockstep driver mirrors.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, CStr};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use mp_abi::game::exports::MpGameExport;
use mp_engine_qcommon::cmd_common::Cbuf_AddText;
use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::cvar_fns::{Cvar_VariableIntegerValue, Cvar_VariableString};
use mp_engine_qcommon::vm::VM_Call;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_string::Q_strncpyz;
use mp_qshared::shared::{qfalse, qtrue};

use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::sv_client::{
    SV_ClientEnterWorld, SV_ClientThink, SV_ExecuteClientCommand, SV_UserinfoChanged,
};
use crate::sv_game::SV_GentityNum;
use crate::sv_referee_fields::{describe, ES, PS};
use crate::Server;

/// FNV-1a 64 offset basis.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64 prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Fold `bytes` into a running FNV-1a 64 hash.
fn fnv1a64(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// The referee operating mode, selected once at `SV_SpawnServer` from the
/// `ref_record` / `ref_replay` cvars.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum RefMode {
    /// Inactive — every tap and hook is a no-op.
    #[default]
    Off,
    /// Appending the input/state tape to the record file.
    Record,
    /// Driving the engine from the tape and comparing digests.
    Replay,
}

/// One tape record. Payload strings/bytes are hex-encoded on the wire so the
/// line format is space-delimited and newline-free regardless of content.
#[derive(Clone)]
enum Rec {
    /// `H <map> <sv_fps> <sv_maxclients> <seed>`.
    Header {
        map: String,
        fps: c_int,
        maxclients: c_int,
        seed: c_int,
    },
    /// `C <clientNum> <hex userinfo>` — human connect.
    Connect { client: c_int, userinfo: String },
    /// `B <clientNum> <hex usercmd bytes>` — the engine-driven world entry
    /// (`SV_ClientEnterWorld`: CS_PRIMED -> CS_ACTIVE + GAME_CLIENT_BEGIN),
    /// which is not a client text command and so is invisible to the `X` tap.
    Begin { client: c_int, cmd: Vec<u8> },
    /// `X <clientNum> <clientOK> <hex command>`.
    Command {
        client: c_int,
        ok: c_int,
        cmd: Vec<u8>,
    },
    /// `T <clientNum> <hex usercmd bytes>`.
    Think { client: c_int, cmd: Vec<u8> },
    /// `D <clientNum>`.
    Drop { client: c_int },
    /// `P <clientNum> <ping>` — the engine-computed ping `SV_CalcPings` writes
    /// straight into the module's `ps.ping` (digested memory). Network reality
    /// is invisible to a replica, so the value is taped as an input; recorded
    /// on change only, for non-bot clients.
    Ping { client: c_int, ping: c_int },
    /// `F <msec>` — the frame's msec input.
    Frame { msec: c_int },
    /// `S <total> E<entities> Y<hash>:<count> <slot>:<ps> ...` — the post-frame
    /// state digest: the combined hash, the entity-block aggregate, the frame's
    /// syscall-stream digest (ordered import numbers; `Y` token optional for
    /// older tapes), and one playerState hash per connected slot, so a
    /// divergence names its component.
    State { digest: StateDigest },
    /// `V <num_entities> <hex entity block> <slot>:<hex ps> ...` — the frame's
    /// verbose state bytes (`ref_state 1`), emitted just before `S` so a
    /// follower can name the first divergent FIELD, not just the component.
    Verbose { state: VState },
    /// `E` — clean end of session, written at `SV_Shutdown`. A follower quits
    /// here; its absence at EOF means the writer is merely lagging (or died).
    End,
}

/// A frame's verbose state (`V` record): the raw digested bytes, so a
/// divergence maps to an exact entity/slot + field offset.
#[derive(Clone)]
pub struct VState {
    /// `sv.sv.num_entities` at digest time.
    pub num_entities: c_int,
    /// Concatenated `entityState_t`-sized prefixes for entities `0..num_entities`.
    pub ents: Vec<u8>,
    /// `(slot, playerState_t bytes)` per connected slot, digest order.
    pub players: Vec<(c_int, Vec<u8>)>,
}

/// The engine referee state, owned as `Server.referee`. Default is [`RefMode::Off`]
/// with empty buffers, so an un-armed server carries zero behavior.
#[derive(Default)]
pub struct Referee {
    pub mode: RefMode,
    /// RECORD: the append stream.
    writer: Option<BufWriter<File>>,
    /// Wire tap (`ref_snaps <file>`): raw client-bound message capture, armed
    /// independently of record/replay. Binary records:
    /// `u32 len, i32 clientNum, i32 svs_time, i32 outgoingSequence, bytes`.
    snap_writer: Option<BufWriter<File>>,
    /// REPLAY: the parsed tape and cursor.
    recs: Vec<Rec>,
    cursor: usize,
    /// FOLLOW: replay is tail-following a tape still being written.
    follow: bool,
    /// FOLLOW: the incremental tape reader (drained into `recs` per frame).
    tail: Option<TailReader>,
    /// REPLAY: header seed to pin GAME_INIT with (mirrored to `Common`).
    header_seed: c_int,
    /// REPLAY: bitmask of client slots the TAPE created (`C` events — humans by
    /// construction). Only these slots' `T`/`X`/`D` events are injected;
    /// module-regenerated bots re-think on their own and injecting their taped
    /// events would double-think them. Cleared per slot on an injected `D`.
    injected_slots: u64,
    /// REPLAY: events buffered for the current frame's injection.
    pending: Vec<Rec>,
    /// RECORD: last `P`-recorded ping per slot (change-dedupe); reset on `C`.
    last_ping: Vec<c_int>,
    /// REPLAY: latest taped ping per injected slot (-1 = none yet); applied by
    /// `SV_CalcPings` in place of the locally-computed value. Reset on `C`.
    replica_ping: Vec<c_int>,
    /// REPLAY: per-slot flag marking a synthetic human replica (materialized
    /// from a `C` event). Set on `C`; not cleared on `D` so the CS_ZOMBIE
    /// teardown window stays I/O-suppressed, and overwritten on slot reuse by
    /// the next `C`. Gates real network I/O (the replica carries a NA_LOOPBACK
    /// address but has no socket). Reset on `C`, mirroring `replica_ping`.
    replica: Vec<bool>,
    /// REPLAY: the current frame's expected digest, if the frame runs the game.
    expected: Option<StateDigest>,
    /// REPLAY: the current frame's tape state bytes (`V` record), if verbose.
    expected_state: Option<VState>,
    /// RECORD: emit `V` verbose state records (`ref_state 1`).
    verbose: bool,
    /// Divergence policy (`ref_haltOnDiverge`): true = freeze both engines
    /// into step mode; false = log, resync from the tape's `V`, continue.
    halt_on_diverge: bool,
    /// The halt-file path (`<tape>.halt`) — the freeze back-channel: the
    /// follower creates it on a halting divergence, the primary polls it and
    /// freezes while it exists, `ref_resume` (primary) removes it.
    halt_path: Option<String>,
    /// FOLLOW: frozen after a halting divergence (halt file written).
    halted: bool,
    /// RECORD: let exactly one frame through the freeze (`ref_step`).
    step_pending: bool,
    /// FOLLOW: resync from the next compared frame's `V` (set on resume).
    resync_next: bool,
    /// FOLLOW: the last divergence's full field-level report (`ref_diff`).
    last_report: String,
    /// Rolling syscall-stream digest for the current frame window (both modes):
    /// ordered import numbers, reset after each `S` boundary.
    sys_hash: u64,
    /// Syscalls folded into `sys_hash` this window.
    sys_count: u32,
    /// REPLAY: a client lifecycle transition (`C`/`B`/`D`) was injected in the
    /// current syscall window. Such frames diverge on syscall COUNT alone
    /// (state digests match) because the transition's module calls are
    /// structurally noisy (reconnect/pak-recheck/slot-reuse cycles); the count
    /// comparison is suppressed for the window (state-digest stays active).
    /// Cleared at each window reset (`ref_sys_reset`).
    injected_transition: bool,
    /// Syscall-stream dump (`ref_calls <file>`): one line per frame window
    /// (`F<n> <import numbers>`), for diffing the two engines' exact call
    /// sequences at a syscall divergence.
    calls_writer: Option<BufWriter<File>>,
    /// The current window's ordered imports (only accumulated when dumping).
    calls_buf: Vec<i32>,
    /// Frame-window counter for the dump (aligned with the compared frames).
    calls_frame: u64,
    /// REPLAY: set once the tape is exhausted; further frames are inert.
    done: bool,
    /// REPLAY: frames whose digest was compared.
    frames: u64,
    /// REPLAY: frames whose digest mismatched.
    divergences: u64,
}

impl Referee {
    /// Whether a referee mode is active (record or replay).
    fn active(&self) -> bool {
        !matches!(self.mode, RefMode::Off)
    }
    /// Append one already-formatted line to the record stream.
    fn emit(&mut self, line: &str) {
        if let Some(w) = self.writer.as_mut() {
            let _ = w.write_all(line.as_bytes());
            let _ = w.write_all(b"\n");
        }
    }
}

/// Lowercase-hex encode a byte slice.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Decode a lowercase-hex string; returns an empty vec on malformed input.
fn hex_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len() / 2);
    let mut i = 0;
    while i + 1 < b.len() + 1 && i + 2 <= b.len() {
        let hi = (b[i] as char).to_digit(16);
        let lo = (b[i + 1] as char).to_digit(16);
        match (hi, lo) {
            (Some(h), Some(l)) => out.push((h * 16 + l) as u8),
            _ => break,
        }
        i += 2;
    }
    out
}

/// The raw `usercmd_t` bytes at `cmd`.
unsafe fn usercmd_bytes(cmd: *const usercmd_t) -> Vec<u8> {
    core::slice::from_raw_parts(cmd as *const u8, core::mem::size_of::<usercmd_t>()).to_vec()
}

/// The client index of `cl` within `svs.clients`.
unsafe fn client_index(sv: &Server, cl: *const client_t) -> c_int {
    ((cl as *const u8).offset_from(sv.svs.clients.as_ptr() as *const u8) as isize
        / core::mem::size_of::<client_t>() as isize) as c_int
}

// ===========================================================================
// State digest
// ===========================================================================

/// A frame's component-level state digest: the combined hash plus the entity-
/// block aggregate and one playerState hash per connected slot, so a
/// divergence names its component rather than just the frame.
#[derive(Clone, PartialEq, Eq)]
pub struct StateDigest {
    /// Combined hash (entities then connected playerStates, fixed order).
    pub total: u64,
    /// Aggregate over every entityState prefix.
    pub entities: u64,
    /// `(slot, fnv1a64(playerState))` per slot in state `>= CS_CONNECTED`.
    pub players: Vec<(c_int, u64)>,
    /// The frame window's syscall-stream digest `(hash, count)` — ordered
    /// import numbers between `S` boundaries. `None` on pre-G4 tapes.
    pub sys: Option<(u64, u32)>,
}

/// Digest the frame's snapshot state: for each entity `0..num_entities` the
/// `entityState_t`-sized prefix of `gentities` (stride `gentitySize`), then
/// for each client slot `0..maxclients` in state `>= CS_CONNECTED` the
/// `playerState_t` bytes at `gameClients + slot*gameClientSize`.
pub fn ref_digest(sv: &Server, maxclients: c_int) -> StateDigest {
    let mut total = FNV_OFFSET;
    let mut entities = FNV_OFFSET;
    let mut players = Vec::new();
    let es_size = core::mem::size_of::<entityState_t>();
    let ps_size = core::mem::size_of::<playerState_t>();

    // Entities: entityState_t `s` is the leading field of sharedEntity_t.
    if !sv.sv.gentities.is_null() && sv.sv.gentitySize > 0 {
        let base = sv.sv.gentities as *const u8;
        let stride = sv.sv.gentitySize as usize;
        for i in 0..sv.sv.num_entities.max(0) as usize {
            let p = unsafe { base.add(i * stride) };
            let es = unsafe { core::slice::from_raw_parts(p, es_size) };
            total = fnv1a64(total, es);
            entities = fnv1a64(entities, es);
        }
    }

    // Clients: playerState_t is the leading field of the game's gclient_s.
    if !sv.sv.gameClients.is_null() && sv.sv.gameClientSize > 0 && !sv.svs.clients.is_empty() {
        let base = sv.sv.gameClients as *const u8;
        let stride = sv.sv.gameClientSize as usize;
        for slot in 0..maxclients.max(0) as usize {
            let cl = &sv.svs.clients[slot];
            if (cl.state as c_int) >= (clientState_t::CS_CONNECTED as c_int) {
                let p = unsafe { base.add(slot * stride) };
                let ps = unsafe { core::slice::from_raw_parts(p, ps_size) };
                total = fnv1a64(total, ps);
                players.push((slot as c_int, fnv1a64(FNV_OFFSET, ps)));
            }
        }
    }

    StateDigest {
        total,
        entities,
        players,
        sys: None,
    }
}

/// Capture the frame's verbose state (`V` record): the same bytes `ref_digest`
/// hashes, kept raw so a follower can byte-diff to a named field.
pub fn ref_vstate(sv: &Server, maxclients: c_int) -> VState {
    let es_size = core::mem::size_of::<entityState_t>();
    let ps_size = core::mem::size_of::<playerState_t>();
    let num_entities = sv.sv.num_entities.max(0);
    let mut ents = Vec::with_capacity(num_entities as usize * es_size);
    if !sv.sv.gentities.is_null() && sv.sv.gentitySize > 0 {
        let base = sv.sv.gentities as *const u8;
        let stride = sv.sv.gentitySize as usize;
        for i in 0..num_entities as usize {
            let p = unsafe { base.add(i * stride) };
            ents.extend_from_slice(unsafe { core::slice::from_raw_parts(p, es_size) });
        }
    }
    let mut players = Vec::new();
    if !sv.sv.gameClients.is_null() && sv.sv.gameClientSize > 0 && !sv.svs.clients.is_empty() {
        let base = sv.sv.gameClients as *const u8;
        let stride = sv.sv.gameClientSize as usize;
        for slot in 0..maxclients.max(0) as usize {
            let cl = &sv.svs.clients[slot];
            if (cl.state as c_int) >= (clientState_t::CS_CONNECTED as c_int) {
                let p = unsafe { base.add(slot * stride) };
                let ps = unsafe { core::slice::from_raw_parts(p, ps_size) };
                players.push((slot as c_int, ps.to_vec()));
            }
        }
    }
    VState {
        num_entities,
        ents,
        players,
    }
}

impl StateDigest {
    /// The tape `S` line payload:
    /// `<total> E<entities> [Y<hash>:<count>] <slot>:<ps> ...`.
    fn to_line(&self) -> String {
        let mut s = format!("{:016x} E{:016x}", self.total, self.entities);
        if let Some((h, n)) = self.sys {
            s.push_str(&format!(" Y{h:016x}:{n}"));
        }
        for (slot, h) in &self.players {
            s.push_str(&format!(" {slot}:{h:016x}"));
        }
        s
    }

    /// Human-readable component delta vs `tape` (which components diverge).
    fn diff_vs(&self, tape: &StateDigest) -> String {
        let mut parts = Vec::new();
        if self.entities != tape.entities {
            parts.push("entities".to_string());
        }
        if let (Some(a), Some(b)) = (self.sys, tape.sys) {
            if a != b {
                parts.push(format!("syscalls(ours={} tape={})", a.1, b.1));
            }
        }
        let ours: std::collections::HashMap<c_int, u64> = self.players.iter().copied().collect();
        let theirs: std::collections::HashMap<c_int, u64> = tape.players.iter().copied().collect();
        for slot in 0..64 {
            match (ours.get(&slot), theirs.get(&slot)) {
                (Some(a), Some(b)) if a != b => parts.push(format!("ps{slot}")),
                (Some(_), None) => parts.push(format!("ps{slot}:ours-only")),
                (None, Some(_)) => parts.push(format!("ps{slot}:tape-only")),
                _ => {}
            }
        }
        if parts.is_empty() {
            parts.push("total-only(?)".to_string());
        }
        parts.join(",")
    }
}

impl VState {
    /// The tape `V` line payload: `<num_entities> <hex ents|-> <slot>:<hex ps> ...`.
    fn to_line(&self) -> String {
        let ents = if self.ents.is_empty() {
            "-".to_string()
        } else {
            hex_encode(&self.ents)
        };
        let mut s = format!("{} {ents}", self.num_entities);
        for (slot, ps) in &self.players {
            s.push_str(&format!(" {slot}:{}", hex_encode(ps)));
        }
        s
    }
}

/// Field-level attribution: byte-diff our state against the tape's `V` record
/// and name the first divergent entity field plus each divergent playerState
/// field (only the FIRST divergent entity — under a cascade the first is the
/// signal, the rest is noise).
fn ref_attribute(ours: &VState, tape: &VState) -> String {
    let es_size = core::mem::size_of::<entityState_t>();
    let mut parts = Vec::new();
    if ours.num_entities != tape.num_entities {
        parts.push(format!(
            "num_entities(ours={} tape={})",
            ours.num_entities, tape.num_entities
        ));
    }
    let n = ours.ents.len().min(tape.ents.len());
    if let Some(off) = (0..n).find(|&i| ours.ents[i] != tape.ents[i]) {
        let slot = off / es_size;
        parts.push(format!("ent{slot}.{}", describe(&ES, off % es_size)));
    }
    let theirs: std::collections::HashMap<c_int, &Vec<u8>> =
        tape.players.iter().map(|(s, b)| (*s, b)).collect();
    for (slot, ours_ps) in &ours.players {
        if let Some(tape_ps) = theirs.get(slot) {
            let m = ours_ps.len().min(tape_ps.len());
            if let Some(off) = (0..m).find(|&i| ours_ps[i] != tape_ps[i]) {
                parts.push(format!("ps{slot}.{}", describe(&PS, off)));
            }
        }
    }
    if parts.is_empty() {
        parts.push("state-bytes-equal".to_string());
    }
    parts.join(" ")
}

/// The aligned 4-byte dword containing byte `off`, as hex (for reports).
fn dword_at(bytes: &[u8], off: usize) -> String {
    let a = off & !3;
    let mut s = String::new();
    for i in a..(a + 4).min(bytes.len()) {
        s.push_str(&format!("{:02x}", bytes[i]));
    }
    s
}

/// Full field-level report for `ref_diff`: every divergent entity (first
/// divergent field + the containing dwords both sides, capped) and every
/// divergent playerState's first field.
fn ref_report(ours: &VState, tape: &VState) -> String {
    let es_size = core::mem::size_of::<entityState_t>();
    let mut lines = Vec::new();
    let n_ents = (ours.ents.len() / es_size).min(tape.ents.len() / es_size);
    let mut shown = 0usize;
    let mut skipped = 0usize;
    for slot in 0..n_ents {
        let a = &ours.ents[slot * es_size..(slot + 1) * es_size];
        let b = &tape.ents[slot * es_size..(slot + 1) * es_size];
        if let Some(off) = (0..es_size).find(|&i| a[i] != b[i]) {
            if shown < 20 {
                lines.push(format!(
                    "  ent{slot} {} ours={} tape={}",
                    describe(&ES, off),
                    dword_at(a, off),
                    dword_at(b, off)
                ));
                shown += 1;
            } else {
                skipped += 1;
            }
        }
    }
    if skipped > 0 {
        lines.push(format!("  ... (+{skipped} more divergent entities)"));
    }
    let theirs: std::collections::HashMap<c_int, &Vec<u8>> =
        tape.players.iter().map(|(s, b)| (*s, b)).collect();
    for (slot, a) in &ours.players {
        if let Some(b) = theirs.get(slot) {
            let m = a.len().min(b.len());
            if let Some(off) = (0..m).find(|&i| a[i] != b[i]) {
                let count = (0..m).filter(|&i| a[i] != b[i]).count();
                lines.push(format!(
                    "  ps{slot} {} ours={} tape={} ({count} divergent bytes)",
                    describe(&PS, off),
                    dword_at(a, off),
                    dword_at(b, off)
                ));
            }
        }
    }
    lines.join("\n")
}

/// Overwrite the module's digested state (entityState prefixes + connected
/// playerStates) with the tape's `V` bytes, written through the
/// LocateGameData-registered memory — the log-mode resync (plan keystone: one
/// divergence must not cascade into noise). Module-private state is untouched
/// (snapshot semantics); genuinely persistent internal drift re-surfaces as a
/// fresh divergence on a later frame.
fn ref_resync(sv: &mut Server, tape: &VState, maxclients: c_int) {
    let es_size = core::mem::size_of::<entityState_t>();
    let ps_size = core::mem::size_of::<playerState_t>();
    if !sv.sv.gentities.is_null() && sv.sv.gentitySize > 0 {
        let base = sv.sv.gentities as *mut u8;
        let stride = sv.sv.gentitySize as usize;
        let n = (sv.sv.num_entities.max(0) as usize)
            .min(tape.num_entities.max(0) as usize)
            .min(tape.ents.len() / es_size);
        for i in 0..n {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    tape.ents.as_ptr().add(i * es_size),
                    base.add(i * stride),
                    es_size,
                );
            }
        }
    }
    if !sv.sv.gameClients.is_null() && sv.sv.gameClientSize > 0 && !sv.svs.clients.is_empty() {
        let base = sv.sv.gameClients as *mut u8;
        let stride = sv.sv.gameClientSize as usize;
        for (slot, bytes) in &tape.players {
            if *slot < 0 || *slot >= maxclients {
                continue;
            }
            let cl = &sv.svs.clients[*slot as usize];
            if (cl.state as c_int) < (clientState_t::CS_CONNECTED as c_int) {
                continue;
            }
            let n = bytes.len().min(ps_size);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    base.add(*slot as usize * stride),
                    n,
                );
            }
        }
    }
}

// ===========================================================================
// Record taps
// ===========================================================================

/// Write the tape `H` header (record mode only), naming the map, `sv_fps`,
/// `sv_maxclients`, and the GAME_INIT `randomSeed` actually used.
pub fn ref_record_header(sv: &mut Server, map: &str, fps: c_int, maxclients: c_int, seed: c_int) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    sv.referee
        .emit(&format!("H {map} {fps} {maxclients} {seed}"));
}

/// RECORD tap at `SV_ClientThink` entry — catches human (`SV_UserMove`) and bot
/// (`BOTLIB_USER_COMMAND` trap) usercmds alike; records the raw struct bytes.
pub fn ref_tap_client_think(sv: &mut Server, cl: *const client_t, cmd: *const usercmd_t) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    let client = unsafe { client_index(sv, cl) };
    let bytes = unsafe { usercmd_bytes(cmd) };
    sv.referee
        .emit(&format!("T {client} {}", hex_encode(&bytes)));
}

/// RECORD tap at `SV_ExecuteClientCommand` entry — catches "begin"/userinfo/say
/// (incl. bot chat via the botlib client-command trap).
pub fn ref_tap_execute_command(sv: &mut Server, cl: *const client_t, s: &str, client_ok: c_int) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    let client = unsafe { client_index(sv, cl) };
    sv.referee.emit(&format!(
        "X {client} {} {}",
        if client_ok != 0 { 1 } else { 0 },
        hex_encode(s.as_bytes())
    ));
}

/// RECORD tap at `SV_DirectConnect` success — human connects only. Bot connects
/// are re-created by the module in replay and are deliberately not recorded.
pub fn ref_tap_direct_connect(sv: &mut Server, client: c_int, userinfo: *const c_char) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    let ui = unsafe { CStr::from_ptr(userinfo) }.to_bytes().to_vec();
    let slot = client as usize;
    if sv.referee.last_ping.len() <= slot {
        sv.referee.last_ping.resize(slot + 1, c_int::MIN);
    }
    sv.referee.last_ping[slot] = c_int::MIN; // new client: force the next P
    sv.referee.emit(&format!("C {client} {}", hex_encode(&ui)));
}

/// RECORD tap at `SV_ClientEnterWorld` — the engine-driven CS_PRIMED ->
/// CS_ACTIVE transition and GAME_CLIENT_BEGIN, with the entering usercmd.
pub fn ref_tap_enter_world(sv: &mut Server, cl: *const client_t, cmd: *const usercmd_t) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    let client = unsafe { client_index(sv, cl) };
    let bytes = unsafe { usercmd_bytes(cmd) };
    sv.referee
        .emit(&format!("B {client} {}", hex_encode(&bytes)));
}

/// RECORD tap at `SV_CalcPings` — the engine writes its network-computed ping
/// into the module's `ps.ping` (digested memory), so the value is a tape
/// input. Emitted on change only.
pub fn ref_tap_ping(sv: &mut Server, client: c_int, ping: c_int) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    let slot = client as usize;
    if sv.referee.last_ping.len() <= slot {
        sv.referee.last_ping.resize(slot + 1, c_int::MIN);
    }
    if sv.referee.last_ping[slot] == ping {
        return;
    }
    sv.referee.last_ping[slot] = ping;
    sv.referee.emit(&format!("P {client} {ping}"));
}

/// REPLAY: the taped ping for an injected slot, if any — `SV_CalcPings`
/// applies it in place of the locally-computed value (a replica has no
/// netchan, so its own computation lands on 999).
pub fn ref_replica_ping(sv: &Server, client: c_int) -> Option<c_int> {
    if sv.referee.mode != RefMode::Replay {
        return None;
    }
    if sv.referee.injected_slots & (1u64 << (client as u32 & 63)) == 0 {
        return None;
    }
    match sv.referee.replica_ping.get(client as usize) {
        Some(&p) if p >= 0 => Some(p),
        _ => None,
    }
}

/// Whether `slot` holds a synthetic replay replica (a `C`-event human the
/// follower materialized). False unless replay is active, so record and normal
/// servers carry no replica behavior. Gates real network I/O to the slot: a
/// replica has a NA_LOOPBACK address but no socket.
pub fn ref_is_replica(sv: &Server, slot: c_int) -> bool {
    sv.referee.mode == RefMode::Replay
        && slot >= 0
        && matches!(sv.referee.replica.get(slot as usize), Some(true))
}

/// Clear a slot's replica marking at reallocation (bot allocate / real
/// connect). The flag survives the drop's CS_ZOMBIE teardown, but a reused
/// slot belongs to its new occupant — leaving it set starves a successor
/// bot's reliable queue via the replica auto-ack (frame-6400-family catch).
pub fn ref_clear_replica(sv: &mut Server, slot: c_int) {
    if slot >= 0 {
        if let Some(r) = sv.referee.replica.get_mut(slot as usize) {
            *r = false;
        }
    }
}

/// Wire tap at `SV_SendMessageToClient` (`ref_snaps`): append the logical
/// message bytes for non-bot clients — byte-for-byte what the client's
/// `CL_ParseServerMessage` consumes after netchan decode (gamestate included,
/// so an offline decoder has the baselines).
pub fn ref_tap_client_message(
    view: &mut EngineHostView,
    sv: &mut Server,
    cl: *const client_t,
    msg: *const msg_t,
) {
    if sv.referee.snap_writer.is_none() {
        return;
    }
    unsafe {
        let client = client_index(sv, cl);
        // Skip real bots (never reach here — SVF_BOT snapshots short-circuit
        // before the send) and synthetic replicas (their taped wire is the
        // primary's, not the follower's regenerated bytes).
        if ref_is_replica(sv, client) || (*cl).netchan.remoteAddress.r#type == netadrtype_t::NA_BOT
        {
            return;
        }
        let len = (*msg).cursize.max(0) as u32;
        let time = sv.svs.time;
        let seq = (*cl).netchan.outgoingSequence;
        let _ = view; // header fields only; kept for signature symmetry
        if let Some(w) = sv.referee.snap_writer.as_mut() {
            let _ = w.write_all(&len.to_le_bytes());
            let _ = w.write_all(&client.to_le_bytes());
            let _ = w.write_all(&time.to_le_bytes());
            let _ = w.write_all(&seq.to_le_bytes());
            let _ = w.write_all(core::slice::from_raw_parts((*msg).data, len as usize));
            let _ = w.flush();
        }
    }
}

/// RECORD tap at `SV_DropClient`.
pub fn ref_tap_drop_client(sv: &mut Server, client: c_int) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    sv.referee.emit(&format!("D {client}"));
}

/// Tap at `SV_GameSystemCalls` entry (record AND replay): fold the ordered
/// import number into the frame window's syscall digest. Pointer-free — only
/// the trap number enters the hash, so differing heap layouts cannot perturb
/// it. Windows run `S` boundary to `S` boundary, covering the packet-loop
/// module calls (human events) that land between `S` and the next `F`.
pub fn ref_tap_syscall(sv: &mut Server, trap: isize) {
    if !sv.referee.active() {
        return;
    }
    sv.referee.sys_hash = fnv1a64(sv.referee.sys_hash, &(trap as i64).to_le_bytes());
    sv.referee.sys_count = sv.referee.sys_count.wrapping_add(1);
    if sv.referee.calls_writer.is_some() {
        sv.referee.calls_buf.push(trap as i32);
    }
}

/// Reset the syscall-digest window (after each `S` emit/compare). When the
/// call dump is armed, flush the window's ordered imports as one `F<n>` line.
fn ref_sys_reset(sv: &mut Server) {
    sv.referee.sys_hash = FNV_OFFSET;
    sv.referee.sys_count = 0;
    sv.referee.injected_transition = false;
    if !sv.referee.calls_buf.is_empty() {
        sv.referee.calls_frame += 1;
        let n = sv.referee.calls_frame;
        let line = sv
            .referee
            .calls_buf
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(w) = sv.referee.calls_writer.as_mut() {
            let _ = writeln!(w, "F{n} {line}");
            let _ = w.flush();
        }
        sv.referee.calls_buf.clear();
    }
}

/// RECORD tap at `SV_Shutdown`: write the tape's `E` end record and flush, so
/// a follower can tell a clean session end from a lagging writer.
pub fn ref_tap_shutdown(sv: &mut Server) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    sv.referee.emit("E");
    if let Some(w) = sv.referee.writer.as_mut() {
        let _ = w.flush();
    }
}

// ===========================================================================
// Per-frame integration (SV_Frame)
// ===========================================================================

/// Follow-mode availability of the next frame block at `cursor`: `Ready` once
/// the block's terminator (its `S`, the next `F`, or the tape `E`) has landed.
enum FollowScan {
    Ready,
    Starved,
    Ended,
}

/// Scan `recs[cursor..]` for a complete next frame block.
fn follow_scan(recs: &[Rec], cursor: usize) -> FollowScan {
    let mut seen_f = false;
    for rec in &recs[cursor..] {
        match rec {
            Rec::End => {
                return if seen_f {
                    FollowScan::Ready
                } else {
                    FollowScan::Ended
                }
            }
            Rec::Frame { .. } if seen_f => return FollowScan::Ready,
            Rec::Frame { .. } => seen_f = true,
            Rec::State { .. } if seen_f => return FollowScan::Ready,
            _ => {}
        }
    }
    FollowScan::Starved
}

/// FOLLOW: drain the tail reader into `recs` (Com_Error on a malformed line —
/// a complete-but-unparsable line is tape corruption, not lag).
fn ref_follow_poll(sv: &mut Server) {
    let Some(tail) = sv.referee.tail.as_mut() else {
        return;
    };
    match tail.poll() {
        Ok(recs) => sv.referee.recs.extend(recs),
        Err(e) => com_error(errorParm_t::ERR_DROP, e),
    }
}

/// Called at `SV_Frame` entry. RECORD: append `F <msec>` and return `msec`
/// unchanged. REPLAY: consume the next `F` (buffering any leading events),
/// return the tape's msec so timeResidual evolves identically; on tape end,
/// print the summary, schedule a quit, and return the input msec inertly.
/// FOLLOW: additionally poll the growing tape first and return `None` —
/// meaning skip this `SV_Frame` call entirely — while the next frame's block
/// is incomplete (starved) or once the tape's `E` end record is reached.
pub fn ref_frame_begin(view: &mut EngineHostView, sv: &mut Server, msec: c_int) -> Option<c_int> {
    match sv.referee.mode {
        RefMode::Off => Some(msec),
        RefMode::Record => {
            // Halt-mode freeze: while the follower's halt file exists, skip
            // frames entirely (`ref_step` lets exactly one through;
            // `ref_resume` removes the file). Console/rcon stay live.
            if sv.referee.halt_on_diverge && ref_halt_file_exists(sv) {
                if sv.referee.step_pending {
                    sv.referee.step_pending = false;
                    // One full game frame per step: force frame_msec so the
                    // stepped frame reaches its S digest (and tape flush)
                    // instead of accumulating a few real-time milliseconds.
                    let fps = view.common.cvar(view.common.sv_fps).integer.max(1);
                    let m = 1000 / fps;
                    sv.referee.emit(&format!("F {m}"));
                    return Some(m);
                }
                sleep(Duration::from_millis(2));
                return None;
            }
            sv.referee.emit(&format!("F {msec}"));
            Some(msec)
        }
        RefMode::Replay => {
            if sv.referee.done {
                return Some(msec);
            }
            // Halt released (`ref_resume` on the primary removed the file):
            // resume, resyncing from the next compared frame's V.
            if sv.referee.halted && !ref_halt_file_exists(sv) {
                sv.referee.halted = false;
                sv.referee.resync_next = true;
                com_printf(
                    view.common,
                    "REF RESUME halt cleared — resyncing from the next frame\n",
                );
            }
            if sv.referee.follow {
                ref_follow_poll(sv);
                match follow_scan(&sv.referee.recs, sv.referee.cursor) {
                    FollowScan::Ready => {}
                    FollowScan::Ended => {
                        ref_replay_finish(view, sv);
                        return None;
                    }
                    FollowScan::Starved => {
                        // Data-paced: yield briefly and let Com_Frame call again.
                        sleep(Duration::from_millis(2));
                        return None;
                    }
                }
            }
            sv.referee.expected = None;
            sv.referee.expected_state = None;
            // Buffer any leading events (e.g. human cmds received before this
            // SV_Frame), then take the frame's F. `pending` must NOT be cleared
            // here: SV_Frame ticks several times per game frame (one F record
            // each), and leading events buffered under an F whose call ran no
            // game step must survive until ref_frame_inject drains them —
            // clearing here ate human C/X/B/T events recorded between ticks
            // (live-session finding, 2026-07-14).
            loop {
                let Some(rec) = sv.referee.recs.get(sv.referee.cursor).cloned() else {
                    ref_replay_finish(view, sv);
                    return Some(msec);
                };
                sv.referee.cursor += 1;
                match rec {
                    Rec::Header { .. } => {}
                    Rec::Frame { msec: m } => return Some(m),
                    Rec::State { .. } | Rec::Verbose { .. } => {}
                    Rec::End => {
                        ref_replay_finish(view, sv);
                        return None;
                    }
                    other => sv.referee.pending.push(other),
                }
            }
        }
    }
}

/// Called in `SV_Frame` just before `SV_BotFrame` (REPLAY only): read this
/// frame's remaining events up to the `S` digest, then inject the buffered
/// tape-created-slot events in exact tape order. The position matters: in the
/// recorded session human events arrived via the packet loop AHEAD of
/// `SV_Frame`, so their module calls (and RNG draws) must land before the bot
/// brains' — injecting after `SV_BotFrame` shifts the module's RNG stream.
/// (The trailing bot thinks read here were recorded during `SV_BotFrame`;
/// they are cursor-alignment data, per-slot-skipped at injection.)
pub fn ref_frame_inject(view: &mut EngineHostView, sv: &mut Server) {
    if sv.referee.mode != RefMode::Replay || sv.referee.done {
        return;
    }
    // Trailing events until the S digest (or the next F, meaning no game ran).
    loop {
        let Some(rec) = sv.referee.recs.get(sv.referee.cursor).cloned() else {
            break;
        };
        match rec {
            Rec::State { digest } => {
                sv.referee.expected = Some(digest);
                sv.referee.cursor += 1;
                break;
            }
            Rec::Verbose { state } => {
                sv.referee.expected_state = Some(state);
                sv.referee.cursor += 1;
            }
            Rec::Frame { .. } => break, // next frame; leave the F for ref_frame_begin
            Rec::End => break,          // session end; leave it for ref_frame_begin
            Rec::Header { .. } => {
                sv.referee.cursor += 1;
            }
            other => {
                sv.referee.pending.push(other);
                sv.referee.cursor += 1;
            }
        }
    }
    // Inject in tape order. ref_inject_one gates per slot: only tape-created
    // (`C`-event) slots inject; module-regenerated bots produce their own
    // thinks, and their taped events serve only to keep the cursor aligned.
    let events = core::mem::take(&mut sv.referee.pending);
    for rec in events {
        ref_inject_one(view, sv, rec);
    }
}

/// Called in `SV_Frame` after the game-run loop. RECORD: if the game ran, append
/// `S <digest>`. REPLAY: if the game ran, compare the digest to the tape's and
/// log/count divergences.
pub fn ref_frame_end(view: &mut EngineHostView, sv: &mut Server, ran_game: bool) {
    if !sv.referee.active() || sv.referee.done {
        return;
    }
    let maxclients = view.common.cvar(view.common.sv_maxclients).integer;
    match sv.referee.mode {
        RefMode::Record => {
            if ran_game {
                if sv.referee.verbose {
                    let v = ref_vstate(sv, maxclients);
                    sv.referee.emit(&format!("V {}", v.to_line()));
                }
                let mut digest = ref_digest(sv, maxclients);
                digest.sys = Some((sv.referee.sys_hash, sv.referee.sys_count));
                sv.referee.emit(&format!("S {}", digest.to_line()));
                // Bound a live follower's lag at one game frame.
                if let Some(w) = sv.referee.writer.as_mut() {
                    let _ = w.flush();
                }
                ref_sys_reset(sv);
            }
        }
        RefMode::Replay => {
            if ran_game {
                if let Some(expected) = sv.referee.expected.take() {
                    let mut ours = ref_digest(sv, maxclients);
                    // Compare syscall streams only when the tape carries a Y.
                    if expected.sys.is_some() {
                        ours.sys = Some((sv.referee.sys_hash, sv.referee.sys_count));
                    }
                    sv.referee.frames += 1;
                    if sv.referee.follow && sv.referee.frames % 500 == 0 {
                        let n = sv.referee.frames;
                        let d = sv.referee.divergences;
                        com_printf(
                            view.common,
                            &format!("REF FOLLOW frames={n} divergences={d}\n"),
                        );
                    }
                    let sys_mismatch = matches!(
                        (ours.sys, expected.sys),
                        (Some(a), Some(b)) if a != b
                    );
                    // A lifecycle transition injected this window makes the
                    // syscall count structurally noisy; suppress the count
                    // comparison (state digest stays authoritative), logging
                    // the suppression so the frame remains visible.
                    let sys_diverged = if sv.referee.injected_transition {
                        if sys_mismatch {
                            let n = sv.referee.frames;
                            com_printf(
                                view.common,
                                &format!(
                                    "REF NOTE frame={n} syscalls suppressed (injected transition)\n"
                                ),
                            );
                        }
                        false
                    } else {
                        sys_mismatch
                    };
                    let diverged = ours.total != expected.total || sys_diverged;
                    let tape_v = sv.referee.expected_state.take();
                    if diverged {
                        sv.referee.divergences += 1;
                        let n = sv.referee.frames;
                        let what = ours.diff_vs(&expected);
                        // Field-level attribution when the tape carries the
                        // frame's verbose state bytes (`ref_state 1`).
                        let first = match &tape_v {
                            Some(tv) if ours.total != expected.total => {
                                let ours_v = ref_vstate(sv, maxclients);
                                sv.referee.last_report = format!(
                                    "REF DIFF frame={n} components={what}\n{}\n",
                                    ref_report(&ours_v, tv)
                                );
                                format!(" first={}", ref_attribute(&ours_v, tv))
                            }
                            _ => {
                                sv.referee.last_report =
                                    format!("REF DIFF frame={n} components={what} (no V record — set ref_state 1 on the primary for field detail)\n");
                                String::new()
                            }
                        };
                        com_printf(
                            view.common,
                            &format!(
                                "REF DIVERGE frame={n} components={what}{first} ours={:016x} tape={:016x}\n",
                                ours.total, expected.total
                            ),
                        );
                        if sv.referee.resync_next {
                            // Post-resume (halt mode): this divergence is the
                            // expected residue of the split — resync instead
                            // of re-halting, then compare cleanly onward.
                            if let Some(tv) = &tape_v {
                                ref_resync(sv, tv, maxclients);
                                sv.referee.resync_next = false;
                                com_printf(view.common, &format!("REF RESYNC frame={n}\n"));
                            }
                        } else if sv.referee.halt_on_diverge {
                            if !sv.referee.halted {
                                sv.referee.halted = true;
                                ref_halt_file_write(sv, n);
                                com_printf(
                                    view.common,
                                    "REF HALT both engines frozen — ref_step (primary) advances one frame, ref_diff (secondary rcon) shows the delta, ref_resume (primary) resyncs and continues\n",
                                );
                            }
                        } else if let Some(tv) = &tape_v {
                            // Log-and-continue: resync from the primary's
                            // authoritative snapshot so one divergence does
                            // not cascade into noise.
                            ref_resync(sv, tv, maxclients);
                            com_printf(view.common, &format!("REF RESYNC frame={n}\n"));
                        }
                    } else {
                        sv.referee.resync_next = false;
                    }
                    ref_sys_reset(sv);
                }
            }
        }
        RefMode::Off => {}
    }
}

/// Whether the referee is in replay mode (used to skip the wall-clock frame
/// sleep — replay runs faster than real time).
pub fn ref_is_replay(sv: &Server) -> bool {
    sv.referee.mode == RefMode::Replay
}

// ===========================================================================
// Divergence UX (halt/step/resync — plan G5)
// ===========================================================================

/// Whether the halt back-channel file currently exists.
fn ref_halt_file_exists(sv: &Server) -> bool {
    sv.referee
        .halt_path
        .as_deref()
        .is_some_and(|p| Path::new(p).exists())
}

/// FOLLOW: create the halt file (freezes the polling primary).
fn ref_halt_file_write(sv: &Server, frame: u64) {
    if let Some(p) = sv.referee.halt_path.as_deref() {
        let _ = std::fs::write(p, format!("diverged frame={frame}\n"));
    }
}

/// `ref_step` console command — primary (record) only: let exactly one frame
/// through the halt freeze; the follower steps automatically when the frame's
/// block lands on the tape.
pub fn ref_step_cmd(view: &mut EngineHostView, sv: &mut Server) {
    match sv.referee.mode {
        RefMode::Record => {
            sv.referee.step_pending = true;
            com_printf(view.common, "REF STEP one frame\n");
        }
        _ => com_printf(
            view.common,
            "ref_step: record-side (primary) only — the follower steps automatically\n",
        ),
    }
}

/// `ref_resume` console command — primary (record) only: remove the halt file;
/// the frozen follower notices, resyncs from the next frame's V, and continues.
pub fn ref_resume_cmd(view: &mut EngineHostView, sv: &mut Server) {
    match sv.referee.mode {
        RefMode::Record => {
            if let Some(p) = sv.referee.halt_path.as_deref() {
                let _ = std::fs::remove_file(p);
            }
            com_printf(view.common, "REF RESUME halt file cleared\n");
        }
        _ => com_printf(view.common, "ref_resume: record-side (primary) only\n"),
    }
}

/// `ref_diff` console command — follower: print the last divergence's full
/// field-level report (works over rcon; the primary redirects the print).
pub fn ref_diff_cmd(view: &mut EngineHostView, sv: &mut Server) {
    if sv.referee.last_report.is_empty() {
        com_printf(view.common, "ref_diff: no divergence recorded\n");
    } else {
        let report = sv.referee.last_report.clone();
        com_printf(view.common, &report);
    }
}

// ===========================================================================
// Replay injection
// ===========================================================================

/// Inject one recorded event into the live engine at the same seam its record
/// tap fired from. Gated per slot: a `C` event marks its slot tape-created and
/// materializes the replica; `T`/`X`/`D` inject only for marked slots —
/// module-regenerated bots re-think on their own, and injecting their taped
/// events on top would double-think them.
fn ref_inject_one(view: &mut EngineHostView, sv: &mut Server, rec: Rec) {
    match rec {
        Rec::Think { client, cmd } => {
            if !ref_slot_injected(sv, view, client) {
                return;
            }
            let cl = &mut sv.svs.clients[client as usize] as *mut client_t;
            let mut ucmd: usercmd_t = unsafe { core::mem::zeroed() };
            let n = cmd.len().min(core::mem::size_of::<usercmd_t>());
            unsafe {
                core::ptr::copy_nonoverlapping(
                    cmd.as_ptr(),
                    &mut ucmd as *mut usercmd_t as *mut u8,
                    n,
                );
                // A replica receives no real packets; the taped event IS the
                // packet — refresh the clock or SV_CheckTimeouts drops it
                // after sv_timeout (live-session finding: phantom drop ~200s in).
                (*cl).lastPacketTime = sv.svs.time;
            }
            SV_ClientThink(view.common, sv, cl, &mut ucmd);
        }
        Rec::Command { client, ok, cmd } => {
            if !ref_slot_injected(sv, view, client) {
                return;
            }
            let cl = &mut sv.svs.clients[client as usize] as *mut client_t;
            unsafe {
                (*cl).lastPacketTime = sv.svs.time; // see Rec::Think
            }
            let cmd_str = String::from_utf8_lossy(&cmd).into_owned();
            SV_ExecuteClientCommand(view, sv, cl, &cmd_str, if ok != 0 { qtrue } else { qfalse });
        }
        Rec::Connect { client, userinfo } => {
            if !ref_client_in_range(view, client) {
                return;
            }
            sv.referee.injected_slots |= 1u64 << (client as u32 & 63);
            sv.referee.injected_transition = true;
            let slot = client as usize;
            if sv.referee.replica_ping.len() <= slot {
                sv.referee.replica_ping.resize(slot + 1, -1);
            }
            sv.referee.replica_ping[slot] = -1; // new client: no taped ping yet
            if sv.referee.replica.len() <= slot {
                sv.referee.replica.resize(slot + 1, false);
            }
            sv.referee.replica[slot] = true;
            ref_inject_connect(view, sv, client, userinfo.as_bytes());
        }
        Rec::Ping { client, ping } => {
            if !ref_slot_injected(sv, view, client) {
                return;
            }
            let slot = client as usize;
            if sv.referee.replica_ping.len() <= slot {
                sv.referee.replica_ping.resize(slot + 1, -1);
            }
            sv.referee.replica_ping[slot] = ping;
        }
        Rec::Begin { client, cmd } => {
            if !ref_slot_injected(sv, view, client) {
                return;
            }
            sv.referee.injected_transition = true;
            let cl = &mut sv.svs.clients[client as usize] as *mut client_t;
            let mut ucmd: usercmd_t = unsafe { core::mem::zeroed() };
            let n = cmd.len().min(core::mem::size_of::<usercmd_t>());
            unsafe {
                core::ptr::copy_nonoverlapping(
                    cmd.as_ptr(),
                    &mut ucmd as *mut usercmd_t as *mut u8,
                    n,
                );
            }
            SV_ClientEnterWorld(view.common, sv, cl, &mut ucmd);
        }
        Rec::Drop { client } => {
            if !ref_slot_injected(sv, view, client) {
                return;
            }
            sv.referee.injected_transition = true;
            // `replica` stays set through the CS_ZOMBIE teardown so its final
            // frames remain I/O-suppressed; the next `C` on this slot resets it.
            sv.referee.injected_slots &= !(1u64 << (client as u32 & 63));
            let cl = &mut sv.svs.clients[client as usize] as *mut client_t;
            crate::SV_DropClient(view.common, sv, cl, "replay drop");
        }
        Rec::Header { .. }
        | Rec::Frame { .. }
        | Rec::State { .. }
        | Rec::Verbose { .. }
        | Rec::End => {}
    }
}

/// Whether `client` is a valid slot index.
fn ref_client_in_range(view: &mut EngineHostView, client: c_int) -> bool {
    let maxclients = view.common.cvar(view.common.sv_maxclients).integer;
    client >= 0 && client < maxclients
}

/// Whether `client` is a valid, tape-created (`C`-event) slot — the injection
/// gate for `T`/`X`/`D` events.
fn ref_slot_injected(sv: &Server, view: &mut EngineHostView, client: c_int) -> bool {
    ref_client_in_range(view, client)
        && sv.referee.injected_slots & (1u64 << (client as u32 & 63)) != 0
}

/// Materialize a synthetic, netchan-less replica of a recorded human client in
/// the given slot: a NA_LOOPBACK (human-class) remoteAddress so NA_BOT-branching
/// engine sites take the human path, with `ref_is_replica` gating the transmit
/// paths that would otherwise do real I/O. Install the recorded userinfo, then run
/// GAME_CLIENT_CONNECT with firstTime=qtrue, isBot=qfalse. The replica stays
/// CS_CONNECTED until its taped `B` event drives `SV_ClientEnterWorld`
/// (CS_ACTIVE + GAME_CLIENT_BEGIN), mirroring the real connect flow.
fn ref_inject_connect(view: &mut EngineHostView, sv: &mut Server, client: c_int, userinfo: &[u8]) {
    if !ref_client_in_range(view, client) {
        return;
    }
    let cl = &mut sv.svs.clients[client as usize] as *mut client_t;
    unsafe {
        // Mirror SV_DirectConnect exactly. It does NOT write ent->s.number
        // (digested memory — the slot keeps its stale value until the module
        // initializes it at spawn); it links the gentity, installs the
        // userinfo, runs GAME_CLIENT_CONNECT, THEN SV_UserinfoChanged.
        let ent = SV_GentityNum(sv, client);
        (*cl).gentity = ent;
        // A human-class address (the primary's real IP is unknowable), so every
        // NA_BOT-branching engine site takes the human path the primary ran.
        // The replica has no socket: `ref_is_replica` gates the transmit paths.
        (*cl).netchan.remoteAddress.r#type = netadrtype_t::NA_LOOPBACK;
        (*cl).rate = 16384;

        let mut ui = userinfo.to_vec();
        ui.push(0);
        Q_strncpyz(
            (*cl).userinfo.as_mut_ptr(),
            ui.as_ptr() as *const c_char,
            (*cl).userinfo.len() as c_int,
        );
    }
    VM_Call(
        view.common,
        sv.gvm,
        MpGameExport::GAME_CLIENT_CONNECT as c_int,
        &[client as isize, qtrue as isize, qfalse as isize],
    );
    unsafe {
        SV_UserinfoChanged(view, cl);
        (*cl).state = clientState_t::CS_CONNECTED;
        (*cl).lastPacketTime = sv.svs.time;
    }
}

/// End of tape (replay): print the summary and schedule a clean quit. Uses
/// `Cbuf_AddText("quit")` rather than an immediate `Com_Quit` so no second
/// `&mut Server` is created while `SV_Frame`'s borrow is live.
fn ref_replay_finish(view: &mut EngineHostView, sv: &mut Server) {
    if sv.referee.done {
        return;
    }
    sv.referee.done = true;
    let n = sv.referee.frames;
    let d = sv.referee.divergences;
    com_printf(
        view.common,
        &format!("REF REPLAY COMPLETE frames={n} divergences={d}\n"),
    );
    Cbuf_AddText(view.common, "quit\n");
}

// ===========================================================================
// Tape (de)serialization
// ===========================================================================

/// Header fields extracted from a parsed tape.
pub struct RefHeader {
    pub map: String,
    pub fps: c_int,
    pub maxclients: c_int,
    pub seed: c_int,
}

/// Open the record file, switching the referee into RECORD mode. Any prior file
/// at `path` is truncated.
pub fn ref_open_record(sv: &mut Server, path: &str) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("ref_record: cannot create {path}: {e}"))?;
    sv.referee.writer = Some(BufWriter::new(file));
    sv.referee.mode = RefMode::Record;
    Ok(())
}

/// The first `H` header among `recs`, if parsed yet.
fn header_of(recs: &[Rec]) -> Option<RefHeader> {
    recs.iter().find_map(|r| match r {
        Rec::Header {
            map,
            fps,
            maxclients,
            seed,
        } => Some(RefHeader {
            map: map.clone(),
            fps: *fps,
            maxclients: *maxclients,
            seed: *seed,
        }),
        _ => None,
    })
}

/// Parse the replay tape, switching the referee into REPLAY mode, and return the
/// tape header for launch-cvar validation.
pub fn ref_open_replay(sv: &mut Server, path: &str) -> Result<RefHeader, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("ref_replay: cannot read {path}: {e}"))?;
    let recs = parse_tape(&text)?;
    let header = header_of(&recs).ok_or_else(|| "ref_replay: tape has no H header".to_string())?;
    sv.referee.header_seed = header.seed;
    sv.referee.recs = recs;
    sv.referee.cursor = 0;
    sv.referee.mode = RefMode::Replay;
    Ok(header)
}

/// FOLLOW: attach to a tape another engine is still writing, switching the
/// referee into REPLAY+follow. Bounded-waits (60s) for the primary to create
/// the file and land its `H` header, since the follower typically boots
/// moments after the primary.
pub fn ref_open_follow(sv: &mut Server, path: &str) -> Result<RefHeader, String> {
    let mut tail: Option<TailReader> = None;
    for _ in 0..600 {
        if tail.is_none() {
            if let Ok(f) = File::open(path) {
                tail = Some(TailReader::new(f));
            }
        }
        if let Some(t) = tail.as_mut() {
            let recs = t.poll()?;
            sv.referee.recs.extend(recs);
            if let Some(header) = header_of(&sv.referee.recs) {
                sv.referee.header_seed = header.seed;
                sv.referee.cursor = 0;
                sv.referee.mode = RefMode::Replay;
                sv.referee.follow = true;
                sv.referee.tail = tail;
                return Ok(header);
            }
        }
        sleep(Duration::from_millis(100));
    }
    Err(format!(
        "ref_follow: no tape header at {path} after 60s (is the primary running?)"
    ))
}

// ===========================================================================
// SV_SpawnServer integration
// ===========================================================================

/// Read the referee cvars and arm the mode for this map load. Called from
/// `SV_SpawnServer` just before `SV_InitGameProgs`, so the seed pin lands on the
/// GAME_INIT `Com_Milliseconds` read. In replay it parses the tape and validates
/// the launch map/fps/maxclients against the `H` header (Com_Error on mismatch).
///
/// The seed pin: `ref_seed` (nonzero) wins for both record and replay; otherwise
/// replay pins the tape header's seed and record leaves the natural clock value
/// (still captured for the header). The pin is armed on `Common` regardless, so
/// the value actually used is captured for the tape `H` header.
pub fn ref_spawn_setup(view: &mut EngineHostView, sv: &mut Server, map: &str) {
    // Reset any prior map's referee state (map_restart / consecutive maps).
    sv.referee = Referee::default();
    view.common.ref_seed_arm = false;
    view.common.ref_seed_pin = 0;
    view.common.ref_seed_used = 0;

    let record = Cvar_VariableString(view.common, "ref_record").to_string();
    let replay = Cvar_VariableString(view.common, "ref_replay").to_string();
    let follow = Cvar_VariableIntegerValue(view.common, "ref_follow") != 0;
    let ref_seed = Cvar_VariableIntegerValue(view.common, "ref_seed");
    let fps = Cvar_VariableIntegerValue(view.common, "sv_fps");
    let maxclients = view.common.cvar(view.common.sv_maxclients).integer;

    if !replay.is_empty() {
        let opened = if follow {
            ref_open_follow(sv, &replay)
        } else {
            ref_open_replay(sv, &replay)
        };
        match opened {
            Ok(header) => {
                if header.map != map {
                    com_error(
                        errorParm_t::ERR_DROP,
                        format!(
                            "ref_replay: tape map {:?} != launch map {:?}",
                            header.map, map
                        ),
                    );
                }
                if header.fps != fps {
                    com_error(
                        errorParm_t::ERR_DROP,
                        format!("ref_replay: tape sv_fps {} != launch {}", header.fps, fps),
                    );
                }
                if header.maxclients != maxclients {
                    com_error(
                        errorParm_t::ERR_DROP,
                        format!(
                            "ref_replay: tape sv_maxclients {} != launch {}",
                            header.maxclients, maxclients
                        ),
                    );
                }
                let mode = if follow { "FOLLOW" } else { "REPLAY" };
                com_printf(
                    view.common,
                    &format!("REF {mode} start map={map} fps={fps} maxclients={maxclients}\n"),
                );
            }
            Err(e) => com_error(errorParm_t::ERR_DROP, e),
        }
    } else if !record.is_empty() {
        if let Err(e) = ref_open_record(sv, &record) {
            com_error(errorParm_t::ERR_DROP, e);
        }
        com_printf(
            view.common,
            &format!("REF RECORD start map={map} -> {record}\n"),
        );
    }

    // Wire tap, armed independently of record/replay.
    let snaps = Cvar_VariableString(view.common, "ref_snaps").to_string();
    if !snaps.is_empty() {
        match File::create(&snaps) {
            Ok(f) => {
                sv.referee.snap_writer = Some(BufWriter::new(f));
                com_printf(view.common, &format!("REF SNAPS tap -> {snaps}\n"));
            }
            Err(e) => com_error(
                errorParm_t::ERR_DROP,
                format!("ref_snaps: cannot create {snaps}: {e}"),
            ),
        }
    }

    // Verbose state records (`ref_state 1`, record side) and the syscall-digest
    // window basis (both modes).
    sv.referee.verbose = sv.referee.mode == RefMode::Record
        && Cvar_VariableIntegerValue(view.common, "ref_state") != 0;
    ref_sys_reset(sv);

    // Syscall-stream dump (`ref_calls <file>`), armed independently.
    let calls = Cvar_VariableString(view.common, "ref_calls").to_string();
    if !calls.is_empty() {
        match File::create(&calls) {
            Ok(f) => {
                sv.referee.calls_writer = Some(BufWriter::new(f));
                com_printf(view.common, &format!("REF CALLS dump -> {calls}\n"));
            }
            Err(e) => com_error(
                errorParm_t::ERR_DROP,
                format!("ref_calls: cannot create {calls}: {e}"),
            ),
        }
    }

    // Divergence policy + the halt back-channel (`<tape>.halt`). The record
    // side clears any stale halt file so a fresh session never boots frozen.
    sv.referee.halt_on_diverge = Cvar_VariableIntegerValue(view.common, "ref_haltOnDiverge") != 0;
    let tape_path = if !replay.is_empty() {
        Some(replay.clone())
    } else if !record.is_empty() {
        Some(record.clone())
    } else {
        None
    };
    sv.referee.halt_path = tape_path.map(|p| format!("{p}.halt"));
    if sv.referee.mode == RefMode::Record {
        if let Some(p) = sv.referee.halt_path.as_deref() {
            let _ = std::fs::remove_file(p);
        }
    }

    // Arm the GAME_INIT seed pin whenever a mode is active, so the seed used is
    // captured for the header and forced when required.
    if sv.referee.active() {
        view.common.ref_seed_pin = if ref_seed != 0 {
            ref_seed
        } else if sv.referee.mode == RefMode::Replay {
            sv.referee.header_seed
        } else {
            0
        };
        view.common.ref_seed_arm = true;
    }
}

/// Write the tape `H` header (record mode) once the GAME_INIT seed has been
/// pinned/captured. Called from `SV_SpawnServer` just after `SV_InitGameProgs`.
pub fn ref_spawn_write_header(view: &mut EngineHostView, sv: &mut Server, map: &str) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    let fps = Cvar_VariableIntegerValue(view.common, "sv_fps");
    let maxclients = view.common.cvar(view.common.sv_maxclients).integer;
    let seed = view.common.ref_seed_used;
    ref_record_header(sv, map, fps, maxclients, seed);
}

/// Parse tape text into records.
fn parse_tape(text: &str) -> Result<Vec<Rec>, String> {
    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        if let Some(rec) = parse_line(line, lineno + 1)? {
            out.push(rec);
        }
    }
    Ok(out)
}

/// Parse one tape line into a record (`None` for a blank line).
fn parse_line(line: &str, lineno: usize) -> Result<Option<Rec>, String> {
    let line = line.trim_end();
    if line.is_empty() {
        return Ok(None);
    }
    let mut it = line.split(' ');
    let tag = it.next().unwrap_or("");
    let bad = || format!("ref tape: malformed line {lineno}: {line:?}");
    let rec = match tag {
        "H" => {
            let map = it.next().ok_or_else(bad)?.to_string();
            let fps = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let maxclients = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let seed = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            Rec::Header {
                map,
                fps,
                maxclients,
                seed,
            }
        }
        "C" => {
            let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let userinfo =
                String::from_utf8_lossy(&hex_decode(it.next().unwrap_or(""))).into_owned();
            Rec::Connect { client, userinfo }
        }
        "X" => {
            let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let ok = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let cmd = hex_decode(it.next().unwrap_or(""));
            Rec::Command { client, ok, cmd }
        }
        "T" => {
            let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let cmd = hex_decode(it.next().unwrap_or(""));
            Rec::Think { client, cmd }
        }
        "B" => {
            let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let cmd = hex_decode(it.next().unwrap_or(""));
            Rec::Begin { client, cmd }
        }
        "D" => {
            let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            Rec::Drop { client }
        }
        "P" => {
            let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let ping = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            Rec::Ping { client, ping }
        }
        "F" => {
            let msec = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            Rec::Frame { msec }
        }
        "S" => {
            let total = it
                .next()
                .and_then(|s| u64::from_str_radix(s, 16).ok())
                .ok_or_else(bad)?;
            let mut entities = 0u64;
            let mut sys = None;
            let mut players = Vec::new();
            for tok in it.by_ref() {
                if let Some(e) = tok.strip_prefix('E') {
                    entities = u64::from_str_radix(e, 16).map_err(|_| bad())?;
                } else if let Some(y) = tok.strip_prefix('Y') {
                    // Y before the generic slot:hash arm — it also has a ':'.
                    let (h, n) = y.split_once(':').ok_or_else(bad)?;
                    sys = Some((
                        u64::from_str_radix(h, 16).map_err(|_| bad())?,
                        n.parse().map_err(|_| bad())?,
                    ));
                } else if let Some((slot, h)) = tok.split_once(':') {
                    players.push((
                        slot.parse().map_err(|_| bad())?,
                        u64::from_str_radix(h, 16).map_err(|_| bad())?,
                    ));
                } else {
                    return Err(bad());
                }
            }
            Rec::State {
                digest: StateDigest {
                    total,
                    entities,
                    players,
                    sys,
                },
            }
        }
        "V" => {
            let num_entities = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let ents_tok = it.next().ok_or_else(bad)?;
            let ents = if ents_tok == "-" {
                Vec::new()
            } else {
                hex_decode(ents_tok)
            };
            let mut players = Vec::new();
            for tok in it.by_ref() {
                let (slot, ps) = tok.split_once(':').ok_or_else(bad)?;
                players.push((slot.parse().map_err(|_| bad())?, hex_decode(ps)));
            }
            Rec::Verbose {
                state: VState {
                    num_entities,
                    ents,
                    players,
                },
            }
        }
        "E" => Rec::End,
        _ => return Err(bad()),
    };
    Ok(Some(rec))
}

/// Incremental tape reader for FOLLOW mode: drains newly appended bytes each
/// poll, holding any trailing partial line until its newline arrives.
struct TailReader {
    file: File,
    /// Bytes after the last complete line (no `\n` seen yet).
    partial: Vec<u8>,
    /// Lines fully consumed so far (for parse-error messages).
    lineno: usize,
}

impl TailReader {
    fn new(file: File) -> Self {
        TailReader {
            file,
            partial: Vec::new(),
            lineno: 0,
        }
    }

    /// Read to the current EOF and parse every newly completed line.
    fn poll(&mut self) -> Result<Vec<Rec>, String> {
        let mut buf = Vec::new();
        self.file
            .read_to_end(&mut buf)
            .map_err(|e| format!("ref_follow: tape read error: {e}"))?;
        if !buf.is_empty() {
            self.partial.extend_from_slice(&buf);
        }
        let mut out = Vec::new();
        while let Some(pos) = self.partial.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.partial.drain(..=pos).collect();
            self.lineno += 1;
            let line = String::from_utf8_lossy(&line[..line.len() - 1]).into_owned();
            if let Some(rec) = parse_line(&line, self.lineno)? {
                out.push(rec);
            }
        }
        Ok(out)
    }
}
