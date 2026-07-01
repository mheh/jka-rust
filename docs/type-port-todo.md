# Type-port TODO — game-code dependency types (MP + SP)

Tracks the types the MP `g_local.h` data model depends on, ported into `crates/mp/game`.
Each was ported for MP, then investigated + ported for SP (SP structures differ).

**Status:** MP `g_local.h` data model **complete** — `mp_game::{client, entity, level}`
(`gclient_s`=7344, `level_locals_t`=47176, `renderInfo_t`=368, all friend types) ported
fresh from oracle and cargo-validated with size/offset asserts. The `spectatorState_t`,
`playerTeamStateState_t`, and `alertEvent*` enums were corrected to real `typedef enum`s
(the retired `src/` scaffolding had wrongly flattened them to `int`). Remaining `mp_game`
work is the gameplay *logic* (`g_*.c`), not types.

Placement principle: put each type in the **lowest tier that needs it** (native < qshared
< bg < game), mirroring which Raven header owns it. `*mut`-only types are forward-declared
now (a faithful C forward-decl) and fully ported later.

See also: [[porting-rules]], [[workspace-architecture]], `GENTITY_TYPE_FOLLOWUPS.md`.

## Legend
- **Port** = full faithful `#[repr(C)]` port with `size_of`/`offset_of!` asserts.
- **Fwd** = forward-declared opaque (`*mut`-only at all current use sites); full port deferred.
- Status: ☐ todo · ◐ in progress · ☑ done (cargo-verified)

## native (Tier -1, cross-mode — identical MP/SP)

Wave 0 foundation. Only genuinely Raven-free types that are byte-identical in both
trees live here; anything that diverges MP/SP stays per-mode in `qshared`.

### `native_math` (q_math section of q_shared.h)
| Type | Oracle (MP / SP) | Kind | native |
|---|---|---|---|
| `vec_t`,`vec2_t`..`vec5_t`,`vec3pair_t` | `q_shared.h:530-537` / `:314-320` | alias | ☑ `math/vector.rs` |
| `ivec3_t`,`ivec4_t`,`ivec5_t` | `q_shared.h:539-541` / `:323-325` | alias | ☑ `math/vector.rs` |
| `fixed4_t`,`fixed8_t`,`fixed16_t` | `q_shared.h:543-545` / `:327-329` | alias | ☑ `math/vector.rs` |
| `orientation_t` | `q_shared.h:1926` / `:1409` | struct (48 B) | ☑ `math/orientation.rs` |
| `Eorientations` | `q_shared.h:3086` / `:2641` | enum (order X,Z,Y) | ☑ `math/eorientations.rs` |

### `native_types` (scalar/handle primitives)
| Type | Oracle (MP / SP) | Kind | native |
|---|---|---|---|
| `byte`,`word`,`ulong` | `q_shared.h:349-351` / `:173-176` | alias | ☑ `types/lib.rs` |
| `qboolean` (+`QFALSE`/`QTRUE`) | `q_shared.h:353` / `:180` | alias `c_int` | ☑ `types/lib.rs` |
| `qhandle_t`,`thandle_t`,`fxHandle_t`,`sfxHandle_t`,`fileHandle_t`,`clipHandle_t` | `q_shared.h:358-363` / `:183-188` | alias | ☑ `types/lib.rs` |
| `mdxaBone_t` | `q_shared.h:3078` / `mdx_format.h:137` | struct | ☑ `types/lib.rs` |
| `MAX_QPATH` | `q_shared.h:393` / `:215` | const `64` | ☑ `types/lib.rs` |

Divergent — deliberately **not** native (kept per-mode): `ivec2_t` (SP-only),
`qint64` (MP-only), `LPCSTR` (SP win32-ism). `native_containers` = C++ track
(idiomatic reimpl); `native_platform` = replacement, not ported.

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

#### Wave 1 trivial batch — `q_shared.h` (ported into `shared/`, cargo-green)
25 batchable types, enum-vs-alias verified against oracle:
- **Enums** (`#[repr(i32)]`): `cbufExec_t`, `WL_e`, `printParm_t`, `errorParm_t`,
  `ha_pref`, `saberBlockType_t`, `saberBlockedType_t`, `sharedERagPhase`,
  `sharedERagEffector`, `sharedEIKMoveState`, `ct_table_t`, `fsOrigin_t`,
  `trackchan_t`, `itemUseFail_t`, `genCmds_t`, `connstate_t`, `ForceReload_e`
- **`typedef int` + consts**: `forcePowers_t`, `soundChannel_t`, `e_status`,
  `flagStatus_t`
- **Structs** (`#[repr(C)]`+size assert): `vec3struct_t`(12, MP `__LCC__`),
  `qint64`(8, MP-only), `markFragment_t`(8), `stringID_table_t`(16),
  `wpneighbor_t`(8)

#### Wave 1 heavy batch — `q_shared.h` (offset-asserted, cargo-green)
- VM-arg effect structs: `addpolyArgStruct_t`(180), `addbezierArgStruct_t`(136),
  `addspriteArgStruct_t`(72), `effectTrailVertStruct_t`(84),
  `effectTrailArgStruct_t`(348), `addElectricityArgStruct_t`(92)
- `gameState_t`(22804) + `MAX_CONFIGSTRINGS`(1700)/`MAX_GAMESTATE_CHARS`(16000)
- `wpobject_t`(300) + `MAX_NEIGHBOR_SIZE`(32)

MP `q_shared.h` now **complete**. Deferred to their tiers: `cvar_t` (engine),
`SSkinGoreData`/`mdxaBone_t` (ghoul2; `mdxaBone_t` already in `native_types`).

### `mp_bg` (bg_public.h / bg_vehicles.h)
| Type / const | Oracle (codemp) | Kind | Value/size | MP |
|---|---|---|---|---|
| `MAX_SPAWN_VARS` | `game/bg_public.h:16` | const | `64` | ☑ `public/spawn.rs` |
| `MAX_SPAWN_VARS_CHARS` | `game/bg_public.h:17` | const | `4096` | ☑ `public/spawn.rs` |
| `team_t` (+`TEAM_NUM_TEAMS`) | `game/bg_public.h:1008-1017` | `typedef int` + enum | ☑ `public/team.rs` |
| `gametype_t` | `game/bg_public.h:183-199` | `typedef int` + anon enum (`GT_FFA`..`GT_MAX_GAME_TYPE`) | ☑ `public/gametype/mod.rs` |
| `powerup_t` | `game/bg_public.h:652-684` | `typedef int` + anon enum (`PW_NONE`..`PW_NUM_POWERUPS`, "may not have more than 16") | ☑ `public/powerup/mod.rs` |
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

#### Wave 1 SP `q_shared.h` batch (ported from SP oracle, cargo-green)
SP `q_shared.h` now **complete** (19 pre-existing + 24 native + 16 trivial + 3
heavy + 3 deferred = 65). Ported from `code/`, not copied from MP:
- **Trivial** (`shared/`): `LPCSTR`(SP-only), `ivec2_t`(SP-only), `cbufExec_t`,
  `printParm_t`, `errorParm_t`, `ct_table_t`(+`CT_TITLE`), `fsOrigin_t`,
  `waterHeightLevel_t`(SP-only), `genCmds_t`(FORCE_* only), `connstate_t`,
  `sharedEIKMoveState`, `ForceReload_e`, `e_status`(named enum), `forcePowers_t`
  (int+consts; no team powers, NUM=16 vs MP 18), `markFragment_t`(8),
  `stringID_table_t`(16)
- **Heavy** (offset-asserted): `gameState_t`(21204; SP `MAX_CONFIGSTRINGS`=1300),
  `sharedRagDollUpdateParams_t`(52), `parseData_t`(88, SP-only, +`MAX_PARSEFILES`)
- **Deferred** to their tiers: `cvar_t`→engine, `SSkinGoreData`→ghoul2,
  `saberInfoRetail_t`→SP savegame system (C++ retail-compat struct)
- **Pre-existing provenance leaks to reconcile** (cite `codemp/`, not SP
  q_shared.h types): `entity_shared.rs`, `qtime.rs`, `pc_token_t.rs`

#### Wave 1 reconcile decisions (stale markers resolved)
- **MP `gentity_t.client`** stays `*mut c_void` **by design**, not a missing type.
  `gclient_s` is ported (mp_game, 7344 B) but `gentity_t` must live in `mp_qshared`
  because the sub-game abi seam (`mp_abi`, below the game tier) names `*mut
  gentity_t` in ~18 syscalls; `mp_qshared` can't depend on `mp_game`. `*mut c_void`
  is ABI-identical to `*mut gclient_s`. **Future refactor** (tracked): move
  `gentity_t` to `mp_game` and switch the 18 abi syscall structs to an opaque
  entity pointer, restoring the real `*mut gclient_s`.
- **SP `playerState_t`** remains a stub — **deferred full heavy-struct port**
  (~284 lines). No longer blocked: SP `saberInfo_t` is ported in-crate and is
  embedded by value as `saber[MAX_SABERS]`. Marked `//TODO: Port playerState_t`.

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

### `sp_bg` (bg_public.h)
| Type / const | SP oracle (`code/…`) | Divergence vs MP | SP |
|---|---|---|---|
| `powerup_t` | `game/bg_public.h:248-267` | **named enum**, member set diverges heavily from MP (`PW_HASTE`, `PW_UNCLOAKING`, `PW_DISRUPTION`, `PW_GALAK_SHIELD`, `PW_SEEKER`, `PW_SHOCKED`, `PW_DRAINED`, `PW_INVINCIBLE`, `PW_FORCE_PUSH*` replace MP's flag/force-power powerups; MP is `typedef int` + anon enum) | ☑ `public/powerup/mod.rs` |

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
