# fx-oracle - differential harness for the MP FX port

Compiles the **unmodified** Raven FX translation units standalone, drives them
over synthetic `.efx` fixtures with a scripted clock, a pinned RNG seed and
scripted trace, point-contents and bolt replies, and captures every outbound
emission as a text stream. The Rust FX port must reproduce the goldens byte for
byte.

The harness executes DEC-61.5 and is the FX port's one behavioural gate: the
demo referee masks `CG_FX_ADDPRIMITIVE`, so no other test sees FX output. It
follows the §F recipe that `tools/gp2-oracle` and `tools/snd-oracle`
established: compile the oracle TU standalone against stub headers, dump
canonical behaviour over committed fixtures, and commit the goldens so
`cargo test` needs no C++ toolchain.

## Usage

```sh
sh build.sh                 # build build/fx_dump
sh run.sh                   # run every scenario, diff against golden/
sh run.sh --regen           # regenerate golden/
```

`build.sh` copies the oracle sources into `build/` so their relative
`#include`s resolve. `oracle/` is never edited.

## What compiles

| TU | Role |
| --- | --- |
| `FxSystem.cpp` | the `SFxHelper` seam wrapper, the FX clock, the bolt lookup |
| `FxScheduler.cpp` | effect registration, the schedule, `CreateEffect`, the 2D list |
| `FxPrimitives.cpp` | the thirteen primitive classes, their update and draw chains |
| `FxTemplate.cpp` | the `.efx` key parsers |
| `FxUtil.cpp` | the live pool, `FX_Add`, the thirteen `FX_Add*` constructors |
| `FXExport.cpp` | the cgame-facing entry points |
| `GenericParser2.cpp` | the `.efx` text parser |
| `q_shared.c`, `q_math.c` | `flrand`, `irand`, `COM_StripExtension`, the vector math |

## Stubbed headers

Four oracle headers drag a whole subsystem into every FX TU, so the harness
replaces them. Each stub keeps the oracle signature of everything it declares.

| Stub | Replaces | Why |
| --- | --- | --- |
| `stubs/client.h` | `codemp/client/client.h` | the real header declares the snapshot ring, the key state, the console, both VMs and the sound cache. FX reaches eight names. |
| `stubs/exe_headers.h` | `codemp/qcommon/exe_headers.h` | it includes `qcommon.h`, which is the netchan, the VM loader, the file system and the zone. FX reaches six names plus the zone pair GP2 needs. |
| `stubs/G2_local.h` | `codemp/ghoul2/G2_local.h` plus the `CGhoul2Info_v` wrapper in `ghoul2_shared.h` | FX uses `mItem`, the int constructor, the int assignment, `kill`, `IsValid` and one `G2API_*` call. |
| `stubs/win_shim.h` | the MSVC forced include | supplies `strlwr`, `strnicmp`, `LPCSTR` and `OutputDebugString`. |

Everything else comes straight from `oracle/`: the five FX headers, `G2.h`,
`q_shared.h`, `tr_types.h`, `cg_public.h`, `GenericParser2.h` and the small
headers those pull in.

## Build flags

```
-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math
-D__linux__ -DFINAL_BUILD -DNDEBUG -include build/inc/win_shim.h
```

- `-ffp-contract=off` is load-bearing. An FMA anywhere in the update chain would
  change the float bits every golden carries.
- `-D__linux__` selects Raven's POSIX branch in `q_shared.h`, which is the
  little-endian branch the shipped x86 build used.
- `-DFINAL_BUILD -DNDEBUG` models the retail release build. It matters:
  `FINAL_BUILD` compiles out the `fx_debug 2` effect-name print
  (`FxScheduler.cpp:851-856`), the "FX system out of effects" print
  (`FxUtil.cpp:155-157`), the `StopEffect` unregistered-effect early return
  (`FxScheduler.cpp:96-102`) and the `PlayEffect(file, ...)` unregistered guard
  (`FxScheduler.cpp:760-766`). `NDEBUG` drops the `assert(0)` in the
  `CreateEffect` default arm and the one in `CParticle::UpdateOrigin`.
  DEC-63.3 rules this posture and the `unregistered` scenario pins the two
  guard sites: both stops are silent and the axis play falls through to the
  id overload's invalid-id print. The port takes the same retail leg.

## Normalisations on the copies

Five edits land on the `build/` copies only, in the `tools/snd-oracle` style.
`oracle/` stays untouched. The six FX `.cpp` TUs carry two of them; the rest are
headers and the shared math file.

1. **`FxPrimitives.cpp:423-451` - `VectorToInt`.** Raven writes this as an MSVC
   32-bit x87 `_asm` block. Clang compiles it on no target we have. The copy
   gets a C body that is bit-identical to the assembly:

   ```c
   int r = (int)nearbyintf(vec[0]) & 0xff;
   int g = (int)nearbyintf(vec[1]) & 0xff;
   int b = (int)nearbyintf(vec[2]) & 0xff;
   retval = (int)(0xff000000u | ((unsigned)b << 16) | ((unsigned)g << 8) | (unsigned)r);
   ```

   The derivation. The assembly pushes `vec[0]`, `vec[1]`, `vec[2]` onto the x87
   stack, so the first `fistp` pops `vec[2]`. It loads `eax` with `0xff00`, sets
   `al` to the low byte of `int(vec[2])`, shifts left by 16, sets `ah` to the low
   byte of `int(vec[1])`, then sets `al` to the low byte of `int(vec[0])`. The
   word is therefore `0xff000000 | b2 << 16 | b1 << 8 | b0`. `fistp` rounds under
   the FPU control word, which is round-half-to-even by default, so the C body
   uses `nearbyintf` and not a C truncating cast. The caller stores the word into
   `mRefEnt.shaderRGBA` on a little-endian target, which gives
   `{ r, g, b, 0xff }`. Every input reaches `VectorToInt` clamped to `[0, 255]`
   (`FxPrimitives.cpp:527-529`), so the out-of-range `fistp` indefinite value is
   unreachable. The Rust port mirrors this with `f32::round_ties_even()`.

2. **`q_math.c:1432` - `holdrand`.** Raven declares the LCG state as
   `unsigned long`, which is 32 bit on the ship and 64 bit under LP64. The copy
   makes it `unsigned int`. Without this every `flrand` draw diverges after the
   first multiply. `tools/snd-oracle` and `tools/rmg-oracle` patch the same line.

3. **`FxSystem.h:218` and `FxPrimitives.h:310,315` - extra qualification.** MSVC
   accepts `SFxHelper::GetOriginAxisFromBolt` and `CParticle::CParticle` as
   in-class declarations. No conforming compiler does. The copies drop the
   redundant class prefix and change nothing else.

4. **`FxTemplate.cpp:2335` - the primitive name copy.** `strcpy( mName, val )`
   writes an unbounded name into a 32-byte field. The copy uses `Q_strncpyz` with
   the field size. Every fixture keeps its names short, so no golden crosses the
   bound. The edit exists so a fixture typo cannot corrupt the neighbouring
   template rather than to change any covered behaviour.

## The harness seam

`host.cpp` supplies the engine seam. Four rules keep every golden run-twice
byte-identical:

1. **No wall clock.** The FX clock only moves when a scenario says `advance`.
2. **No pointer, address or size ever enters a record.**
3. **Every float prints as its raw IEEE-754 bit pattern**, eight lowercase hex
   digits, never `%f`. Ints print as decimal and bytes print as decimal.
4. **Every engine answer comes from a scripted queue**, never from a simulation.

### Scripted queues

Each queue is FIFO, and its last entry repeats forever once the queue drains. An
empty queue answers with a miss:

| Queue | Miss reply |
| --- | --- |
| `trace` | `fraction` 1, `endpos` equal to the requested end, normal `0 0 1`, `entityNum` `ENTITYNUM_NONE` |
| `pointcontents` | `0` |
| `bolt` | the bolt does not exist |
| `lerporigin` | `0 0 0` |

### The media registry

`RegisterShader`, `RegisterModel` and `RegisterSound` each keep an ordered name
table: a new name gets `index + 1`, a repeat gets the stored handle. Every call
prints its record, repeats included, so a golden carries the registration order
the effect files drive. The three tables are independent, so a shader and a
sound can both hold handle 1.

### The file system

`FS_FOpenFileByMode` resolves under `fixtures/` and refuses every write.
`CFxScheduler::RegisterEffect` prepends `effects/` to any name that does not
already start with it (`FxScheduler.cpp:295-302`), so the resolver strips that
prefix back off and the fixture tree stays flat: a scenario says
`register particle` and the harness opens `fixtures/particle.efx`.

### The `SOUND` volume and radius

`SFxHelper::PlaySound` drops its volume and radius arguments before it calls
`S_StartSound` (`FxSystem.h:91-95`). Nothing reaches the sound seam with them.
The harness declares `S_StartSound` with two defaulted trailing parameters, so
the record carries `volume -1 radius -1` and the loss is visible rather than
silent. A `SOUND` record always shows `entnum 1023` and `entchannel 0` for the
same reason: the wrapper hardcodes `ENTITYNUM_NONE` and `CHAN_AUTO`.

### Reading private scheduler state

`dumpstate` reads `mNextFree2DEffect` and `dumptemplate` reads
`mEffectTemplates` and the ordered handle list inside `CMediaHandles`. All three
are private and the oracle offers no ordered read accessor: `CMediaHandles::
GetHandle` picks at random. `main.cpp` relaxes the access with
`#define private public` around the `FxScheduler.h` include. The relaxation runs
in the driver only, changes no oracle file, and changes no layout, because clang
lays fields out in declaration order whatever their access. Every standard
header lands ahead of the relaxation so it cannot reach the standard library.

## Scenario language

A scenario is a text file under `scenarios/`. Blank lines and `#` lines are
comments. One command per line, tokens separated by whitespace. `<v3>` is three
floats. Every scenario starts with `seed` and ends with `dumpstate` and `free`.

| Command | Effect |
| --- | --- |
| `seed <uint>` | `Rand_Init(seed)`. Always the first command. |
| `refdef <v3 vieworg> <v3 viewangles> <v3 axis0> <v3 axis1> <v3 axis2> <fov_x> <fov_y>` | fills the view the FX system culls and projects against |
| `cvar <name> <value>` | sets `fx_debug`, `fx_countScale` or `fx_nearCull` |
| `init` | `FX_InitSystem(&refdef)` |
| `register <path>` | `FX_RegisterEffect(path)`, and prints `REGISTER <path> -> <handle>` |
| `dumptemplate <handle>` | prints the whole parsed `SEffectTemplate` |
| `trace <fraction> <v3 endpos> <v3 normal> <startsolid> <allsolid> <surfaceFlags> <entityNum>` | pushes one scripted trace reply |
| `pointcontents <int>` | pushes one scripted `CG_POINT_CONTENTS` reply |
| `bolt <0\|1> <v3 origin> <v3 axis0> <v3 axis1> <v3 axis2>` | pushes one scripted bolt reply |
| `lerporigin <v3>` | pushes one scripted `CG_GET_LERP_ORIGIN` reply |
| `playid <id> <v3 org> <v3 fwd> <vol> <rad> <portal>` | `FX_PlayEffectID` |
| `play <path> <v3 org> <v3 fwd> <vol> <rad>` | `FX_PlayEffect` |
| `playbolted <id> <v3 org> <boltInfo> <iGhoul2> <iLoopTime> <isRelative>` | `FX_PlayBoltedEffectID` |
| `playentity <id> <v3 org> <v3 axis0> <v3 axis1> <v3 axis2> <boltInfo> <entNum> <vol> <rad>` | `FX_PlayEntityEffectID` |
| `stop <path> <boltInfo> <portal>` | `CFxScheduler::StopEffect` |
| `addline <v3 start> <v3 end> <size1> <size2> <sizeParm> <a1> <a2> <aParm> <v3 sRGB> <v3 eRGB> <rgbParm> <killTime> <shader> <flags>` | the `CG_FX_ADDLINE` arm |
| `addelectricity <...> <chaos> <killTime> <shader> <flags>` | the `CG_FX_ADDELECTRICITY` arm |
| `addbezier <v3 start> <v3 end> <v3 c1> <v3 c1vel> <v3 c2> <v3 c2vel> <...>` | the `CG_FX_ADDBEZIER` arm |
| `addpoly <numVerts> <3 verts> <3 st pairs> <...>` | the `CG_FX_ADDPOLY` arm, always three verts |
| `addsprite <v3 org> <v3 vel> <v3 accel> <scale> <dscale> <sAlpha> <eAlpha> <rotation> <bounce> <life> <shader> <flags>` | the `CG_FX_ADDSPRITE` arm, which is `FX_AddParticle` with rgb `1 1 1` |
| `addtrail <v3 o0> <v3 o1> <v3 o2> <v3 o3> <shader> <setFlags> <killTime>` | `FX_FeedTrail` over a unit-coloured quad |
| `advance <ms>` | adds `<ms>` to the running clock and calls `FX_AdjustTime` with the absolute value |
| `addscheduled <portal>` | `FX_AddScheduledEffects(portal)` |
| `draw2d <xscale> <yscale>` | `FX_Draw2DEffects` |
| `dumpstate` | prints the pool census |
| `free` | `FX_FreeSystem` |
| `reset` | `FX_Free(false)` |

`advance` accumulates because `SFxHelper::AdjustTime` takes an absolute time,
not a delta (`FxSystem.cpp:53-80`).

## Golden format

```
== fx-oracle <scenario-stem> ==
...records...
== end ==
```

`F(x)` below is the eight-hex-digit float bit pattern.

```
REGISTER <path> -> <handle>
REGSHADER <name> -> <handle>
REGMODEL <name> -> <handle>
REGSOUND <name> -> <handle>
TIME <absolute-ms>
REFENT reType <i> renderfx <i> hModel <i> origin <F F F> oldorigin <F F F> axis <F F F F F F F F F> nonNormalizedAxes <i> radius <F> rotation <F> shaderTime <F> customShader <i> shaderRGBA <b b b b> shaderTexCoord <F F> frame <i>
MINIREFENT <same field list as REFENT>
NULLREFENT
LIGHT origin <F F F> radius <F> rgb <F F F>
POLY shader <i> count <i>
POLYV <index> xyz <F F F> st <F F> modulate <b b b b>
DECAL shader <i> origin <F F F> dir <F F F> orientation <F> rgba <F F F F> alphaFade <i> radius <F> temporary <i>
G2DECAL shader <i> start <F F F> dir <F F F> size <F>
SOUND origin <F F F> entnum <i> entchannel <i> sfx <i> volume <i> radius <i>
LOCALSOUND sfx <i> entchannel <i>
SHAKE origin <F F F> intensity <F> radius <i> time <i>
STRETCHPIC x <F> y <F> w <F> h <F> shader <i>
TRACE start <F F F> mins <F F F> maxs <F F F> end <F F F> skip <i> mask <i> g2 <0|1>
POINTCONTENTS point <F F F> passent <i> -> <i>
BOLT ent <i> model <i> bolt <i> -> <0|1>
LERPORIGIN ent <i> -> <F F F>
PRINT <text, trailing newline stripped>
```

The `REFENT` and `MINIREFENT` field list is the subset of `refEntity_t` and
`miniRefEntity_t` the FX code writes. `refEntity_t` opens with a byte-identical
copy of `miniRefEntity_t` (`tr_types.h:131-163`), so both records share one
printer. `AddFxToScene((miniRefEntity_t*)0)`, the emitter's attached-model call
at `FxPrimitives.cpp:1345`, prints `NULLREFENT`.

`dumptemplate` prints:

```
TEMPLATE <handle> name <effectName> repeatDelay <i> primitiveCount <i>
PRIM <index> name <mName> type <i> flags <i> spawnFlags <i> matImpactFX <i> cullRange <i> soundRadius <i> soundVolume <i>
PRIMRANGE <index> <fieldName> <F> <F>
PRIMVEC <index> min <F F F> max <F F F>
PRIMMEDIA <index> <listName> <count> <handle> <handle> ...
```

One `PRIMRANGE` line per `CFxRange` field in the declaration order of
`CPrimitiveTemplate` (`FxScheduler.h:167-254`), and one `PRIMMEDIA` line per
handle list in the order `mediaHandles`, `impactFxHandles`, `deathFxHandles`,
`emitterFxHandles`, `playFxHandles`. A handle that names no live template prints
`TEMPLATE <handle> MISSING`.

`dumpstate` prints:

```
STATE activeFx <i> drawnFx <i> scheduledFx <i> nextFree2DEffect <i>
```

`activeFx` and `drawnFx` are the `FxUtil.cpp` file-scope counters, read through
`extern` rather than by patching the TU.

## Fixture matrix

Every fixture is hand-authored. Retail `.efx` files are Raven content and never
enter the repo.

| Fixture | Primitive groups | What it pins |
| --- | --- | --- |
| `particle.efx` | `particle` | the particle chain with rgb, alpha and size ramps and a physics probe |
| `line.efx` | `line` | a line with an explicit `origin2` and `cheapOrg2Calc` |
| `traceline.efx` | `line` | `org2fromTrace`, `org2isOffset`, `traceImpactFx` and an `impactfx` cross-reference |
| `tail.efx` | `tail` | the length group and absolute velocity and acceleration |
| `sound.efx` | `sound` | a single-sound primitive |
| `cylinder.efx` | `cylinder` | the size2 group alongside size and length |
| `electricity.efx` | `electricity` | the chaos value, which shares the elasticity field |
| `emitter.efx` | `emitter` | `useModel`, `emitFx`, `deathFx`, angle deltas, density and variance |
| `decal.efx` | `decal` | the world decal plus the `ghoul2Decals` mark |
| `orientedparticle.efx` | `orientedparticle` | the oriented quad and its `origin2` angular offset |
| `fxrunner.efx` | `fxrunner` | `playfx` chaining |
| `light.efx` | `light` | the dynamic-light seam |
| `camerashake.efx` | `cameraShake` | the shake trap, with `intensity` and `radius` |
| `flash.efx` | `flash` (x2) | the scene flash and the `localizedFlash` 2D flash |
| `transitions.efx` | `cylinder` (x5) | every transition token (`linear`, `nonlinear`, `wave`, `random`, `clamp`) in every group (`rgb`, `alpha`, `size`, `size2`, `length`) |
| `allflags.efx` | `particle` | every token `ParseFlags` accepts, plus `materialImpact shellsound` |
| `allspawnflags.efx` | `particle` | every token `ParseSpawnFlags` accepts |
| `lists.efx` | `particle`, `sound`, `emitter` | list-valued `shaders`, `sounds` and `models` |
| `crossref.efx` | `particle`, `emitter`, `fxrunner` | `impactfx`, `deathfx`, `emitfx` and `playfx` together |
| `target.efx` | `particle` | the effect every cross-reference points at |
| `spawnshapes.efx` | `particle` (x3) | `orgOnSphere`, `orgOnCylinder`, `axisFromSphere`, `randrotaroundfwd` |
| `scheduling.efx` | `particle` (x3) | a wide count range for `fx_countScale`, `evenDistribution` with a delay range, and a one-wide count range that must not scale |
| `rgbinterp.efx` | `particle` | `rgbComponentInterpolation` |
| `relative.efx` | `particle` | a `relative` effect with a 50 ms `repeatDelay`, for the bolted and looped paths |
| `overflow.efx` | `particle` (x25) | one primitive past `FX_MAX_EFFECT_COMPONENTS`, so the overflow arm drops the last |
| `badkeys.efx` | `particle` plus an unknown group | an unknown primitive key, an unknown sub-group, an unknown key inside a group, an unknown `materialImpact`, silently dropped unknown flag tokens and an unknown top-level group |

## Scenarios

| Scenario | Covers |
| --- | --- |
| `templates` | one fixture per primitive group name, parsed and dumped. No play. |
| `parsing` | every flag, spawn flag, transition, media list and cross-reference key, parsed and dumped. No play. |
| `particles` | the particle chain over six frames, from spawn to death |
| `lines` | a plain line, an electricity arc and a trace-driven line with its impact effect |
| `tails` | the tail and cylinder pair, the two types that read the length group |
| `sound` | the world sound arm, the portal local-sound arm and the sound list |
| `emitter` | the attached model, the `NULLREFENT` pair and the death effect |
| `decal` | the world decal, the ghoul2 mark and an oriented particle |
| `runner` | the fxrunner chain, the light seam and the camera shake trap |
| `flash` | the scene flash, the 2D list and `draw2d` at two scales |
| `direct` | all six direct `FX_Add*` trap arms |
| `bolted` | bolted scheduling, the looped reschedule, `stop`, and the entity arm |
| `scheduling` | `fx_countScale` at 1, 0.5 and 4, the delay arms, and the reset door |
| `spawnshapes` | the sphere, cylinder and random-rotation spawn arms, and component colour interpolation |
| `crossref` | a bounce into an impact effect, an emit chain and a runner chain |
| `parseerrors` | the overflow arm, every unknown-key print, a missing file and three invalid play calls |
| `culldebug` | the three cull arms, the two flags that skip them, and the `fx_debug` census |

## Quirks the goldens pin

These are Raven behaviours the port must reproduce, not harness artefacts.

- **`AddLoopedEffects` shifts its arguments.** The call at `FxScheduler.cpp:135`
  passes nine arguments to a signature whose sixth is `fxParm`, so the portal
  flag lands in `vol`, `0` lands in `rad`, and the relative flag lands in
  `isPortal`. Every reschedule from a relative loop therefore belongs to the
  portal pass, and only a portal pass drains it. `bolted` pins this.
- **`FX_PlayEntityEffectID` drops its `entNum`.** `FXExport.cpp:64-75` never
  forwards the argument, so the entity comes out of `boltInfo` alone.
- **`SFxHelper::Trace` swaps its `memset` arguments.** `FxSystem.h:121` writes
  `memset(td, sizeof(*td), 0)`, which clears zero bytes. The shared block keeps
  whatever the previous trap left in it.
- **`SFxHelper::PlaySound` hardcodes the entity and channel** and drops the
  volume and radius.
- **A bolted effect starts one millisecond late.** `FxScheduler.cpp:991`
  increments `mStartTime` so the ghoul2 bolt has a frame to appear.
- **A count range exactly one wide skips the scale.** `FxScheduler.cpp:884`
  compares `fabsf(max - min) > 1.0f`, so `count 2 2` never scales.

## Uncovered

The harness makes no claim on these, and no golden hides them:

- **Real ghoul2 bolts.** `G2API_GetBoltMatrix` answers from the scripted queue,
  so the bolt matrix the goldens carry is the harness's, not a skeleton's. The
  FX side of the seam is covered in full: the origin and axis extraction, the
  bolt-missing arm and the relative update chain all run.
- **`CG_GET_LERP_DATA`.** The arm clears the block and preserves the entity
  number, because the harness answers the bolt question itself. It prints no
  record of its own; the `BOLT` record carries the answer.
- **Wind.** `FX_AFFECTED_BY_WIND` parses and reaches its branch, but Raven's
  wind body is commented out (`FxScheduler.cpp:1361-1369`), so there is nothing
  to reproduce.
- **`MaterialImpact`.** `materialImpact shellsound` parses and the template dump
  carries it, but `CFxScheduler::MaterialImpact` is commented out whole
  (`FxScheduler.cpp:660-682`).
- **`GetEffectCopy` and `GetPrimitiveCopy`.** The FX override API has no caller
  in the MP tree, so no scenario drives it.
- **The `_SOF2DEV_` arms.** `fx_freeze` and its guards never compile.
- **`VV_LIGHTING`.** The alternate light path never compiles, so `CLight` always
  takes the `re.AddLightToScene` arm.
