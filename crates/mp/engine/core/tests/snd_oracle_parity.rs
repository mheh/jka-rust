//! The snd-oracle mixer gate (ticket gh#24, DEC-57.2 and DEC-62).
//!
//! # What this drives
//! `tools/snd-oracle` compiles the unmodified Raven sound TUs, runs a scripted
//! command list over committed PCM fixtures, and dumps the `dma_t` ring. This
//! rig reads the same scripts, drives the Rust `S_*` port through the same
//! scripted clock and DMA cursor, and compares two things per scenario:
//!
//! 1. the text dump, line for line, against `golden/<name>.txt`, and
//! 2. the ring bytes, byte for byte, against `golden/<name>.bin`.
//!
//! The dump format is defined by `tools/snd-oracle/main.cpp`, and the writer
//! below mirrors it exactly.
//!
//! # Assets
//! The rig writes a temporary game tree from the committed generated fixtures
//! (`tools/snd-oracle/fixtures/sound/*.wav`) plus an empty `mpdefault.cfg`, so it
//! needs no retail content and no C++ toolchain. It runs in CI.
//!
//! # The device
//! DEC-57.1 puts the read cursor on `SoundSystem` and leaves the device end to
//! write it. The `advance` command writes it here, exactly the way the harness's
//! `SNDDMA_GetDMAPos` reports the cursor the script set.

#![allow(non_snake_case)]

use std::fs;
use std::path::{Path, PathBuf};

use mp_engine_client::client_host::snd_from_view;
use mp_engine_client::snd_ambient::{
    AS_AddPrecacheEntry, AS_GetBModelSound, AS_ParseSets, S_AddLocalSet, S_UpdateAmbientSet,
};
use mp_engine_client::snd_dma::{
    sfxHandle_t, S_AddAmbientLoopingSound, S_AddLoopingSound, S_BeginRegistration,
    S_ClearLoopingSounds, S_DisableSounds, S_GetSampleLengthInMilliSeconds, S_Init, S_MuteSound,
    S_RawSamples, S_RegisterSound, S_Respatialize, S_RestartMusic, S_Shutdown, S_StartAmbientSound,
    S_StartBackgroundTrack, S_StartLocalLoopingSound, S_StartLocalSound, S_StartSound,
    S_StopAllSounds, S_StopBackgroundTrack, S_StopLoopingSound, S_StopSounds, S_Update,
    S_UpdateEntityPosition,
};
use mp_engine_client::snd_music::{
    Music_AllowedToTransition, Music_DynamicDataAvailable, Music_GetFileNameForState,
    Music_GetLevelSetName, Music_GetRandomEntryTime, Music_StateCanBeInterrupted,
};
use mp_engine_client::snd::music_state_e::MusicState_e;
use mp_engine_client::snd::sfx_sample_data::SfxSampleData;
use mp_engine_client::snd::sound_system::{SoundSystem, MAX_CHANNELS};
use mp_engine_client::{Client, SoundSystem as ClientSoundSystem};
use mp_engine_core::{com_init, engine_host_view, install_engine_hooks, Engine};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::ComError;
use mp_engine_qcommon::cvar_fns::{Cvar_FindVar, Cvar_Set, Cvar_Set2};
use mp_engine_qcommon::files::files_consts::BASEGAME;
use mp_engine_qcommon::files_common::{FS_Shutdown, FS_Startup};
use mp_qshared::shared::{qfalse, vec3_t};
use native_math::rng::HoldrandLcg;

/// The scenarios the harness ships. DEC-62.7 adds `lipsync` with gh#24 and the
/// `music`, `musicdata`, and `ambient` trio with gh#25.
/// Source: `tools/snd-oracle/scenarios/`
const SCENARIOS: [&str; 14] = [
    "ambient",
    "badfiles",
    "basic",
    "channels",
    "khz11",
    "khz44",
    "lipsync",
    "loops",
    "music",
    "musicdata",
    "rawstream",
    "resample",
    "ringwrap",
    "spatialize",
];

/// The ring dump splits the buffer into 4 KB blocks.
/// Source: `tools/snd-oracle/main.cpp` `SND_ORACLE_RING_BLOCK`
const RING_BLOCK: usize = 4096;

/// Harness slot indices, 0 to 15, that `register` fills with a handle.
/// Source: `tools/snd-oracle/main.cpp` `SND_ORACLE_MAX_SLOTS`
const MAX_SLOTS: usize = 16;

/// Entity slots the lip-sync dump covers.
/// Source: `tools/snd-oracle/main.cpp` `SND_ORACLE_LIPSYNC_ENTS`
const LIPSYNC_ENTS: usize = 8;

fn repo_tool_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/snd-oracle")
}

/// FNV-1a, 32 bit. The harness uses the same digest.
/// Source: `tools/snd-oracle/main.cpp` `snd_oracle_fnv1a`
fn fnv1a(data: &[u8]) -> u32 {
    let mut hash: u32 = 2166136261;
    for &b in data {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

// ===========================================================================
// The scripted driver
// ===========================================================================

/// One scenario run: the engine island, the scripted clock, and the dump.
struct Rig {
    engine: Box<Engine>,
    /// The device has consumed this many stereo frames since `S_Init`.
    /// Source: `tools/snd-oracle/main.cpp:35`
    framesConsumed: i64,
    clock_ms: u32,
    dma_pos: i32,
    slots: [sfxHandle_t; MAX_SLOTS],
    /// The console counters at the start of the run, so the dump reports the
    /// sound system's own prints and not the boot's.
    printBase: u64,
    dprintBase: u64,
    out: String,
    /// The last ring the script dumped, which becomes the `.bin` comparison.
    ringSnapshot: Vec<u8>,
}

impl Rig {
    fn new(home: &Path, assets: &Path) -> Rig {
        let mut engine: Box<Engine> = Engine::new();
        install_engine_hooks(&mut engine);
        // `+echo` is a startup command, so the boot never queues Raven's
        // `cinematic openinglogos.roq` default action.
        let cmdline = format!(
            "+set fs_basepath {} +set fs_homepath {} +set dedicated 0 +echo snd-oracle",
            assets.display(),
            home.display()
        );
        com_init(&mut engine, &cmdline);

        // The temp tree has no `productid.txt`, so `FS_SetRestrictions` put the
        // file system on the `demo` directory and set `fs_restrict`, which
        // refuses every loose `.wav`. Put it back on `base` with add-ons
        // allowed. The oracle harness reads the same fixtures straight off disk.
        {
            let mut view = engine_host_view(&mut engine);
            Cvar_Set(&mut view, "fs_restrict", "0");
            FS_Shutdown(view.common, qfalse);
            FS_Startup(&mut view, BASEGAME);
        }

        // `Com_Init` reseeds `holdrand` from the wall clock. The ambient system
        // draws every subwave and every gap through `Q_irand`, and the oracle
        // harness has no `Com_Init`, so its generator sits at the compile-time
        // seed. Put ours back there. Harness policy, the scripted-clock posture.
        engine.common.qrand.Rand_Init(HoldrandLcg::INIT as i32);
        engine.common.qrand.srand(0);

        engine.cl = Some(Client::default());
        engine.snd = Some(ClientSoundSystem::default());

        let printBase = engine.common.com_printCount;
        let dprintBase = engine.common.com_dprintCount;

        Rig {
            engine,
            framesConsumed: 0,
            clock_ms: 0,
            dma_pos: 0,
            slots: [0; MAX_SLOTS],
            printBase,
            dprintBase,
            out: String::new(),
            ringSnapshot: Vec::new(),
        }
    }

    fn with<R>(&mut self, body: impl FnOnce(&mut EngineHostView, &mut SoundSystem) -> R) -> R {
        let mut view = engine_host_view(&mut self.engine);
        // SAFETY: the slot came from the live `Engine.snd`, and no other cast of
        // the slot is live here.
        let snd = unsafe { snd_from_view(&mut view) };
        body(&mut view, snd)
    }

    /// Seed a cvar before `S_Init`, the way a config file would.
    ///
    /// The harness's flat cvar table holds no reset string, so the seed clears
    /// ours too. `Cvar_Get` then adopts the Raven default as the reset string
    /// without its duplicate-initial-value warning, and the dprint counts match.
    /// Source: `tools/snd-oracle/host.cpp:103-118`
    fn seed_cvar(&mut self, name: &str, value: &str) {
        let mut view = engine_host_view(&mut self.engine);
        Cvar_Set2(&mut view, name, Some(value), true);
        let h = Cvar_FindVar(view.common, name).expect("seeded cvar");
        view.common.cvar_mut(h).resetString = String::new();
    }

    /// The device consumed `frames` stereo frames.
    /// Source: `tools/snd-oracle/main.cpp:197-202`
    fn advance(&mut self, frames: i32) {
        let channels = self.with(|_, snd| snd.dma.channels);
        let speed = self.with(|_, snd| snd.dma.speed);
        self.framesConsumed += i64::from(frames);
        self.dma_pos += frames * channels;
        self.clock_ms = ((self.framesConsumed * 1000) / i64::from(speed)) as u32;
        let pos = self.dma_pos;
        self.with(|_, snd| snd.dma_pos = pos);
    }

    /// Source: `tools/snd-oracle/main.cpp:52-77`
    fn dump_state(&mut self, tag: &str) {
        let prints = self.engine.common.com_printCount - self.printBase;
        let dprints = self.engine.common.com_dprintCount - self.dprintBase;
        let (clock, dmapos, frames) = (self.clock_ms, self.dma_pos, self.framesConsumed);

        let mut out = String::new();
        out.push_str(&format!("STATE {tag}\n"));
        out.push_str(&format!("  clock {clock} dmapos {dmapos} frames {frames}\n"));

        let snd = self.engine.snd.as_ref().expect("sound system seated");
        out.push_str(&format!(
            "  started {} muted {} soundtime {} paintedtime {} rawend {}\n",
            snd.s_soundStarted,
            i32::from(snd.s_soundMuted),
            snd.s_soundtime,
            snd.s_paintedtime,
            snd.s_rawend
        ));
        out.push_str(&format!(
            "  dma channels {} samples {} samplebits {} speed {} chunk {}\n",
            snd.dma.channels,
            snd.dma.samples,
            snd.dma.samplebits,
            snd.dma.speed,
            snd.dma.submission_chunk
        ));
        out.push_str(&format!(
            "  listener {} org {}\n",
            snd.listener_number,
            fmt_vec(snd.listener_origin)
        ));
        for a in 0..3 {
            out.push_str(&format!("  axis{a} {}\n", fmt_vec(snd.listener_axis[a])));
        }
        out.push_str(&format!(
            "  loops {} prints {prints} dprints {dprints}\n",
            snd.numLoopSounds
        ));

        for i in 0..MAX_CHANNELS {
            let ch = &snd.s_channels[i];
            let Some(sfx) = ch.thesfx else {
                continue;
            };
            out.push_str(&format!(
                "  ch {i} ent {} chan {} lv {} rv {} mv {} start {} fixed {} loop {} org {} sfx {}\n",
                ch.entnum,
                ch.entchannel,
                ch.leftvol,
                ch.rightvol,
                ch.master_vol,
                ch.startSample,
                i32::from(ch.fixed_origin),
                i32::from(ch.loopSound),
                fmt_vec(ch.origin),
                snd.s_knownSfx[sfx].sSoundName
            ));
        }

        self.out.push_str(&out);
    }

    /// Source: `tools/snd-oracle/main.cpp:79-92`
    fn dump_sfx(&mut self, tag: &str) {
        let snd = self.engine.snd.as_ref().expect("sound system seated");
        let mut out = format!("SFX {tag} count {}\n", snd.s_knownSfx.len());
        for (i, sfx) in snd.s_knownSfx.iter().enumerate() {
            // The harness digests `iSoundLengthInSamples * 2` bytes off
            // `pSoundData`, which is the PCM block for a `ct_16` sound and the
            // raw MP3 image for a `ct_MP3` one.
            let digest = match sfx.pSoundData.as_ref() {
                Some(data) if sfx.iSoundLengthInSamples > 0 => {
                    let want = (sfx.iSoundLengthInSamples as usize) * 2;
                    match data {
                        SfxSampleData::Pcm(pcm) => {
                            let bytes: Vec<u8> = pcm[..(want / 2).min(pcm.len())]
                                .iter()
                                .flat_map(|s| s.to_le_bytes())
                                .collect();
                            fnv1a(&bytes)
                        }
                        SfxSampleData::Mp3(raw) => fnv1a(&raw[..want.min(raw.len())]),
                    }
                }
                _ => 0,
            };
            out.push_str(&format!(
                "  sfx {i} name {} samples {} volrange {:.6} method {} default {} inmem {} data {digest:08x}\n",
                sfx.sSoundName,
                sfx.iSoundLengthInSamples,
                f64::from(sfx.fVolRange),
                sfx.eSoundCompressionMethod as i32,
                i32::from(sfx.bDefaultSound),
                i32::from(sfx.bInMemory),
            ));
        }
        self.out.push_str(&out);
    }

    /// Source: `tools/snd-oracle/main.cpp` `snd_oracle_dump_music`
    fn dump_music(&mut self, tag: &str) {
        let snd = self.engine.snd.as_ref().expect("sound system seated");
        let mut out = format!("MUSIC {tag}\n");
        out.push_str(&format!(
            "  dynamic {} actual {} request {} loop {}\n",
            i32::from(snd.bMusic_IsDynamic),
            snd.eMusic_StateActual as i32,
            snd.eMusic_StateRequest as i32,
            snd.sMusic_BackgroundLoop
        ));
        out.push_str(&format!("  set {}\n", snd.sInfoOnly_CurrentDynamicMusicSet));
        for track in 0..MusicState_e::eBGRNDTRACK_NUMBEROF as usize {
            let info = &snd.tMusic_Info[track];
            out.push_str(&format!(
                "  track {track} mp3 {} file {} active {} exists {} xfade {} samples {} rate {} chans {} width {}\n",
                i32::from(info.bIsMP3),
                info.s_backgroundFile,
                i32::from(info.bActive),
                i32::from(info.bExists),
                info.iXFadeVolume,
                info.s_backgroundSamples,
                info.s_backgroundInfo.rate,
                info.s_backgroundInfo.channels,
                info.s_backgroundInfo.width
            ));
        }
        self.out.push_str(&out);
    }

    /// Source: `tools/snd-oracle/main.cpp` `snd_oracle_dump_lipsync`
    fn dump_lipsync(&mut self, tag: &str) {
        let snd = self.engine.snd.as_ref().expect("sound system seated");
        let mut out = format!("LIPSYNC {tag}\n");
        for i in 0..LIPSYNC_ENTS {
            out.push_str(&format!(
                "  ent {i} vol {} back {}\n",
                snd.s_entityWavVol[i], snd.s_entityWavVol_back[i]
            ));
        }
        self.out.push_str(&out);
    }

    /// Source: `tools/snd-oracle/main.cpp:104-140`
    fn dump_ring(&mut self, tag: &str) {
        let snd = self.engine.snd.as_ref().expect("sound system seated");
        let bytes = (snd.dma.samples * (snd.dma.samplebits / 8)) as usize;
        if snd.dma.buffer.is_empty() || bytes == 0 || bytes > 0x10000 {
            self.out.push_str(&format!("RING {tag} bytes 0 whole 00000000\n"));
            return;
        }
        let ring = snd.dma.buffer[..bytes].to_vec();
        self.ringSnapshot = ring.clone();

        let mut out = format!("RING {tag} bytes {bytes} whole {:08x}\n", fnv1a(&ring));

        for (block, chunk) in ring.chunks(RING_BLOCK).enumerate() {
            let mut lo = 0i32;
            let mut hi = 0i32;
            let mut nonzero = 0usize;
            for pair in chunk.chunks_exact(2) {
                let s = i32::from(i16::from_le_bytes([pair[0], pair[1]]));
                if s < lo {
                    lo = s;
                }
                if s > hi {
                    hi = s;
                }
                if s != 0 {
                    nonzero += 1;
                }
            }
            out.push_str(&format!(
                "  blk {block} crc {:08x} min {lo} max {hi} nonzero {nonzero}\n",
                fnv1a(chunk)
            ));
        }

        self.out.push_str(&out);
    }

    /// Keep the last ring copy the way `main.cpp` does before `S_Shutdown` frees it.
    /// Source: `tools/snd-oracle/main.cpp:370-378`
    fn snapshot_ring(&mut self) {
        let snd = self.engine.snd.as_ref().expect("sound system seated");
        let bytes = (snd.dma.samples * (snd.dma.samplebits / 8)) as usize;
        if !snd.dma.buffer.is_empty() && bytes > 0 && bytes <= 0x10000 {
            self.ringSnapshot = snd.dma.buffer[..bytes].to_vec();
        }
    }
}

/// The `MusicState_e` a scripted state number names, the way the harness casts
/// its integer argument.
fn music_state(value: i32) -> MusicState_e {
    match value {
        0 => MusicState_e::eBGRNDTRACK_EXPLORE,
        1 => MusicState_e::eBGRNDTRACK_ACTION,
        2 => MusicState_e::eBGRNDTRACK_BOSS,
        3 => MusicState_e::eBGRNDTRACK_DEATH,
        4 => MusicState_e::eBGRNDTRACK_ACTIONTRANS0,
        5 => MusicState_e::eBGRNDTRACK_ACTIONTRANS1,
        6 => MusicState_e::eBGRNDTRACK_ACTIONTRANS2,
        7 => MusicState_e::eBGRNDTRACK_ACTIONTRANS3,
        8 => MusicState_e::eBGRNDTRACK_EXPLORETRANS0,
        9 => MusicState_e::eBGRNDTRACK_EXPLORETRANS1,
        10 => MusicState_e::eBGRNDTRACK_EXPLORETRANS2,
        11 => MusicState_e::eBGRNDTRACK_EXPLORETRANS3,
        12 => MusicState_e::eBGRNDTRACK_NONDYNAMIC,
        13 => MusicState_e::eBGRNDTRACK_SILENCE,
        _ => MusicState_e::eBGRNDTRACK_FADE,
    }
}

/// `%.6f %.6f %.6f` over a vector, the shape every dump line uses.
fn fmt_vec(v: vec3_t) -> String {
    format!(
        "{:.6} {:.6} {:.6}",
        f64::from(v[0]),
        f64::from(v[1]),
        f64::from(v[2])
    )
}

/// Runs one scenario script and returns the text dump plus the final ring.
fn run_scenario(name: &str) -> (String, Vec<u8>) {
    let tool = repo_tool_dir();
    let script_path = tool.join(format!("scenarios/{name}.snd"));
    let script = fs::read_to_string(&script_path).expect("scenario script");

    let home = tempdir(&format!("snd-home-{name}"));
    let assets = tempdir(&format!("snd-assets-{name}"));
    seat_fixture_tree(&tool, &assets);

    let mut rig = Rig::new(&home, &assets);
    rig.out
        .push_str(&format!("== snd-oracle scenarios/{name}.snd ==\n"));

    for line in script.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let Some(&cmd) = words.first() else {
            continue;
        };
        if cmd.starts_with('#') {
            continue;
        }
        let int = |i: usize| -> i32 { words[i].parse().expect("integer argument") };
        let flt = |i: usize| -> f32 { words[i].parse().expect("float argument") };
        let vec = |i: usize| -> vec3_t { [flt(i), flt(i + 1), flt(i + 2)] };
        let slot = |rig: &Rig, i: usize| -> sfxHandle_t { rig.slots[int(i) as usize] };

        match cmd {
            "cvar" => rig.seed_cvar(words[1], words[2]),
            "init" => rig.with(|view, snd| S_Init(view, snd)),
            "beginreg" => rig.with(|view, snd| S_BeginRegistration(view, snd)),
            "register" => {
                let index = int(1) as usize;
                let path = words[2].to_string();
                let handle = rig.with(|view, snd| S_RegisterSound(view, snd, &path));
                rig.slots[index] = handle;
                rig.out
                    .push_str(&format!("REGISTER slot {index} handle {handle} path {path}\n"));
            }
            "length" => {
                let h = slot(&rig, 1);
                let ms = rig.with(|_, snd| S_GetSampleLengthInMilliSeconds(snd, h));
                rig.out
                    .push_str(&format!("LENGTH handle {h} ms {:.6}\n", f64::from(ms)));
            }
            "respatialize" => {
                let (ent, head, inwater) = (int(1), vec(2), int(5));
                let axis = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
                rig.with(|view, snd| S_Respatialize(view.common, snd, ent, head, axis, inwater));
            }
            "respatializeaxis" => {
                let ent = int(1);
                let head = vec(2);
                let axis = [vec(5), vec(8), vec(11)];
                let inwater = int(14);
                rig.with(|view, snd| S_Respatialize(view.common, snd, ent, head, axis, inwater));
            }
            "entitypos" => {
                let (ent, org) = (int(1), vec(2));
                rig.with(|_, snd| S_UpdateEntityPosition(snd, ent, org));
            }
            "startsound" => {
                let (org, ent, chan, h) = (vec(1), int(4), int(5), slot(&rig, 6));
                rig.with(|view, snd| S_StartSound(view, snd, Some(org), ent, chan, h));
            }
            "startsoundent" => {
                // A null origin makes the channel follow the entity position.
                let (ent, chan, h) = (int(1), int(2), slot(&rig, 3));
                rig.with(|view, snd| S_StartSound(view, snd, None, ent, chan, h));
            }
            "startlocal" => {
                let (h, chan) = (slot(&rig, 1), int(2));
                rig.with(|view, snd| S_StartLocalSound(view, snd, h, chan));
            }
            "startlocalloop" => {
                let h = slot(&rig, 1);
                rig.with(|view, snd| S_StartLocalLoopingSound(view, snd, h));
            }
            "startambient" => {
                let (org, ent, volume, h) = (vec(1), int(4), int(5) as u8, slot(&rig, 6));
                rig.with(|view, snd| S_StartAmbientSound(view, snd, Some(org), ent, volume, h));
            }
            "ambientloop" => {
                let (org, volume, h) = (vec(1), int(4) as u8, slot(&rig, 5));
                rig.with(|view, snd| S_AddAmbientLoopingSound(view, snd, org, volume, h));
            }
            "clearloops" => rig.with(|_, snd| S_ClearLoopingSounds(snd)),
            "music" => {
                let intro = words[1].to_string();
                let loop_name = if words.len() > 2 {
                    words[2].to_string()
                } else {
                    String::new()
                };
                let called_by_cgame = words.len() > 3 && int(3) != 0;
                rig.with(|view, snd| {
                    S_StartBackgroundTrack(view, snd, &intro, &loop_name, called_by_cgame)
                });
            }
            "restartmusic" => rig.with(|view, snd| S_RestartMusic(view, snd)),
            "musicdata" => {
                let label = words[1].to_string();
                let available =
                    rig.with(|view, snd| Music_DynamicDataAvailable(view, snd, &label));
                rig.out.push_str(&format!(
                    "MUSICDATA label {label} available {}\n",
                    i32::from(available)
                ));
            }
            "musicsetname" => {
                let name = rig.with(|_, snd| Music_GetLevelSetName(&snd.music));
                rig.out.push_str(&format!("MUSICSETNAME {name}\n"));
            }
            "musicfile" => {
                let state = int(1);
                let name = rig
                    .with(|_, snd| Music_GetFileNameForState(&snd.music, music_state(state)))
                    .unwrap_or_else(|| "<none>".to_string());
                rig.out
                    .push_str(&format!("MUSICFILE state {state} name {name}\n"));
            }
            "musicinterrupt" => {
                let (a, b) = (int(1), int(2));
                let allowed = Music_StateCanBeInterrupted(music_state(a), music_state(b));
                rig.out.push_str(&format!(
                    "MUSICINTERRUPT from {a} to {b} allowed {}\n",
                    i32::from(allowed)
                ));
            }
            "musictransition" => {
                let (elapsed, state) = (flt(1), int(2));
                let answer = rig.with(|view, snd| {
                    Music_AllowedToTransition(view, &snd.music, elapsed, music_state(state))
                });
                let (allowed, to, entry) = match answer {
                    Some((transition, entry)) => (1, transition as i32, entry),
                    None => (0, -1, 0.0),
                };
                rig.out.push_str(&format!(
                    "MUSICTRANSITION at {:.6} state {state} allowed {allowed} to {to} entry {:.6}\n",
                    f64::from(elapsed),
                    f64::from(entry)
                ));
            }
            "musicentrytime" => {
                let state = int(1);
                let time = rig.with(|view, snd| {
                    Music_GetRandomEntryTime(&mut snd.music, &mut view.common.qrand, music_state(state))
                });
                rig.out.push_str(&format!(
                    "MUSICENTRYTIME state {state} time {:.6}\n",
                    f64::from(time)
                ));
            }
            "stopmusic" => rig.with(|view, snd| S_StopBackgroundTrack(view.common, snd)),
            "asprecache" => {
                let name = words[1].to_string();
                rig.with(|_, snd| AS_AddPrecacheEntry(&mut snd.ambient, &name));
            }
            "asparse" => rig.with(|view, snd| AS_ParseSets(view, snd)),
            "asupdate" => {
                let (name, origin, realtime) = (words[1].to_string(), vec(2), int(5));
                rig.with(|view, snd| S_UpdateAmbientSet(view, snd, &name, origin, realtime));
            }
            "aslocal" => {
                let name = words[1].to_string();
                let (listener, origin) = (vec(2), vec(5));
                let (entID, time, realtime) = (int(8), int(9), int(10));
                let answer = rig.with(|view, snd| {
                    S_AddLocalSet(view, snd, &name, listener, origin, entID, time, realtime)
                });
                rig.out
                    .push_str(&format!("LOCALSET name {name} time {answer}\n"));
            }
            "asbmodel" => {
                let (name, stage) = (words[1].to_string(), int(2));
                let handle = rig.with(|_, snd| AS_GetBModelSound(&snd.ambient, &name, stage));
                rig.out
                    .push_str(&format!("BMODELSOUND name {name} stage {stage} handle {handle}\n"));
            }
            "dumpmusic" => rig.dump_music(words[1]),
            "addloop" => {
                let (ent, org, vel, h) = (int(1), vec(2), vec(5), slot(&rig, 8));
                rig.with(|view, snd| S_AddLoopingSound(view, snd, ent, org, vel, h));
            }
            "stoploop" => {
                let ent = int(1);
                rig.with(|_, snd| S_StopLoopingSound(snd, ent));
            }
            "mute" => {
                let (ent, chan) = (int(1), int(2));
                rig.with(|view, snd| S_MuteSound(view.common, snd, ent, chan));
            }
            "stopsounds" => rig.with(|_, snd| S_StopSounds(snd)),
            "stopall" => rig.with(|view, snd| S_StopAllSounds(view.common, snd)),
            "disable" => rig.with(|view, snd| S_DisableSounds(view.common, snd)),
            "rawsamples" => {
                // A scripted stereo ramp, so the raw path has deterministic content.
                let (frames, rate, amplitude) = (int(1), int(2), int(3));
                let mut block: Vec<u8> = Vec::with_capacity(frames as usize * 4);
                for i in 0..frames {
                    let value = ((i % 64) * amplitude / 64) as i16;
                    block.extend_from_slice(&value.to_le_bytes());
                    block.extend_from_slice(&(-value).to_le_bytes());
                }
                rig.with(|view, snd| {
                    S_RawSamples(view.common, snd, frames, rate, 2, 2, &block, 1.0, true)
                });
            }
            "advance" => rig.advance(int(1)),
            "update" => rig.with(|view, snd| S_Update(view, snd)),
            "dumpstate" => rig.dump_state(words[1]),
            "dumpsfx" => rig.dump_sfx(words[1]),
            "dumpring" => rig.dump_ring(words[1]),
            "dumplipsync" => rig.dump_lipsync(words[1]),
            "shutdown" => {
                rig.snapshot_ring();
                rig.with(|view, snd| S_Shutdown(view, snd));
            }
            other => panic!("snd-oracle: unknown command '{other}'"),
        }
    }

    rig.snapshot_ring();
    rig.out.push_str("== end ==\n");

    let _ = fs::remove_dir_all(&home);
    let _ = fs::remove_dir_all(&assets);

    (rig.out.clone(), rig.ringSnapshot.clone())
}

// ===========================================================================
// The temporary game tree
// ===========================================================================

fn tempdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jka-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// Write the game tree the boot needs: a `mpdefault.cfg` plus the committed
/// generated fixtures under `sound/`.
///
/// The tree has no `productid.txt`, so the boot drops into restricted demo mode
/// and restarts the file system on the `demo` game directory. Both directories
/// get the same pair, so the boot finds `mpdefault.cfg` either way.
fn seat_fixture_tree(tool: &Path, assets: &Path) {
    // `fixtures/mp3` stays out: it belongs to the decoder pin
    // (`crates/mp/engine/client/tests/mp3_decode_fixtures.rs`), not the game tree.
    for game in ["base", "demo"] {
        let dir = assets.join(game);
        fs::create_dir_all(&dir).expect("game dir");
        fs::write(dir.join("mpdefault.cfg"), b"// snd-oracle parity rig\n")
            .expect("mpdefault.cfg");
        for tree in ["sound", "music", "ext_data"] {
            copy_tree(&tool.join("fixtures").join(tree), &dir.join(tree));
        }
    }
}

/// Copy one fixture directory, subdirectories included (`sound/chars`).
fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("fixture dir");
    for entry in fs::read_dir(src).expect("fixture dir") {
        let entry = entry.expect("fixture entry");
        let target = dst.join(entry.file_name());
        if entry.file_type().expect("fixture type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("fixture copy");
        }
    }
}

// ===========================================================================
// The gate
// ===========================================================================

#[test]
fn every_scenario_reproduces_its_goldens() {
    let golden = repo_tool_dir().join("golden");
    let mut failures: Vec<String> = Vec::new();

    for name in SCENARIOS {
        let run = std::panic::catch_unwind(|| run_scenario(name));
        let (text, ring) = match run {
            Ok(pair) => pair,
            Err(payload) => {
                let msg = payload
                    .downcast_ref::<ComError>()
                    .map(|e| e.msg.clone())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".to_string());
                failures.push(format!("{name}: the run died: {msg}"));
                continue;
            }
        };

        if std::env::var("SND_ORACLE_DUMP").is_ok() {
            let out = std::env::temp_dir().join(format!("snd-rust-{name}.txt"));
            fs::write(&out, &text).expect("dump write");
        }

        let want_text = fs::read_to_string(golden.join(format!("{name}.txt")))
            .expect("text golden");
        let want_ring = fs::read(golden.join(format!("{name}.bin"))).expect("ring golden");

        for (i, (got, want)) in text.lines().zip(want_text.lines()).enumerate() {
            if got != want {
                failures.push(format!(
                    "{name}: line {} differs\n  rust:   {got}\n  oracle: {want}",
                    i + 1
                ));
                break;
            }
        }
        if text.lines().count() != want_text.lines().count() {
            failures.push(format!(
                "{name}: the dump has {} lines and the golden has {}",
                text.lines().count(),
                want_text.lines().count()
            ));
        }

        if ring != want_ring {
            let first = ring
                .iter()
                .zip(want_ring.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(ring.len().min(want_ring.len()));
            failures.push(format!(
                "{name}: the ring bytes differ, first at offset {first}"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
