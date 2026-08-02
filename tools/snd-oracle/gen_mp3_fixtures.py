#!/usr/bin/env python3
"""Write the synthetic MP3 fixtures the decoder pin reads.

DEC-57.3 keeps the MP3 decoder outside the byte gate, so it gets pinned decode
fixtures of its own. Every byte here comes from integer arithmetic, so a
regenerated fixture is identical on any host, and no retail audio is involved.

    python3 gen_mp3_fixtures.py

The `.mp3` files land in `fixtures/mp3/`. The expected PCM beside them is minted
by `cargo test -p mp_engine_client --test mp3_decode_fixtures` with
`MP3_FIXTURES_REGEN=1`, because the PCM is what our own decoder answers.

Every frame is MPEG-1 Layer III, 44100 Hz, 128 kbit/s, so the frame is 417 bytes
and carries 1152 samples per channel. The bodies are crafted by hand:

  silence  side info all zero, so `part2_3_length` is zero and no spectral data
           follows. The decode is exact silence.
  tone     valid side info with `big_values` zero, `global_gain` 180, and the
           main data a repeating `0x55`. The count1 region then decodes to a
           steady deterministic signal that peaks near a third of full scale.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "fixtures", "mp3")

FRAME_BYTES = 417
SIDE_INFO_MONO = 17
SIDE_INFO_STEREO = 32


def header(stereo):
    """The four-byte frame header: MPEG-1, Layer III, no CRC, 128 kbit/s, 44100 Hz."""
    # bitrate index 9 is 128 kbit/s for MPEG-1 Layer III, rate index 0 is 44100.
    return bytes([0xFF, 0xFB, 0x90, 0x00 if stereo else 0xC0])


class BitWriter:
    def __init__(self):
        self.bits = []

    def write(self, value, count):
        for i in range(count - 1, -1, -1):
            self.bits.append((value >> i) & 1)

    def to_bytes(self, size):
        out = bytearray()
        for i in range(0, len(self.bits), 8):
            chunk = self.bits[i:i + 8]
            while len(chunk) < 8:
                chunk.append(0)
            value = 0
            for bit in chunk:
                value = (value << 1) | bit
            out.append(value)
        while len(out) < size:
            out.append(0)
        return bytes(out[:size])


def side_info_mono(part2_3_length, global_gain):
    """Layer III mono side info: two granules, no window switching, no scalefactors."""
    bw = BitWriter()
    bw.write(0, 9)   # main_data_begin
    bw.write(0, 5)   # private_bits
    bw.write(0, 4)   # scfsi
    for _ in range(2):
        bw.write(part2_3_length, 12)
        bw.write(0, 9)             # big_values
        bw.write(global_gain, 8)
        bw.write(0, 4)             # scalefac_compress, so no scalefactor bits
        bw.write(0, 1)             # window_switching_flag
        bw.write(0, 5)             # table_select[0]
        bw.write(0, 5)             # table_select[1]
        bw.write(0, 5)             # table_select[2]
        bw.write(0, 4)             # region0_count
        bw.write(0, 3)             # region1_count
        bw.write(0, 1)             # preflag
        bw.write(0, 1)             # scalefac_scale
        bw.write(0, 1)             # count1table_select
    return bw.to_bytes(SIDE_INFO_MONO)


def silence_stream(frames, stereo):
    side = SIDE_INFO_STEREO if stereo else SIDE_INFO_MONO
    body = bytes(FRAME_BYTES - 4 - side)
    return (header(stereo) + bytes(side) + body) * frames


def tone_stream(frames):
    side = side_info_mono(600, 180)
    body = bytes([0x55]) * (FRAME_BYTES - 4 - SIDE_INFO_MONO)
    return (header(False) + side + body) * frames


def id3v1_tag(title, uncompressed, max_vol):
    """Raven's tagger writes `#UNCOMP %d` in the album field and `#MAXVOL %g` in
    the comment field. `MP3_ReadSpecialTagInfo` reads exactly those two.
    Source: `oracle/codemp/client/snd_mp3.cpp:158-218`"""

    def field(text, size):
        raw = text.encode("ascii")[:size]
        return raw + bytes(size - len(raw))

    return (
        b"TAG"
        + field(title, 30)
        + field("Raven Software", 30)
        + field("#UNCOMP %d" % uncompressed, 30)
        + field("2000", 4)
        + field("#MAXVOL %g" % max_vol, 28)
        + bytes([0, 0, 0])
    )


def write(name, data):
    path = os.path.join(OUT, name)
    with open(path, "wb") as f:
        f.write(data)
    print("%-28s %6d bytes" % (name, len(data)))


def main():
    os.makedirs(OUT, exist_ok=True)

    write("silence_stereo.mp3", silence_stream(8, True))
    write("silence_mono.mp3", silence_stream(8, False))

    tone = tone_stream(8)
    write("tone_mono.mp3", tone)
    write("tone_tagged.mp3", tone + id3v1_tag("tone_mono", 18432, 4768))

    # A stream that stops in the middle of its second frame, so the walker runs
    # out of source with a partial frame in hand.
    write("truncated.mp3", tone[: FRAME_BYTES + 200])

    # No sync word anywhere, so validation fails before any decode.
    write("notmp3.mp3", bytes(range(0, 256)) * 4)
    return 0


if __name__ == "__main__":
    sys.exit(main())
