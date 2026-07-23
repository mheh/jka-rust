//! Offline `ref_snaps` wire-capture decoder — replays every captured
//! client-bound server message through the ported read side, structured
//! exactly like the retail client's parse loop (`CL_ParseServerMessage` /
//! `CL_ParseGamestate` / `CL_ParseSnapshot` / `CL_ParsePacketEntities`,
//! `oracle/codemp/client/cl_parse.cpp`), to localize wire-stream corruption
//! to a message, section, and entity.
//!
//! Diagnostic harness for the 2026-07-14 connect-drop hunt (client dropped
//! with `CL_ParsePacketEntities: end of message` / `invalid entityState
//! field count`). Skips unless `SNAPS_BIN` names a capture file:
//!
//! ```sh
//! SNAPS_BIN=/path/to/snaps.bin cargo test -p jampgame --test snapdecode -- --nocapture
//! ```
//!
//! Capture record format (`sv_referee.rs::ref_tap_client_message`):
//! `u32 len, i32 clientNum, i32 svs_time, i32 outgoingSequence, len bytes`,
//! all little-endian, one record per `SV_SendMessageToClient` call.
#![allow(non_snake_case)]

use std::env;
use std::fs;

use mp_engine_core::engine::Engine;
use mp_engine_core::host_view::engine_host_view;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::msg::{
    MSG_BeginReading, MSG_Bitstream, MSG_Init, MSG_ReadBigString, MSG_ReadBits, MSG_ReadByte,
    MSG_ReadData, MSG_ReadDeltaEntity, MSG_ReadDeltaPlayerstate, MSG_ReadLong, MSG_ReadShort,
    MSG_ReadString,
};
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::shared::limits::GENTITYNUM_BITS;
use mp_qshared::shared::{qfalse, qtrue, MAX_GENTITIES};

// Client-side constants (`oracle/codemp/client/client.h`).
const MAX_PARSE_ENTITIES: i64 = 2048;
const PACKET_BACKUP: i32 = 32;
const PACKET_MASK: i32 = PACKET_BACKUP - 1;

// `svc_ops_e` (`oracle/codemp/qcommon/qcommon.h:233-250`, !_XBOX).
const SVC_NOP: i32 = 1;
const SVC_GAMESTATE: i32 = 2;
const SVC_CONFIGSTRING: i32 = 3;
const SVC_BASELINE: i32 = 4;
const SVC_SERVERCOMMAND: i32 = 5;
const SVC_DOWNLOAD: i32 = 6;
const SVC_SNAPSHOT: i32 = 7;
const SVC_SETGAME: i32 = 8;
const SVC_MAPCHANGE: i32 = 9;
const SVC_EOF: i32 = 10;

/// One captured message.
struct Record {
    client: i32,
    time: i32,
    seq: i32,
    data: Vec<u8>,
}

/// Client-side `clSnapshot_t` mirror (only what parsing needs).
struct Snap {
    valid: bool,
    message_num: i32,
    delta_num: i32,
    ps: playerState_t,
    vps: playerState_t,
    parse_entities_num: i64,
    num_entities: i32,
}

impl Snap {
    fn zeroed() -> Snap {
        Snap {
            valid: false,
            message_num: 0,
            delta_num: 0,
            ps: unsafe { core::mem::zeroed() },
            vps: unsafe { core::mem::zeroed() },
            parse_entities_num: 0,
            num_entities: 0,
        }
    }
    fn copy_from(&mut self, o: &Snap) {
        self.valid = o.valid;
        self.message_num = o.message_num;
        self.delta_num = o.delta_num;
        unsafe {
            core::ptr::copy_nonoverlapping(&o.ps, &mut self.ps, 1);
            core::ptr::copy_nonoverlapping(&o.vps, &mut self.vps, 1);
        }
        self.parse_entities_num = o.parse_entities_num;
        self.num_entities = o.num_entities;
    }
}

/// Client-side parse state (`cl.` / `clc.` mirror).
struct Cl {
    baselines: Vec<entityState_t>,
    parse_entities: Vec<entityState_t>,
    parse_entities_num: i64,
    snapshots: Vec<Snap>,
    snap: Snap,
    have_snap: bool,
    server_command_sequence: i32,
    /// (name, offset, bits) copies of the delta tables, for novelty analysis.
    ps_fields: Vec<(String, i32, i32)>,
    ent_fields: Vec<(String, i32, i32)>,
    nov: Novelty,
}

/// Diff two structs over a (name, offset, bits) table, reporting each changed
/// field to the novelty tracker.
fn diff_fields<T>(
    nov: &mut Novelty,
    mi: usize,
    table: u8,
    tname: &str,
    fields: &[(String, i32, i32)],
    from: &T,
    to: &T,
) {
    for (fi, (name, offset, bits)) in fields.iter().enumerate() {
        let f = unsafe { *((from as *const T as *const u8).add(*offset as usize) as *const i32) };
        let t = unsafe { *((to as *const T as *const u8).add(*offset as usize) as *const i32) };
        if f != t {
            nov.check(mi, table, tname, name, fi, *bits, t);
        }
    }
}

/// First-occurrence tracker for (table, field index, encode class): the
/// wire-framing class of every changed field, so the fatal message's novel
/// encode paths stand out. Classes: 0 float-zero (ent only), 1 float-as-int,
/// 2 float-full, 3 int-zero (ent only), 4 int, 5 array-block bit.
struct Novelty {
    seen: std::collections::HashSet<(u8, u16, u8)>,
}

impl Novelty {
    fn new() -> Novelty {
        Novelty {
            seen: std::collections::HashSet::new(),
        }
    }
    fn check(
        &mut self,
        mi: usize,
        table: u8,
        tname: &str,
        fname: &str,
        fi: usize,
        bits: i32,
        v: i32,
    ) {
        let class: u8 = if bits == 0 {
            let f = f32::from_bits(v as u32);
            let trunc = f as i32;
            if f == 0.0 {
                0
            } else if trunc as f32 == f && trunc + 8192 >= 0 && trunc + 8192 < (1 << 14) {
                1
            } else {
                2
            }
        } else if v == 0 {
            3
        } else {
            4
        };
        if self.seen.insert((table, fi as u16, class)) {
            let fv = f32::from_bits(v as u32);
            println!(
                "   NOVEL msg={mi} {tname}[{fi}] {fname} bits={bits} class={class} val={v} (0x{v:08x} f={fv})"
            );
        }
    }
    fn check_array(&mut self, mi: usize, which: u8, wname: &str, idx: usize, v: i32) {
        if self.seen.insert((100 + which, idx as u16, 5)) {
            println!("   NOVEL msg={mi} array {wname}[{idx}] val={v} (0x{v:08x})");
        }
    }
}

fn read_records(path: &str) -> Vec<Record> {
    let raw = fs::read(path).expect("read SNAPS_BIN");
    let mut recs = Vec::new();
    let mut p = 0usize;
    while p + 16 <= raw.len() {
        let len = u32::from_le_bytes(raw[p..p + 4].try_into().unwrap()) as usize;
        let client = i32::from_le_bytes(raw[p + 4..p + 8].try_into().unwrap());
        let time = i32::from_le_bytes(raw[p + 8..p + 12].try_into().unwrap());
        let seq = i32::from_le_bytes(raw[p + 12..p + 16].try_into().unwrap());
        p += 16;
        if p + len > raw.len() {
            println!(
                "!! truncated final record (len={len}, remaining={})",
                raw.len() - p
            );
            break;
        }
        recs.push(Record {
            client,
            time,
            seq,
            data: raw[p..p + len].to_vec(),
        });
        p += len;
    }
    recs
}

fn main_impl() {
    let Ok(path) = env::var("SNAPS_BIN") else {
        println!("SNAPS_BIN not set — skipping snapdecode");
        return;
    };
    let recs = read_records(&path);
    println!("capture: {} messages", recs.len());
    if recs.is_empty() {
        return;
    }
    let target = recs[0].client;
    println!("decoding client {target}\n");

    let mut engine = Engine::new();
    let mut view = engine_host_view(&mut engine);
    // Both retail override files are fully commented out (verified 2026-07-14),
    // so skipping the FS-dependent override pass is behavior-identical and
    // spares the harness a filesystem boot.
    view.common.g_nOverrideChecked = true;

    // Arm the delta tables (lazily built by MSG_Init), then copy them for the
    // novelty analyzer.
    {
        let mut scratch = [0u8; 8];
        let mut m: msg_t = unsafe { core::mem::zeroed() };
        MSG_Init(
            &mut view,
            &mut m,
            scratch.as_mut_ptr(),
            scratch.len() as i32,
        );
    }
    let ps_fields: Vec<(String, i32, i32)> = view
        .common
        .player_state_fields
        .iter()
        .map(|f| (f.name.to_string(), f.offset, f.bits))
        .collect();
    let ent_fields: Vec<(String, i32, i32)> = view
        .common
        .entity_state_fields
        .iter()
        .map(|f| (f.name.to_string(), f.offset, f.bits))
        .collect();

    let mut cl = Cl {
        baselines: (0..MAX_GENTITIES)
            .map(|_| unsafe { core::mem::zeroed() })
            .collect(),
        parse_entities: (0..MAX_PARSE_ENTITIES)
            .map(|_| unsafe { core::mem::zeroed() })
            .collect(),
        parse_entities_num: 0,
        snapshots: (0..PACKET_BACKUP).map(|_| Snap::zeroed()).collect(),
        snap: Snap::zeroed(),
        have_snap: false,
        server_command_sequence: 0,
        ps_fields,
        ent_fields,
        nov: Novelty::new(),
    };

    for (mi, rec) in recs.iter().enumerate() {
        if rec.client != target {
            continue;
        }
        parse_server_message(&mut view, &mut cl, rec, mi);
    }
    println!(
        "\ndecode complete: {} messages, no framing errors",
        recs.len()
    );
}

fn parse_server_message(view: &mut EngineHostView, cl: &mut Cl, rec: &Record, mi: usize) {
    let mut buf = rec.data.clone();
    let mut m: msg_t = unsafe { core::mem::zeroed() };
    MSG_Init(view, &mut m, buf.as_mut_ptr(), buf.len() as i32);
    m.cursize = rec.data.len() as i32;
    MSG_BeginReading(&mut m);
    MSG_Bitstream(&mut m);

    let rel_ack = MSG_ReadLong(view.common, &mut m);
    println!(
        "== msg {mi}: seq={} svtime={} len={} relAck={rel_ack}",
        rec.seq, rec.time, m.cursize
    );

    loop {
        // The tap sits in SV_SendMessageToClient, BEFORE SV_Netchan_Transmit
        // appends the terminating svc_EOF (sv_net_chan.rs:138) — so a captured
        // message ends at its last real op and the boundary is the implicit
        // EOF. Garbage below the boundary is still real corruption.
        if m.readcount >= m.cursize {
            println!(
                "   implicit EOF at readcount={} / cursize={}",
                m.readcount, m.cursize
            );
            break;
        }
        let cmd = MSG_ReadByte(view.common, &mut m);
        if cmd == -1 && m.readcount >= m.cursize {
            println!("   implicit EOF (padding read) at cursize={}", m.cursize);
            break;
        }
        match cmd {
            SVC_EOF => {
                println!(
                    "   EOF at readcount={} / cursize={}",
                    m.readcount, m.cursize
                );
                break;
            }
            SVC_NOP => {}
            SVC_SERVERCOMMAND => {
                let seq = MSG_ReadLong(view.common, &mut m);
                let s = MSG_ReadString(view.common, &mut m);
                if seq > cl.server_command_sequence {
                    cl.server_command_sequence = seq;
                }
                let shown: String = s.chars().take(70).collect();
                println!("   svcmd {seq}: {shown:?}");
            }
            SVC_GAMESTATE => parse_gamestate(view, cl, &mut m, mi),
            SVC_SNAPSHOT => parse_snapshot(view, cl, &mut m, rec, mi),
            SVC_SETGAME => {
                let mut game = String::new();
                for _ in 0..64 {
                    let c = MSG_ReadByte(view.common, &mut m);
                    if c <= 0 {
                        break;
                    }
                    game.push(c as u8 as char);
                }
                println!("   setgame {game:?}");
            }
            SVC_MAPCHANGE => println!("   mapchange"),
            SVC_DOWNLOAD => panic!("msg {mi}: unexpected svc_download"),
            other => panic!(
                "msg {mi} seq={}: Illegible server message (op {other} at readcount={} cursize={})",
                rec.seq, m.readcount, m.cursize
            ),
        }
    }
    if m.readcount != m.cursize {
        println!(
            "   NOTE: {} trailing bytes after EOF (readcount={} cursize={})",
            m.cursize - m.readcount,
            m.readcount,
            m.cursize
        );
    }
}

fn parse_gamestate(view: &mut EngineHostView, cl: &mut Cl, m: &mut msg_t, mi: usize) {
    cl.server_command_sequence = MSG_ReadLong(view.common, m);
    println!("   gamestate: cmdSeq={}", cl.server_command_sequence);
    let mut n_cs = 0;
    let mut n_base = 0;
    loop {
        let cmd = MSG_ReadByte(view.common, m);
        if cmd == SVC_EOF {
            break;
        }
        match cmd {
            SVC_CONFIGSTRING => {
                let idx = MSG_ReadShort(view.common, m);
                if !(0..1700).contains(&idx) {
                    panic!("msg {mi}: configstring index {idx} out of range");
                }
                let s = MSG_ReadBigString(view.common, m);
                let len = s.len();
                n_cs += 1;
                if idx <= 1 {
                    let shown: String = s.chars().take(60).collect();
                    println!("     cs {idx} ({len} bytes): {shown}");
                }
            }
            SVC_BASELINE => {
                let newnum = MSG_ReadBits(view.common, m, GENTITYNUM_BITS);
                if !(0..MAX_GENTITIES as i32).contains(&newnum) {
                    panic!("msg {mi}: baseline number out of range: {newnum}");
                }
                let mut nullstate: entityState_t = unsafe { core::mem::zeroed() };
                MSG_ReadDeltaEntity(
                    view,
                    m,
                    &mut nullstate,
                    &mut cl.baselines[newnum as usize],
                    newnum,
                );
                n_base += 1;
            }
            other => panic!(
                "msg {mi}: CL_ParseGamestate bad command byte {other} (readcount={})",
                m.readcount
            ),
        }
    }
    let client_num = MSG_ReadLong(view.common, m);
    let checksum_feed = MSG_ReadLong(view.common, m);
    // CL_ParseRMG: zero heightmap size on non-RMG maps, then nothing follows.
    let rmg = MSG_ReadShort(view.common, m) as u16;
    println!(
        "     {n_cs} configstrings, {n_base} baselines, clientNum={client_num} feed={checksum_feed:x} rmg={rmg}"
    );
    if rmg != 0 {
        panic!("msg {mi}: unexpected RMG heightmap on this map");
    }
}

fn parse_snapshot(view: &mut EngineHostView, cl: &mut Cl, m: &mut msg_t, rec: &Record, mi: usize) {
    let server_time = MSG_ReadLong(view.common, m);
    let message_num = rec.seq;
    let delta_byte = MSG_ReadByte(view.common, m);
    let delta_num = if delta_byte == 0 {
        -1
    } else {
        message_num - delta_byte
    };
    let snap_flags = MSG_ReadByte(view.common, m);

    let mut new = Snap::zeroed();
    new.message_num = message_num;
    new.delta_num = delta_num;

    // Mirror CL_ParseSnapshot: `old` is used for parsing even when the delta
    // window checks fail; `valid` only gates whether the result is kept.
    let mut old: Option<usize> = None;
    if delta_num <= 0 {
        new.valid = true;
    } else {
        let oi = (delta_num & PACKET_MASK) as usize;
        old = Some(oi);
        let o = &cl.snapshots[oi];
        if !o.valid {
            println!("   !! delta from invalid frame");
        } else if o.message_num != delta_num {
            println!(
                "   !! delta frame too old (slot has {}, want {delta_num})",
                o.message_num
            );
        } else if cl.parse_entities_num - o.parse_entities_num > MAX_PARSE_ENTITIES - 128 {
            println!("   !! delta parseEntitiesNum too old");
        } else {
            new.valid = true;
        }
    }

    let area_len = MSG_ReadByte(view.common, m);
    if !(0..=32).contains(&area_len) {
        panic!("msg {mi} seq={message_num}: areamask len {area_len} (max 32) — misaligned stream");
    }
    let mut areamask = [0u8; 32];
    MSG_ReadData(view.common, m, areamask.as_mut_ptr() as *mut (), area_len);

    // Copy the old snap's headers/ps out to end the borrow before ring writes.
    let mut old_copy = Snap::zeroed();
    if let Some(oi) = old {
        old_copy.copy_from(&cl.snapshots[oi]);
    }
    let ps_at = m.readcount;
    if old.is_some() {
        MSG_ReadDeltaPlayerstate(view.common, m, &mut old_copy.ps, &mut new.ps, qfalse);
        if new.ps.m_iVehicleNum != 0 {
            MSG_ReadDeltaPlayerstate(view.common, m, &mut old_copy.vps, &mut new.vps, qtrue);
        }
    } else {
        MSG_ReadDeltaPlayerstate(view.common, m, core::ptr::null_mut(), &mut new.ps, qfalse);
        if new.ps.m_iVehicleNum != 0 {
            MSG_ReadDeltaPlayerstate(view.common, m, core::ptr::null_mut(), &mut new.vps, qtrue);
        }
    }
    let ents_at = m.readcount;

    // Novelty analysis: what the server encoded in this ps delta.
    {
        let fields = std::mem::take(&mut cl.ps_fields);
        diff_fields(&mut cl.nov, mi, 0, "psf", &fields, &old_copy.ps, &new.ps);
        cl.ps_fields = fields;
        for i in 0..16 {
            if new.ps.stats[i] != old_copy.ps.stats[i] {
                cl.nov.check_array(mi, 0, "stats", i, new.ps.stats[i]);
            }
            if new.ps.persistant[i] != old_copy.ps.persistant[i] {
                cl.nov
                    .check_array(mi, 1, "persistant", i, new.ps.persistant[i]);
            }
            if new.ps.ammo[i] != old_copy.ps.ammo[i] {
                cl.nov.check_array(mi, 2, "ammo", i, new.ps.ammo[i]);
            }
            if new.ps.powerups[i] != old_copy.ps.powerups[i] {
                cl.nov.check_array(mi, 3, "powerups", i, new.ps.powerups[i]);
            }
        }
    }

    // CL_ParsePacketEntities.
    new.parse_entities_num = cl.parse_entities_num;
    let mut ent_log = String::new();
    let mut oldindex: i32 = 0;
    let (mut oldnum, mut oldstate_slot): (i32, usize) = if old.is_none() {
        (99999, 0)
    } else if oldindex >= old_copy.num_entities {
        (99999, 0)
    } else {
        let slot =
            ((old_copy.parse_entities_num + oldindex as i64) & (MAX_PARSE_ENTITIES - 1)) as usize;
        (cl.parse_entities[slot].number, slot)
    };
    let advance_old = |oldindex: &mut i32,
                       oldnum: &mut i32,
                       oldstate_slot: &mut usize,
                       parse_entities: &Vec<entityState_t>| {
        *oldindex += 1;
        if *oldindex >= old_copy.num_entities {
            *oldnum = 99999;
        } else {
            *oldstate_slot = ((old_copy.parse_entities_num + *oldindex as i64)
                & (MAX_PARSE_ENTITIES - 1)) as usize;
            *oldnum = parse_entities[*oldstate_slot].number;
        }
    };

    loop {
        let newnum = MSG_ReadBits(view.common, m, GENTITYNUM_BITS);
        if newnum == MAX_GENTITIES as i32 - 1 {
            break;
        }
        if m.readcount > m.cursize {
            panic!(
                "msg {mi} seq={message_num}: CL_ParsePacketEntities: end of message \
                 (newnum={newnum} readcount={} cursize={})\n   entity log: {ent_log}",
                m.readcount, m.cursize
            );
        }
        while oldnum < newnum {
            ent_log.push_str(&format!("u{oldnum} "));
            delta_entity(
                view,
                cl,
                &mut new,
                m,
                oldnum,
                Src::Unchanged(oldstate_slot),
                mi,
            );
            advance_old(
                &mut oldindex,
                &mut oldnum,
                &mut oldstate_slot,
                &cl.parse_entities,
            );
        }
        if oldnum == newnum {
            ent_log.push_str(&format!("d{newnum}@{} ", m.readcount));
            delta_entity(view, cl, &mut new, m, newnum, Src::Old(oldstate_slot), mi);
            advance_old(
                &mut oldindex,
                &mut oldnum,
                &mut oldstate_slot,
                &cl.parse_entities,
            );
            continue;
        }
        // oldnum > newnum: delta from baseline.
        ent_log.push_str(&format!("b{newnum}@{} ", m.readcount));
        delta_entity(view, cl, &mut new, m, newnum, Src::Baseline, mi);
    }
    while oldnum != 99999 {
        ent_log.push_str(&format!("u{oldnum} "));
        delta_entity(
            view,
            cl,
            &mut new,
            m,
            oldnum,
            Src::Unchanged(oldstate_slot),
            mi,
        );
        advance_old(
            &mut oldindex,
            &mut oldnum,
            &mut oldstate_slot,
            &cl.parse_entities,
        );
    }

    println!(
        "   snap t={server_time} delta={delta_num} flags={snap_flags:x} area={area_len} \
         ps@{ps_at} veh={} ents@{ents_at}: {} ents [{}]",
        new.ps.m_iVehicleNum,
        new.num_entities,
        ent_log.trim_end()
    );

    if !new.valid {
        println!("   (snapshot not stored — invalid delta source)");
        return;
    }

    // Wrap-clear + store (CL_ParseSnapshot tail).
    let mut old_message_num = if cl.have_snap {
        cl.snap.message_num + 1
    } else {
        new.message_num
    };
    if new.message_num - old_message_num >= PACKET_BACKUP {
        old_message_num = new.message_num - (PACKET_BACKUP - 1);
    }
    while old_message_num < new.message_num {
        cl.snapshots[(old_message_num & PACKET_MASK) as usize].valid = false;
        old_message_num += 1;
    }
    cl.snap.copy_from(&new);
    cl.have_snap = true;
    cl.snapshots[(new.message_num & PACKET_MASK) as usize].copy_from(&new);
}

#[derive(Clone, Copy)]
enum Src {
    Unchanged(usize),
    Old(usize),
    Baseline,
}

/// `CL_DeltaEntity`.
fn delta_entity(
    view: &mut EngineHostView,
    cl: &mut Cl,
    frame: &mut Snap,
    m: &mut msg_t,
    newnum: i32,
    src: Src,
    mi: usize,
) {
    let slot = (cl.parse_entities_num & (MAX_PARSE_ENTITIES - 1)) as usize;
    match src {
        Src::Unchanged(old_slot) => unsafe {
            let old = &cl.parse_entities[old_slot] as *const entityState_t;
            let dst = &mut cl.parse_entities[slot] as *mut entityState_t;
            core::ptr::copy(old, dst, 1);
        },
        Src::Old(old_slot) => {
            let old = &cl.parse_entities[old_slot] as *const entityState_t as *mut entityState_t;
            let dst = &mut cl.parse_entities[slot] as *mut entityState_t;
            MSG_ReadDeltaEntity(view, m, old, dst, newnum);
        }
        Src::Baseline => {
            let old = &mut cl.baselines[newnum as usize] as *mut entityState_t;
            let dst = &mut cl.parse_entities[slot] as *mut entityState_t;
            MSG_ReadDeltaEntity(view, m, old, dst, newnum);
        }
    }
    if m.readcount > m.cursize {
        panic!(
            "msg {mi}: entity {newnum} read past end (readcount={} cursize={})",
            m.readcount, m.cursize
        );
    }
    // Novelty analysis on the changed fields (from vs decoded to).
    if let Src::Old(old_slot) = src {
        let fields = std::mem::take(&mut cl.ent_fields);
        let from = unsafe { core::ptr::read(&cl.parse_entities[old_slot]) };
        diff_fields(
            &mut cl.nov,
            mi,
            1,
            "netf",
            &fields,
            &from,
            &cl.parse_entities[slot],
        );
        cl.ent_fields = fields;
    } else if let Src::Baseline = src {
        let fields = std::mem::take(&mut cl.ent_fields);
        let from = unsafe { core::ptr::read(&cl.baselines[newnum as usize]) };
        diff_fields(
            &mut cl.nov,
            mi,
            1,
            "netf",
            &fields,
            &from,
            &cl.parse_entities[slot],
        );
        cl.ent_fields = fields;
    }
    if cl.parse_entities[slot].number == MAX_GENTITIES as i32 - 1 {
        return; // entity was delta removed
    }
    cl.parse_entities_num += 1;
    frame.num_entities += 1;
}

#[test]
fn snapdecode() {
    main_impl();
}
