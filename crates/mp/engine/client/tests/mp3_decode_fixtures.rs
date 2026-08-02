//! The MP3 decoder pin (DEC-57.3).
//!
//! # Why this rig exists
//! DEC-03 and DEC-57.3 keep the MP3 decoder outside the snd-oracle byte gate:
//! Raven's `codemp/mp3code` C decoder does not port, so decoded PCM cannot match
//! it byte for byte. The decoder gets pinned decode fixtures of its own instead.
//! A change in the decoder crate, in the framing walk, or in the reduction and
//! downmix arithmetic moves these goldens and fails the test.
//!
//! # The fixtures
//! `tools/snd-oracle/gen_mp3_fixtures.py` writes the `.mp3` files from integer
//! arithmetic, so no retail audio is involved and a regenerated fixture is
//! identical on any host. The `.pcm` goldens beside them are the decoder's own
//! answer, minted by this test with `MP3_FIXTURES_REGEN=1`.

#![allow(non_snake_case)]

use std::fs;
use std::path::PathBuf;

use mp_engine_client::mp3::mp3_frame_header::Mp3FrameHeader;
use mp_engine_client::mp3::mp3_stream_state::MP3StreamState;

/// One pinned decode: the MP3 file, the settings, and the golden it answers.
struct Pin {
    mp3: &'static str,
    /// The game mixer rate the packet output is reduced to.
    rate: i32,
    /// Raven's `convert_code`: false downmixes to mono, true keeps the pair.
    stereo: bool,
    pcm: &'static str,
}

/// The pinned set. `silence_*` proves the framing, the packet accounting, and
/// every reduction step; `tone_*` proves the granule decode itself.
const PINS: [Pin; 7] = [
    Pin {
        mp3: "silence_stereo.mp3",
        rate: 44100,
        stereo: true,
        pcm: "silence_stereo.44k_stereo.pcm",
    },
    Pin {
        mp3: "silence_stereo.mp3",
        rate: 22050,
        stereo: true,
        pcm: "silence_stereo.22k_stereo.pcm",
    },
    Pin {
        mp3: "silence_mono.mp3",
        rate: 11025,
        stereo: false,
        pcm: "silence_mono.11k_mono.pcm",
    },
    Pin {
        mp3: "tone_mono.mp3",
        rate: 44100,
        stereo: false,
        pcm: "tone_mono.44k_mono.pcm",
    },
    Pin {
        mp3: "tone_mono.mp3",
        rate: 22050,
        stereo: false,
        pcm: "tone_mono.22k_mono.pcm",
    },
    Pin {
        mp3: "tone_mono.mp3",
        rate: 11025,
        stereo: false,
        pcm: "tone_mono.11k_mono.pcm",
    },
    Pin {
        mp3: "tone_tagged.mp3",
        rate: 22050,
        stereo: false,
        pcm: "tone_tagged.22k_mono.pcm",
    },
];

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/snd-oracle/fixtures/mp3")
}

/// Decode one whole file the way `MP3_UnpackRawPCM` does, and answer the bytes.
fn decode_all(data: &[u8], rate: i32, stereo: bool) -> Vec<u8> {
    let mut stream = MP3StreamState::default();
    assert!(
        stream
            .DecodeInit(data, data.len() as i32, rate, stereo)
            .is_none(),
        "the decoder refused a pinned fixture"
    );

    let mut out = Vec::new();
    loop {
        let iBytes = stream.Decode(data, 0);
        if iBytes == 0 {
            break;
        }
        out.extend_from_slice(&stream.bDecodeBuffer[..iBytes as usize]);
    }
    out
}

#[test]
fn every_pinned_fixture_decodes_to_its_golden() {
    let dir = fixture_dir();
    let regen = std::env::var("MP3_FIXTURES_REGEN").is_ok();
    let mut failures: Vec<String> = Vec::new();

    for pin in PINS {
        let data = fs::read(dir.join(pin.mp3)).expect("pinned MP3 fixture");
        let got = decode_all(&data, pin.rate, pin.stereo);

        let goldenPath = dir.join(pin.pcm);
        if regen {
            fs::write(&goldenPath, &got).expect("golden write");
            continue;
        }

        let want = fs::read(&goldenPath).expect("pinned PCM golden");
        if got.len() != want.len() {
            failures.push(format!(
                "{}: decoded {} bytes, the golden has {}",
                pin.pcm,
                got.len(),
                want.len()
            ));
            continue;
        }
        if got != want {
            let first = got
                .iter()
                .zip(want.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(0);
            failures.push(format!("{}: the PCM differs, first at byte {first}", pin.pcm));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The packet accounting is analytic: eight frames of 1152 samples each, times
/// the output channel count, times two bytes, divided by the rate reduction.
#[test]
fn the_packet_accounting_matches_the_frame_walk() {
    let dir = fixture_dir();
    let data = fs::read(dir.join("silence_stereo.mp3")).expect("pinned MP3 fixture");

    let (offset, head) = Mp3FrameHeader::find(&data, data.len() / 2).expect("a frame header");
    assert_eq!(offset, 0);
    assert_eq!(head.sampleRate, 44100);
    assert_eq!(head.channels, 2);
    assert_eq!(head.frameBytes, 417);
    assert_eq!(head.samplesPerFrame, 1152);

    let frames = data.len() / head.frameBytes;
    for (rate, reduction) in [(44100, 1), (22050, 2), (11025, 4)] {
        for (stereo, channels) in [(false, 1), (true, 2)] {
            let got = decode_all(&data, rate, stereo);
            assert_eq!(
                got.len(),
                frames * head.samplesPerFrame * channels * 2 / reduction,
                "rate {rate} stereo {stereo}"
            );
        }
    }
}

/// A file with no sync word anywhere is refused before any decode runs, and a
/// stream that stops mid-frame stops the walk rather than reading past the end.
#[test]
fn the_bad_files_are_refused() {
    let dir = fixture_dir();

    let notmp3 = fs::read(dir.join("notmp3.mp3")).expect("pinned MP3 fixture");
    assert!(Mp3FrameHeader::find(&notmp3, notmp3.len() / 2).is_none());

    let mut stream = MP3StreamState::default();
    assert!(stream
        .DecodeInit(&notmp3, notmp3.len() as i32, 22050, false)
        .is_some());

    let truncated = fs::read(dir.join("truncated.mp3")).expect("pinned MP3 fixture");
    let got = decode_all(&truncated, 22050, false);
    // One whole frame decodes; the partial second frame does not.
    assert_eq!(got.len(), 1152 * 2 / 2);
}

/// The trailing ID3v1 tag is taken off the byte count, so the decoder never
/// treats the tag as audio, and the two Raven keys read back.
#[test]
fn the_id3v1_tag_is_read_and_skipped() {
    let dir = fixture_dir();
    let tagged = fs::read(dir.join("tone_tagged.mp3")).expect("pinned MP3 fixture");
    let plain = fs::read(dir.join("tone_mono.mp3")).expect("pinned MP3 fixture");

    assert_eq!(tagged.len(), plain.len() + 128);
    assert_eq!(
        decode_all(&tagged, 22050, false),
        decode_all(&plain, 22050, false)
    );
}
