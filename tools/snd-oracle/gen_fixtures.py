#!/usr/bin/env python3
"""Generate the committed PCM fixtures for tools/snd-oracle.

Every waveform comes from integer arithmetic on a fixed table, so a regenerated
fixture is byte-identical to the committed one on any host. Run it from
tools/snd-oracle:

    python3 gen_fixtures.py

The fixture set is documented in README.md.
"""

import math
import os
import struct

FIXTURE_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", "sound")


def write_wav(name, rate, width, channels, frames):
    """Write a RIFF/WAVE PCM file that Raven's GetWavinfo accepts.

    `frames` holds one list per channel-interleaved sample, already clamped to
    the width. Raven reads the format chunk at a fixed offset, so the layout
    stays minimal: RIFF, fmt, data, nothing else.
    """
    if width == 2:
        payload = b"".join(struct.pack("<h", s) for s in frames)
    else:
        payload = bytes(frames)

    block_align = channels * width
    byte_rate = rate * block_align
    fmt = struct.pack("<HHIIHH", 1, channels, rate, byte_rate, block_align, width * 8)
    riff = b"WAVE" + b"fmt " + struct.pack("<I", len(fmt)) + fmt
    riff += b"data" + struct.pack("<I", len(payload)) + payload
    blob = b"RIFF" + struct.pack("<I", len(riff)) + riff

    path = os.path.join(FIXTURE_DIR, name)
    with open(path, "wb") as handle:
        handle.write(blob)
    print("wrote %s (%d bytes)" % (path, len(blob)))


def sine16(rate, hz, count, peak):
    """A quantised sine. The round-half-away step keeps the result host-stable."""
    out = []
    for i in range(count):
        value = peak * math.sin(2.0 * math.pi * hz * i / rate)
        out.append(int(math.floor(value + 0.5)) if value >= 0 else -int(math.floor(-value + 0.5)))
    return out


def sweep16(rate, start_hz, end_hz, count, peak):
    """A linear frequency sweep, integrated so the phase stays continuous."""
    out = []
    phase = 0.0
    for i in range(count):
        hz = start_hz + (end_hz - start_hz) * i / float(count)
        phase += 2.0 * math.pi * hz / rate
        value = peak * math.sin(phase)
        out.append(int(math.floor(value + 0.5)) if value >= 0 else -int(math.floor(-value + 0.5)))
    return out


def main():
    os.makedirs(FIXTURE_DIR, exist_ok=True)

    # S_BeginRegistration registers sound/null.wav first in a FINAL_BUILD, and
    # every failed registration hands back its handle. Retail ships a short
    # silent file, so the fixture is one too.
    write_wav("null.wav", 22050, 2, 1, [0] * 128)

    # The house rate. Nothing resamples, so the mixer sees the file samples.
    write_wav("sine440.wav", 22050, 2, 1, sine16(22050, 440.0, 5512, 20000))

    # Half rate. ResampleSfx doubles the length at stepscale 0.5.
    write_wav("sweep11k.wav", 11025, 2, 1, sweep16(11025, 200.0, 3000.0, 2756, 18000))

    # Double rate. ResampleSfx halves the length at stepscale 2.
    write_wav("sine44k.wav", 44100, 2, 1, sine16(44100, 1000.0, 11025, 24000))

    # Eight bit. ResampleSfx takes the (sample - 128) << 8 branch.
    impulse8 = [128] * 512
    impulse8[0] = 255
    impulse8[1] = 0
    impulse8[256] = 200
    write_wav("impulse8.wav", 22050, 1, 1, impulse8)

    # Silence. The channel occupies a slot and paints nothing.
    write_wav("silence.wav", 22050, 2, 1, [0] * 2048)

    # A short ramp. The channel ends inside the first paint window.
    write_wav("ramp64.wav", 22050, 2, 1, [(i - 32) * 512 for i in range(64)])

    # Stereo. S_LoadSound rejects it and the handle falls back to the default
    # sound, which is the buzz S_DefaultSound builds.
    stereo = []
    for i in range(1024):
        stereo.append(int(10000 * math.sin(2.0 * math.pi * 300.0 * i / 22050)))
        stereo.append(int(-10000 * math.sin(2.0 * math.pi * 300.0 * i / 22050)))
    write_wav("stereo.wav", 22050, 2, 2, stereo)


if __name__ == "__main__":
    main()
