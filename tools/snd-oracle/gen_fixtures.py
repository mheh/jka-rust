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

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE_ROOT = os.path.join(HERE, "fixtures")
FIXTURE_DIR = os.path.join(FIXTURE_ROOT, "sound")


def write_file(relpath, blob):
    """Write one fixture under fixtures/, creating the directory it needs."""
    path = os.path.join(FIXTURE_ROOT, relpath)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as handle:
        handle.write(blob)
    print("wrote %s (%d bytes)" % (path, len(blob)))


def wav_bytes(rate, width, channels, frames):
    """Build a RIFF/WAVE PCM image that Raven's GetWavinfo accepts.

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
    return b"RIFF" + struct.pack("<I", len(riff)) + riff


def write_wav(name, rate, width, channels, frames):
    """Write one WAV under fixtures/sound/."""
    write_file(os.path.join("sound", name), wav_bytes(rate, width, channels, frames))


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

    # A voice line for the lip-sync path. It sits under sound/chars, which is the
    # directory S_LoadSound_FileLoadAndNameAdjuster tests for a language pack.
    # Four equal blocks step the amplitude down and back up, so S_CheckAmplitude
    # reports a different bucket in each paint frame.
    steps = [30000, 3000, 300, 6000]
    voice = []
    for level in steps:
        for i in range(1102):
            voice.append(level if (i // 2) % 2 == 0 else -level)
    write_wav(os.path.join("chars", "voice1.wav"), 22050, 2, 1, voice)

    # Stereo. S_LoadSound rejects it and the handle falls back to the default
    # sound, which is the buzz S_DefaultSound builds.
    stereo = []
    for i in range(1024):
        stereo.append(int(10000 * math.sin(2.0 * math.pi * 300.0 * i / 22050)))
        stereo.append(int(-10000 * math.sin(2.0 * math.pi * 300.0 * i / 22050)))
    write_wav("stereo.wav", 22050, 2, 2, stereo)

    write_music_fixtures()
    write_ambient_fixtures()


def write_music_fixtures():
    """The background-music tree: one streamed WAV track and one dynamic set."""
    # The streamed track. S_StartBackgroundTrack_Actual reads the header off
    # this and Sys_StreamedRead feeds the raw ring from it. 22 kHz stereo is what
    # Raven warns about not getting, so the fixture is one.
    track = []
    for i in range(4096):
        value = int(12000 * math.sin(2.0 * math.pi * 220.0 * i / 22050))
        track.append(value)
        track.append(-value)
    write_file(os.path.join("music", "track.wav"), wav_bytes(22050, 2, 2, track))

    # A second, shorter track, so the loop hand-over is observable.
    loop = []
    for i in range(1024):
        value = int(9000 * math.sin(2.0 * math.pi * 660.0 * i / 22050))
        loop.append(value)
        loop.append(-value)
    write_file(os.path.join("music", "loop.wav"), wav_bytes(22050, 2, 2, loop))

    # The dynamic-music description. `Music_ParseLeveldata` checks that every
    # file it names exists, so each one gets a one-frame MP3 beside it. Nothing
    # in the covered scenarios decodes them (DEC-57.3).
    dms = """musicfiles
{
	explore_piece
	{
		entry
		{
			marker0		0.000
			marker1		4.000
		}
		exit
		{
			nextfile	explore_tr0
			time0		8.000
		}
	}
	action_piece
	{
		entry
		{
			marker0		0.000
		}
		exit
		{
			nextfile	action_tr0
			nextmark	marker1
			time0		6.000
			time1		12.000
		}
	}
	boss_piece
	{
	}
}

levelmusic
{
	testmap
	{
		explore		explore_piece
		action		action_piece
		boss		boss_piece
	}
	usesmap
	{
		uses		testmap
	}
}
"""
    write_file(os.path.join("ext_data", "dms.dat"), dms.encode("ascii"))

    # One MPEG-1 Layer III silent frame, which is enough for S_FileExists.
    frame = bytes([0xFF, 0xFB, 0x90, 0x00]) + bytes(417 - 4)
    for name in ["explore_piece", "action_piece", "boss_piece", "explore_tr0", "action_tr0"]:
        write_file(os.path.join("music", "testmap", name + ".mp3"), frame)
    write_file(os.path.join("music", "death_music.mp3"), frame)


def write_ambient_fixtures():
    """The ambient-set file and the waves it names."""
    # Four short waves the sets pick between, plus one looping bed.
    for i, peak in enumerate([8000, 12000, 16000, 20000]):
        wave = [peak if (n // 8) % 2 == 0 else -peak for n in range(512)]
        write_wav(os.path.join("amb", "sub%d.wav" % (i + 1)), 22050, 2, 1, wave)
    bed = [4000 if (n // 32) % 2 == 0 else -4000 for n in range(2048)]
    write_wav(os.path.join("amb", "bed.wav"), 22050, 2, 1, bed)

    # `AS_ParseHeader` demands the type line first. Only a set the precache list
    # names is kept, so the file carries one the scenario never asks for.
    #
    # The file is written with CRLF, which is what the shipped `sound.txt` has,
    # and `AS_GetSubWaves` depends on it: after the last wave of a line it steps
    # one character past the name, and only a two-character line ending leaves
    # the cursor on a newline for the break test. Under LF the parser walks
    # straight into the next line and reads its keyword as a wave name.
    text = """type ambientSet

generalSet cave
timeBetweenWaves 2 4
subWaves amb sub1 sub2 sub3
loopedWave amb/bed
volRange 100 200

generalSet unused
timeBetweenWaves 1 1
subWaves amb sub4

localSet vent
timeBetweenWaves 1 3
subWaves amb sub2 sub4
radius 400
volRange 50 150

bmodelSet door
subWaves amb sub1 sub2 sub3 sub4
"""
    write_file(
        os.path.join("sound", "sound.txt"),
        text.replace("\n", "\r\n").encode("ascii"),
    )


if __name__ == "__main__":
    main()
