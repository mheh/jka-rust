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

#### Wave 3 batch — `cgame/tr_types.h` (ported into `common/mp/cgame/`, cargo-green)
All 9 qshared-tier types (`stereoFrame_t` pre-existed in `sp_abi`):
- **Trivial**: `color4ub_t` (alias), `refEntityType_t`, `textureCompression_t`
- **Medium**: `polyVert_t`(24), `poly_s`(16), `miniRefEntity_s`(108)
- **Heavy** (offset-asserted): `glconfig_t`(96), `refEntity_t`(216),
  `refdef_t`(384)

#### Wave 4 batch — `game/g_public.h` (ported into `common/mp/qcommon/`, cargo-green)
23 seam types beside the `parms_t`/`failedEdge_e` precedent: the 17
`T_G_ICARUS_*` VM-transport structs (12–4104 B, offset-asserted),
`sharedEntity_t`(976 — engine-side `gentity_t` view; `m_pVehicle` stays opaque
below the bg tier), and the seam enums `gameImport_t`/`gameExport_t`/`bSet_t`/
`bState_t`/`taskID_t`.

### `mp_bg` (bg_public.h / bg_vehicles.h / bg_weapons.h)
| Type / const | Oracle (codemp) | Kind | Value/size | MP |
|---|---|---|---|---|
| `MAX_SPAWN_VARS` | `game/bg_public.h:16` | const | `64` | ☑ `public/spawn.rs` |
| `MAX_SPAWN_VARS_CHARS` | `game/bg_public.h:17` | const | `4096` | ☑ `public/spawn.rs` |
| `team_t` (+`TEAM_NUM_TEAMS`) | `game/bg_public.h:1008-1017` | `typedef int` + enum | ☑ `public/team.rs` |
| `gametype_t` | `game/bg_public.h:183-199` | `typedef int` + anon enum (`GT_FFA`..`GT_MAX_GAME_TYPE`) | ☑ `public/gametype/mod.rs` |
| `powerup_t` | `game/bg_public.h:652-684` | `typedef int` + anon enum (`PW_NONE`..`PW_NUM_POWERUPS`, "may not have more than 16") | ☑ `public/powerup/mod.rs` |
| `Vehicle_t` | `game/bg_vehicles.h:477-623` | struct (976 B, ptr+fn-ptr fields) | ☑ `vehicles/vehicle_s.rs` (Wave 2; was **Fwd-deferred**, now full-ported — `gentity_s.m_pVehicle` still stays `*mut c_void` in `mp_qshared`, which sits below `mp_bg` and can't name `Vehicle_t` directly) |

#### Wave 2 batch — `bg_public.h` (ported into `public/`, cargo-green)
- **Enums** (`#[repr(i32)]`): `animEventType_t`, `brokenLimb_t`, `ctfMsg_t`,
  `duelTeam_t`, `effectTypes_t`, `entityType_t`, `entity_event_t`, `fieldtype_t`,
  `footstepType_t`, `forceHandAnims_t`, `g2ModelParts_t`, `gender_t`,
  `global_team_sound_t`, `meansOfDeath_t`, `pdSounds_t`, `persEnum_t`,
  `pmtype_t`, `saberQuadrant_t`, `statIndex_t`, `teamtask_t`, `weaponstate_t`
- **`typedef int` + consts**: `holdable_t`, `saberMoveName_t`
- **Structs** (`#[repr(C)]`+size assert): `animation_s`(7 B, `repr(C,packed)`
  per Raven's `#pragma pack(push,1)`), `animevent_s`(32 B), `BG_field_t`(24 B),
  `bgLoadedAnim_t`(72 B), `bgLoadedEvents_t`(19272 B), `saberMoveData_t`(48 B),
  `bgEntity_s`(576 B — shared head-of-`gentity_t`/`centity_t` view),
  `pmove_t`(336 B — ghoul2 bolt/gametype/duel-loss fields vs SP's simpler
  block)

#### Wave 2 batch — `bg_weapons.h` (ported into `weapons/`, cargo-green)
- `ammo_t` (enum), `weapon_t` (`typedef int` + consts), `ammoData_s`(4 B),
  `weaponData_s`(56 B)

### `mp_uishared` (ui/ui_shared.h) — Wave 3, **complete**
All 15 types, cargo-green, offset-asserted:
- **Medium**: `rectDef_t`(16), `scriptDef_t`(104), `colorRangeDef_t`(24),
  `columnInfo_s`(12), `commandDef_t`(16), `editFieldDef_s`(28),
  `modelDef_s`(136)
- **Heavy**: `windowDef_t`(192), `listBoxDef_s`(240), `multiDef_s`(648),
  `itemDef_s`(704), `menuDef_t`(2400), `cachedAssets_t`(272),
  `textScrollDef_s`(2072), `displayContextDef_t`(872)

#### Wave 2 batch — `bg_vehicles.h` (ported into `vehicles/`, cargo-green)
- **Enums**: `EWeaponPose`, `vehFlags_t`, `vehicleType_t`
- **Structs**: `vehTurretStatus_t`(20 B), `vehWeaponStats_t`(28 B),
  `vehWeaponStatus_t`(16 B), `turretStats_t`(96 B), `vehWeaponInfo_t`(104 B,
  ptr-bearing), `vehicleInfo_t`(952 B, ptr+fn-ptr "virtual interface" table),
  `Vehicle_t`(976 B, see Wave-2 note in the table above)

#### Wave 4 batch — `bg_saga.h` + `bg_local.h` (ported into `saga/`, `local/`, cargo-green)
- **saga/**: `siegeClassDesc_t`(4096), `siegeClass_t`(1548), `siegeTeam_t`(648,
  real `*mut siegeClass_t` roster), `siegeClassFlags_t`, `siegePlayerClassFlags_t`
- **local/**: `pml_t`(132 — pmove-internal scratch state)

### `mp_abi` seam batches — Wave 4, cargo-green
- **cgame/public/** (cg_public.h remainder, beside the pre-existing
  `shared_buffer.rs` TCG* set): `snapshot_t`(139352 — embeds
  `entityState_t[256]` + `playerState_t`), `TCGIncomingConsoleCommand`(1024),
  `TCGPositionOnBolt`(272), 6 `ragCallback*` structs, `cgameImport_t`/
  `cgameExport_t`
- **ui/public/** (ui_public.h): `uiClientState_t`(3084), `uiImport_t`,
  `uiExport_t`, `uiMenuCommand_t`

### `mp_cgame` (cg_local.h / cg_lights.h) — Wave 4, **complete**
All 27 `cg_local.h` types in `local/` + `clightstyle_t`(264) in `lights/`,
offset-asserted. Heavies: `cg_t`(295424), `cgs_t`(229576), `centity_t`(1984,
real `*mut Vehicle_t`), `clientInfo_t`(5920), `cgMedia_t`(1716),
`cgEffects_t`(356), `localEntity_s`(472), `weaponInfo_t`(232, typed
`centity_t` trail callbacks), `lerpFrame_t`(80), `markPoly_s`(304),
`playerEntity_t`(264).

### `mp_ui` (ui_local.h / keycodes.h) — Wave 4, **complete**
All 30 `ui_local.h` types in `local/` + `fakeAscii_t` in `keycodes/`,
offset-asserted. Heavies: `uiInfo_t`(342384), `serverStatus_s`(11484),
`playerInfo_t`(11056), `playerSpeciesInfo_t`(7760), `menuframework_s`(2096),
`pendingServerStatus_t`(2244), `serverStatusInfo_t`(5288), the Q3 menu-widget
family (`menucommon_s`(88) embedded by value in `menuaction/bitmap/list/
radiobutton/slider/text/field`), `mfield_t`(272), `uiClientState_t` → abi.

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
| `gNPC_t` | `game/b_public.h:116-264` | struct (896 B) | ☑ `npc/g_npc_t.rs` (Wave 4 — full-ported; `gentity_s.NPC` still `*mut c_void` in qshared by tier) |

#### Wave 4 batch — `ai_main.h` / `b_public.h` / `b_local.h` / `say.h` / `w_saber.h` (cargo-green)
- **botai/**: `bot_state_t`(5096 — embeds the existing `level/bot_settings`
  port), `bot_ctf_state_t`, `bot_siege_state_t`, `bot_teamplay_state_t`,
  `botattachment_s`(68), `boteventtracker_s`(16), `botskills_s`(24),
  `nodeobject_s`(28)
- **npc/**: `gNPCstats_e`(68), `navInfo_s`(88), `jumpState_t`, `spot_t`,
  `visibility_t`
- **saber/**: `evasionType_t` · **say/**: `saying_t`

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
- **SP `playerState_t`** ~~remains a stub~~ — **full-ported in Wave 3**
  (offset-asserted, 4992 B, matches the v7 clang ground truth), incl.
  `saber[MAX_SABERS]` by value.

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
| `playerState_t` | `game/q_shared.h:2066-2361` | **4992 B** — SP-only forceData/vehicle-less block; embeds `saberInfo_t[2]` by value (MP layout differs heavily) | ☑ `qcommon/player_state.rs` (Wave 3) |
| `gentity_s` | `game/g_shared.h:514-825` | **1496 B** — SP concrete entity (MP's lives in q_shared tier as 976 B `sharedEntity_t` split). Game-tier by-value enums (`material_t`/`moverState_t`/`team_t`) stay documented `c_int` aliases; gclient/gNPC/Vehicle ptrs opaque per MP precedent | ☑ `common/sp/gentity.rs` (Wave 4 — stub replaced) |

#### Wave 3 batch — `renderer/tr_types.h` (ported into `common/sp/renderer/`, cargo-green)
8 types, SP-as-diff vs MP `cgame/tr_types.h` (no `miniRefEntity_s` in SP):
- **Trivial**: `color4ub_t`, `refEntityType_t`, `textureCompression_t`
- **Medium**: `polyVert_t`(24), `poly_s`(16), `refdef_t`(116 — vs MP 384)
- **Heavy** (offset-asserted): `glconfig_t`(96), `refEntity_t`(176 — vs MP 216)
- `sp_abi`'s opaque `refdef_t` stub replaced by a re-export of the real layout.

### `sp_uishared` (ui/ui_shared.h) — Wave 3, **complete**
All 15 crate-owned types (`pc_token_s` lives in `sp_qshared`), cargo-green,
offset-asserted. Divergences vs MP: `displayContextDef_t`(792 vs 872),
`menuDef_t`(1568 vs 2400), `multiDef_s`(1288 vs 648), `cachedAssets_t`(212 vs
272), `itemDef_s`(712 vs 704), `windowDef_t`(208 vs 192); no `scriptDef_t`.

### `sp_bg` (bg_public.h)
| Type / const | SP oracle (`code/…`) | Divergence vs MP | SP |
|---|---|---|---|
| `powerup_t` | `game/bg_public.h:248-267` | **named enum**, member set diverges heavily from MP (`PW_HASTE`, `PW_UNCLOAKING`, `PW_DISRUPTION`, `PW_GALAK_SHIELD`, `PW_SEEKER`, `PW_SHOCKED`, `PW_DRAINED`, `PW_INVINCIBLE`, `PW_FORCE_PUSH*` replace MP's flag/force-power powerups; MP is `typedef int` + anon enum) | ☑ `public/powerup/mod.rs` |

#### Wave 2 SP `bg_public.h` batch (ported into `public/`, cargo-green)
| Type | SP oracle (`code/…`) | Divergence vs MP | SP |
|---|---|---|---|
| `animEventType_t` | `game/bg_public.h:520-532` | identical member set (incl. `AEV_SABER_SWING`/`AEV_SABER_SPIN`) | ☑ |
| `animation_s` | `game/bg_public.h:468-475` | **8 B, not packed** — adds a `glaIndex` byte (MP: 7 B `repr(C,packed)`, no `glaIndex`) | ☑ |
| `animevent_s` | `game/bg_public.h:537-545` | **40 B** — adds `modelOnly`/`glaIndex` fields, `MAX_RANDOM_ANIM_SOUNDS=8` (MP: 32 B, `=4`) | ☑ |
| `entityType_t` | `game/bg_public.h:713-732` | **15 variants**, no `ET_HOLOCRON`/`ET_NPC`/`ET_TEAM`/`ET_BODY`/`ET_FX` (MP 19) | ☑ |
| `entity_event_t` | `game/bg_public.h:283-465` | **142 variants** — SP story/NPC/AI sound events replace MP's CTF/vehicle events (MP 192) | ☑ |
| `footstepType_t` | `game/bg_public.h:550-557` | identical (4 + terminator) | ☑ |
| `meansOfDeath_t` | `game/bg_public.h:560-617` | same count (46) but reordered/renamed members (e.g. `MOD_BRYAR` vs MP `MOD_BRYAR_PISTOL`) | ☑ |
| `persEnum_t` | `game/bg_public.h:195-208` | **10 variants**, no rank/impressive/excellent/defend/assist/gauntlet/capture stats (MP 15) | ☑ |
| `pmtype_t` | `game/bg_public.h:63-70` | **6 variants**, no `PM_JETPACK`/`PM_FLOAT`/`PM_SPINTERMISSION` (MP 9) | ☑ |
| `weaponstate_t` | `game/bg_public.h:72-80` | identical (7 variants incl. `WEAPON_IDLE`) | ☑ |
| `pmove_t` | `game/bg_public.h:130-163` | **248 B** — `gent: *mut gentity_t` (SP's concrete entity type) instead of MP's `bgEntity_t*`/ghoul2-bolt/gametype/duel-loss fields; `trace` callback carries an extra `eG2TraceType` param (MP 336 B) | ☑ `public/pmove_t.rs` |

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
| `animFileSet_t` (+`MAX_ANIM_EVENTS=300`) | `game/g_local.h:68-76` | **absent in MP** — SP-only per-model anim config, embeds `animation_t[MAX_ANIMATIONS]`/`animevent_t[MAX_ANIM_EVENTS]` (36416 B) | ☑ `local/anim_file_set_t.rs` |

### `sp_game` — Wave 4, **complete** (80 types across 18 header folders)
- **shared/** (g_shared.h): `gclient_t`(7384), `clientSession_t`(1036, embeds
  `missionStats_s`(228) + `objectives_s`(8)), `clientPersistant_t`(128),
  `clientInfo_t`(496), `renderInfo_t`(468), `weaponInfo_t`(160 — cgame-tier
  `centity_t` callback param stays opaque), `playerTeamState_t`(44) + the
  g_shared enums (`material_t`, `moverState_t`, `movetype_t`, `taskID_t`,
  `targetModel_t`, `clientConnected_t`, `playerTeamStateState_t`,
  `saberBlock*_t`)
- **local/** (g_local.h): `level_locals_t`(620536, real `*mut gclient_t`),
  `alertEvent_s`(56) + level enums, `combatPoint_t`(28), `interestPoint_t`(24),
  `waypointData_t`(324), `reference_tag_s`(64)
- **vehicles/** (G_Vehicles.h — SP vehicle system un-deferred): `Vehicle_t`
  (1760 vs MP 976), `vehicleInfo_t`(904 vs MP 952), `Muzzle`(32),
  `turretStats_t`(96), `vehWeapon*` set, `vehicleType_t`, `EWeaponPose`
- **saber/** (wp_saber.h): `saberMoveName_t`(316-variant enum),
  `saberMoveData_t`(48, typed chain fields), `saberQuadrant_t`,
  `sabersLockMode_t`, `saberLockResult_t`, `evasionType_t`, `swingType_t`
- **weapons/** (weapons.h): `weaponData_t`(1536 vs MP 56!), `ammoData_t`(36),
  `weapon_t`, `ammo_t`
- **npc/** (b_public.h/b_local.h): `gNPC_t`(984), `navInfo_s`(1120),
  `gNPCstats_e`(72), `jumpState_t`, `sexType_t`, `spot_t`, `visibility_t`
- **functions/** (g_functions.h): the 8 savegame fn-enums (`thinkFunc_t`,
  `useFunc_t`, `painFunc_t`, `dieFunc_t`, `touchFunc_t`, `blockedFunc_t`,
  `reachedFunc_t`, `clThinkFunc_t`)
- **one-header folders**: `roff/`(5 structs), `objectives/`(3 enums),
  `fields/`(`save_field_t`+`fieldtypeSAVE_t`), `characters/`, `hitlocs/`,
  `events/`, `say/`, `bset/`, `bstate/`, `dmstates/`

### `sp_abi` seam batches — Wave 4, cargo-green
- **game/public/** (g_public.h): `game_import_t`(1048, ~150 fn ptrs — matches
  v7 clang ground truth), `game_export_t`(144), `SavedGameJustLoaded_e`;
  `CMiniHeap`/`CRagDoll*` params stay opaque (C++ track).
  `G2API_SetBoneAngles` uses real `Eorientations` (new `sp_qshared::shared`
  re-export from `native_math`)
- **cgame/public/**: `snapshot_s`(144328), `cgameImport_t`
- **ui/public/**: `uiimport_t`(528 fn table, `R_LerpTag` takes real
  `*mut orientation_t`), `uiImport_t`, `dpTypes_t`

### `sp_cgame` (cg_local.h / cg_media.h / cg_camera.h / cg_lights.h) — Wave 4, **complete**
`local/`: 13 types (`cg_t` 321248, `centity_t` 488, `localEntity_s` 336,
`markPoly_s` 304, `screengraphics_s` 72, `lerpFrame_t` 56 + enums).
`media/`: `cgs_t`(35232 — `clientInfo_t` embed is an ABI-sized blob + TODO,
game-tier type unreachable from cgame), `cgMedia_t`(1640), `cgEffects_t`(136),
`HUDMenuItem_s`(56), `footstep_t`, `otherhudbits_t`. `camera/`:
`camera_t`(500). `lights/`: `clightstyle_t`(264).

### `sp_ui` (ui_local.h / gameinfo.h) — Wave 4, **complete**
`local/`: `uiInfo_t`(251568), `playerSpeciesInfo_t`(7760), `uifield_t`(288),
`uiStatic_t`(144), `modInfo_t`. `gameinfo/`: `gameinfo_import_t`(72).

### SP deferred
| Type | Note | SP |
|---|---|---|
| `Vehicle_t` | ~~deferred~~ **ported in Wave 4** (`sp_game::vehicles`, 1760 B) | ☑ |
| `gNPC_t` | ~~deferred~~ **ported in Wave 4** (`sp_game::npc`, 984 B) | ☑ |
| `saberInfoRetail_t` | SP-only savegame-compat struct; port with SP savegame system | ◐ deferred |
| `animNumber_t` | `anims.h:1789`; ~1500-entry enum. `animFileSet_t.animations[MAX_ANIMATIONS]` sized from the verified packet offset (12344 B / 8) instead, pending this enum's port | ◐ deferred |

## Engine tier — Wave 5, **complete** (263 types, all ☑)

### `mp_engine_qcommon` / `sp_engine_qcommon` (MP 77 / SP 67)
`qcommon/`: `msg_t`, `netadr_t`, `netchan_t` (MP 98364 B / SP 17448 B),
MP `huff_t`/`huffman_t`/`nodetype`, `sysEvent_t`, wire enums
(`netsrc_t`/`netadrtype_t`/`svc_ops_e`/`clc_ops_e`, MP `sharedTraps_t`/
`vmInterpret_t`), `xcommand_t`. `qfiles/`: full disk formats — BSP lumps
(`dheader_t`…`dsurface_t`), MD3 (`md3Header_t`…), `dfontdat_t`(7180),
`pcx_t`/`_TargaHeader`, `vmHeader_t`, SP `hunkAllocType_t`. `cm/`:
`clipMap_t`, `cGrid_t`(199708), `facet_t`, `patchCollide_t`, `winding_t`,
`traceWork_t` (MP 296 / SP 1376), `CCMShader`, `leafList_t` (fn-ptr field
typed). `files/`: `pack_t`, `searchpath_t`, `fileHandleData_t`, `qfile_us/gus`.
MP `vm/`: `vm_t`, `opcode_t`, `vmSymbol_t`, `vmptr_t`. `miniheap/`
`CMiniHeap`(24), SP `hstring/` `hstring`(4), `timing/` `timing_c` (win32
`rdtsc` fields compiled out in clang ground truth — documented), MP `gp2/`
`TGPGroup`/`TGPValue`/`TGenericParser2` void* handles.

### `mp_engine_server` / `sp_engine_server` (8 each)
`server/`: `server_t` (MP 664960 B / SP 397528 B), `client_t` (MP 332960 /
SP 100048, embeds `netchan_t`), `serverStatic_t`, `clientSnapshot_t`,
`svEntity_t`, `challenge_t`, `serverState_t`/`clientState_t`.

### `mp_engine_botlib` (42 internals; MP-only — SP ships no botlib)
`aasfile/` 16 on-disk AAS types; `be_aas_def/` runtime state (`aas_t` 14272,
`aas_entity_t`, routing cache/links); `be_ai_weight/` fuzzy weights
(`weightconfig_t` 2120); `l_script//l_precomp//l_struct/` lexer (`script_t`,
`token_t`, `source_t` 3184, `define_t`, `indent_t`, field/struct defs);
`l_libvar//l_crc/`; `be_interface/` `botlib_globals_t`.

### `mp_qshared::common::mp::botlib` — game↔engine seam (23)
From `game/botlib.h` + `game/be_*.h` (joins `bot_goal_t`/`aas_areainfo_t`):
fn tables `botlib_export_t`(1104)/`botlib_import_t`/`aas_export_t`/
`ai_export_t`(600)/`ea_export_t`, `bot_input_t`, `bot_entitystate_t`,
`bsp_trace_t`/`bsp_surface_t`, `aas_clientmove_t`, `aas_entityinfo_t`,
`aas_trace_t`, `aas_altroutegoal_t`, `aas_predictroute_t`, `solid_t`,
chat/move/weapon info (`bot_consolemessage_t`, `bot_match_t`,
`weaponinfo_t` 552, `projectileinfo_t`, …).

### ghoul2 — `mp/sp_engine_ghoul2` + `sp_qshared::common::sp::ghoul2`
Faithful subset: `boneInfo_t` (768/760, embeds `mdxaBone_t`), `boltInfo_t`
(MP 64 / SP 16), `surfaceInfo_t`, gore PODs (`SGoreSurface`,
`GoreTextureCoordinates`, `SRagDollEffectorCollision`). SP module-visible
types live at qshared tier (embedded by value in `gentity_t`/`itemDef_s`,
passed in G2API tables): `CGhoul2Info_v` (4 B handle), `EG2_Collision`,
`CRagDollParams`, `CCollisionRecord` (shared-layout alias). Reconciled into
`gentity_t.ghoul2`, `refEntity_t.ghoul2`, `pmove_t.trace`,
`game_import_t`, `displayContextDef_t`, `itemDef_s`.

### icarus — `mp/sp_engine_icarus` (MP 11 / SP 3)
`CBlockMember`/`CBlockStream`(1056) POD-layout classes,
`interface_export_t`(320 fn table), `variable_t`, `keywordArray_t`,
`bstream_t` (→ real `*mut CBlockStream`), `pscript_t`,
`playType_t`/`setType_t`, `vector_t`, `LPTokenizerErrorProc`.

### rmg — `mp/sp_engine_rmg` (2 MP / 3 SP)
`symmetry_t`, `ERMDir` (`DIR_FIRST` as const alias — duplicate
discriminant), SP `CRMAutomapSymbol`.

### Wave 5 C++-track deferrals (idiomatic reimpl, not byte-faithful)
| Group | Types | Why |
|---|---|---|
| Terrain/RMG | `CCMLandScape`, `CCMPatch`, `CArea`, `CPathInfo`, `CRandomTerrain`, `CTerrainMap`(2 MB), `CRM*` classes, `CCGPatch`, `CRandomModel`, `areaType_t` | RMG subsystem; std:: members; OpenJK dropped it entirely |
| GP2 / ROFF / stringed | `CGenericParser2` family, `CROFFSystem`, stringed classes | bases / std:: members; module access is via handles |
| ghoul2 classes | `CGhoul2Info` (std::vector members), `IGhoul2InfoArray` (virtual), `CGoreSet` (multimap), `CRagDollUpdateParams` (virtual), `CBoneCache` (renderer, Wave 7) | not standard-layout |
| icarus managers | `CIcarus`, `CSequencer`, `CTaskManager`, `CBlock`, `CInterpreter`, tokenizer class family | virtual / std:: members |
| net profiling | `INetProfile`/`CNetProfile` | win32-only C++ |
| misc | SP `CMapPoolLow`/`CMapBlock` (hstring pools), `unzip.h` (vendored minizip) | std:: members / vendored |

## Related pre-existing gaps (out of scope, tracked)
- ~~SP `gentity_t` is an opaque stub~~ — full-ported in Wave 4 (1496 B,
  offset-asserted). Remaining: SP `entity_shared.rs` / `collision.rs` are
  mis-provenanced MP copies. See scout findings / `GENTITY_TYPE_FOLLOWUPS.md`.
- Cross-tier `c_int` aliases with `//TODO: Port` markers, kept deliberately
  (ABI-identical; real type lives above the referencing crate): SP `gentity.rs`
  (`material_t`/`moverState_t`/`team_t`), SP `cgs_t.clientinfo` blob, SP
  `weaponInfo_t` cgame-callback params, MP `sharedEntity_t.m_pVehicle`.
  Wave 5 resolved the ghoul2 ones (`gentity_t.ghoul2` et al. are real types
  now); still opaque by tier: sp_abi `G2API_CollisionDetect` `*mut CMiniHeap`
  (engine-tier type) and `CGhoul2Info`/`CRagDollUpdateParams` pointers
  (C++ track).
