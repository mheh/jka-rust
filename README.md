# jka-rust

An idiomatic Rust reimplementation of Star Wars Jedi Knight: Jedi Academy. MP comes first (the game module, then cgame, ui, and the engine), and SP follows it. The target is a drop-in replacement for each binary Raven shipped.

## The oracle (`oracle/`)

[`oracle/`](oracle) is a git submodule of [mheh/jediacademy](https://github.com/mheh/jediacademy), a fork of [jedis/jedi-academy](https://github.com/jedis/jedi-academy) that holds Raven's released Jedi Academy source (SP under `code/`, MP under `codemp/`). We never edit it. Every port is verified against it: clang-verified layout static-asserts for the types, and committed golden fixtures plus differential parity tests for the behavior. There is no FFI and no extracted C.

The repo takes its own structure. Per-module logic crates (`crates/mp/*`, `crates/sp/*`) sit under thin cdylib shells (`crates/jampgame`, `crates/cgame`, `crates/ui`, `crates/jagame`) that export the exact symbols the engines load. See [`docs/workspace-architecture.md`](docs/workspace-architecture.md) for the crate graph and [`docs/porting-rules.md`](docs/porting-rules.md) for the rules a port follows.

## Status

The MP game module (`jampgame`) and the MP dedicated-server engine are complete. The lockstep referee verifies them against Raven's binaries, and they host live play. The idiomatic consolidation campaigns (owned strings, `bool`, threaded state, model-data views) are done on top of that parity, and the `ui` module port closed on 2026-08-01.

The active track is the full `jamp` client. The live work plan is the wayfinder map, [issue #2](../../issues/2), and the design groundwork sits in [`docs/plans/2026-07-24-client-port/`](docs/plans/2026-07-24-client-port/). The frontier is the renderer census, [issue #31](../../issues/31), plus the open live-play defect tickets.

In the census, the model blocks publish to the render thread, and the draw arms read the published registry. The MD3 arm draws un-gated on the live client under a committed entity image golden. The Ghoul2 bone matrices cross at scene-add (DEC-65 ruling 2, merge `ec7c934c` on 2026-08-05). Mark fragments, the polygon-offset depth bias, and the dlight projection passes landed after that. The 2D closure step is in flight, and DEC-66 settled the render-side RNG owner, which unblocks the FX mini-refent arms.

Architectural rulings live in [`docs/decisions.md`](docs/decisions.md). Each push to `master` builds the workspace, runs the workspace test suite, cross-checks the ILP32 layout asserts, and publishes the `jampgame` modules and the `jampded` server to the rolling [`latest` release](../../releases/tag/latest). The lockstep referee and the image goldens run locally, because they need the retail assets and a GPU.

## Build and verify

Run these three steps from the repo root.

1. Fetch the oracle source: `git submodule update --init`. The parity tests read it.
2. Build the workspace: `cargo build --workspace`. The layout static-asserts are compile-time, so a green build proves the ABI layouts.
3. Run the tests on one thread: `cargo test --workspace -- --test-threads=1`. The world-golden gate aborts when the tests run in parallel.

## For readers of Raven's `g_*.c`

Files mirror Raven's subsystems, functions keep Raven's names, fields keep Raven's names, and Raven's comments stay in place. `grep -rn "G_RadiusDamage"` lands where you expect. Where the later reshaping renames anything, `#[doc(alias = "G_Damage")]` keeps grep, rustdoc search, and IDE lookup working. The port stays as faithful as Rust allows, so a reader who knows the original can read it, work on it, and build on it.

Here is one loop in three forms: 2003, today, and the endgame.

```c
/* 2003 - oracle/codemp/game/g_combat.c, G_RadiusDamage */
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
// Today: crates/mp/game/src/g_combat.rs, G_RadiusDamage.
// The entities are ids, and the seam call spells out the syscall args.
let numListedEntities = trap::EntitiesInBox(
    ctx.engine,
    mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs::new(
        &mins as *const vec3_t,
        &maxs as *const vec3_t,
        entityList.as_mut_ptr(),
        (MAX_GENTITIES) as i32,
    ),
);

for e in 0..numListedEntities {
    let ent = EntityId(entityList[e as usize] as u32);

    if Some(ent) == ignore {
        continue;
    }
    if ctx.entity(ent).takedamage == qfalse {
        continue;
    }
    // ...
    G_Damage(
        ctx,
        Some(ent),
        missile,
        attacker,
        Some(&mut dir),
        origin,
        points as c_int,
        DAMAGE_RADIUS,
        r#mod,
    );
}
```

```rust
// The endgame: the same architecture, stated precisely.
// No pointer can dangle and no NULL goes unchecked, and the compiler proves both with no runtime cost.
for id in game.entities_in_box(mins, maxs) {
    if ignore == Some(id) || !game.ent(id).takedamage { continue; }
    // ...
    damage(game, id, None, attacker, Some(dir), origin, points as i32,
           DamageFlags::RADIUS, mod);
}
```

`qboolean` does not reach the third form.

### The middle state and the machine transcription

A codebase this size invites silent drift under hand translation. Small improvements enter mid-translation, and nobody can audit them afterward. This port removes that risk by construction. LLM agents did the transcription with no creative latitude. The tooling parses the oracle with libclang and emits self-contained work orders, the agents transcribe them blind, and mechanical checks judge the output.

Three checks carry that judgment. Clang-derived layout asserts cover every ABI-crossing struct. Committed golden fixtures cover the C++ subsystems. The lockstep referee runs Raven's compiled `jampgame` and ours side by side on a live server, and it compares the entity state, the player state, and the syscall stream every frame, byte for byte. Human rulings settle the judgment calls, and [`docs/decisions.md`](docs/decisions.md) records them.

The referee has caught real divergences. A one-ULP head-angle error traced to C's unsuffixed double literals, where `0.4` promotes to a double multiply and `0.4f` does not. Retail never shipped `bg_lib.c`'s `rand()`, because the native DLL links MSVC's CRT LCG, so faithful bot behavior needs `holdrand * 214013 + 2531011`. A `vec3_t` parameter writes back through array decay, and a by-value port dropped that write. Each reshaping step lands behind a green referee, one verified step at a time.

### Where it lands

```rust
// `gentity_t` no longer exists.
// The bytes the engine reads live in one #[repr(C)] seam array, and everything else is plain Rust.
#[repr(C)]
pub struct EntitySeam { pub s: entityState_t, pub r: entityShared_t }

pub struct Entity {               // module-private, no pointers anywhere
    pub enemy: Option<EntityId>,  // was *mut gentity_t
    pub think: Think,             // enum dispatch, was a function pointer
    pub classname: Classname,     // was char*
    // ...
}

let mut ent = world.ent_mut(id);      // EntityMut<'_>, both halves under one borrow
ent.s.pos.trDelta[2] = 237.3;         // seam: the engine reads this byte
ent.p.enemy = Some(other);            // private: the engine never reads this
```

`LocateGameData` hands the engine a base pointer and a module-chosen stride. The engine only dereferences the `sharedEntity_t` prefix (`s` and `r`) and each client's `playerState_t`. Everything Raven packed after that prefix was module-private all along, so it can become `Option`s, enums, and `String`s while a 2003 `jampded` binary loads the module unchanged. There is no marshaling layer, because the seam array is the live storage the engine snapshots in place.

After parity, the reshaping (see [`docs/roadmap-final-stages.md`](docs/roadmap-final-stages.md)) makes the world one owned, copyable value. A single input queue in the `Cbuf_AddText` tradition feeds it, with typed, whitelisted commands in place of text, and it logs every input. The simulation is deterministic, so an input log plus occasional world keyframes is a complete recording. That recording holds the world state rather than the wire traffic, so a tool can re-simulate it, rewind it, and inspect it.

The same property gives headless runs with no clock and no renderer, at thousands of frames per second. Those runs serve balance testing, fuzzing, and soak farms, and the lockstep referee is the first consumer. External tools such as a Discord bot or an MCP server can also drive a live server through the same audited command queue that rcon uses.

## Ship targets

- **MP** (`jamp` engine): 3 loadable modules, `jampgame`, `cgame`, and `ui`.
- **SP** (`jasp` engine): `jagame` only. SP cgame and ui link statically into the engine.
- **The MP dedicated server engine (the `jampDed` equivalent) is done and hosts live sessions** (see Status). The client engine and the renderer are the active track, and they are no longer deferred (DEC ledger).
