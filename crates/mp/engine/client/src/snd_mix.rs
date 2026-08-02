//! `snd_mix.cpp` — the paint chain and the ring transfer.
//!
//! Raven mixes each channel into a 1024-pair integer paint buffer, then shifts
//! and clamps that buffer into the `dma_t` ring. The `id386` MMX arm is not
//! ported: it saturates exactly as the portable C arm does, and OpenJK ships the
//! C arm.
//!
//! Source: `oracle/codemp/client/snd_mix.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::common::Common;
use mp_qshared::shared::sound_channel::{CHAN_VOICE, CHAN_VOICE_ATTEN, CHAN_VOICE_GLOBAL};

use crate::snd::sound_compression_method_t::SoundCompressionMethod_t;
use crate::snd::sound_system::{SoundSystem, MAX_CHANNELS, MAX_RAW_SAMPLES, PAINTBUFFER_SIZE};

/// Raven `S_WriteLinearBlastStereo16` — shift down by 8, clamp to 16 bits, and
/// write `count` interleaved samples into the ring.
///
/// `paint_base` and `out_base` are both counted in samples, not in pairs, the
/// way Raven's `snd_p` and `snd_out` pointers are.
/// Source: `oracle/codemp/client/snd_mix.cpp:21-44`
fn S_WriteLinearBlastStereo16(
    snd: &mut SoundSystem,
    paint_base: usize,
    out_base: usize,
    count: usize,
) {
    let paint = &snd.paintbuffer;
    let out = &mut snd.dma.buffer;

    let mut i = 0usize;
    while i < count {
        for lane in 0..2usize {
            let index = paint_base + i + lane;
            let raw = if index & 1 == 0 {
                paint[index >> 1].left
            } else {
                paint[index >> 1].right
            };
            let val = raw >> 8;
            let clamped: i16 = if val > 0x7fff {
                0x7fff
            } else if val < -0x8000 {
                -0x8000
            } else {
                val as i16
            };
            let byte = (out_base + i + lane) * 2;
            out[byte..byte + 2].copy_from_slice(&clamped.to_le_bytes());
        }
        i += 2;
    }
}

/// Raven `S_TransferStereo16` — walk the paint buffer into the ring, wrapping at
/// the ring end.
///
/// Source: `oracle/codemp/client/snd_mix.cpp:137-164`
fn S_TransferStereo16(snd: &mut SoundSystem, endtime: c_int) {
    let mut snd_p: usize = 0;
    let mut ls_paintedtime = snd.s_paintedtime;

    while ls_paintedtime < endtime {
        // handle recirculating buffer issues
        let lpos = ls_paintedtime & ((snd.dma.samples >> 1) - 1);

        let snd_out = (lpos << 1) as usize;

        let mut snd_linear_count = (snd.dma.samples >> 1) - lpos;
        if ls_paintedtime + snd_linear_count > endtime {
            snd_linear_count = endtime - ls_paintedtime;
        }

        snd_linear_count <<= 1;

        // write a linear blast of samples
        S_WriteLinearBlastStereo16(snd, snd_p, snd_out, snd_linear_count as usize);

        snd_p += snd_linear_count as usize;
        ls_paintedtime += snd_linear_count >> 1;
    }
}

/// Raven `S_TransferPaintBuffer` — the format switch in front of the ring write.
///
/// The `s_testsound` arm overwrites the whole window with a sine before the
/// transfer, so a cheat-enabled run hears the test tone and nothing else.
/// Source: `oracle/codemp/client/snd_mix.cpp:172-239`
fn S_TransferPaintBuffer(common: &mut Common, snd: &mut SoundSystem, endtime: c_int) {
    if common.cvar(snd.s_testsound).integer != 0 {
        // write a fixed sine wave
        let count = endtime - snd.s_paintedtime;
        for i in 0..count {
            let value = (f64::sin(f64::from(snd.s_paintedtime + i) * 0.1) * 20000.0 * 256.0) as c_int;
            snd.paintbuffer[i as usize].left = value;
            snd.paintbuffer[i as usize].right = value;
        }
    }

    if snd.dma.samplebits == 16 && snd.dma.channels == 2 {
        // optimized case
        S_TransferStereo16(snd, endtime);
        return;
    }

    // general case
    let count = (endtime - snd.s_paintedtime) * snd.dma.channels;
    let out_mask = snd.dma.samples - 1;
    let mut out_idx = snd.s_paintedtime * snd.dma.channels & out_mask;
    let step = (3 - snd.dma.channels) as usize;

    let mut p: usize = 0;
    if snd.dma.samplebits == 16 {
        for _ in 0..count {
            let raw = if p & 1 == 0 {
                snd.paintbuffer[p >> 1].left
            } else {
                snd.paintbuffer[p >> 1].right
            };
            let mut val = raw >> 8;
            p += step;
            if val > 0x7fff {
                val = 0x7fff;
            } else if val < -0x8000 {
                val = -0x8000;
            }
            let byte = out_idx as usize * 2;
            snd.dma.buffer[byte..byte + 2].copy_from_slice(&(val as i16).to_le_bytes());
            out_idx = (out_idx + 1) & out_mask;
        }
    } else if snd.dma.samplebits == 8 {
        for _ in 0..count {
            let raw = if p & 1 == 0 {
                snd.paintbuffer[p >> 1].left
            } else {
                snd.paintbuffer[p >> 1].right
            };
            let mut val = raw >> 8;
            p += step;
            if val > 0x7fff {
                val = 0x7fff;
            } else if val < -0x8000 {
                val = -0x8000;
            }
            snd.dma.buffer[out_idx as usize] = ((val >> 8) + 128) as u8;
            out_idx = (out_idx + 1) & out_mask;
        }
    }
}

/// Raven `S_PaintChannelFrom16` — add one 16-bit channel into the paint buffer.
///
/// Source: `oracle/codemp/client/snd_mix.cpp:249-267`
fn S_PaintChannelFrom16(
    snd: &mut SoundSystem,
    channel: usize,
    sfx: usize,
    count: c_int,
    sampleOffset: c_int,
    bufferOffset: c_int,
) {
    let iLeftVol = snd.s_channels[channel].leftvol * snd.snd_vol;
    let iRightVol = snd.s_channels[channel].rightvol * snd.snd_vol;

    let data = &snd.s_knownSfx[sfx].pSoundData;
    let dest = &mut snd.paintbuffer;

    let mut sampleOffset = sampleOffset as usize;
    let bufferOffset = bufferOffset as usize;
    for i in 0..count as usize {
        let iData = c_int::from(data[sampleOffset]);
        sampleOffset += 1;

        dest[bufferOffset + i].left += (iData * iLeftVol) >> 8;
        dest[bufferOffset + i].right += (iData * iRightVol) >> 8;
    }
}

/// Raven `ChannelPaint` — the compression-method switch in front of the two painters.
///
/// Source: `oracle/codemp/client/snd_mix.cpp:320-339`
fn ChannelPaint(
    snd: &mut SoundSystem,
    channel: usize,
    sfx: usize,
    count: c_int,
    sampleOffset: c_int,
    bufferOffset: c_int,
) {
    match snd.s_knownSfx[sfx].eSoundCompressionMethod {
        SoundCompressionMethod_t::ct_16 => {
            S_PaintChannelFrom16(snd, channel, sfx, count, sampleOffset, bufferOffset);
        }
        SoundCompressionMethod_t::ct_MP3 => {
            //TODO: Port S_PaintChannelFromMP3
            // Source: oracle/codemp/client/snd_mix.cpp:270. The decoder is gh#25 under
            // DEC-57.3, and no gh#24 load path produces a `ct_MP3` sfx.
            todo!("Port S_PaintChannelFromMP3 — oracle/codemp/client/snd_mix.cpp:270 (gh#25)")
        }
        // Raven's `default` arm is an `assert(0)` that the release build drops,
        // so a method outside the two above paints nothing.
        SoundCompressionMethod_t::ct_NUMBEROF => {}
    }
}

/// Raven `S_PaintChannels` — fill the ring up to `endtime`.
///
/// One pass clears the paint buffer to the raw stream or to silence, adds every
/// audible channel, and transfers the result. A looping channel wraps inside the
/// pass, so it can paint twice.
/// Source: `oracle/codemp/client/snd_mix.cpp:343-470`
pub fn S_PaintChannels(common: &mut Common, snd: &mut SoundSystem, endtime: c_int) {
    let normal_vol = (common.cvar(snd.s_volume).value * 256.0) as c_int;
    let voice_vol = (common.cvar(snd.s_volumeVoice).value * 256.0) as c_int;
    snd.snd_vol = normal_vol;

    while snd.s_paintedtime < endtime {
        // if paintbuffer is smaller than DMA buffer
        // we may need to fill it multiple times
        let mut end = endtime;
        if endtime - snd.s_paintedtime > PAINTBUFFER_SIZE as c_int {
            end = snd.s_paintedtime + PAINTBUFFER_SIZE as c_int;
        }

        // clear the paint buffer to either music or zeros
        if snd.s_rawend < snd.s_paintedtime {
            for pair in snd.paintbuffer[..(end - snd.s_paintedtime) as usize].iter_mut() {
                pair.left = 0;
                pair.right = 0;
            }
        } else {
            // copy from the streaming sound source
            let stop = if end < snd.s_rawend { end } else { snd.s_rawend };

            let painted = snd.s_paintedtime;
            let mut i = painted;
            while i < stop {
                let s = (i & (MAX_RAW_SAMPLES as c_int - 1)) as usize;
                let value = snd.s_rawsamples[s];
                snd.paintbuffer[(i - painted) as usize] = value;
                i += 1;
            }
            while i < end {
                let pair = &mut snd.paintbuffer[(i - painted) as usize];
                pair.right = 0;
                pair.left = 0;
                i += 1;
            }
        }

        // paint in the channels.
        for channel in 0..MAX_CHANNELS {
            let ch = snd.s_channels[channel];
            let Some(sfx) = ch.thesfx else {
                continue;
            };
            if f64::from(ch.leftvol) < 0.25 && f64::from(ch.rightvol) < 0.25 {
                continue;
            }

            if ch.entchannel == CHAN_VOICE
                || ch.entchannel == CHAN_VOICE_ATTEN
                || ch.entchannel == CHAN_VOICE_GLOBAL
            {
                snd.snd_vol = voice_vol;
            } else {
                snd.snd_vol = normal_vol;
            }

            let mut ltime = snd.s_paintedtime;

            // we might have to make 2 passes if it is
            //	a looping sound effect and the end of
            //	the sameple is hit...
            loop {
                let length = snd.s_knownSfx[sfx].iSoundLengthInSamples;
                let sampleOffset = if ch.loopSound {
                    ltime % length
                } else {
                    ltime - ch.startSample as c_int
                };

                let mut count = end - ltime;
                if sampleOffset + count > length {
                    count = length - sampleOffset;
                }

                if count > 0 {
                    let bufferOffset = ltime - snd.s_paintedtime;
                    ChannelPaint(snd, channel, sfx, count, sampleOffset, bufferOffset);
                    ltime += count;
                }

                if !(ltime < end && ch.loopSound) {
                    break;
                }
            }
        }

        // transfer out according to DMA format
        S_TransferPaintBuffer(common, snd, end);
        snd.s_paintedtime = end;
    }
}
