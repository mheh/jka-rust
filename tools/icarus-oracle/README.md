# icarus-oracle — differential golden harness for the ICARUS sequencer port

Verifies the `mp_engine_icarus` §F reimplementation (`docs/subsystems/icarus.md`,
FROZEN) against the **unmodified** Raven ICARUS TUs, exactly like `tools/gp2-oracle`
and `tools/jampgame-oracle` (porting-rules §18). The oracle `.cpp`/`.h` are copied
into `build/` and compiled standalone against the real oracle header closure;
canonical dumps are stored under `goldens/` and committed, so the Rust parity
tests need **no** C++ toolchain — only `build.sh` does, to (re)generate or check.

`oracle/` is never edited.

## Usage

```sh
sh build.sh          # build ibi-gen + dumpers, diff dumps against goldens/
sh build.sh --regen  # recompile fixtures/*.IBI and regenerate goldens/*
```

Toolchain: Homebrew `g++-16` (override with `CXX=`). Oracle-parity flags match
the sibling harnesses: `-fsigned-char -ffp-contract=off -fno-fast-math`, plus
`-std=c++14 -D__linux__` (POSIX branch of the real headers), `-DDEDICATED`, and
`-fpermissive` (see Normalizations). A green `build.sh` proves the port target's
behaviour byte-for-byte against the oracle over the committed corpus.

## The three goldens (one per § Verification strategy unit)

| Golden | Pins | Doc unit |
| --- | --- | --- |
| `goldens/q3_registers.txt` | `Q3_Registers` script-variable store: a scripted `Declare`/`Set`/`Get`/`Free`/`VariableDeclared` sequence over `varStrings`/`varFloats`/`varVectors`, the duplicate/bad-type/undeclared no-ops, and the `MAX_VARIABLES`(32) cap — final sorted state + `numVariables`. | unit 2 "Q3_Registers" |
| `goldens/blockstream_<fixture>.txt` (×4) | `CBlockStream` **reader**: `Open`→`BlockAvailable`→`ReadBlock`→`ReadMember` over each committed `.IBI`, dumped as the parsed `(block id, flags, members)` + per-member `(id, type, size, bytes)` record stream. Exercises the live `CBlock::Write`/`WriteData` in-memory builders transitively (their member bytes appear in the dump). | unit 1 "BlockStream (read-only)" |
| `goldens/endtoend_e2e.txt` | End-to-end `ICARUS_Init`→`InitEnt`→`RunScript`→per-frame `CTaskManager::Update` on `e2e.IBI`: the ordered outbound `VM_Call(gvm, GAME_ICARUS_*)` callback trace (`SET`,`PLAYSOUND`,`USE`,`KILL`) + final variable/signal state. Drives `Sequencer`/`TaskManager`/`Instance`, `Parse*`/`Check*`, `wait`, signals, `Callback`. | unit 3 "Sequencer + TaskManager + Instance (end-to-end)" |

The Rust port's parity test (in `mp_engine_icarus`, once landed) reads
`fixtures/*.IBI` + `goldens/*` from here and must reproduce every golden exactly.

## Fixture corpus (`fixtures/*.icarus` → committed `fixtures/*.IBI`)

Hand-authored ICARUS scripts, compiled once by `ibi-gen` (below). No retail `.IBI`
assets are committed (icarus.md ruling 14).

- `vars.icarus` — `declare`/`set`/`print`/`wait`: float + string members.
- `control.icarus` — `loop`, `task`/`do`/`dowait`, `sound`, `signal`/`waitsignal`, block nesting.
- `motion.icarus` — `affect{...}` block with `move`/`rotate` (vectors), `use`, `wait`.
- `e2e.icarus` — the end-to-end driver: variable + outbound vmcall (`sound`/`use`/`kill`) + `signal`.

Together they cover every block/member type the reader parses (string=4, float=6,
identifier=7, vector=14) across declare/set/loop/print/wait/task/sound/do/signal/
affect/move/rotate/use/kill.

## ibi-gen — the fixture compiler (icarus.md ruling 14)

`ibigen.cpp` drives the oracle's **out-of-set** `Interpreter.cpp` + `Tokenizer.cpp`
front-end (permitted in the fixture-generator **only** — they are not in the WinDed
link set and are **not** part of the ported scope) plus the in-scope `CBlockStream`
**writer** half (§20-dropped from the port, permitted here) and `Memory.cpp`, to
compile a `.icarus` script into a precompiled `.IBI` block-instruction blob. The
goldens themselves are produced by the **in-scope** reader/registers/sequencer TUs
over these blobs — never by the generator.

## Normalizations (documented, semantics-preserving; applied to `build/` COPIES only)

Everything below is a compiler-portability or LP64 platform-width fix that
preserves the 32-bit ship behaviour the Rust port models (porting-rules §19); none
changes observable behaviour. `oracle/` is untouched.

- **`BlockStream.cpp` `return false;`→`return 0;`** (GetMember :333 live, Duplicate
  :367 dead) — MSVC lets `false` initialise a pointer return; `false==0==NULL`.
- **`BlockStream.cpp` `ReadMember` `long`→`int`** (:110,:117,:118) — the member
  size field is *written* as `int`(4) but *read* through `*(long*)`/`sizeof(long)`;
  self-consistent on the 4-byte-`long` ship, mis-parses at LP64. Normalising to
  `int` is exactly the port's `i32` model and matches the fixtures.
- **`BlockStream.cpp` `Create` header write** (:546) `sizeof(id_header)`(char\*)→
  `sizeof(IBI_HEADER_ID)`(4) — the reader `Open` expects a 4-byte `"IBI\0"` header;
  the writer's pointer-width `sizeof` is 4 on the ship, 8 here. Generator-only.
- **`GameInterface.h`** `entlist_t`/`bufferlist_t` drop the explicit
  `less<string>,allocator<T>` args — an MSVC-STL laxity (`value_type != allocator
  value_type`) modern libstdc++ rejects; the default allocator yields an identical
  container.
- **`tokenizer.h`** (generator-only) restore two `CSymbolLookup` accessor
  declarations (`GetChild`, `GetChildAddress`) that the committed oracle header
  omits but `Tokenizer.cpp` both defines and uses — an oracle header/impl mismatch
  no conforming compiler accepts. Signatures taken verbatim from the `.cpp` defs.
- **`stubs/prelude.h`** force-include restores the STL/`<cstring>` ambient state
  Raven's MSVC precompiled header (`exe_headers.h`) supplied before any icarus
  header parsed (include-order only), and hoists the two `ICARUS_Malloc`/`_Free`
  decls for GCC two-phase template lookup.

## Deviations from the doc's plan (with reason)

1. **ibi-gen needed a two-line generator-only header repair.** The doc names
   `Interpreter.cpp`/`Tokenizer.cpp` as the ibi-gen source and assumes they
   compile. The committed `tokenizer.h` is missing two `CSymbolLookup` accessor
   declarations its own `Tokenizer.cpp` requires (see Normalizations). Restoring
   exactly those two decls in the `build/` copy keeps the doc-specified
   Interpreter/Tokenizer front-end; it is confined to the generator and never
   touches the ported reader/registers/sequencer TUs.
2. **The MockHost's `VM_Call` returns 0 for every `GAME_ICARUS_*` arm.** The
   end-to-end golden records the outbound call *sequence* (the seam signal); the
   game module's return values are a defined mock convention (0 = "not handled"),
   documented so the Rust `MockHost` uses the same convention. Consequently
   `set("hp",100)` routes through the engine `SET` vmcall (returning 0) rather than
   mutating the local var store — faithful to "game returns 0", captured in the
   golden. The `svs.time` clock advances 200 ms/frame for 30 frames.

## On-disk size

Committed artifacts: 4 `.icarus` + 4 `.IBI` + 6 goldens = 14 files, ~4.6 KB total.
`build/` is git-ignored.
