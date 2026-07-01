# Type-port TODO — game-code dependency types (MP + SP)

Tracks the types the dormant `src/game/{client,entity,level}` scaffolding depends on
that were **not yet ported**, discovered while migrating that code into `crates/mp/game`.
Each must be ported for MP, then investigated + ported for SP (SP structures differ).

Placement principle: put each type in the **lowest tier that needs it** (native < qshared
< bg < game), mirroring which Raven header owns it. `*mut`-only types are forward-declared
now (a faithful C forward-decl) and fully ported later.

See also: [[porting-rules]], [[workspace-architecture]], `GENTITY_TYPE_FOLLOWUPS.md`.

## Legend
- **Port** = full faithful `#[repr(C)]` port with `size_of`/`offset_of!` asserts.
- **Fwd** = forward-declared opaque (`*mut`-only at all current use sites); full port deferred.
- Status: ☐ todo · ◐ in progress · ☑ done (cargo-verified)

## MP

### `mp_qshared` (q_shared.h)
| Type / const | Oracle (codemp) | Kind | Value/size | MP |
|---|---|---|---|---|
| `MAX_CLIENTS` | `game/q_shared.h:1985` (non-Xbox) | const | `32` | ☑ `shared/limits.rs` |
| `MAX_STRING_CHARS` | `game/q_shared.h:380` | const | `1024` | ☑ `shared/limits.rs` |
| `MAX_SABERS` | `game/q_shared.h:841` | const | `2` | ☑ `qcommon/saber/` |
| `saber_colors_t` | `game/q_shared.h:575-588` | `typedef int` + enum | ☑ | ☑ |
| `saberType_t` | `game/q_shared.h:601-631` | enum | ☑ | ☑ |
| `saber_styles_t` | `game/q_shared.h:671-683` | enum | ☑ | ☑ |
| `saberTrail_t` | `game/q_shared.h:633-650` | struct (116 B) | ☑ | ☑ |
| `bladeInfo_t` (+`MAX_BLADES=8`) | `game/q_shared.h:652-670` | struct (204 B) | ☑ | ☑ |
| `saberInfo_t` | `game/q_shared.h:735-840` | struct (2156 B, by-value in `gclient_s.saber[]`) | ☑ full port | ☑ |

### `mp_bg` (bg_public.h / bg_vehicles.h)
| Type / const | Oracle (codemp) | Kind | Value/size | MP |
|---|---|---|---|---|
| `MAX_SPAWN_VARS` | `game/bg_public.h:16` | const | `64` | ☑ `public/spawn.rs` |
| `MAX_SPAWN_VARS_CHARS` | `game/bg_public.h:17` | const | `4096` | ☑ `public/spawn.rs` |
| `team_t` (+`TEAM_NUM_TEAMS`) | `game/bg_public.h:1008-1017` | `typedef int` + enum | ☑ `public/team.rs` |
| `Vehicle_t` | `game/bg_vehicles.h:477-623` (~146 ln) | struct (`*mut` only via `gentity_s`) | **Fwd — deferred** (not needed by client/level; `gentity_s.m_pVehicle` stays `*mut c_void` in qshared) |

### `mp_game` (ai.h / teams.h / b_public.h)
| Type / const | Oracle (codemp) | Kind | Value/size | MP |
|---|---|---|---|---|
| `class_t` | `game/teams.h:17-77` | enum (56 variants) | ☑ `teams/class.rs` |
| `npcteam_t` | `game/teams.h:4-14` | `typedef int` + enum | ☑ `teams/npcteam.rs` |
| `lookMode_t` | `game/b_public.h:70-75` | enum (`LM_ENT`,`LM_INTEREST`) | ☑ `npc/look_mode.rs` |
| `MAX_FRAME_GROUPS` | `game/ai.h:85` | const `32` | ☑ `ai/consts.rs` |
| `MAX_GROUP_MEMBERS` | `game/ai.h:95` | const `32` | ☑ `ai/consts.rs` |
| `NUM_SQUAD_STATES` | `game/ai.h:19-29` (anon enum) | const `7` | ☑ `ai/consts.rs` |
| `AIGroupMember_t` | `game/ai.h:87-93` | struct (16 B) | ☑ `ai/group_member.rs` |
| `AIGroupInfo_t` | `game/ai.h:97-116` | struct (ptrs → align 8, 624 B) | ☑ `ai/group_info.rs` |
| `gNPC_t` | `game/b_public.h:…-264` (large) | struct (`*mut` only via `gentity_s`) | **Fwd — deferred** (not needed by client/level; `gentity_s.NPC` stays `*mut c_void` in qshared) |

### `mp_qshared::gentity` (reconcile, not new)
| Task | Detail | MP |
|---|---|---|
| Layout asserts | Port `size_of`/`offset_of!` asserts from dormant `src/game/entity` into qshared `gentity.rs` (first parity contract for `gentity_t`) | ☐ |
| Entity consts | `FL_*`, full `HL_*` list, `MOVER_*`, `DAMAGEREDIRECT_*` → `mp_game::entity` (game-private) | ☐ |

## SP — investigated (3 scouts) + ported

SP structures diverge from MP. Investigated via oracle scouts; ported to how **SP** actually
defines it (SP tiering follows SP's headers: `team_t`/`class_t` in `teams.h` → `sp_game`;
spawn-vars in `g_local.h` → `sp_game`; saber in `q_shared.h` → `sp_qshared`).

### `sp_qshared`
| Type / const | SP oracle (`code/…`) | Divergence vs MP | SP |
|---|---|---|---|
| `MAX_CLIENTS` | `game/q_shared.h:1447` | **1** (single-player; MP 32) | ☑ `shared/limits.rs` |
| `MAX_STRING_CHARS` | `game/q_shared.h:206` | same (1024) | ☑ `shared/limits.rs` |
| `saber_colors_t` | `game/q_shared.h:474-483` | **named enum**, no `NUM_SABER_COLORS` (MP: `typedef int`) | ☑ |
| `saberType_t` / `saber_styles_t` | `game/q_shared.h:1561`,`1660` | identical | ☑ |
| `saberTrail_t` | `game/q_shared.h:1616-1630` | **92 B** — no `dualbase`/`dualtip` (MP 116) | ☑ |
| `bladeInfo_t` (+`MAX_BLADES=8`) | `game/q_shared.h:1634-1658` | **164 B** — no `desiredLength`/3 debounce ints (MP 204) | ☑ |
| `saberInfo_t` (+`MAX_SABERS=2`) | `game/q_shared.h:1724-1944` | **1952 B, pointer-bearing** — `char*` name/model/skin/broken1/2, `char[]` shaders, SP-only `fallSound[3]` (MP 2156, buffers/handles) | ☑ |

### `sp_game`
| Type / const | SP oracle (`code/…`) | Divergence vs MP | SP |
|---|---|---|---|
| `team_t` | `game/teams.h:4-13` | **named enum** `FREE/PLAYER/ENEMY/NEUTRAL` (MP `typedef int`, diff values) | ☑ `teams/team.rs` |
| `class_t` | `game/teams.h:18-88` | **64 variants**, SP story-NPC roster (MP 56, different) | ☑ `teams/class.rs` |
| `npcteam_t` | — | **absent in SP** — folded into `team_t` | ☑ n/a |
| `lookMode_t` | `game/b_public.h:91-95` | identical | ☑ `npc/look_mode.rs` |
| `AIGroupMember_t` | `game/ai.h:96-102` | identical (16 B) | ☑ `ai/group_member.rs` |
| `AIGroupInfo_t` | `game/ai.h:106-125` | byte-identical layout (624 B); `team` is SP `team_t` | ☑ `ai/group_info.rs` |
| ai consts | `game/ai.h:18,94,104` | identical (32/32/7) | ☑ `ai/consts.rs` |
| `MAX_SPAWN_VARS` | `game/g_local.h:143` | same value (64), **game-tier** in SP | ☑ `local/spawn.rs` |
| `MAX_SPAWN_VARS_CHARS` | `game/g_local.h:144` | **2048** (MP 4096) | ☑ `local/spawn.rs` |

### SP deferred
| Type | Note | SP |
|---|---|---|
| `Vehicle_t` | Not needed yet (SP `gentity` is an opaque stub); SP vehicle system TBD | ◐ deferred |
| `gNPC_t` | SP NPC struct is large; SP `gentity` opaque; no consumer yet | ◐ deferred |
| `saberInfoRetail_t` | SP-only savegame-compat struct; port with SP savegame system | ◐ deferred |

## Related pre-existing gaps (out of scope, tracked)
- SP `gentity_t`, `playerState_t` are opaque stubs; SP `entity_shared.rs` / `collision.rs`
  are mis-provenanced MP copies. See scout findings / `GENTITY_TYPE_FOLLOWUPS.md`.
