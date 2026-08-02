//! Golden-shape gate for the sound harness (`tools/snd-oracle`, DEC-57.2).
//!
//! The harness compiles the unmodified Raven sound TUs and dumps two goldens per
//! scenario: `<name>.txt` holds the state and ring digests, and `<name>.bin`
//! holds the final `dma_t` ring bytes. This test reads the committed goldens and
//! checks that the set is complete and self-consistent, so a truncated or
//! half-regenerated golden fails here rather than inside the mixer comparison.
//!
//! The mixer comparison itself is `mp_engine_core`'s `snd_oracle_parity` test,
//! which replays every scenario through the Rust port (gh#24). Nothing in either
//! test needs a C++ toolchain.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// The scenarios the harness ships. A new scenario must be added here, or the
/// completeness check fails.
/// Source: `tools/snd-oracle/scenarios/`
const SCENARIOS: [&str; 11] = [
    "badfiles",
    "basic",
    "channels",
    "khz11",
    "khz44",
    "lipsync",
    "loops",
    "rawstream",
    "resample",
    "ringwrap",
    "spatialize",
];

/// The retail DirectSound secondary buffer is 65536 bytes at every `s_khz`.
/// Source: `oracle/codemp/win32/win_snd.cpp:12,246`
const RING_BYTES: usize = 0x10000;

/// The dump splits the ring into 4 KB blocks.
/// Source: `tools/snd-oracle/main.cpp` `SND_ORACLE_RING_BLOCK`
const RING_BLOCK: usize = 4096;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/snd-oracle/golden")
}

/// One `RING` header line plus the block lines that follow it.
struct RingDump {
    tag: String,
    bytes: usize,
    whole: u32,
    blocks: Vec<(usize, u32, i32, i32, usize)>,
}

/// This reads every `RING` block out of one scenario text dump.
/// The parser is strict, so a malformed golden fails the test instead of parsing
/// to an empty result.
fn parse_ring_dumps(text: &str) -> Vec<RingDump> {
    let mut dumps: Vec<RingDump> = Vec::new();

    for line in text.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();

        if words.first() == Some(&"RING") {
            assert_eq!(words.len(), 6, "malformed RING header: {line}");
            assert_eq!(words[2], "bytes");
            assert_eq!(words[4], "whole");
            dumps.push(RingDump {
                tag: words[1].to_string(),
                bytes: words[3].parse().expect("RING byte count"),
                whole: u32::from_str_radix(words[5], 16).expect("RING digest"),
                blocks: Vec::new(),
            });
        } else if words.first() == Some(&"blk") {
            assert_eq!(words.len(), 10, "malformed block line: {line}");
            let dump = dumps.last_mut().expect("a block line before any RING header");
            dump.blocks.push((
                words[1].parse().expect("block index"),
                u32::from_str_radix(words[3], 16).expect("block digest"),
                words[5].parse().expect("block minimum"),
                words[7].parse().expect("block maximum"),
                words[9].parse().expect("block nonzero count"),
            ));
        }
    }

    dumps
}

#[test]
fn every_scenario_ships_both_goldens() {
    let dir = golden_dir();
    for name in SCENARIOS {
        let text = dir.join(format!("{name}.txt"));
        let ring = dir.join(format!("{name}.bin"));

        assert!(text.is_file(), "missing text golden for {name}");
        assert!(ring.is_file(), "missing ring golden for {name}");

        let ring_len = fs::metadata(&ring).expect("ring golden").len() as usize;
        assert_eq!(ring_len, RING_BYTES, "{name}: the ring golden is the wrong size");
    }
}

#[test]
fn text_goldens_are_well_formed() {
    let dir = golden_dir();
    for name in SCENARIOS {
        let text = fs::read_to_string(dir.join(format!("{name}.txt"))).expect("text golden");

        assert!(text.starts_with("== snd-oracle "), "{name}: missing the header line");
        assert!(text.trim_end().ends_with("== end =="), "{name}: the run did not finish");
        assert!(
            text.contains("STATE "),
            "{name}: the scenario dumps no state at all"
        );

        // The harness aborts on the dropped OpenAL and EAX arm, so a golden that
        // names it means the arm ran.
        assert!(
            !text.contains("snd-oracle: the dropped"),
            "{name}: the dropped arm reached the goldens"
        );

        // Every scenario runs at one of the three rates Raven offers.
        assert!(
            text.contains("speed 22050") || text.contains("speed 11025") || text.contains("speed 44100"),
            "{name}: no known sample rate in the state dump"
        );
    }
}

#[test]
fn ring_dumps_cover_the_whole_buffer() {
    let dir = golden_dir();
    for name in SCENARIOS {
        let text = fs::read_to_string(dir.join(format!("{name}.txt"))).expect("text golden");
        let dumps = parse_ring_dumps(&text);
        assert!(!dumps.is_empty(), "{name}: no ring dump in the golden");

        for dump in &dumps {
            assert_eq!(dump.bytes, RING_BYTES, "{name}/{}: wrong ring size", dump.tag);
            assert_eq!(
                dump.blocks.len(),
                RING_BYTES / RING_BLOCK,
                "{name}/{}: wrong block count",
                dump.tag
            );

            for (i, block) in dump.blocks.iter().enumerate() {
                assert_eq!(block.0, i, "{name}/{}: block {i} is out of order", dump.tag);
                assert!(block.2 <= 0, "{name}/{}: block {i} minimum is positive", dump.tag);
                assert!(block.3 >= 0, "{name}/{}: block {i} maximum is negative", dump.tag);
                assert!(
                    block.4 <= RING_BLOCK / 2,
                    "{name}/{}: block {i} counts more samples than it holds",
                    dump.tag
                );
                // A silent block is exactly the zero block, and the digest proves it.
                let silent = block.2 == 0 && block.3 == 0;
                assert_eq!(
                    silent,
                    block.4 == 0,
                    "{name}/{}: block {i} disagrees on silence",
                    dump.tag
                );
            }

            // The digests must distinguish the blocks. Two blocks that share a
            // digest also share their extremes.
            let mut seen: BTreeMap<u32, (i32, i32, usize)> = BTreeMap::new();
            for block in &dump.blocks {
                let stats = (block.2, block.3, block.4);
                if let Some(previous) = seen.insert(block.1, stats) {
                    assert_eq!(
                        previous, stats,
                        "{name}/{}: one digest covers two different blocks",
                        dump.tag
                    );
                }
            }

            assert_ne!(dump.whole, 0, "{name}/{}: the whole-ring digest is empty", dump.tag);
        }
    }
}

#[test]
fn every_scenario_paints_something() {
    let dir = golden_dir();
    for name in SCENARIOS {
        let text = fs::read_to_string(dir.join(format!("{name}.txt"))).expect("text golden");
        let dumps = parse_ring_dumps(&text);

        // The final ring can be silent, because `channels` ends on
        // S_StopAllSounds and that clears the buffer. At least one dump in the
        // run must carry audio, or the scenario proves nothing.
        let painted = dumps
            .iter()
            .any(|dump| dump.blocks.iter().any(|block| block.4 > 0));
        assert!(painted, "{name}: no ring dump in the run carries audio");
    }
}

#[test]
fn ring_goldens_match_their_digests() {
    let dir = golden_dir();
    for name in SCENARIOS {
        let text = fs::read_to_string(dir.join(format!("{name}.txt"))).expect("text golden");
        let ring = fs::read(dir.join(format!("{name}.bin"))).expect("ring golden");

        // The .bin holds the last ring the scenario dumped, so it must match the
        // last RING block in the text.
        let dumps = parse_ring_dumps(&text);
        let last = dumps.last().expect("a ring dump");
        assert_eq!(
            fnv1a(&ring),
            last.whole,
            "{name}: the ring bytes and the last text digest disagree"
        );
    }
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
