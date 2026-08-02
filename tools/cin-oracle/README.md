# cin-oracle: differential harness for the MP RoQ cinematic decoder

Compiles the **unmodified** Raven `cl_cin.cpp` standalone, drives its decode core over synthetic RoQ streams, and dumps the colour tables, the vector-quantizer codebooks, the quad cel map, the motion table, the decoded surface and the RLL audio output. The Rust port in `crates/mp/engine/client/src/cl_cin.rs` must reproduce the goldens byte for byte.

The harness executes DEC-55.3 and closes wayfinder ticket [#28](https://github.com/mheh/jka-rust/issues/28). It follows the §F recipe that `tools/gp2-oracle`, `tools/ghoul2-server-oracle` and `tools/snd-oracle` established: compile the oracle TU standalone against stub headers, dump canonical behaviour over committed fixtures, and commit the goldens so `cargo test` needs no C++ toolchain.

## Usage

```sh
sh build.sh                 # build build/cin_dump
sh run.sh                   # run every scenario, diff against golden/
sh run.sh --regen           # regenerate golden/
python3 gen_fixtures.py     # regenerate fixtures/*.roq
cargo test -p mp_engine_client --test cin_oracle_goldens
```

`build.sh` copies the oracle sources into `build/` so their relative `#include`s resolve. `oracle/` is never edited.

## What compiles

`main.cpp` `#include`s the copied `cl_cin.cpp`, so the driver and the oracle text form one translation unit. Every gated function is `static` in that file, and the include is what reaches them. `host.cpp` supplies the engine seam.

`cl_cin.cpp` pulls in `client.h` and `snd_local.h`, which include the OpenAL and EAX headers with a Windows path separator. `build.sh` copies the same four stubs `tools/snd-oracle` uses to build-tree file names that carry the backslash. Nothing in the gate reaches that arm.

## What the gate proves, and what it does not

Proven byte for byte, over the scenarios below:

| Function | Oracle source |
| --- | --- |
| `RllSetupTable` | `cl_cin.cpp:159-167` |
| `RllDecodeMonoToMono` | `cl_cin.cpp:184-198` |
| `RllDecodeMonoToStereo` | `cl_cin.cpp:215-231` |
| `RllDecodeStereoToStereo` | `cl_cin.cpp:247-269` |
| `RllDecodeStereoToMono` | `cl_cin.cpp:285-305` |
| `move8_32`, `move4_32`, `blit8_32`, `blit4_32`, `blit2_32` | `cl_cin.cpp:315-446` |
| `blitVQQuad32fs` | `cl_cin.cpp:456-532` |
| `ROQ_GenYUVTables` | `cl_cin.cpp:542-560` |
| `yuv_to_rgb24` | `cl_cin.cpp:625-637` |
| `decodeCodeBook` and the `VQ2TO4` macro | `cl_cin.cpp:562-583,648-693` |
| `recurseQuad` | `cl_cin.cpp:703-733` |
| `setupQuad` | `cl_cin.cpp:744-778` |
| `readQuadInfo` | `cl_cin.cpp:788-827` |
| `RoQPrepMcomp` | `cl_cin.cpp:837-856` |
| `initRoQ` | `cl_cin.cpp:866-874` |

**`RoQInterrupt` is outside the byte gate.** Its body is `Sys_StreamedRead`, console printing, the looping and EOF control, the `inMemory` `goto redump` re-entry, and the `S_RawSamples` hand-off. Driving it needs the whole engine seam on the Rust side. Both drivers replicate its chunk-dispatch switch over an in-memory fixture instead, so they stay symmetric, and `RoQInterrupt`'s own loop control is covered by code audit rather than by the byte gate. The same holds for `RoQ_init`: both drivers replicate its header parse.

**`readQuadInfo` is gated on the oracle side only.** The Rust `readQuadInfo` takes a `&mut Common` for one `Com_Printf` on the `maxTextureSize <= 256` arm, and no test can build a `Common`. The Rust driver writes the same fields and dumps them. Both drivers hold `maxTextureSize` at 2048, so the clamp and its print never run and the rest of the body is exactly those field writes.

## Build flags

```
-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math
-D__linux__ -DFINAL_BUILD -DNDEBUG -include build/inc/win_shim.h
```

- `-D__linux__` selects Raven's POSIX branch in `q_shared.h`, the little-endian branch that matches the shipped x86 build. It also selects the PC arms of `cl_cin.cpp` itself: the `MACOS_X` branch has a second `yuv_to_rgb24` body and the `drawX = 256; drawX = 256;` typo block, neither of which the PC build compiles.
- `LittleLong` resolves to an empty macro on that branch, so `yuv_to_rgb24` returns the packed word unswapped.
- `stubs/win_shim.h` leads every translation unit with the MSVC names `client.h` and `snd_local.h` call: `LPCSTR`, `strlwr`, `strnicmp`, `timeGetTime`, `OutputDebugString`, and a two-`long` `min`. `timeGetTime` returns a constant, never a wall clock.

## Pointer width and determinism

Raven shipped a 32-bit build. Three places in `cl_cin.cpp` do arithmetic whose result depends on that, so the harness normalises the **copies** in `build/` and the dump derives every value so it cannot carry an address or a word width. `oracle/` stays untouched.

1. `cl_cin.cpp:806-807` casts `cin.linbuf` to `unsigned int`. That is exact on the 32-bit ship and a hard error under LP64, so the copy routes the cast through `size_t`. The surrounding algebra cancels the address at either width: `t[0]` lands on `screenDelta` and `t[1]` on `-screenDelta`. The dump narrows both to `(int)` anyway, so the golden is width-independent by construction.
2. `cl_cin.cpp:515,523` add `cin.mcomp[...]`, an `unsigned int` holding a signed delta, to a `byte *`. On the 32-bit ship the addition wraps and walks backwards. An LP64 pointer zero-extends the same value and walks off the surface, so the copy casts to `int` first. The Rust port spells the identical `as i32 as isize` step. Every `mcomp` entry dumps as `(int)`, the 32-bit reinterpretation the pointer arithmetic means.
3. `cl_cin.cpp:1197` calls `abs` on an `unsigned int` difference. MSVC resolves that to `abs(int)`; libc++ finds the float overloads too and calls it ambiguous. The copy spells the MSVC choice. The site is `CIN_RunCinematic`, outside the gate and off every golden path.

`cin.qStatus` holds raw pointers. The dump never prints one: each entry becomes a signed byte offset from `cin.linbuf`, and an end-of-quad null becomes -1. Every digest is FNV-1a over value-derived little-endian bytes rather than over raw memory, so no padding or word width reaches a golden.

## Fixtures

`gen_fixtures.py` writes every stream from integer arithmetic, so a regenerated fixture is byte-identical on any host. No retail game content enters the repo: there is no pak file, no retail `.roq` movie and no retail audio.

The container is hand-built to the layout `RoQ_init` and `RoQInterrupt` read (`cl_cin.cpp:1069-1077,1026-1030`): an 8-byte file header of `u16` id `0x1084`, `u32` size and `u16` framerate, then chunks of an 8-byte header (`u16` id, `u32` size, two argument bytes) followed by the payload. Raven reads three of the four size bytes and takes the arguments as `roq_flags = arg0 + arg1*256`, `roqF0 = (char)arg1`, `roqF1 = (char)arg0`. Every fixture ends with an all-zero terminator chunk, which both drivers use as the stop mark.

## Scenarios

| Scenario | Fixture | Covers |
| --- | --- | --- |
| `quadinfo` | 64x64 quad info alone | `readQuadInfo`, `setupQuad`, `recurseQuad` at an aligned size |
| `quadinfo_ragged` | 40x24 quad info alone | `recurseQuad`'s bounds rejection, where neither edge is a multiple of 16 |
| `codebook` | quad info plus a 256-entry codebook | `decodeCodeBook` with `roq_flags == 0`, the `VQ2TO4` expansion, and the whole YUV table chain |
| `codebook_partial` | two codebooks, `0x8040` then `0x0040` | both non-zero `roq_flags` branches, including the `two == 0` fallback to 256 |
| `vq_frames` | 64x64, codebook, three VQ frames | every arm of `blitVQQuad32fs`: the `0x8000` 8x8 vq code, the `0xc000` drop into all four sub-arms, the top-level `0x4000` motion, and both skips. Frame 1 carries `roqF0 = -4`, frame 2 `roqF1 = -6`, so the `(char)` sign extension of the header bytes is on the golden path. Frames alternate banks, so `t[0]`/`t[1]` and both `qStatus` banks run. |
| `vq_nonsquare` | 64x16, codebook, two VQ frames | `RoQPrepMcomp`'s `xsize == ysize*4` branch, which doubles both strides |
| `sound_mono` | three `ZA_SOUND_MONO` chunks | `RllDecodeMonoToStereo` over two full 0..255 delta sweeps per chunk, including the sign-table wrap, at three predictor flags |
| `sound_stereo` | three `ZA_SOUND_STEREO` chunks | `RllDecodeStereoToStereo`, whose flag splits into two channel predictors |
| `rll_direct` | none, driver only | all four `RllDecode*` entry points over a deterministic byte sweep, at five flags and both `signedOutput` values. `RoQInterrupt` never reaches `RllDecodeMonoToMono` or `RllDecodeStereoToMono`, so this is their only cover. |

## Golden format

One fact per line, stable order, no address and no timestamp. Every scenario opens with the `TABLES` block (the five `ROQ_*_tab` digests and samples, the `sqrTable`, and a `yuv_to_rgb24` sweep over a 6x6x6 grid), then walks its chunks. A `FRAME` block carries an FNV-1a digest of the whole live `linbuf` area, one digest per half, and an 8x8 grid of sampled RGBA texels so a mismatch localises. A `QUADS` block carries the `qStatus` digests as linbuf offsets plus the first entries and the terminator. `MCOMP`, `CODEBOOK` and `AUDIO` blocks follow the same digest-plus-samples shape.

## Uncovered

The harness makes no claim on these, and no golden hides them:

- **`RoQInterrupt`'s loop control**, as stated above. Its `ROQ_PACKET` and `ROQ_QUAD_HANG` arms and the `inMemory` `goto redump` re-entry drive `RoQFrameSize` and the stream cursor, so they belong with the file seam, not the decode core.
- **`ROQ_QUAD_JPEG`**, which Raven leaves empty, and `ROQ_QUAD` and `ROQ_PACKET`, which no shipped stream carries into the decode core.
- **The playback shell**: `CIN_PlayCinematic`, `CIN_RunCinematic`, `CIN_DrawCinematic`, `CIN_UploadCinematic`, `RoQShutdown` and `RoQReset`. `host.cpp` gives every symbol they reach an aborting body, so a hit stops the run and names the function.
- **Odd texel motion displacements.** `RoQPrepMcomp` builds `temp = (x+xoff-8)*4`, so an odd horizontal motion vector puts the source rows on a 4-byte boundary and the blitters then read a `double` off its natural alignment. That is UB in C, which x86 and ARM64 both tolerate. Porting-rules §19 keeps UB out of the shared fixtures, so `gen_fixtures.py` rounds every displacement to an even texel count. Flip the parity clamp in `motion_byte` to reproduce the case.
- **Streams wider or taller than 512x512.** The port bounds `recurseQuad` and `setupQuad` against `linbuf` and `qStatus`, which Raven does not. No fixture reaches that bound, so the two sides never diverge on the golden path.
