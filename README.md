# jka-rust

An idiomatic, opinionated Rust reimplementation of Star Wars Jedi Knight:
Jedi Academy — MP first (game module, then cgame/ui/engine), SP behind it —
built toward drop-in replacements for Raven's shipped binaries.

## The oracle (`oracle/`)

[`oracle/`](oracle) is a git submodule of
[mheh/jediacademy](https://github.com/mheh/jediacademy) — a fork of
[jedis/jedi-academy](https://github.com/jedis/jedi-academy), Raven's released
Jedi Academy source (SP under `code/`, MP under `codemp/`). It is **never
edited**: every port is verified against it — clang-verified layout
static-asserts for types, committed golden fixtures and differential parity
tests for behavior. No FFI, no extracted C.

This repo does not mirror Raven's layout — it takes its own structure:
per-module logic crates (`crates/mp/*`, `crates/sp/*`) under thin cdylib shells
(`crates/jampgame`, `crates/cgame`, `crates/ui`, `crates/jagame`) that export
the exact symbols the engines load. See
[`docs/workspace-architecture.md`](docs/workspace-architecture.md) for the
crate graph and [`docs/porting-rules.md`](docs/porting-rules.md) for how code
is ported.

## Status (2026-07-11)

- **Type port: complete** (Waves 0–7, both trees). Every ABI-crossing struct
  carries `size_of`/`offset_of!` static-asserts — a green build is the layout
  test — with `target_pointer_width = "32"` twins (clang i386 ground truth)
  across the `jampgame` crate tree.
- **MP game module (`jampgame`): transcribed and integrated.** `mp_game`
  compiles green with zero `todo!()` stubs and zero open `TODO: Port` markers;
  all `vmMain` dispatch arms are wired. The built cdylib exports
  `dllEntry`/`vmMain`/`GetModuleAPI`.
- **MP dedicated-server engine: closure complete.** The full `jampDed` link
  set is transcribed and the workspace is **green** — `cargo check --workspace`
  builds every crate and **343 tests pass**, including the §F oracle-parity
  golden suites (GP2, ghoul2 bone/bolt/collision, ICARUS, RMG, StringEd, ROFF,
  tr_model). `qcommon`, `botlib`, and `server` are **closed** — zero `todo!()`
  stubs, zero open `TODO: Port` markers, zero extern forward-declaration
  blocks. The seven C++ subsystems are golden-verified against unmodified
  oracle TUs; the platform layer (`Sys_*`/console/sockets) is implemented
  natively in Rust; the client draw surface is out of the dedicated scope by
  ruling (DEC-01).
- **Host seam: live.** The engine island threads one borrowed world bundle
  (`EngineHostView`, DEC-23) — the live `EngineHost` implementation behind the
  §F subsystems and the `EngineHooks` upcall table — replacing the transitional
  receiver-list convention.
- **CI**: pushes to `master` run the full-workspace compile+test gate
  (`cargo build --workspace` + the oracle parity suites, single-threaded),
  build `jampgame` for Windows/Linux × x86/x86_64 × release/debug (all lanes
  enforced), and publish the zips to the rolling
  [`latest` release](../../releases/tag/latest) under the exact filenames the
  engine loads (`jampgamex86.dll`, `jampgamex86_64.so`, …).
- **Remaining roadmap** (compiling green is not parity): boot/lifecycle wiring
  (in progress — `SV_Frame`/net bring-up so the dedicated server actually
  runs); the **referee swap** — oracle differential tests (single-threaded,
  replay-based) become the ground truth; then warning-zero and the safe-state
  migration that retires the transcription's raw-pointer scaffolding.
- Architectural decisions live in
  [`docs/decisions.md`](docs/decisions.md); remaining port surface is
  machine-audited against the link-set manifest
  (`tools/closure-prototype/out/engine/engine-port-order.tsv`); older audits
  live under [`docs/audits/`](docs/audits/).

## If you've spent twenty years in `g_*.c`

You already know this codebase. Files mirror Raven's subsystems, functions
keep Raven's names, fields keep Raven's names, Raven's comments ride along.
`grep -rn "G_RadiusDamage"` lands where you expect, and where the later
reshaping renames anything, `#[doc(alias = "G_Damage")]` keeps grep, rustdoc
search, and IDE lookup working. The port tries to stay as faithful as
possible so that anyone who knows the original can read it, work on it, and
build on it.

The same loop, three snapshots — 2003, today, and where this lands:

```c
/* 2003 — oracle/codemp/game/g_combat.c, G_RadiusDamage */
numListedEntities = trap_EntitiesInBox( mins, maxs, entityList, MAX_GENTITIES );
for ( e = 0 ; e < numListedEntities ; e++ ) {
    ent = &g_entities[entityList[ e ]];
    if (ent == ignore)
        continue;
    if (!ent->takedamage)
        continue;
    /* ... */
    G_Damage( ent, NULL, attacker, dir, origin, (int)points, DAMAGE_RADIUS, mod );
}
```

```rust
// Today — crates/mp/game/src/g_combat.rs, mid-migration. Honest snapshot:
// noisier than the C. Ids at the boundaries, pointers still in the bodies —
// scaffolding, kept while every commit byte-verifies against the oracle.
let numListedEntities = trap::EntitiesInBox(ctx.engine, /* seam args */);
for e in 0..numListedEntities {
    let ent = &mut ctx.world.g_entities[entityList[e as usize] as usize] as *mut gentity_t;
    if ent == ignore { continue; }
    if (*ent).takedamage == qfalse { continue; }
    // ...
    G_Damage(ctx, ctx.entity_id_of(ent), None, ctx.entity_id_of(attacker),
        Some(&mut dir), origin, points as c_int, DAMAGE_RADIUS, r#mod);
}
```

```rust
// The endgame — the same architecture, stated precisely. No pointer can
// dangle, no NULL goes unchecked, and the compiler verifies it at compile
// time with no runtime cost.
for id in game.entities_in_box(mins, maxs) {
    if ignore == Some(id) || !game.ent(id).takedamage { continue; }
    // ...
    damage(game, id, None, attacker, Some(dir), origin, points as i32,
           DamageFlags::RADIUS, mod);
}
```

(`qboolean` doesn't survive to the third snapshot. It had a good run.)

### Why the middle state exists — and why machines did the typing

Hand-translating a codebase this size invites silent drift: small
"improvements" made mid-translation that nobody can audit afterward. This
port avoids that by construction. The transcription was executed by LLM
agents that were given no creative latitude: tooling parses the oracle with
libclang and emits self-contained work orders, agents transcribe them
blind, and mechanical checks judge the output — clang-derived layout
asserts on every ABI-crossing struct, committed golden fixtures for the C++
subsystems, and a lockstep referee that runs Raven's compiled `jampgame`
and ours side by side on a live server, comparing entity/player state and
the syscall stream every frame, byte for byte. Judgment calls are human
rulings, recorded in [`docs/decisions.md`](docs/decisions.md).

Examples of what the referee has caught: a one-ULP head-angle divergence
traced to C's unsuffixed double literals (`0.4` promotes to a double
multiply; `0.4f` does not); retail never shipping `bg_lib.c`'s `rand()` —
the native DLL links MSVC's CRT LCG, so faithful bot behavior means
reproducing `holdrand * 214013 + 2531011`; and a `vec3_t` parameter's
array-decay write-back that a by-value port silently dropped. The pointers
you see in today's snapshot are scaffolding kept for exactly this reason:
reshaping happens only behind a green referee, one verified step at a time.

### Where it lands

```rust
// `gentity_t` no longer exists. The bytes the engine actually reads live in
// ONE #[repr(C)] seam array; everything else is plain Rust.
#[repr(C)]
pub struct EntitySeam { pub s: entityState_t, pub r: entityShared_t }

pub struct Entity {               // module-private — no pointers, anywhere
    pub enemy: Option<EntityId>,  // was *mut gentity_t
    pub think: Think,             // enum dispatch, was a function pointer
    pub classname: Classname,     // was char*
    // ...
}

let mut ent = world.ent_mut(id);      // EntityMut<'_> — both halves, one borrow
ent.s.pos.trDelta[2] = 237.3;         // seam — the engine sees this byte
ent.p.enemy = Some(other);            // private — the engine never will
```

What makes this compatible: `LocateGameData` hands the engine a base pointer
and a module-chosen stride, and the engine only ever dereferences the
`sharedEntity_t` prefix (`s`+`r`) and each client's `playerState_t`.
Everything Raven packed after that prefix was module-private all along — so
it can become `Option`s, enums, and `String`s while a 2003 `jampded` binary
loads the module unchanged. There is no marshaling layer: the seam array is
the live storage the engine snapshots in place.

After parity, the reshaping (see
[`docs/roadmap-final-stages.md`](docs/roadmap-final-stages.md)) makes the
world one owned, copyable value, with a single input queue in the
`Cbuf_AddText` tradition — typed, whitelisted commands instead of text —
and every input logged. Because the simulation is deterministic, an input
log plus occasional world keyframes is a complete recording: like a demo,
but of the world rather than the wire, so it can be re-simulated, rewound,
and inspected rather than only rewatched. The same property gives headless
runs — no clock, no renderer, thousands of frames per second — for balance
testing, fuzzing, and soak farms (the lockstep referee is the first
consumer), and lets external tools such as a Discord bot or an MCP server
drive a live server through the same audited command queue rcon uses.

## Ship targets

- **MP** (`jamp` engine): 3 loadable modules — `jampgame`, `cgame`, `ui`.
- **SP** (`jasp` engine): `jagame` only (SP cgame/ui are statically linked into
  the engine).
- **The MP dedicated server engine (`jampDed` equivalent) — in progress** (see
  Status); client engine and renderer are deferred by decision (DEC ledger).
