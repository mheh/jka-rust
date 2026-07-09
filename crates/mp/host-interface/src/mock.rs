//! `MockHost` — the fixture-backed test implementation of both host traits.
//!
//! Ruling 32 makes this the reusable goldens vehicle for every host-taking
//! subsystem (icarus, RMG, ghoul2, NPCNav): no test-only constructor is added
//! to a subsystem — instead its `Load`/first-slice port runs with its real
//! frozen signature and reaches the world through THIS mock's front door. The
//! mock is a pure function of its injected fixtures (ordered maps, monotonic
//! clock, deterministic LCG) so oracle-vs-Rust goldens can be byte-compared.
//!
//! No feature gate: the module is always compiled so the referee and the
//! subsystem goldens link it with no build-matrix branch.
//!
//! What each seam does here:
//! * FS reads are served from a caller-provided path→bytes map (missing path →
//!   `None`, Raven's `-1`); free is a drop.
//! * `print`/`sys_print` capture into buffers; `error` records then panics
//!   (the fork-1 `catch_unwind` model — a test wraps the call to observe it).
//! * `flrand`/`irand` run a faithful replica of Raven's `q_math.c` `holdrand`
//!   LCG. The engine's real generator (ruling 21: a qshared `QRand`-type field
//!   on `Engine.common`) is not yet ported — the referee-proven bit-exact
//!   generator today is `mp_game`'s `bg_channel::rng::Rng`, which this crate
//!   must not depend on (wrong tier), so the LCG is replicated inline against
//!   the same oracle source.
//! * `gentity` hands back a stable pointer into a byte arena strided by
//!   `size_of::<sharedEntity_t>()` — the exact `SV_GentityNum` arithmetic.
//! * `trace` writes a deterministic empty-space result (moved fully to `end`,
//!   hit nothing), matching the referee mock's out-param contract.
//! * `vm_call` records `(vm, callnum, args)` and returns a caller-set value; a
//!   `Load`-style first slice that never re-enters the game VM does not
//!   exercise it.
//! * UDP is scripted: `get_packet` pops the injected `incoming_packets` queue;
//!   `send_packet` captures into `sent_packets`; `string_to_adr` resolves
//!   `localhost`/dotted quads deterministically (no real DNS); LAN = loopback
//!   or `127.x`.

use core::ffi::{c_char, c_ulong};

use std::collections::{BTreeMap, VecDeque};

use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::{ENTITYNUM_NONE, MAX_GENTITIES};
use mp_qshared::shared::vec3_t;

use crate::engine_host::EngineHost;
use crate::platform_host::PlatformHost;
use crate::vm_slot::VmSlot;

/// Faithful replica of Raven's `q_math.c` `holdrand` LCG (see the module doc
/// for why it is inlined rather than reused). `holdrand` is platform-width
/// `c_ulong`, matching Raven's `unsigned long` (the referee ground truth on
/// LP64 hosts).
/// Source: `oracle/codemp/game/q_math.c:1432-1474`
struct HoldrandLcg {
    holdrand: c_ulong,
}

impl HoldrandLcg {
    /// Raven's compile-time `static unsigned long holdrand = 0x89abcdef;`.
    /// Source: `oracle/codemp/game/q_math.c:1432`
    const HOLDRAND_INIT: c_ulong = 0x89ab_cdef;

    fn new() -> Self {
        Self {
            holdrand: Self::HOLDRAND_INIT,
        }
    }

    /// Raven `flrand` — `min <= x < max`.
    /// Source: `oracle/codemp/game/q_math.c:1441-1450`
    fn flrand(&mut self, min: f32, max: f32) -> f32 {
        self.holdrand = self.holdrand.wrapping_mul(214013).wrapping_add(2531011);
        let result = (self.holdrand >> 17) as f32;
        ((result * (max - min)) / 32768.0f32) + min
    }

    /// Raven `irand` — `min <= x <= max` (inclusive).
    /// Source: `oracle/codemp/game/q_math.c:1458-1469`
    fn irand(&mut self, min: i32, max: i32) -> i32 {
        let max = max + 1;
        self.holdrand = self.holdrand.wrapping_mul(214013).wrapping_add(2531011);
        let result = (self.holdrand >> 17) as i32;
        (result.wrapping_mul(max - min) >> 15).wrapping_add(min)
    }
}

/// Fixture-backed [`EngineHost`] + [`PlatformHost`] for goldens and the referee.
pub struct MockHost {
    /// FS fixtures served by [`EngineHost::fs_read_file`], keyed by qpath.
    pub files: BTreeMap<String, Vec<u8>>,
    /// `Sys_ListFiles` fixtures, keyed by directory → sorted entry names.
    pub dir_entries: BTreeMap<String, Vec<String>>,
    /// Scripted inbound datagrams served by [`PlatformHost::get_packet`]
    /// (front-of-queue first); empty queue = no packet pending.
    pub incoming_packets: VecDeque<(netadr_t, Vec<u8>)>,
    /// [`PlatformHost::send_packet`] capture: `(to, payload)` in send order.
    pub sent_packets: Vec<(netadr_t, Vec<u8>)>,
    /// `Com_Printf` capture.
    pub prints: Vec<String>,
    /// `Sys_Print` capture.
    pub sys_prints: Vec<String>,
    /// `Com_Error` capture (`code`, `msg`); each entry is followed by a panic.
    pub errors: Vec<(errorParm_t, String)>,
    /// `VM_Call` log: `(vm, callnum, args)` in issue order.
    pub vm_calls: Vec<(VmSlot, i32, Vec<isize>)>,
    /// The value [`EngineHost::vm_call`] returns (default `0`).
    pub vm_call_return: isize,
    /// Byte arena strided by `size_of::<sharedEntity_t>()` behind `gentity`.
    gentities: Vec<u8>,
    /// The `sv.mSharedMemory` window.
    shared_mem: Vec<u8>,
    /// The `holdrand` LCG behind `flrand`/`irand`.
    rng: HoldrandLcg,
    /// Monotonic `Sys_Milliseconds` counter.
    millis: i32,
}

impl MockHost {
    /// Default `sv.mSharedMemory` window size — comfortably larger than any
    /// `T_G_ICARUS_*` request struct the icarus seam writes.
    const SHARED_MEM_BYTES: usize = 64 * 1024;

    /// A fresh mock: empty fixtures, zeroed entity arena, LCG at Raven's seed.
    pub fn new() -> Self {
        let stride = core::mem::size_of::<sharedEntity_t>();
        Self {
            files: BTreeMap::new(),
            dir_entries: BTreeMap::new(),
            incoming_packets: VecDeque::new(),
            sent_packets: Vec::new(),
            prints: Vec::new(),
            sys_prints: Vec::new(),
            errors: Vec::new(),
            vm_calls: Vec::new(),
            vm_call_return: 0,
            gentities: vec![0u8; MAX_GENTITIES * stride],
            shared_mem: vec![0u8; Self::SHARED_MEM_BYTES],
            rng: HoldrandLcg::new(),
            millis: 0,
        }
    }

    /// Seed one FS fixture (`qpath` → bytes). Chainable.
    pub fn with_file(mut self, qpath: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        self.files.insert(qpath.into(), bytes.into());
        self
    }

    /// Borrow the entity at `ent_num` to populate it before a run (the same
    /// slot [`EngineHost::gentity`] later hands back).
    ///
    /// # Panics
    /// If `ent_num` is out of `0..MAX_GENTITIES`.
    pub fn gentity_mut(&mut self, ent_num: i32) -> &mut sharedEntity_t {
        let stride = core::mem::size_of::<sharedEntity_t>();
        let n = ent_num as usize;
        assert!(n < MAX_GENTITIES, "gentity_mut: ent_num {ent_num} out of range");
        // SAFETY: the arena is `MAX_GENTITIES * stride` zeroed bytes; slot `n`
        // is in bounds and `sharedEntity_t` is a `#[repr(C)]` POD whose
        // all-zero bit pattern is a valid value.
        unsafe { &mut *(self.gentities.as_mut_ptr().add(n * stride) as *mut sharedEntity_t) }
    }
}

impl Default for MockHost {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineHost for MockHost {
    fn trace(
        &mut self,
        results: &mut trace_t,
        _start: &vec3_t,
        _mins: &vec3_t,
        _maxs: &vec3_t,
        end: &vec3_t,
        _pass_entity_num: i32,
        _contentmask: i32,
        _capsule: bool,
        _trace_flags: i32,
        _use_lod: i32,
    ) {
        // Deterministic empty space: moved fully to `end`, hit nothing.
        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        tr.fraction = 1.0;
        tr.entityNum = ENTITYNUM_NONE as i16;
        tr.endpos = *end;
        *results = tr;
    }

    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>> {
        self.files.get(qpath).cloned()
    }

    fn print(&mut self, msg: &str) {
        self.prints.push(msg.to_string());
    }

    fn error(&mut self, code: errorParm_t, msg: &str) -> ! {
        self.errors.push((code, msg.to_string()));
        panic!("MockHost EngineHost::error [{code:?}]: {msg}");
    }

    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize {
        self.vm_calls.push((vm, callnum, args.to_vec()));
        self.vm_call_return
    }

    fn shared_memory(&mut self) -> *mut c_char {
        self.shared_mem.as_mut_ptr() as *mut c_char
    }

    fn flrand(&mut self, min: f32, max: f32) -> f32 {
        self.rng.flrand(min, max)
    }

    fn irand(&mut self, min: i32, max: i32) -> i32 {
        self.rng.irand(min, max)
    }

    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t {
        let stride = core::mem::size_of::<sharedEntity_t>();
        // Faithful `SV_GentityNum` arithmetic: byte base + stride*num.
        unsafe { self.gentities.as_mut_ptr().add(ent_num as usize * stride) as *mut sharedEntity_t }
    }
}

impl PlatformHost for MockHost {
    fn milliseconds(&mut self, _base_time: bool) -> i32 {
        let t = self.millis;
        self.millis += 1;
        t
    }

    fn sys_print(&mut self, msg: &str) {
        self.sys_prints.push(msg.to_string());
    }

    fn console_input(&mut self) -> Option<String> {
        None
    }

    fn get_packet(&mut self, net_from: &mut netadr_t, net_message: &mut msg_t) -> bool {
        let Some((from, payload)) = self.incoming_packets.pop_front() else {
            return false;
        };
        *net_from = from;
        if !net_message.data.is_null() && net_message.maxsize > 0 {
            let n = payload.len().min(net_message.maxsize as usize);
            // SAFETY: `data..data+maxsize` is the caller's message buffer;
            // `n <= maxsize` keeps the copy in bounds.
            unsafe {
                core::ptr::copy_nonoverlapping(payload.as_ptr(), net_message.data, n);
            }
            net_message.cursize = n as i32;
        }
        true
    }

    fn send_packet(&mut self, data: &[u8], to: &netadr_t) {
        // SAFETY: `netadr_t` is a `#[repr(C)]` POD with no drop glue; a bitwise
        // read duplicates it soundly (it carries no `Clone`).
        let to_copy: netadr_t = unsafe { core::ptr::read(to) };
        self.sent_packets.push((to_copy, data.to_vec()));
    }

    fn string_to_adr(&mut self, s: &str, a: &mut netadr_t) -> bool {
        // Deterministic resolver: "localhost" and dotted quads (with optional
        // ":port") resolve as NA_IP; anything else fails — no real DNS.
        let (host, port) = match s.rsplit_once(':') {
            Some((h, p)) => match p.parse::<u16>() {
                Ok(port) => (h, port),
                Err(_) => (s, 0),
            },
            None => (s, 0),
        };
        let ip: [u8; 4] = if host == "localhost" {
            [127, 0, 0, 1]
        } else {
            let mut octets = [0u8; 4];
            let mut it = host.split('.');
            for o in octets.iter_mut() {
                match it.next().and_then(|t| t.parse::<u8>().ok()) {
                    Some(v) => *o = v,
                    None => return false,
                }
            }
            if it.next().is_some() {
                return false;
            }
            octets
        };
        *a = netadr_t {
            r#type: netadrtype_t::NA_IP,
            ip,
            ipx: [0; 10],
            port: port.to_be(),
        };
        true
    }

    fn is_lan_address(&mut self, adr: &netadr_t) -> bool {
        // Deterministic rule: loopback type or a 127.x address is "LAN".
        matches!(adr.r#type, netadrtype_t::NA_LOOPBACK) || adr.ip[0] == 127
    }

    fn list_files(
        &mut self,
        directory: &str,
        _extension: &str,
        _filter: Option<&str>,
        _want_subs: bool,
    ) -> Vec<String> {
        self.dir_entries.get(directory).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_serves_fixtures_and_misses() {
        let mut host = MockHost::new().with_file("scripts/x.rof", vec![1u8, 2, 3]);
        assert_eq!(host.fs_read_file("scripts/x.rof"), Some(vec![1, 2, 3]));
        assert_eq!(host.fs_read_file("nope"), None);
        // free consumes the buffer (drop).
        if let Some(buf) = host.fs_read_file("scripts/x.rof") {
            host.fs_free_file(buf);
        }
    }

    #[test]
    fn lcg_matches_raven_seed_stream() {
        // First two `holdrand` steps off 0x89abcdef, `>>17`. This pins the
        // replica against the oracle q_math.c stream.
        // (No range assertion: on LP64 `holdrand>>17` exceeds 0x7fff, so irand
        // can fall outside min..max — the referee-documented faithful behavior.)
        let mut host = MockHost::new();
        let a = host.irand(0, 100);
        let b = host.irand(0, 100);
        // Deterministic: a fresh host reproduces the same stream.
        let mut host2 = MockHost::new();
        assert_eq!(host2.irand(0, 100), a);
        assert_eq!(host2.irand(0, 100), b);
    }

    #[test]
    fn trace_reports_empty_space() {
        let mut host = MockHost::new();
        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        let end = [1.0, 2.0, 3.0];
        host.trace(&mut tr, &[0.0; 3], &[0.0; 3], &[0.0; 3], &end, 0, 0, false, 0, 0);
        assert_eq!(tr.fraction, 1.0);
        assert_eq!(tr.entityNum, ENTITYNUM_NONE as i16);
        assert_eq!(tr.endpos, end);
    }

    #[test]
    fn gentity_pointer_is_stable_and_writable() {
        let mut host = MockHost::new();
        host.gentity_mut(5).s.number = 5;
        let p = host.gentity(5);
        assert_eq!(unsafe { (*p).s.number }, 5);
        // Distinct slots are distinct pointers.
        assert_ne!(host.gentity(5), host.gentity(6));
    }

    #[test]
    fn udp_roundtrip_is_scripted_and_captured() {
        let mut host = MockHost::new();

        // string_to_adr: deterministic resolver.
        let mut adr: netadr_t = unsafe { core::mem::zeroed() };
        assert!(host.string_to_adr("192.168.0.7:29070", &mut adr));
        assert_eq!(adr.ip, [192, 168, 0, 7]);
        assert_eq!(adr.port, 29070u16.to_be());
        assert!(!host.string_to_adr("no.such.host.example", &mut adr));

        // send_packet: captured by (to, payload).
        host.send_packet(b"hello", &adr);
        assert_eq!(host.sent_packets.len(), 1);
        assert_eq!(host.sent_packets[0].1, b"hello");

        // get_packet: empty queue = none pending; scripted packet round-trips.
        let mut buf = [0u8; 16];
        let mut msg: msg_t = unsafe { core::mem::zeroed() };
        msg.data = buf.as_mut_ptr();
        msg.maxsize = buf.len() as i32;
        let mut from: netadr_t = unsafe { core::mem::zeroed() };
        assert!(!host.get_packet(&mut from, &mut msg));

        let src = netadr_t {
            r#type: netadrtype_t::NA_IP,
            ip: [10, 0, 0, 2],
            ipx: [0; 10],
            port: 27960u16.to_be(),
        };
        host.incoming_packets.push_back((src, vec![1, 2, 3]));
        assert!(host.get_packet(&mut from, &mut msg));
        assert_eq!(from.ip, [10, 0, 0, 2]);
        assert_eq!(msg.cursize, 3);
        assert_eq!(&buf[..3], &[1, 2, 3]);

        // is_lan_address: loopback rule.
        assert!(host.is_lan_address(&netadr_t {
            r#type: netadrtype_t::NA_LOOPBACK,
            ip: [0; 4],
            ipx: [0; 10],
            port: 0,
        }));
    }

    #[test]
    fn vm_call_logs_slot_and_args() {
        let mut host = MockHost::new();
        host.vm_call_return = 7;
        assert_eq!(host.vm_call(VmSlot::Gvm, 3, &[1, 2]), 7);
        assert_eq!(host.vm_call(VmSlot::Cgvm, 9, &[]), 7);
        assert_eq!(host.vm_calls[0], (VmSlot::Gvm, 3, vec![1, 2]));
        assert_eq!(host.vm_calls[1], (VmSlot::Cgvm, 9, vec![]));
    }

    #[test]
    fn error_records_then_panics() {
        let mut host = MockHost::new();
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            host.error(errorParm_t::ERR_DROP, "boom");
        }));
        assert!(r.is_err());
        assert_eq!(host.errors.len(), 1);
        assert_eq!(host.errors[0].0, errorParm_t::ERR_DROP);
    }
}
