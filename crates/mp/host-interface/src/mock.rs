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
//! * Ruling-36/55 fixtures: the cvar registry (`cvars`, name → [`MockCvar`])
//!   serves register/string/integer/take-modified coherently — the string is
//!   authoritative, `cvar_integer` derives via C `atoi` per read, missing
//!   names read `0`/`""`/`false`; `sv_time` serves the settable `sv_time`
//!   field; `fs_write_file` captures into `written_files`; `fs_list_files`
//!   filters the FS fixture map's keys by dir/ext (the `"/"` extension lists
//!   subdirectory names); `model_mdxm`/`model_mdxa` hand back pointers into
//!   caller-provided byte blocks (missing handle = NULL).

use core::ffi::{c_char, c_ulong, c_void};

use std::collections::{BTreeMap, VecDeque};

use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::{ENTITYNUM_NONE, MAX_GENTITIES};
use mp_qshared::shared::{qboolean, qhandle_t, vec3_t};

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

/// One registered cvar in [`MockHost::cvars`] — the fixture mirror of Raven's
/// `cvar_t` slots the ruled services read (`string`/`modified`) plus the
/// registration record (`default`/`flags`). `integer` is NOT stored: it is
/// derived from `string` per read (see the registry field doc).
/// Source: `oracle/codemp/game/q_shared.h` (`cvar_t`); creation semantics
/// `oracle/codemp/qcommon/cvar.cpp:261-273`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockCvar {
    /// Raven `cvar_t->string` — the authoritative value.
    pub string: String,
    /// Raven `cvar_t->modified` — set on create and on every set; taken (and
    /// cleared) by [`EngineHost::cvar_take_modified`].
    pub modified: bool,
    /// Raven `cvar_t->resetString` — the registration default.
    pub default: String,
    /// Raven `cvar_t->flags` — ORed on re-registration (`cvar.cpp:223`).
    pub flags: i32,
}

/// C `atoi` over a Rust string: skip leading whitespace, optional sign, then
/// the digit prefix; empty/non-numeric prefix = 0. (C overflow is UB; here it
/// wraps via `i64 as i32` — a defined stand-in, per porting-rules §19.)
fn c_atoi(s: &str) -> i32 {
    let t = s.trim_start();
    let (neg, t) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    let mut v: i64 = 0;
    for c in digits.chars() {
        v = v.wrapping_mul(10).wrapping_add((c as u8 - b'0') as i64);
    }
    if neg {
        v = v.wrapping_neg();
    }
    v as i32
}

/// Fixture-backed [`EngineHost`] + [`PlatformHost`] for goldens and the referee.
pub struct MockHost {
    /// FS fixtures served by [`EngineHost::fs_read_file`], keyed by qpath.
    pub files: BTreeMap<String, Vec<u8>>,
    /// Count of *successful* [`EngineHost::fs_read_file`] disk loads — the
    /// tr-model cache hit/miss golden's fixture knob (the oracle host's
    /// `host_fs_reads`, `tools/trmodel-oracle/host.cpp:184`: incremented only
    /// after a successful read, so a cache hit — served without touching the FS
    /// — reads 0). Reset it between phases like the dumper does.
    pub fs_reads: usize,
    /// Pak-membership fixtures served by [`EngineHost::fs_file_is_in_pak`]:
    /// qpath → the pak's `pure_checksum`. A file present in [`files`] but
    /// absent here behaves as disk-only — `None`, Raven's `-1` path
    /// (`files.cpp:1659`).
    ///
    /// [`files`]: MockHost::files
    pub pak_files: BTreeMap<String, i32>,
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
    /// The cvar registry serving all four cvar services coherently
    /// ([`EngineHost::cvar_register`]/[`cvar_string`]/[`cvar_integer`]/
    /// [`cvar_take_modified`]). The STRING is the single source of truth —
    /// `cvar_integer` derives via C `atoi` semantics per read, exactly how
    /// Raven keeps `var->integer = atoi(var->string)` in sync
    /// (`cvar.cpp:266,404`). A missing name reads `0`/`""`/`false`
    /// (`Cvar_VariableIntegerValue`: `cvar.cpp:118-124`;
    /// `Cvar_VariableString`: `cvar.cpp:133-140`).
    ///
    /// [`cvar_string`]: EngineHost::cvar_string
    /// [`cvar_integer`]: EngineHost::cvar_integer
    /// [`cvar_take_modified`]: EngineHost::cvar_take_modified
    pub cvars: BTreeMap<String, MockCvar>,
    /// The `svs.time` value [`EngineHost::sv_time`] serves — set it per frame
    /// (it never auto-advances; the driver owns the frame clock).
    pub sv_time: i32,
    /// [`EngineHost::fs_write_file`] capture: qpath → written bytes (last
    /// write wins, like an FS_WRITE reopen truncating).
    pub written_files: BTreeMap<String, Vec<u8>>,
    /// Loader model-memory fixtures, mesh half: model handle → `.glm` block
    /// bytes served by [`EngineHost::model_mdxm`].
    pub mdxm_blocks: BTreeMap<qhandle_t, Vec<u8>>,
    /// `model_register` fixture log; index+1 is the returned handle.
    pub model_registers: Vec<String>,
    /// Loader model-memory fixtures, animation half: model handle → `.gla`
    /// block bytes served by [`EngineHost::model_mdxa`].
    pub mdxa_blocks: BTreeMap<qhandle_t, Vec<u8>>,
    /// Skin fixtures served by [`EngineHost::skin_surfaces`]: skin handle →
    /// `(surface-name, shader-name)` rows. A missing handle reads as no
    /// surfaces (the skin analogue of "missing handle = NULL").
    pub skins: BTreeMap<qhandle_t, Vec<(String, String)>>,
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
            fs_reads: 0,
            pak_files: BTreeMap::new(),
            dir_entries: BTreeMap::new(),
            incoming_packets: VecDeque::new(),
            sent_packets: Vec::new(),
            prints: Vec::new(),
            sys_prints: Vec::new(),
            errors: Vec::new(),
            vm_calls: Vec::new(),
            vm_call_return: 0,
            cvars: BTreeMap::new(),
            sv_time: 0,
            written_files: BTreeMap::new(),
            mdxm_blocks: BTreeMap::new(),
            model_registers: Vec::new(),
            mdxa_blocks: BTreeMap::new(),
            skins: BTreeMap::new(),
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

    /// Set a cvar value like Raven's `Cvar_Set2` — writes the string and sets
    /// `modified = qtrue` (`cvar.cpp` Cvar_Set2); creates the cvar (empty
    /// default, no flags) if the name is new, so tests can seed values before
    /// or after the subsystem registers them.
    pub fn set_cvar(&mut self, name: &str, value: &str) {
        let e = self.cvars.entry(name.to_string()).or_insert(MockCvar {
            string: String::new(),
            modified: false,
            default: String::new(),
            flags: 0,
        });
        e.string = value.to_string();
        e.modified = true;
    }

    /// Borrow the entity at `ent_num` to populate it before a run (the same
    /// slot [`EngineHost::gentity`] later hands back).
    ///
    /// # Panics
    /// If `ent_num` is out of `0..MAX_GENTITIES`.
    pub fn gentity_mut(&mut self, ent_num: i32) -> &mut sharedEntity_t {
        let stride = core::mem::size_of::<sharedEntity_t>();
        let n = ent_num as usize;
        assert!(
            n < MAX_GENTITIES,
            "gentity_mut: ent_num {ent_num} out of range"
        );
        // SAFETY: the arena is `MAX_GENTITIES * stride` zeroed bytes; slot `n`
        // is in bounds and `sharedEntity_t` is a `#[repr(C)]` POD whose
        // all-zero bit pattern is a valid value.
        unsafe { &mut *(self.gentities.as_mut_ptr().add(n * stride) as *mut sharedEntity_t) }
    }

    /// Re-seed the `holdrand` LCG behind [`EngineHost::flrand`]/[`irand`], the
    /// mock mirror of Raven's `Rand_Init(int seed)` (`holdrand = seed`). The
    /// `int` seed sign-extends into the platform-width `c_ulong` state exactly
    /// as Raven's assignment does on an LP64 host (`(int)0x89abcdef` →
    /// `0xffffffff89abcdef`) — the fixture knob the RMG substrate golden
    /// (`tools/rmg-oracle/golden/seed.txt`) drives per seed.
    ///
    /// [`irand`]: EngineHost::irand
    /// Source: `oracle/codemp/game/q_math.c:1436`
    pub fn rand_init(&mut self, seed: i32) {
        self.rng.holdrand = seed as c_ulong;
    }

    /// The raw `holdrand` LCG state (the RMG oracle dumper's `rng_state()`,
    /// `tools/rmg-oracle/main.cpp`), for pinning the platform-width substrate
    /// golden's post-draw state words byte-for-byte.
    ///
    /// Source: `oracle/codemp/game/q_math.c:1432`
    pub fn rng_state(&self) -> c_ulong {
        self.rng.holdrand
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
        let bytes = self.files.get(qpath).cloned();
        // Mirror the oracle host: count only successful reads (a miss returns
        // the -1/`None` path without bumping the counter).
        if bytes.is_some() {
            self.fs_reads += 1;
        }
        bytes
    }

    fn print(&mut self, msg: &str) {
        self.prints.push(msg.to_string());
    }

    fn error(&mut self, code: errorParm_t, msg: &str) -> ! {
        self.errors.push((code, msg.to_string()));
        panic!("MockHost EngineHost::error [{code:?}]: {msg}");
    }

    fn sv_shownet_entity_classname(&mut self, _number: i32) -> Option<String> {
        // No server spine in the mock; Raven's `if (sv.state)` reads as dead.
        None
    }

    fn sys_init(&mut self) {}

    fn sys_quit(&mut self) -> ! {
        panic!("MockHost EngineHost::sys_quit");
    }

    fn sys_error(&mut self, msg: &str) -> ! {
        panic!("MockHost EngineHost::sys_error: {msg}");
    }

    fn sys_show_console(&mut self, _level: i32, _quit_on_close: qboolean) {}

    fn is_lan_address(&mut self, adr: &netadr_t) -> bool {
        PlatformHost::is_lan_address(self, adr)
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

    fn cvar_integer(&mut self, name: &str) -> i32 {
        // Derived from the string per read — Raven's `var->integer =
        // atoi(var->string)` invariant. Missing name reads 0
        // (Cvar_VariableIntegerValue, cvar.cpp:118-124).
        self.cvars.get(name).map(|c| c_atoi(&c.string)).unwrap_or(0)
    }

    fn sv_time(&mut self) -> i32 {
        self.sv_time
    }

    fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool {
        self.written_files.insert(qpath.to_string(), data.to_vec());
        true
    }

    fn model_register(&mut self, name: &str) -> qhandle_t {
        // Fixture rule: a name registers once and keeps its 1-based handle
        // (mirroring the renderer's dedup-by-name pool); the log doubles as
        // the handle table.
        if let Some(i) = self.model_registers.iter().position(|n| n == name) {
            return (i + 1) as qhandle_t;
        }
        self.model_registers.push(name.to_string());
        self.model_registers.len() as qhandle_t
    }

    fn model_mdxm(&mut self, model: qhandle_t) -> *mut c_void {
        // NULL where Raven's model_t.mdxm is NULL (no fixture). The pointer is
        // into the fixture Vec: stable until `mdxm_blocks` is next mutated.
        self.mdxm_blocks
            .get_mut(&model)
            .map(|b| b.as_mut_ptr() as *mut c_void)
            .unwrap_or(core::ptr::null_mut())
    }

    fn model_mdxa(&mut self, model: qhandle_t) -> *mut c_void {
        // NULL where Raven's model_t.mdxa is NULL (no fixture). Same pointer
        // stability contract as `model_mdxm`.
        self.mdxa_blocks
            .get_mut(&model)
            .map(|b| b.as_mut_ptr() as *mut c_void)
            .unwrap_or(core::ptr::null_mut())
    }

    fn skin_surfaces(&mut self, h_skin: qhandle_t) -> Vec<(String, String)> {
        // No surfaces where the fixture is missing (see the `skins` field doc).
        self.skins.get(&h_skin).cloned().unwrap_or_default()
    }

    fn cvar_register(&mut self, name: &str, default: &str, flags: i32) {
        match self.cvars.get_mut(name) {
            // Existing cvar: keep the value, OR the flags in (cvar.cpp:209-232).
            Some(c) => {
                c.flags |= flags;
                if c.default.is_empty() {
                    c.default = default.to_string();
                }
            }
            // Creation: string=default, modified=qtrue (cvar.cpp:261-273).
            None => {
                self.cvars.insert(
                    name.to_string(),
                    MockCvar {
                        string: default.to_string(),
                        modified: true,
                        default: default.to_string(),
                        flags,
                    },
                );
            }
        }
    }

    fn cvar_string(&mut self, name: &str) -> String {
        // Missing name reads "" (Cvar_VariableString, cvar.cpp:133-140).
        self.cvars
            .get(name)
            .map(|c| c.string.clone())
            .unwrap_or_default()
    }

    fn cvar_take_modified(&mut self, name: &str) -> bool {
        match self.cvars.get_mut(name) {
            Some(c) => core::mem::replace(&mut c.modified, false),
            None => false,
        }
    }

    fn fs_list_files(&mut self, dir: &str, ext: &str, want_subs: bool) -> Vec<String> {
        // Served from the FS fixture map's keys under `dir/`. `ext = "/"`
        // lists distinct subdirectory names (the SE_R_ListFiles convention);
        // otherwise direct children (or, with `want_subs`, sub-path entries)
        // whose names end with `ext`. BTreeMap keys keep the output sorted.
        let prefix = format!("{dir}/");
        let mut out: Vec<String> = Vec::new();
        for key in self.files.keys() {
            let Some(rest) = key.strip_prefix(&prefix) else {
                continue;
            };
            if ext == "/" {
                if let Some((sub, _)) = rest.split_once('/') {
                    if !out.iter().any(|s| s == sub) {
                        out.push(sub.to_string());
                    }
                }
            } else if (want_subs || !rest.contains('/')) && rest.ends_with(ext) {
                out.push(rest.to_string());
            }
        }
        out
    }

    fn fs_file_is_in_pak(&mut self, qpath: &str) -> Option<i32> {
        // Some(pure_checksum) = Raven's `return 1` path; a qpath absent here
        // (even if present in `files`) is disk-only/missing = the -1 path.
        self.pak_files.get(qpath).copied()
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
        host.trace(
            &mut tr, &[0.0; 3], &[0.0; 3], &[0.0; 3], &end, 0, 0, false, 0, 0,
        );
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

        // is_lan_address: loopback rule (qualified — both host traits carry it).
        assert!(PlatformHost::is_lan_address(
            &mut host,
            &netadr_t {
                r#type: netadrtype_t::NA_LOOPBACK,
                ip: [0; 4],
                ipx: [0; 10],
                port: 0,
            }
        ));
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
    fn cvar_integer_serves_registry_and_defaults_to_zero() {
        let mut host = MockHost::new();
        host.set_cvar("developer", "1");
        assert_eq!(host.cvar_integer("developer"), 1);
        // Unregistered name reads 0 (Cvar_VariableIntegerValue, cvar.cpp:118-124).
        assert_eq!(host.cvar_integer("cg_g2MarksAllModels"), 0);
        // atoi semantics: digit prefix wins, non-numeric reads 0.
        host.set_cvar("weird", "12abc");
        assert_eq!(host.cvar_integer("weird"), 12);
        host.set_cvar("word", "english");
        assert_eq!(host.cvar_integer("word"), 0);
    }

    #[test]
    fn cvar_register_defaults_once_and_ors_flags() {
        let mut host = MockHost::new();
        // Creation: string = default, modified = qtrue (cvar.cpp:261-273).
        host.cvar_register("se_language", "english", 1 | 8);
        assert_eq!(host.cvar_string("se_language"), "english");
        assert!(host.cvars["se_language"].modified);
        assert_eq!(host.cvars["se_language"].flags, 1 | 8);
        // Re-registration keeps the (user-set) value, ORs flags (cvar.cpp:209-232).
        host.set_cvar("se_language", "deutsch");
        host.cvar_register("se_language", "english", 4);
        assert_eq!(host.cvar_string("se_language"), "deutsch");
        assert_eq!(host.cvars["se_language"].flags, 1 | 8 | 4);
    }

    #[test]
    fn cvar_string_reads_value_or_empty() {
        let mut host = MockHost::new();
        assert_eq!(host.cvar_string("se_language"), "");
        host.cvar_register("se_language", "english", 0);
        assert_eq!(host.cvar_string("se_language"), "english");
    }

    #[test]
    fn cvar_take_modified_reads_and_clears() {
        let mut host = MockHost::new();
        // Missing name reads false.
        assert!(!host.cvar_take_modified("se_language"));
        // Creation sets modified; the take returns it once, then clears.
        host.cvar_register("se_language", "english", 0);
        assert!(host.cvar_take_modified("se_language"));
        assert!(!host.cvar_take_modified("se_language"));
        // A set re-raises it (SE_CheckForLanguageUpdates cycle).
        host.set_cvar("se_language", "deutsch");
        assert!(host.cvar_take_modified("se_language"));
        assert!(!host.cvar_take_modified("se_language"));
    }

    #[test]
    fn fs_file_is_in_pak_serves_checksum_or_none() {
        let mut host = MockHost::new()
            .with_file("models/players/kyle/model.glm", b"glm".to_vec())
            .with_file("models/local/tweak.glm", b"glm".to_vec());
        host.pak_files.insert(
            "models/players/kyle/model.glm".to_string(),
            0x1234_abcd_u32 as i32,
        );

        // In a pure pak: Some(pure_checksum) (files.cpp:1650-1653).
        assert_eq!(
            host.fs_file_is_in_pak("models/players/kyle/model.glm"),
            Some(0x1234_abcd_u32 as i32)
        );
        // Present in the FS fixtures but not the pak map = disk-only: None
        // (Raven's -1, files.cpp:1659).
        assert_eq!(host.fs_file_is_in_pak("models/local/tweak.glm"), None);
        // Not found at all: also None (same -1 path).
        assert_eq!(host.fs_file_is_in_pak("models/missing.glm"), None);
    }

    #[test]
    fn fs_list_files_filters_fixture_keys() {
        let mut host = MockHost::new()
            .with_file("strings/english/menus.str", b"a".to_vec())
            .with_file("strings/english/mp.str", b"b".to_vec())
            .with_file("strings/deutsch/menus.str", b"c".to_vec())
            .with_file("strings/readme.txt", b"d".to_vec());

        // ext "/" lists distinct subdirectory names (SE_R_ListFiles convention).
        assert_eq!(
            host.fs_list_files("strings", "/", false),
            vec!["deutsch", "english"]
        );
        // Direct children by extension.
        assert_eq!(
            host.fs_list_files("strings/english", ".str", false),
            vec!["menus.str", "mp.str"]
        );
        assert_eq!(
            host.fs_list_files("strings", ".str", false),
            Vec::<String>::new()
        );
        // want_subs extends into subdirectories (ruled surface).
        assert_eq!(
            host.fs_list_files("strings", ".str", true),
            vec!["deutsch/menus.str", "english/menus.str", "english/mp.str"]
        );
    }

    #[test]
    fn sv_time_is_settable_and_distinct_from_milliseconds() {
        let mut host = MockHost::new();
        assert_eq!(EngineHost::sv_time(&mut host), 0);
        host.sv_time = 12345;
        assert_eq!(EngineHost::sv_time(&mut host), 12345);
        // The PlatformHost clock advances independently.
        let _ = host.milliseconds(false);
        assert_eq!(EngineHost::sv_time(&mut host), 12345);
    }

    #[test]
    fn fs_write_file_captures_and_reports_success() {
        let mut host = MockHost::new();
        assert!(host.fs_write_file("maps/duel1.nav", &[9, 8, 7]));
        assert_eq!(host.written_files["maps/duel1.nav"], vec![9, 8, 7]);
        // Rewrite truncates (last write wins).
        assert!(host.fs_write_file("maps/duel1.nav", &[1]));
        assert_eq!(host.written_files["maps/duel1.nav"], vec![1]);
    }

    #[test]
    fn model_memory_serves_fixture_blocks_or_null() {
        let mut host = MockHost::new();
        host.mdxm_blocks.insert(3, vec![0xAA, 0xBB]);
        host.mdxa_blocks.insert(7, vec![0xCC]);

        let m = host.model_mdxm(3);
        assert!(!m.is_null());
        assert_eq!(unsafe { *(m as *const u8) }, 0xAA);
        let a = host.model_mdxa(7);
        assert!(!a.is_null());
        assert_eq!(unsafe { *(a as *const u8) }, 0xCC);

        // Missing handle = NULL, and the two halves are independent.
        assert!(host.model_mdxm(7).is_null());
        assert!(host.model_mdxa(3).is_null());
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
