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
use std::io::{BufWriter, Write};

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
    /// `F <msec>` — the frame's msec input.
    Frame { msec: c_int },
    /// `S <total> E<entities> <slot>:<ps> ...` — the post-frame state digest:
    /// the combined hash, the entity-block aggregate, and one playerState hash
    /// per connected slot, so a divergence names its component.
    State { digest: StateDigest },
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
    /// REPLAY: header seed to pin GAME_INIT with (mirrored to `Common`).
    header_seed: c_int,
    /// REPLAY: bitmask of client slots the TAPE created (`C` events — humans by
    /// construction). Only these slots' `T`/`X`/`D` events are injected;
    /// module-regenerated bots re-think on their own and injecting their taped
    /// events would double-think them. Cleared per slot on an injected `D`.
    injected_slots: u64,
    /// REPLAY: events buffered for the current frame's injection.
    pending: Vec<Rec>,
    /// REPLAY: the current frame's expected digest, if the frame runs the game.
    expected: Option<StateDigest>,
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
    ((cl as *const u8).offset_from(sv.svs.clients as *const u8) as isize
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
    if !sv.sv.gameClients.is_null() && sv.sv.gameClientSize > 0 && !sv.svs.clients.is_null() {
        let base = sv.sv.gameClients as *const u8;
        let stride = sv.sv.gameClientSize as usize;
        for slot in 0..maxclients.max(0) as usize {
            let cl = unsafe { &*sv.svs.clients.add(slot) };
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
    }
}

impl StateDigest {
    /// The tape `S` line payload: `<total> E<entities> <slot>:<ps> ...`.
    fn to_line(&self) -> String {
        let mut s = format!("{:016x} E{:016x}", self.total, self.entities);
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
pub fn ref_tap_execute_command(
    sv: &mut Server,
    cl: *const client_t,
    s: *const c_char,
    client_ok: c_int,
) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    let client = unsafe { client_index(sv, cl) };
    let cmd = unsafe { CStr::from_ptr(s) }.to_bytes().to_vec();
    sv.referee.emit(&format!(
        "X {client} {} {}",
        if client_ok != 0 { 1 } else { 0 },
        hex_encode(&cmd)
    ));
}

/// RECORD tap at `SV_DirectConnect` success — human connects only. Bot connects
/// are re-created by the module in replay and are deliberately not recorded.
pub fn ref_tap_direct_connect(sv: &mut Server, client: c_int, userinfo: *const c_char) {
    if sv.referee.mode != RefMode::Record {
        return;
    }
    let ui = unsafe { CStr::from_ptr(userinfo) }.to_bytes().to_vec();
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
        if (*cl).netchan.remoteAddress.r#type == netadrtype_t::NA_BOT {
            return;
        }
        let client = client_index(sv, cl);
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

// ===========================================================================
// Per-frame integration (SV_Frame)
// ===========================================================================

/// Called at `SV_Frame` entry. RECORD: append `F <msec>` and return `msec`
/// unchanged. REPLAY: consume the next `F` (buffering any leading events),
/// return the tape's msec so timeResidual evolves identically; on tape end,
/// print the summary, schedule a quit, and return the input msec inertly.
pub fn ref_frame_begin(view: &mut EngineHostView, sv: &mut Server, msec: c_int) -> c_int {
    match sv.referee.mode {
        RefMode::Off => msec,
        RefMode::Record => {
            sv.referee.emit(&format!("F {msec}"));
            msec
        }
        RefMode::Replay => {
            if sv.referee.done {
                return msec;
            }
            sv.referee.expected = None;
            sv.referee.pending.clear();
            // Buffer any leading events (e.g. human cmds received before this
            // SV_Frame), then take the frame's F.
            loop {
                let Some(rec) = sv.referee.recs.get(sv.referee.cursor).cloned() else {
                    ref_replay_finish(view, sv);
                    return msec;
                };
                sv.referee.cursor += 1;
                match rec {
                    Rec::Header { .. } => {}
                    Rec::Frame { msec: m } => return m,
                    Rec::State { .. } => {}
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
            Rec::Frame { .. } => break, // next frame; leave the F for ref_frame_begin
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
    let maxclients = unsafe { (*view.common.sv_maxclients).integer };
    match sv.referee.mode {
        RefMode::Record => {
            if ran_game {
                let digest = ref_digest(sv, maxclients);
                sv.referee.emit(&format!("S {}", digest.to_line()));
            }
        }
        RefMode::Replay => {
            if ran_game {
                if let Some(expected) = sv.referee.expected.take() {
                    let ours = ref_digest(sv, maxclients);
                    sv.referee.frames += 1;
                    if ours.total != expected.total {
                        sv.referee.divergences += 1;
                        let n = sv.referee.frames;
                        let what = ours.diff_vs(&expected);
                        com_printf(
                            view.common,
                            &format!(
                                "REF DIVERGE frame={n} components={what} ours={:016x} tape={:016x}\n",
                                ours.total, expected.total
                            ),
                        );
                    }
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
            let cl = unsafe { sv.svs.clients.add(client as usize) };
            let mut ucmd: usercmd_t = unsafe { core::mem::zeroed() };
            let n = cmd.len().min(core::mem::size_of::<usercmd_t>());
            unsafe {
                core::ptr::copy_nonoverlapping(
                    cmd.as_ptr(),
                    &mut ucmd as *mut usercmd_t as *mut u8,
                    n,
                );
            }
            SV_ClientThink(view.common, sv, cl, &mut ucmd);
        }
        Rec::Command { client, ok, cmd } => {
            if !ref_slot_injected(sv, view, client) {
                return;
            }
            let cl = unsafe { sv.svs.clients.add(client as usize) };
            let mut cstr = cmd.clone();
            cstr.push(0);
            SV_ExecuteClientCommand(
                view,
                sv,
                cl,
                cstr.as_ptr() as *const c_char,
                if ok != 0 { qtrue } else { qfalse },
            );
        }
        Rec::Connect { client, userinfo } => {
            if !ref_client_in_range(view, client) {
                return;
            }
            sv.referee.injected_slots |= 1u64 << (client as u32 & 63);
            ref_inject_connect(view, sv, client, userinfo.as_bytes());
        }
        Rec::Begin { client, cmd } => {
            if !ref_slot_injected(sv, view, client) {
                return;
            }
            let cl = unsafe { sv.svs.clients.add(client as usize) };
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
            sv.referee.injected_slots &= !(1u64 << (client as u32 & 63));
            let cl = unsafe { sv.svs.clients.add(client as usize) };
            crate::SV_DropClient(view.common, sv, cl, c"replay drop".as_ptr());
        }
        Rec::Header { .. } | Rec::Frame { .. } | Rec::State { .. } => {}
    }
}

/// Whether `client` is a valid slot index.
fn ref_client_in_range(view: &mut EngineHostView, client: c_int) -> bool {
    let maxclients = unsafe { (*view.common.sv_maxclients).integer };
    client >= 0 && client < maxclients
}

/// Whether `client` is a valid, tape-created (`C`-event) slot — the injection
/// gate for `T`/`X`/`D` events.
fn ref_slot_injected(sv: &Server, view: &mut EngineHostView, client: c_int) -> bool {
    ref_client_in_range(view, client)
        && sv.referee.injected_slots & (1u64 << (client as u32 & 63)) != 0
}

/// Materialize a synthetic, netchan-less replica of a recorded human client in
/// the given slot (the `SV_BotAllocateClient` pattern: NA_BOT remoteAddress so
/// snapshots skip it), install the recorded userinfo, then run
/// GAME_CLIENT_CONNECT with firstTime=qtrue, isBot=qfalse. The replica stays
/// CS_CONNECTED until its taped `B` event drives `SV_ClientEnterWorld`
/// (CS_ACTIVE + GAME_CLIENT_BEGIN), mirroring the real connect flow.
fn ref_inject_connect(view: &mut EngineHostView, sv: &mut Server, client: c_int, userinfo: &[u8]) {
    if !ref_client_in_range(view, client) {
        return;
    }
    let cl = unsafe { sv.svs.clients.add(client as usize) };
    unsafe {
        // Mirror SV_DirectConnect exactly. It does NOT write ent->s.number
        // (digested memory — the slot keeps its stale value until the module
        // initializes it at spawn); it links the gentity, installs the
        // userinfo, runs GAME_CLIENT_CONNECT, THEN SV_UserinfoChanged.
        let ent = SV_GentityNum(sv, client);
        (*cl).gentity = ent;
        (*cl).netchan.remoteAddress.r#type = netadrtype_t::NA_BOT;
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
    Cbuf_AddText(view.common, c"quit\n".as_ptr());
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

/// Parse the replay tape, switching the referee into REPLAY mode, and return the
/// tape header for launch-cvar validation.
pub fn ref_open_replay(sv: &mut Server, path: &str) -> Result<RefHeader, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("ref_replay: cannot read {path}: {e}"))?;
    let recs = parse_tape(&text)?;
    let header = recs.iter().find_map(|r| match r {
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
    });
    let header = header.ok_or_else(|| "ref_replay: tape has no H header".to_string())?;
    sv.referee.header_seed = header.seed;
    sv.referee.recs = recs;
    sv.referee.cursor = 0;
    sv.referee.mode = RefMode::Replay;
    Ok(header)
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

    let record = cvar_string(view, c"ref_record".as_ptr());
    let replay = cvar_string(view, c"ref_replay".as_ptr());
    let ref_seed = Cvar_VariableIntegerValue(view.common, c"ref_seed".as_ptr());
    let fps = Cvar_VariableIntegerValue(view.common, c"sv_fps".as_ptr());
    let maxclients = unsafe { (*view.common.sv_maxclients).integer };

    if !replay.is_empty() {
        match ref_open_replay(sv, &replay) {
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
                com_printf(
                    view.common,
                    &format!("REF REPLAY start map={map} fps={fps} maxclients={maxclients}\n"),
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
    let snaps = cvar_string(view, c"ref_snaps".as_ptr());
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
    let fps = Cvar_VariableIntegerValue(view.common, c"sv_fps".as_ptr());
    let maxclients = unsafe { (*view.common.sv_maxclients).integer };
    let seed = view.common.ref_seed_used;
    ref_record_header(sv, map, fps, maxclients, seed);
}

/// The value of cvar `name` as an owned `String` (empty if unset/blank).
fn cvar_string(view: &mut EngineHostView, name: *const c_char) -> String {
    let p = Cvar_VariableString(view.common, name);
    if p.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

/// Parse tape text into records.
fn parse_tape(text: &str) -> Result<Vec<Rec>, String> {
    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let mut it = line.split(' ');
        let tag = it.next().unwrap_or("");
        let bad = || format!("ref tape: malformed line {}: {line:?}", lineno + 1);
        match tag {
            "H" => {
                let map = it.next().ok_or_else(bad)?.to_string();
                let fps = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let maxclients = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let seed = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                out.push(Rec::Header {
                    map,
                    fps,
                    maxclients,
                    seed,
                });
            }
            "C" => {
                let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let userinfo =
                    String::from_utf8_lossy(&hex_decode(it.next().unwrap_or(""))).into_owned();
                out.push(Rec::Connect { client, userinfo });
            }
            "X" => {
                let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let ok = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let cmd = hex_decode(it.next().unwrap_or(""));
                out.push(Rec::Command { client, ok, cmd });
            }
            "T" => {
                let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let cmd = hex_decode(it.next().unwrap_or(""));
                out.push(Rec::Think { client, cmd });
            }
            "B" => {
                let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let cmd = hex_decode(it.next().unwrap_or(""));
                out.push(Rec::Begin { client, cmd });
            }
            "D" => {
                let client = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                out.push(Rec::Drop { client });
            }
            "F" => {
                let msec = it.next().and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                out.push(Rec::Frame { msec });
            }
            "S" => {
                let total = it
                    .next()
                    .and_then(|s| u64::from_str_radix(s, 16).ok())
                    .ok_or_else(bad)?;
                let mut entities = 0u64;
                let mut players = Vec::new();
                for tok in it.by_ref() {
                    if let Some(e) = tok.strip_prefix('E') {
                        entities = u64::from_str_radix(e, 16).map_err(|_| bad())?;
                    } else if let Some((slot, h)) = tok.split_once(':') {
                        players.push((
                            slot.parse().map_err(|_| bad())?,
                            u64::from_str_radix(h, 16).map_err(|_| bad())?,
                        ));
                    } else {
                        return Err(bad());
                    }
                }
                out.push(Rec::State {
                    digest: StateDigest {
                        total,
                        entities,
                        players,
                    },
                });
            }
            _ => return Err(bad()),
        }
    }
    Ok(out)
}
