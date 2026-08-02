# snd-oracle — differential harness for the MP sound port

Compiles the **unmodified** Raven sound TUs standalone, drives them over
committed PCM fixtures with a scripted command list, and dumps the `dma_t` ring
plus the sound state. The Rust mixer port must reproduce the goldens byte for
byte.

The harness executes DEC-57.2 and closes the first half of wayfinder ticket
[#23](https://github.com/mheh/jka-rust/issues/23). It follows the §F recipe that
`tools/gp2-oracle` and `tools/ghoul2-server-oracle` established: compile the
oracle TU standalone against stub headers, dump canonical behaviour over
committed fixtures, and commit the goldens so `cargo test` needs no C++
toolchain.

## Usage

```sh
sh build.sh                 # build build/snd_dump
sh run.sh                   # run every scenario, diff against golden/
sh run.sh --regen           # regenerate golden/
python3 gen_fixtures.py     # regenerate fixtures/sound/*.wav
cargo test -p mp_engine_client --test snd_oracle_goldens
```

`build.sh` copies the oracle sources into `build/` so their relative `#include`s
resolve. `oracle/` is never edited.

## What compiles

| TU | Role |
| --- | --- |
| `snd_dma.cpp` | channels, spatialization, the frame driver, the sfx cache |
| `snd_mem.cpp` | WAV parsing and `ResampleSfx` |
| `snd_mix.cpp` | the paint chain and the ring transfer |
| `snd_music.cpp` | the dynamic-music state machine |
| `snd_ambient.cpp` | the ambient-set parser |
| `q_shared.c`, `q_math.c` | the shared string and math helpers, so name handling stays faithful |
| `GenericParser2.cpp` | the parser `snd_music` and `snd_ambient` read their sets with |

`snd_mp3.cpp` and `codemp/mp3code` are **out**. DEC-57.3 keeps the decoder
outside the byte gate, so MP3 content enters as decoded PCM fixtures. `host.cpp`
gives every `MP3_*` entry point an aborting body: a hit means a script fed the
harness an `.mp3` name.

## The dropped OpenAL and EAX arm

DEC-57.4 drops the OpenAL and EAX arm. Raven gates that arm on the `s_UseOpenAL`
cvar rather than a preprocessor symbol, so the TU still compiles and links it.
The harness stubs declare the whole API with aborting bodies
(`stubs/openal_al.h`, `stubs/openal_alc.h`, `stubs/eax_eax.h`,
`stubs/eax_eaxman.h`), and `s_UseOpenAL` stays at its `0` default. A golden can
therefore never contain output from the arm: reaching it aborts the run.

Raven writes those four includes with a Windows path separator
(`#include "openal\al.h"`, `snd_local.h:12-15`). Clang resolves the name
literally on macOS, so `build.sh` copies each stub to a build-tree file whose
name carries the backslash. The repo keeps the portable names.

## Build flags

```
-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math
-D__linux__ -DFINAL_BUILD -DNDEBUG -include build/inc/win_shim.h
```

- `-D__linux__` selects Raven's POSIX branch in `q_shared.h`. That branch is the
  little-endian one, which matches the shipped x86 build: `LittleShort` is a
  no-op and `BigShort` swaps. The `MACOS_X` branch has the pair the other way
  round, because it targeted PPC, and it would byte-swap every WAV sample.
- `-DFINAL_BUILD -DNDEBUG` models the retail release build. It matters:
  `S_BeginRegistration` registers `sound/null.wav` under `FINAL_BUILD` and calls
  `S_DefaultSound` under `_DEBUG` instead.
- `id386` resolves to 0 under `__linux__`, so `S_WriteLinearBlastStereo16`
  compiles as portable C. The shipped x86 build took the MMX assembly path. Both
  clamp identically (`packssdw` saturates the same way the C branch does), and
  OpenJK ships the C path.
- `stubs/win_shim.h` leads every TU with the MSVC names Raven calls: `LPCSTR`,
  `strlwr`, `strnicmp`, `timeGetTime`, `OutputDebugString`, and a two-`long`
  `min`. `timeGetTime` is the harness clock, never a wall clock.

## Normalisations on the copies

Three edits land on the `build/` copies only, in the `tools/icarus-oracle`
style. `oracle/` stays untouched.

1. `snd_dma.cpp:4572` — MSVC leaks `for (int i=...)` into the enclosing scope and
   a later loop reuses the name. The copy declares it. Site:
   `S_StartBackgroundTrack`, off every golden path.
2. `snd_dma.cpp:5187,5194` — the same leak with `iChannel`. The copy hoists the
   declaration. Site: `SND_FreeOldestSound`, off every golden path.
3. `snd_mem.cpp:98` — `(int)(data_p - 4)` is exact on the 32-bit ship and lossy
   under LP64. The copy widens the cast. The line sits inside `DumpChunks`, which
   only runs under `s_show`, and no golden prints an address.

## The harness seam

`host.cpp` supplies the engine seam. Three rules keep every golden run-twice
byte-identical: the clock only moves when the script says so, no address is ever
printed, and the file system reads from `fixtures/` and refuses every write.

The five `SNDDMA_*` functions model the retail DirectSound secondary buffer
(`oracle/codemp/win32/win_snd.cpp:12,183-250`): 65536 bytes, stereo, 16 bit, at
the rate `s_khz` picks. `SNDDMA_GetDMAPos` reports the cursor the **script** set
rather than a timed cursor, so the mix window is scripted. That is harness
policy, not oracle behaviour: DEC-57.1 dissolves the five functions into the cpal
wrapper, so the Rust side supplies its own device model and drives the same
scripted cursor.

`Com_Printf` and `Com_DPrintf` count their calls and drop the text. Raven's
messages carry file names and byte sizes that the port words differently, so the
goldens carry the counts instead. A count only means something when the stub
prints where Raven prints, so `FS_ReadFile` and `FS_FOpenFileRead` emit the
`Can't find %s` line that Raven's own `FS_FOpenFileRead` emits on a miss
(`oracle/codemp/qcommon/files.cpp:1387`).

`snd_oracle_host_init` registers `com_buildScript`, which Raven's engine creates
in `Com_Init`. `S_LoadSound_FileLoadAndNameAdjuster` dereferences it for every
name under `sound/chars`, so a null pointer there would crash the `lipsync` run.

## Scripted command schema

A scenario is a text file under `scenarios/`. Blank lines and `#` lines are
comments. One command per line, arguments separated by spaces. `<slot>` is a
harness slot index, 0 to 15, that `register` fills with the returned
`sfxHandle_t`.

| Command | Effect |
| --- | --- |
| `cvar <name> <value>` | Seeds a cvar before `init`, the way a config file would. |
| `init` | `S_Init` |
| `beginreg` | `S_BeginRegistration` |
| `register <slot> <path>` | `S_RegisterSound`, and prints the handle |
| `length <slot>` | `S_GetSampleLengthInMilliSeconds`, and prints it |
| `respatialize <ent> <x> <y> <z> <inwater>` | `S_Respatialize` with an identity axis |
| `respatializeaxis <ent> <x y z> <fwd> <right> <up> <inwater>` | `S_Respatialize` with a named axis |
| `entitypos <ent> <x> <y> <z>` | `S_UpdateEntityPosition` |
| `startsound <x> <y> <z> <ent> <chan> <slot>` | `S_StartSound` at a fixed origin |
| `startsoundent <ent> <chan> <slot>` | `S_StartSound` with a null origin, so the channel follows the entity |
| `startlocal <slot> <chan>` | `S_StartLocalSound` |
| `startlocalloop <slot>` | `S_StartLocalLoopingSound` |
| `startambient <x> <y> <z> <ent> <volume> <slot>` | `S_StartAmbientSound` |
| `ambientloop <x> <y> <z> <volume> <slot>` | `S_AddAmbientLoopingSound` |
| `clearloops` | `S_ClearLoopingSounds` |
| `addloop <ent> <x y z> <vx vy vz> <slot>` | `S_AddLoopingSound` |
| `stoploop <ent>` | `S_StopLoopingSound` |
| `mute <ent> <chan>` | `S_MuteSound` |
| `stopsounds` | `S_StopSounds` |
| `stopall` | `S_StopAllSounds` |
| `disable` | `S_DisableSounds` |
| `rawsamples <frames> <rate> <amplitude>` | `S_RawSamples` over a scripted stereo ramp |
| `advance <frames>` | The device consumed `frames` stereo frames. Moves the cursor and the clock together. |
| `update` | `S_Update` |
| `dumpstate <tag>` | Writes the state block |
| `dumpsfx <tag>` | Writes the sfx table with a digest per sound |
| `dumpring <tag>` | Writes the ring digests and keeps the ring copy for the `.bin` |
| `dumplipsync <tag>` | Writes `s_entityWavVol` and `s_entityWavVol_back` for entities 0 to 7 |
| `shutdown` | `S_Shutdown` |

`<chan>` is the integer `soundChannel_t` (`oracle/codemp/game/q_shared.h:1945-1961`):
0 `CHAN_AUTO`, 1 `CHAN_LOCAL`, 2 `CHAN_WEAPON`, 3 `CHAN_VOICE`,
4 `CHAN_VOICE_ATTEN`, 5 `CHAN_ITEM`, 6 `CHAN_BODY`, 7 `CHAN_AMBIENT`,
8 `CHAN_LOCAL_SOUND`, 9 `CHAN_ANNOUNCER`, 10 `CHAN_LESS_ATTEN`, 11 `CHAN_MENU1`,
12 `CHAN_VOICE_GLOBAL`, 13 `CHAN_MUSIC`.

**Frame order matters.** `S_Respatialize` is the call that recomputes every
channel volume and merges the loop list into channels, so a frame runs in the
client order: place the sources, `respatialize`, `advance`, `update`. A scenario
that skips `respatialize` keeps the volumes `S_StartSound` set, and its loops
never become channels.

## Fixture set

`gen_fixtures.py` writes every file from integer arithmetic, so a regenerated
fixture is byte-identical on any host.

| File | Format | What it exercises |
| --- | --- | --- |
| `sound/null.wav` | 22050 Hz, 16 bit, mono, 128 samples of silence | The file `S_BeginRegistration` registers first, and the handle every failed registration returns |
| `sound/sine440.wav` | 22050 Hz, 16 bit, mono, 5512 samples | The house rate. Nothing resamples. |
| `sound/sweep11k.wav` | 11025 Hz, 16 bit, mono, 2756 samples | `ResampleSfx` upsampling at stepscale 0.5 |
| `sound/sine44k.wav` | 44100 Hz, 16 bit, mono, 11025 samples | `ResampleSfx` downsampling at stepscale 2 |
| `sound/impulse8.wav` | 22050 Hz, 8 bit, mono, 512 samples | The `(sample - 128) << 8` branch, plus a full-scale impulse |
| `sound/silence.wav` | 22050 Hz, 16 bit, mono, 2048 samples | A channel that holds a slot and paints nothing |
| `sound/ramp64.wav` | 22050 Hz, 16 bit, mono, 64 samples | A sound that ends inside the first paint window |
| `sound/stereo.wav` | 22050 Hz, 16 bit, stereo | The stereo reject in `S_LoadSound_Actual` |
| `sound/chars/voice1.wav` | 22050 Hz, 16 bit, mono, 4408 samples | The lip-sync path, plus the `chars` language-pack branch of the loader. Four equal blocks step the amplitude, so each frame reports a different bucket |

`sound/missing.wav` has no fixture on purpose: `badfiles` registers it to drive
the not-found fallback.

## Scenarios and goldens

Each scenario writes two goldens: `golden/<name>.txt`, the state and ring
digests, and `golden/<name>.bin`, the final 65536-byte ring.

| Scenario | Covers |
| --- | --- |
| `basic` | The shortest chain: init, register, start, three paint frames |
| `spatialize` | `S_SpatializeOrigin` over four fixed sources and a listener that moves and turns |
| `resample` | Every fixture rate and width through `ResampleSfx`, with the resampled digests |
| `channels` | `S_PickChannel`, the entity and channel stomp, `S_MuteSound`, and the two stop doors |
| `loops` | `S_AddLoopingSound`, `S_AddAmbientLoopingSound`, `S_StopLoopingSound` across four frames |
| `rawstream` | `S_RawSamples` alone, mixed with a channel, and running dry |
| `ringwrap` | Ten frames that carry the ring past its 16384-frame wrap twice |
| `khz11`, `khz44` | `S_Init` at the other two `s_khz` rates |
| `badfiles` | A missing file, a stereo file, and a repeated registration |
| `lipsync` | `S_DoLipSynchs` and `S_CheckAmplitude` over a `sound/chars` voice line, on all three voice channels |

## How gh#24 consumes the goldens

Ticket [#24](https://github.com/mheh/jka-rust/issues/24) landed the Rust paint
chain. `crates/mp/engine/core/tests/snd_oracle_parity.rs` reads the same scenario
scripts, drives the Rust sound state with the same scripted clock and DMA cursor,
and compares:

1. the text dump, line for line, against `golden/<name>.txt`, and
2. the ring bytes against `golden/<name>.bin`.

The dump format is defined by `main.cpp`, so the Rust dumper mirrors it exactly,
the way `tests/gp2_parity.rs` mirrors `tools/gp2-oracle/main.cpp`. The rig writes
a temporary game tree from these fixtures, so it needs no retail content and no
C++ toolchain, and it runs in CI. Set `SND_ORACLE_DUMP=1` to write each Rust dump
to the temp directory for a manual diff.

`crates/mp/engine/client/tests/snd_oracle_goldens.rs` gates the golden set
itself: it checks that both goldens exist for every scenario, that the ring size
and block layout are right, that every run paints, and that each `.bin` matches
the digest its text dump records.

## Uncovered

The harness makes no claim on these, and no golden hides them:

- **The MP3 decoder.** DEC-57.3 puts it outside the byte gate. It gets pinned
  decode fixtures of its own.
- **The background music track.** `S_StartBackgroundTrack` needs the streamed
  file seam and the MP3 path, so `snd_music.cpp` compiles and links but nothing
  drives it. The `Sys_*StreamedFile` stubs abort if a script reaches them.
- **The ambient-set parser.** `AS_Init` runs during `S_Init` and finds no set
  file, so the parser itself is unexercised.
- **The OpenAL and EAX arm.** Dropped by DEC-57.4.
