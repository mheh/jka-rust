//! The cpal device end (DEC-57.1) — Raven's `win_snd.cpp` replacement.
//!
//! Raven mixed into a looping DirectSound secondary buffer and read the play
//! cursor back with `SNDDMA_GetDMAPos`. The port keeps that whole model: the
//! paint chain writes `SoundSystem.dma.buffer` at Raven's internal format
//! (16-bit stereo, `s_khz` rate), `SNDDMA_Submit` republishes the ring to the
//! device, and the cpal callback plays it back, converting rate and format to
//! whatever the output device negotiated. The callback's own read cursor is the
//! play cursor `SNDDMA_GetDMAPos` reports.
//!
//! Raven picked this device at link time (`win_snd.cpp` for the client,
//! `null_snddma.cpp` for `jampded`). The `sound_device` cargo feature is that
//! arm, so a dedicated build carries no cpal edge at all. A client build that
//! finds no usable output device keeps the silent ring and says so.
//!
//! Source: `oracle/codemp/win32/win_snd.cpp:105-355`

#![allow(non_snake_case)]

use core::ffi::c_int;

#[cfg(feature = "sound_device")]
use std::sync::atomic::{AtomicI64, Ordering};
#[cfg(feature = "sound_device")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "sound_device")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "sound_device")]
use cpal::{SampleFormat, StreamConfig};

/// The engine ring's own format. The paint chain only ever writes 16-bit
/// stereo, so the device end reads sample pairs and nothing else.
///
/// Source: `oracle/codemp/win32/win_snd.cpp:184-185`
#[cfg(feature = "sound_device")]
const RING_BYTES_PER_FRAME: usize = 4;

/// The play cursor is masked against `dma.samples`, so only the low bits
/// matter. This cap keeps the reported cursor a positive `int` forever.
#[cfg(feature = "sound_device")]
const CURSOR_MASK: i64 = 0x3FFF_FFFF;

/// The ring the cpal callback plays and the play cursor it advances.
#[cfg(feature = "sound_device")]
struct DeviceRing {
    /// The engine ring's bytes, republished by every `SNDDMA_Submit`.
    /// The sim thread holds this lock only for one memcpy of the ring.
    pcm: Mutex<Vec<u8>>,
    /// Source frames the callback has played, free running.
    played: AtomicI64,
}

/// An open output stream plus the ring it plays.
///
/// Dropping this stops playback and closes the device, which is what
/// `SNDDMA_Shutdown` wants.
#[cfg(feature = "sound_device")]
pub struct SoundDevice {
    ring: Arc<DeviceRing>,
    channels: i64,
    /// Held to keep the stream alive; cpal stops it on drop.
    _stream: cpal::Stream,
    /// The negotiated device format, for `S_SoundInfo_f`-style reporting.
    description: String,
}

/// The no-device build. `SoundDevice::open` always fails, so no value of this
/// type ever exists.
#[cfg(not(feature = "sound_device"))]
pub struct SoundDevice(core::convert::Infallible);

#[cfg(feature = "sound_device")]
impl SoundDevice {
    /// Open the default output device and start playing `ring_bytes` of 16-bit
    /// stereo at `speed`. The error string is the line `SNDDMA_Init` prints.
    ///
    /// Source: `oracle/codemp/win32/win_snd.cpp:122-257`
    pub fn open(speed: c_int, channels: c_int, ring_bytes: usize) -> Result<SoundDevice, String> {
        if channels != 2 {
            return Err(format!("the ring is stereo, not {channels} channel"));
        }

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "no default output device".to_string())?;
        let supported = device
            .default_output_config()
            .map_err(|error| format!("no default output config ({error})"))?;

        let format = supported.sample_format();
        let config: StreamConfig = supported.config();
        let device_channels = config.channels.max(1) as usize;
        let device_rate = config.sample_rate.max(1);
        // cpal 0.18 gives the device name through `Display`.
        let name = device.to_string();
        let description =
            format!("{name}, {device_rate} Hz, {device_channels} channel, {format:?}");

        let ring = Arc::new(DeviceRing {
            pcm: Mutex::new(vec![0u8; ring_bytes]),
            played: AtomicI64::new(0),
        });

        // Q32 source frames per device frame. The ring runs at Raven's internal
        // rate and the device at its own, so the callback steps by this.
        let step = ((speed.max(1) as u64) << 32) / device_rate as u64;
        let frames = (ring_bytes / RING_BYTES_PER_FRAME).max(1) as u64;

        let stream = match format {
            SampleFormat::F32 => build_stream::<f32>(&device, &config, &ring, step, frames, device_channels),
            SampleFormat::I16 => build_stream::<i16>(&device, &config, &ring, step, frames, device_channels),
            SampleFormat::U16 => build_stream::<u16>(&device, &config, &ring, step, frames, device_channels),
            SampleFormat::I32 => build_stream::<i32>(&device, &config, &ring, step, frames, device_channels),
            SampleFormat::F64 => build_stream::<f64>(&device, &config, &ring, step, frames, device_channels),
            other => return Err(format!("unsupported sample format {other:?}")),
        }
        .map_err(|error| format!("could not build the output stream ({error})"))?;

        stream
            .play()
            .map_err(|error| format!("could not start the output stream ({error})"))?;

        Ok(SoundDevice {
            ring,
            channels: channels as i64,
            _stream: stream,
            description,
        })
    }

    /// Republish the engine ring to the device. This is `SNDDMA_Submit`'s work
    /// once the DirectSound lock/unlock pair is gone.
    ///
    /// Source: `oracle/codemp/win32/win_snd.cpp:350-355`
    pub fn publish(&self, pcm: &[u8]) {
        let mut shared = match self.ring.pcm.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let len = shared.len().min(pcm.len());
        shared[..len].copy_from_slice(&pcm[..len]);
    }

    /// The play cursor in Raven's units: one per interleaved sample, so
    /// `dma.channels` per source frame.
    ///
    /// Source: `oracle/codemp/win32/win_snd.cpp:267-286`
    pub fn play_cursor(&self) -> c_int {
        let frames = self.ring.played.load(Ordering::Relaxed);
        ((frames * self.channels) & CURSOR_MASK) as c_int
    }

    /// The negotiated device format, for the `soundinfo` banner.
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Build the output stream for one cpal sample type. Every arm shares the same
/// resample-and-convert body, so the type parameter is the only difference.
#[cfg(feature = "sound_device")]
fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    ring: &Arc<DeviceRing>,
    step: u64,
    frames: u64,
    device_channels: usize,
) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let ring = Arc::clone(ring);
    // Q32 read position, owned by the callback: only the audio thread runs it.
    let mut phase: u64 = 0;
    device.build_output_stream::<T, _, _>(
        config.clone(),
        move |out: &mut [T], _| {
            let pcm = match ring.pcm.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            for out_frame in out.chunks_mut(device_channels) {
                let source = ((phase >> 32) % frames) as usize * RING_BYTES_PER_FRAME;
                let (left, right) = match pcm.get(source..source + RING_BYTES_PER_FRAME) {
                    Some(bytes) => (
                        i16::from_le_bytes([bytes[0], bytes[1]]) as f32 / 32768.0,
                        i16::from_le_bytes([bytes[2], bytes[3]]) as f32 / 32768.0,
                    ),
                    None => (0.0, 0.0),
                };
                write_frame(out_frame, left, right);
                phase = phase.wrapping_add(step);
            }
            ring.played.store((phase >> 32) as i64, Ordering::Relaxed);
        },
        |error| eprintln!("sound device error: {error}"),
        None,
    )
}

/// Spread one stereo sample pair over the device's channels. A mono device gets
/// the average, and any channel past the second gets silence.
#[cfg(feature = "sound_device")]
fn write_frame<T: cpal::SizedSample + cpal::FromSample<f32>>(
    out_frame: &mut [T],
    left: f32,
    right: f32,
) {
    match out_frame.len() {
        0 => {}
        1 => out_frame[0] = T::from_sample((left + right) * 0.5),
        _ => {
            out_frame[0] = T::from_sample(left);
            out_frame[1] = T::from_sample(right);
            for slot in &mut out_frame[2..] {
                *slot = T::from_sample(0.0f32);
            }
        }
    }
}

#[cfg(not(feature = "sound_device"))]
impl SoundDevice {
    /// The no-device arm: Raven's dedicated build had no `win_snd.cpp` linked.
    pub fn open(_speed: c_int, _channels: c_int, _ring_bytes: usize) -> Result<SoundDevice, String> {
        Err("built without the sound_device feature".to_string())
    }

    pub fn publish(&self, _pcm: &[u8]) {
        match self.0 {}
    }

    pub fn play_cursor(&self) -> c_int {
        match self.0 {}
    }

    pub fn description(&self) -> &str {
        match self.0 {}
    }
}
