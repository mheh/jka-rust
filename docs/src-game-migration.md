# `src/game` Migration Map

`src/game` is a legacy staging area for MP server-game material copied from Raven `codemp/game/g_local.h`. The intended Rust-module destination is `src/modules/mp/game/` once the missing dependency types are ported.

| Current file | Migration target | Raven source | Notes |
| --- | --- | --- | --- |
| `src/game/mod.rs` | `src/modules/mp/game/mod.rs` | `oracle/oracle/codemp/game/g_local.h:28` | MP game constants: `GAMEVERSION`, frame/reward timings, intermission timings, shared buffer size, podium model. |
| `src/game/client/mod.rs` | `src/modules/mp/game/client.rs` or `src/modules/mp/game/client/mod.rs` | `oracle/oracle/codemp/game/g_local.h:366` | Client/session/persistent/render structures ending in `gclient_s`; these are MP game-private except for the leading engine-visible `playerState_t ps`. |
| `src/game/entity/mod.rs` | `src/modules/mp/game/entity.rs` or `src/modules/mp/game/entity/mod.rs` | `oracle/oracle/codemp/game/g_local.h:52` | Entity flags, mover state, hit locations, `gentity_s`, and damage redirect constants. The `gentity_s` prefix is engine-sensitive and must preserve Raven layout. |
| `src/game/level/mod.rs` | `src/modules/mp/game/level.rs` or `src/modules/mp/game/level/mod.rs` | `oracle/oracle/codemp/game/g_local.h:750` | World/level-local containers: interest points, combat points, alert events, waypoint data, `level_locals_t`, damage flags, reference tags, bot settings. |
| `src/bg/mod.rs` | `src/common/mp/bg/` or `src/common/sp/bg/` after provenance is known | Raven `bg_*` files differ between `code/game` and `codemp/game` | This file is currently only a placeholder; do not collapse SP and MP `bg_*` into one module until a specific component is proven shared. |

## Migration Rule

Keep structures sourced from `codemp/game/g_local.h` under `src/modules/mp/game`. Promote only narrower pieces into `src/common/mp/game` or `src/common/mp/qcommon` when the Raven source proves they are shared support code instead of runtime module state.

## Comment Rule

When moving a structure, preserve its Raven comments and add source references in this style:

```rust
/// `type_or_const_name`.
///
/// Raven: original Raven comment text.
/// Source: `oracle/oracle/codemp/game/g_local.h:123`
```
