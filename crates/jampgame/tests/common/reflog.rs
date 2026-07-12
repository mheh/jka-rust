//! `.reflog` — the referee input-log format, parser, serializer and the
//! deterministic scenario generator.
//!
//! A `.reflog` is a committed, human-legible text fixture: a header (map-entity
//! variant, `randomSeed` for `GAME_INIT`, connected-client count + per-client
//! userinfo, msec-per-frame, start level.time, frame count) followed by
//! per-frame per-client `usercmd_t` inputs. `serverTime` is NOT stored — the
//! driver derives it from the fixed frame stepping (`starttime + frame*msec`).
//!
//! The committed logs under `tools/referee-oracle/logs/*.reflog` ARE the
//! fixtures; [`gen_idle`]/[`gen_melee_brawl`] are the committed generators that
//! produced them. `referee.rs`'s `reflog_roundtrip` test parses each committed
//! log and re-runs its generator, asserting byte-identical output — so a change
//! to a generator that would silently drift the fixtures fails CI until the log
//! is regenerated deliberately (never silently).

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::ffi::c_int;

use mp_qshared::common::mp::qcommon::usercmd_t;

/// WP_MELEE (`bg_public.h`) — the brawl weapon: exercises pmove + attack anims
/// without saber `saberEntityNum` bookkeeping. Mirrors `mp_bg`'s `WP_MELEE = 2`.
pub const WP_MELEE: u8 = 2;

/// One client's replay input for one frame — the `usercmd_t` fields the log
/// carries (everything except the derived `serverTime`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CmdInput {
    pub angles: [i32; 3],
    pub buttons: i32,
    pub forwardmove: i8,
    pub rightmove: i8,
    pub upmove: i8,
    pub weapon: u8,
    pub forcesel: u8,
}

impl CmdInput {
    /// Materialize the wire `usercmd_t`, stamping the derived `serverTime`.
    pub fn to_usercmd(self, server_time: c_int) -> usercmd_t {
        usercmd_t {
            serverTime: server_time,
            angles: self.angles,
            buttons: self.buttons,
            weapon: self.weapon,
            forcesel: self.forcesel,
            invensel: 0,
            generic_cmd: 0,
            forwardmove: self.forwardmove,
            rightmove: self.rightmove,
            upmove: self.upmove,
        }
    }
}

/// A parsed / generated replay scenario.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub name: String,
    /// Map-entity variant name (selects the `G_GET_ENTITY_TOKEN` stream).
    pub map: String,
    /// `GAME_INIT` randomSeed.
    pub seed: i32,
    /// Connected client count.
    pub clients: i32,
    /// Milliseconds per frame (fixed level.time step).
    pub msec: i32,
    /// Initial level.time handed to `GAME_INIT` and the first frame.
    pub starttime: i32,
    /// Number of replay frames.
    pub frames: i32,
    /// Per-client userinfo strings.
    pub userinfos: BTreeMap<i32, String>,
    /// Per (frame, client) input; absent entries default to an all-zero idle cmd.
    pub cmds: BTreeMap<(i32, i32), CmdInput>,
}

impl Scenario {
    /// The input for `(frame, client)` — the idle default if unset.
    pub fn cmd(&self, frame: i32, client: i32) -> CmdInput {
        self.cmds.get(&(frame, client)).copied().unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Serialize
// ---------------------------------------------------------------------------

/// Render a scenario to the committed `.reflog` text form.
pub fn to_text(s: &Scenario) -> String {
    let mut out = String::new();
    out.push_str("# jka-rust referee input log (.reflog v1)\n");
    out.push_str("# Regenerate deliberately via tests/referee.rs; never edit by hand.\n");
    out.push_str("version 1\n");
    out.push_str(&format!("name {}\n", s.name));
    out.push_str(&format!("map {}\n", s.map));
    out.push_str(&format!("seed {}\n", s.seed));
    out.push_str(&format!("clients {}\n", s.clients));
    out.push_str(&format!("msec {}\n", s.msec));
    out.push_str(&format!("starttime {}\n", s.starttime));
    out.push_str(&format!("frames {}\n", s.frames));
    for (num, ui) in &s.userinfos {
        out.push_str(&format!("userinfo {num} {ui}\n"));
    }
    out.push_str(
        "# cmd <frame> <client> <ang0> <ang1> <ang2> <buttons> <fwd> <right> <up> <weapon> <forcesel>\n",
    );
    for (&(frame, client), c) in &s.cmds {
        out.push_str(&format!(
            "cmd {frame} {client} {} {} {} {} {} {} {} {} {}\n",
            c.angles[0],
            c.angles[1],
            c.angles[2],
            c.buttons,
            c.forwardmove,
            c.rightmove,
            c.upmove,
            c.weapon,
            c.forcesel,
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// Parse
// ---------------------------------------------------------------------------

/// Parse a `.reflog`. Panics with a precise message on malformed input (a test
/// fixture is either well-formed or a bug).
pub fn parse(text: &str) -> Scenario {
    let mut name = String::new();
    let mut map = String::new();
    let mut seed = 0i32;
    let mut clients = 0i32;
    let mut msec = 50i32;
    let mut starttime = 0i32;
    let mut frames = 0i32;
    let mut userinfos = BTreeMap::new();
    let mut cmds = BTreeMap::new();

    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.split_whitespace();
        let key = it.next().unwrap();
        let ln = lineno + 1;
        match key {
            "version" => {
                let v: i32 = it.next().unwrap().parse().unwrap();
                assert_eq!(v, 1, "unsupported reflog version at line {ln}");
            }
            "name" => name = it.next().unwrap().to_string(),
            "map" => map = it.next().unwrap().to_string(),
            "seed" => seed = it.next().unwrap().parse().unwrap(),
            "clients" => clients = it.next().unwrap().parse().unwrap(),
            "msec" => msec = it.next().unwrap().parse().unwrap(),
            "starttime" => starttime = it.next().unwrap().parse().unwrap(),
            "frames" => frames = it.next().unwrap().parse().unwrap(),
            "userinfo" => {
                let num: i32 = it.next().unwrap().parse().unwrap();
                // Userinfo is the remainder of the line verbatim (may contain
                // backslashes and be the empty string).
                let prefix = format!("userinfo {num}");
                let rest = line[prefix.len()..].trim_start().to_string();
                userinfos.insert(num, rest);
            }
            "cmd" => {
                let nums: Vec<i32> = it
                    .map(|t| {
                        t.parse()
                            .unwrap_or_else(|_| panic!("bad cmd field {t:?} at line {ln}"))
                    })
                    .collect();
                assert_eq!(nums.len(), 11, "cmd needs 11 fields at line {ln}");
                let frame = nums[0];
                let client = nums[1];
                cmds.insert(
                    (frame, client),
                    CmdInput {
                        angles: [nums[2], nums[3], nums[4]],
                        buttons: nums[5],
                        forwardmove: nums[6] as i8,
                        rightmove: nums[7] as i8,
                        upmove: nums[8] as i8,
                        weapon: nums[9] as u8,
                        forcesel: nums[10] as u8,
                    },
                );
            }
            other => panic!("unknown reflog key {other:?} at line {ln}"),
        }
    }

    Scenario {
        name,
        map,
        seed,
        clients,
        msec,
        starttime,
        frames,
        userinfos,
        cmds,
    }
}

// ---------------------------------------------------------------------------
// Deterministic generator
// ---------------------------------------------------------------------------

/// Tiny VC-libc-style LCG for scenario synthesis (NOT the game RNG — this only
/// shapes the committed input stream, deterministically from a fixed seed).
struct Lcg(u32);

impl Lcg {
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(214013).wrapping_add(2531011);
        self.0
    }
    /// Uniform integer in `[min, max]` inclusive.
    fn range(&mut self, min: i32, max: i32) -> i32 {
        let span = (max - min + 1) as u32;
        min + (((self.next_u32() >> 8) % span) as i32)
    }
    /// True with probability `pct`/100.
    fn chance(&mut self, pct: i32) -> bool {
        self.range(0, 99) < pct
    }
}

/// Canonical force-power userinfo tail — the `<rank>-<side>-<18 digits>` string
/// `WP_InitForcePowers` reads. REQUIRED: an empty `forcepowers` leaves the
/// oracle's stack array uninitialized and it segfaults (see the note in
/// `mod.rs`'s `CLIENT0_USERINFO`).
const FORCEPOWERS: &str = "7-1-032330000000001333";

fn userinfo_for(client: i32, model: &str) -> String {
    format!(
        "\\name\\Player{client}\\rate\\25000\\snaps\\20\\model\\{model}\
\\team_model\\{model}\\color1\\4\\color2\\4\\handicap\\100\\sex\\male\
\\cg_predictItems\\1\\teamtask\\0\\forcepowers\\{FORCEPOWERS}\\password\\"
    )
}

/// `idle.reflog`: 2 clients, 100 frames, NO input. The pipeline's baseline —
/// any divergence here is a frame-0/spawn-level finding.
pub fn gen_idle() -> Scenario {
    let mut userinfos = BTreeMap::new();
    userinfos.insert(0, userinfo_for(0, "kyle/default"));
    userinfos.insert(1, userinfo_for(1, "jan/default"));
    Scenario {
        name: "idle".into(),
        map: "arena3".into(),
        seed: 42,
        clients: 2,
        msec: 50,
        starttime: 600,
        frames: 100,
        userinfos,
        cmds: BTreeMap::new(),
    }
}

/// `solo.reflog`: 1 client, 120 frames of deterministic movement + jumps +
/// melee attacks. A single client never triggers `CalculateRanks`' `qsort`
/// comparator (n<2), so it exercises the full snapshot/diff pipeline on the
/// non-crashing path — the referee's "it completes" proof.
pub fn gen_solo() -> Scenario {
    let mut userinfos = BTreeMap::new();
    userinfos.insert(0, userinfo_for(0, "kyle/default"));

    const BUTTON_ATTACK: i32 = 1;
    let frames = 120;
    let mut cmds = BTreeMap::new();
    let mut rng = Lcg(0x50_10_u32);
    let mut yaw: i32 = rng.range(0, 65535);
    for frame in 0..frames {
        yaw = (yaw + rng.range(-1200, 1200)).rem_euclid(65536);
        let buttons = if rng.chance(30) { BUTTON_ATTACK } else { 0 };
        let upmove = if rng.chance(15) { 127 } else { 0 } as i8;
        cmds.insert(
            (frame, 0),
            CmdInput {
                angles: [rng.range(-3000, 3000), yaw, 0],
                buttons,
                forwardmove: rng.range(-127, 127) as i8,
                rightmove: rng.range(-127, 127) as i8,
                upmove,
                weapon: WP_MELEE,
                forcesel: 0,
            },
        );
    }

    Scenario {
        name: "solo".into(),
        map: "arena3".into(),
        seed: 7,
        clients: 1,
        msec: 50,
        starttime: 600,
        frames,
        userinfos,
        cmds,
    }
}

/// `melee-brawl.reflog`: 2 clients, 300 frames of deterministic pseudo-random
/// movement / jump / melee-attack button streams (WP_MELEE). Exercises pmove +
/// combat without saber entityNum complexity.
pub fn gen_melee_brawl() -> Scenario {
    let mut userinfos = BTreeMap::new();
    userinfos.insert(0, userinfo_for(0, "kyle/default"));
    userinfos.insert(1, userinfo_for(1, "desann/default"));

    const BUTTON_ATTACK: i32 = 1;
    const BUTTON_WALKING: i32 = 16;

    let frames = 300;
    let mut cmds = BTreeMap::new();
    // Independent input streams per client, each seeded distinctly.
    for client in 0..2i32 {
        let mut rng = Lcg(0x1337_0000u32.wrapping_add(client as u32 * 0x9E37_79B9));
        // Absolute view yaw as an ANGLE2SHORT-scale int, wandering.
        let mut yaw: i32 = rng.range(0, 65535);
        for frame in 0..frames {
            // Yaw random walk (wrap into the 16-bit short range).
            yaw = (yaw + rng.range(-1500, 1500)).rem_euclid(65536);
            let pitch = rng.range(-4000, 4000);

            let mut buttons = 0;
            if rng.chance(35) {
                buttons |= BUTTON_ATTACK;
            }
            if rng.chance(10) {
                buttons |= BUTTON_WALKING;
            }
            let forwardmove = rng.range(-127, 127) as i8;
            let rightmove = rng.range(-127, 127) as i8;
            let upmove = if rng.chance(12) { 127 } else { 0 } as i8;

            cmds.insert(
                (frame, client),
                CmdInput {
                    angles: [pitch, yaw, 0],
                    buttons,
                    forwardmove,
                    rightmove,
                    upmove,
                    weapon: WP_MELEE,
                    forcesel: 0,
                },
            );
        }
    }

    Scenario {
        name: "melee-brawl".into(),
        map: "arena3".into(),
        seed: 1337,
        clients: 2,
        msec: 50,
        starttime: 600,
        frames,
        userinfos,
        cmds,
    }
}

/// `real-duel1-idle.reflog` (referee swap, plan §3c): map `mp/duel1`, 4 clients,
/// 900 frames, NO input. The clients spawn on the map's REAL spawn points and
/// fall/settle via REAL traces against real geometry — the first real-engine
/// scenario, so any divergence is a spawn/ground-trace finding.
pub fn gen_real_duel1_idle() -> Scenario {
    let models = [
        "kyle/default",
        "jan/default",
        "luke/default",
        "desann/default",
    ];
    let mut userinfos = BTreeMap::new();
    for c in 0..4i32 {
        userinfos.insert(c, userinfo_for(c, models[c as usize]));
    }
    Scenario {
        name: "real-duel1-idle".into(),
        map: "mp/duel1".into(),
        seed: 4001,
        clients: 4,
        msec: 50,
        starttime: 600,
        frames: 900,
        userinfos,
        cmds: BTreeMap::new(),
    }
}

/// `real-duel1-walk.reflog` (referee swap, plan §3c): map `mp/duel1`, 2 clients,
/// 1500 frames, walking (forwardmove 127) with a slow deterministic yaw sweep
/// (the `gen_solo` LCG pattern) so the clients drive into real walls/geometry —
/// exercising the real `SV_Trace` collision arm at scale.
pub fn gen_real_duel1_walk() -> Scenario {
    let mut userinfos = BTreeMap::new();
    userinfos.insert(0, userinfo_for(0, "kyle/default"));
    userinfos.insert(1, userinfo_for(1, "jan/default"));

    let frames = 1500;
    let mut cmds = BTreeMap::new();
    // Independent input streams per client, each seeded distinctly.
    for client in 0..2i32 {
        let mut rng = Lcg(0xD0E1_0000u32.wrapping_add(client as u32 * 0x9E37_79B9));
        // Absolute view yaw as an ANGLE2SHORT-scale int, sweeping slowly so the
        // constant forward walk carries the client along real corridors/walls.
        let mut yaw: i32 = rng.range(0, 65535);
        for frame in 0..frames {
            yaw = (yaw + rng.range(-800, 800)).rem_euclid(65536);
            cmds.insert(
                (frame, client),
                CmdInput {
                    angles: [0, yaw, 0],
                    buttons: 0,
                    forwardmove: 127,
                    rightmove: 0,
                    upmove: 0,
                    weapon: WP_MELEE,
                    forcesel: 0,
                },
            );
        }
    }

    Scenario {
        name: "real-duel1-walk".into(),
        map: "mp/duel1".into(),
        seed: 4002,
        clients: 2,
        msec: 50,
        starttime: 600,
        frames,
        userinfos,
        cmds,
    }
}
