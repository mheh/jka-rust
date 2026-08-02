# The logic-port pipeline

This doc records the packet pipeline that turns an oracle subsystem into machine-verifiable transcription work orders. The principle (CLAUDE.md): tooling turns the oracle into self-contained packets, agents transcribe, and a mechanical referee judges. The pipeline ran the jampgame port (pass 3), the engine C-pass, and now the jamp client island. All tools live in `tools/closure-prototype/` and run on its venv: `.venv/bin/python <tool>`.

## The chain

1. **Profile.** Each module has one spec file, `tools/closure-prototype/modules/<name>.py`. `closure.py` loads every spec into `RAVEN_MODULES`. A spec pins the language, the entry headers, the include dirs, the vcproj Release defines, and a `srcglob` naming the exact transcription TU set. The srcglob is scope law: dead TUs, replaced TUs (DEC rulings), and design-track C++ subsystems stay out. `--list-modules` prints the table.

2. **Sweep.** `enginesweep.py --module <m> --out out/<label>` does one unity libclang parse and emits the function manifest, per-subsystem stats, the call matrix, and the SCC wave partition. Parse errors of the known MSVC class (the `__asm` SnapVector block, unity redefinitions) are benign and reported, never silent.

3. **Order.** `engineorder.py --module <m>` builds the stub-free port order: every function sorted so its in-set callees precede it, cyclic SCCs grouped as one unit.

4. **Packets.** `enginepackets.py --module <m>` derives one work order per C-track unit: the verbatim oracle slice with cites, the derived resolved Rust signature (porting-rules §C defaults, rosetta type rows), the touched globals and statics with their classification hooks, and the resolved callee surface. A shared `_PREAMBLE.md` carries the rulings digest for the module (engine: the 48 fork rulings, client: the DEC-55..58 digest), the type map, and the zero-park discipline. Rule-20 drops (dead or replaced arms, for example the client's OpenAL/EAX functions) become manifest markers, never packets. Built-in machine checks: zero dangling in-set callees, missing rosetta types reported, undocumented C++ methods flagged.

5. **Transcribe.** A workflow (`.claude/workflows/engine-cpass-transcribe.js` is the engine template, `port-jampgame-pass3.js` the game one) fans blind porters over the shards. Porters never explore, never run cargo, never park: missing symbols are reported for memoized fixers, ambiguity becomes a `// PORT-NOTE`, and the packet signature is law. The one porter-side gate is the rustfmt parse check.

6. **Integrate.** The sibling integrate workflow (`integrate-engine-cpass.js`) runs triage, bounded parallel fix rounds under the symbol-resolver contract, and a serial finisher with delta tripwires, ending at `cargo build` zero errors.

7. **Judge.** The referees gate the result: the lockstep suite for server-visible code, the replay referee for cgame, the subsystem golden rigs (gp2, ghoul2, snd) for C++-track and seam code. Green referees, not porter claims, close a wave.

## Module spec inputs

The tools read no per-module data outside the spec files. One spec, `modules/<name>.py`, carries the full variable surface of a module:

- **Parse profile:** `lang`, `entry`, `includes`, `defines`, `flags`, `srcglob`.
- **Sweep fields:** `label` (the `out/<label>/` prefix), `subsystems` (the subsystem map, so no `?` rows), `sweep_title`, `sweep_desc`.
- **Order fields (`order` block):** an optional `vcproj` link-set source with its exclude sets (the engine), or the srcglob default (the client), plus `extra_subsystems` and `md_title`.
- **Packet fields (`packets` block):** the rosetta path, the classification sets (`doc`, `doc_kind`, file routes, class sets), the destination `crate_src` map, and three declarative inputs:
  - **Rulings digest file** (`digest`): a verbatim text file that `_PREAMBLE.md` embeds under `digest_heading`. Engine: `docs/handoffs/engine-fork-discovery.md`. Client: `modules/mp-client-rulings-digest.md` (DEC-55..58).
  - **Rule-20 drop list** (`drop_list`): a JSON file of verified dead-arm functions, each with `symbol`, `file`, `reason`, and `cite`. A listed function gets no packet. The manifest records it as a drop with reason and cite. Client: `modules/mp-client-drops.json` (the OpenAL/EAX arm, DEC-57).
  - **Dual-mode signature resolver** (`law_from_tree`): when true, an out-of-set callee that exists under `crates/mp` or `crates/native` stamps its real worktree signature as LAW in the packet. In-set callees keep the derived mechanical signatures.

Every generated manifest carries a freshness stamp: `generated_at_commit` (git HEAD) and `generated_at_tree_dirty` (from `git status --porcelain`).

## Adding a module leg

To point the pipeline at a new island: write `modules/<name>.py` with the profile and chain fields (defines from the owning vcproj config, win32 spellings from the `mp-engine-ded` precedent when headers assume a platform section), author the module rulings digest file from the settled DEC entries, and write the rule-20 drop-list file. Everything else is module-keyed. The historical outputs of earlier modules must regenerate byte-identical - the tools never fork per module.

## The client leg (2026-08-01)

`mp-client`: 18 TUs (16 `codemp/client/*.cpp` + `cm_terrainmap.cpp` + `cm_draw.cpp`), 617 functions, 22,414 LOC. FX C++ TUs are design-track (ticket #26) and stay out of the srcglob. Dropped per DEC-56/57: `win32/`, `mp3code/`, the OpenAL/EAX arm, the console/Xbox twins, `0_SH_Leak.cpp`. The rulings digest cites DEC-55..DEC-58 and the survey's Class A/B/C state classification.
