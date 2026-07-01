# Oracle type index

Mechanically extracted type definitions from the Raven headers under
`oracle/oracle/`. Separate MP (`codemp/`) and SP (`code/`) sections, mirroring
how the Rust crates are split. Line = the definition's opening declaration line
(clickable `file:line`).

**Scope:** header files (`*.h`) only. Type definitions living in `.cpp` files are
not indexed here. Forward declarations are skipped in favour of the definition
site. Anonymous types (`typedef enum {...} X` with no name, bare `enum {...}`
const groups) are not types and are omitted.

**Kind:** `struct` / `enum` / `union` / `typedef (alias)` (e.g. `typedef int team_t`)
/ `fn-ptr typedef` / `class (C++)`.

**Totals:** MP 1334 types across 155 headers · SP 1238 types across 198 headers.

---

## MP (`codemp/`)

### `codemp/RMG`

#### `codemp/RMG/RM_Area.h`

| Type | Kind | Line |
|------|------|------|
| `CRMArea` | class (C++) | 17 |
| `rmAreaVector_t` | typedef (alias) | 74 |
| `CRMAreaManager` | class (C++) | 76 |

#### `codemp/RMG/RM_Headers.h`

| Type | Kind | Line |
|------|------|------|
| `symmetry_t` | enum | 29 |

#### `codemp/RMG/RM_Instance.h`

| Type | Kind | Line |
|------|------|------|
| `CRMInstance` | class (C++) | 25 |
| `rmInstanceIter_t` | typedef (alias) | 119 |
| `rmInstanceList_t` | typedef (alias) | 120 |

#### `codemp/RMG/RM_InstanceFile.h`

| Type | Kind | Line |
|------|------|------|
| `CRMInstanceFile` | class (C++) | 11 |

#### `codemp/RMG/RM_Instance_BSP.h`

| Type | Kind | Line |
|------|------|------|
| `CRMBSPInstance` | class (C++) | 9 |

#### `codemp/RMG/RM_Instance_Group.h`

| Type | Kind | Line |
|------|------|------|
| `CRMGroupInstance` | class (C++) | 9 |

#### `codemp/RMG/RM_Instance_Random.h`

| Type | Kind | Line |
|------|------|------|
| `CRMRandomInstance` | class (C++) | 11 |

#### `codemp/RMG/RM_Instance_Void.h`

| Type | Kind | Line |
|------|------|------|
| `CRMVoidInstance` | class (C++) | 9 |

#### `codemp/RMG/RM_Manager.h`

| Type | Kind | Line |
|------|------|------|
| `CRMManager` | class (C++) | 9 |

#### `codemp/RMG/RM_Mission.h`

| Type | Kind | Line |
|------|------|------|
| `rmIntVector_t` | typedef (alias) | 12 |
| `CRMMission` | class (C++) | 15 |

#### `codemp/RMG/RM_Objective.h`

| Type | Kind | Line |
|------|------|------|
| `CRMObjective` | class (C++) | 9 |
| `rmObjectiveIter_t` | typedef (alias) | 61 |
| `rmObjectiveList_t` | typedef (alias) | 62 |

#### `codemp/RMG/RM_Path.h`

| Type | Kind | Line |
|------|------|------|
| `ERMDir` | enum | 24 |
| `CRMNode` | class (C++) | 41 |
| `rmNodeVector_t` | typedef (alias) | 72 |
| `CRMLoc` | class (C++) | 75 |
| `rmLocVector_t` | typedef (alias) | 110 |
| `CRMCell` | struct | 114 |
| `rmCellVector_t` | typedef (alias) | 132 |
| `CRMPathManager` | class (C++) | 135 |

#### `codemp/RMG/RM_Terrain.h`

| Type | Kind | Line |
|------|------|------|
| `CRandomModel` | class (C++) | 7 |
| `CCGHeightDetails` | class (C++) | 30 |
| `CCGPatch` | class (C++) | 50 |
| `CRMLandScape` | class (C++) | 59 |

### `codemp/Ratl`

#### `codemp/Ratl/bits_vs.h`

| Type | Kind | Line |
|------|------|------|
| `bits_vs` | class (C++) | 37 |

#### `codemp/Ratl/ratl_common.h`

| Type | Kind | Line |
|------|------|------|
| `alignStruct` | struct | 120 |
| `T` | class (C++) | 152 |
| `compile_assert` | class (C++) | 261 |
| `ratl_base` | class (C++) | 300 |
| `bits_base` | class (C++) | 319 |
| `ratl_compare` | struct | 463 |
| `bits_true` | class (C++) | 478 |

#### `codemp/Ratl/vector_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 37 |

### `codemp/Ravl`

#### `codemp/Ravl/CVec.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 51 |
| `ESide` | enum | 71 |
| `CVec4` | class (C++) | 88 |
| `CVec3` | class (C++) | 559 |

### `codemp/Splines`

#### `codemp/Splines/math_angles.h`

| Type | Kind | Line |
|------|------|------|
| `vec3_p` | typedef (alias) | 12 |
| `angles_t` | class (C++) | 14 |

#### `codemp/Splines/math_matrix.h`

| Type | Kind | Line |
|------|------|------|
| `mat3_t` | class (C++) | 18 |

#### `codemp/Splines/math_quaternion.h`

| Type | Kind | Line |
|------|------|------|
| `quat_t` | class (C++) | 11 |

#### `codemp/Splines/math_vector.h`

| Type | Kind | Line |
|------|------|------|
| `idVec3_t` | class (C++) | 104 |
| `Bounds` | class (C++) | 357 |
| `idVec2_t` | class (C++) | 484 |
| `vec4_t` | class (C++) | 506 |
| `idVec5_t` | class (C++) | 536 |

#### `codemp/Splines/q_shared.h`

| Type | Kind | Line |
|------|------|------|
| `qboolean` | enum | 202 |
| `byte` | typedef (alias) | 207 |
| `qhandle_t` | typedef (alias) | 211 |
| `sfxHandle_t` | typedef (alias) | 212 |
| `fileHandle_t` | typedef (alias) | 213 |
| `clipHandle_t` | typedef (alias) | 214 |
| `jointHandle_t` | enum | 216 |
| `cbufExec_t` | enum | 258 |
| `errorParm_t` | enum | 274 |
| `vec3_p` | typedef (alias) | 330 |
| `vec3_c` | typedef (alias) | 331 |
| `angles_p` | typedef (alias) | 334 |
| `angles_c` | typedef (alias) | 335 |
| `mat3_p` | typedef (alias) | 338 |
| `mat3_c` | typedef (alias) | 339 |
| `fsMode_t` | enum | 590 |
| `fsOrigin_t` | enum | 597 |
| `ePair_t` | struct | 653 |
| `mapSide_t` | struct | 658 |
| `mapBrush_t` | struct | 664 |
| `patchVertex_t` | struct | 669 |
| `mapPatch_t` | struct | 674 |
| `mapModel_t` | struct | 680 |
| `mapPrimitive_t` | struct | 685 |
| `mapEntity_t` | struct | 695 |
| `mapFile_t` | struct | 703 |
| `drawVert_t` | struct | 720 |
| `drawVertMesh_t` | struct | 728 |
| `growList_t` | struct | 753 |

#### `codemp/Splines/splines.h`

| Type | Kind | Line |
|------|------|------|
| `fileHandle_t` | typedef (alias) | 15 |
| `idPointListInterface` | class (C++) | 23 |
| `idSplineList` | class (C++) | 143 |
| `idVelocity` | struct | 330 |
| `idCameraPosition` | class (C++) | 343 |
| `idFixedPosition` | class (C++) | 459 |
| `idInterpolatedPosition` | class (C++) | 514 |
| `idSplinePosition` | class (C++) | 600 |
| `idCameraFOV` | class (C++) | 666 |
| `idCameraEvent` | class (C++) | 721 |
| `idCameraDef` | class (C++) | 792 |

#### `codemp/Splines/util_list.h`

| Type | Kind | Line |
|------|------|------|
| `type` | class (C++) | 7 |

#### `codemp/Splines/util_str.h`

| Type | Kind | Line |
|------|------|------|
| `strdata` | class (C++) | 16 |
| `idStr` | class (C++) | 45 |

### `codemp/botlib`

#### `codemp/botlib/aasfile.h`

| Type | Kind | Line |
|------|------|------|
| `aas_bbox_t` | struct | 97 |
| `aas_reachability_t` | struct | 107 |
| `aas_areasettings_t` | struct | 119 |
| `aas_portal_t` | struct | 132 |
| `aas_portalindex_t` | typedef (alias) | 141 |
| `aas_cluster_t` | struct | 144 |
| `aas_vertex_t` | typedef (alias) | 154 |
| `aas_plane_t` | struct | 157 |
| `aas_edge_t` | struct | 165 |
| `aas_edgeindex_t` | typedef (alias) | 171 |
| `aas_face_t` | struct | 174 |
| `aas_faceindex_t` | typedef (alias) | 185 |
| `aas_area_t` | struct | 188 |
| `aas_node_t` | struct | 200 |
| `aas_lump_t` | struct | 210 |
| `aas_header_t` | struct | 217 |

#### `codemp/botlib/be_aas_def.h`

| Type | Kind | Line |
|------|------|------|
| `aas_stringindex_t` | struct | 43 |
| `aas_link_t` | struct | 50 |
| `bsp_link_t` | struct | 59 |
| `bsp_entdata_t` | struct | 67 |
| `aas_entity_t` | struct | 78 |
| `aas_settings_t` | struct | 88 |
| `aas_routingcache_t` | struct | 133 |
| `aas_routingupdate_t` | struct | 150 |
| `aas_reversedlink_t` | struct | 163 |
| `aas_reversedreachability_t` | struct | 171 |
| `aas_reachabilityareas_t` | struct | 178 |
| `aas_t` | struct | 183 |

#### `codemp/botlib/be_ai_weight.h`

| Type | Kind | Line |
|------|------|------|
| `fuzzyseperator_t` | struct | 19 |
| `weight_t` | struct | 32 |
| `weightconfig_t` | struct | 39 |

#### `codemp/botlib/be_interface.h`

| Type | Kind | Line |
|------|------|------|
| `botlib_globals_t` | struct | 19 |

#### `codemp/botlib/l_crc.h`

| Type | Kind | Line |
|------|------|------|
| `crc_t` | typedef (alias) | 10 |

#### `codemp/botlib/l_libvar.h`

| Type | Kind | Line |
|------|------|------|
| `libvar_t` | struct | 16 |

#### `codemp/botlib/l_precomp.h`

| Type | Kind | Line |
|------|------|------|
| `define_t` | struct | 55 |
| `indent_t` | struct | 71 |
| `source_t` | struct | 80 |
| `pc_token_t` | struct | 149 |

#### `codemp/botlib/l_script.h`

| Type | Kind | Line |
|------|------|------|
| `punctuation_t` | struct | 133 |
| `token_t` | struct | 141 |
| `script_t` | struct | 158 |

#### `codemp/botlib/l_struct.h`

| Type | Kind | Line |
|------|------|------|
| `fielddef_t` | struct | 31 |
| `structdef_t` | struct | 43 |

### `codemp/cgame`

#### `codemp/cgame/cg_lights.h`

| Type | Kind | Line |
|------|------|------|
| `clightstyle_t` | struct | 5 |

#### `codemp/cgame/cg_local.h`

| Type | Kind | Line |
|------|------|------|
| `footstep_t` | enum | 90 |
| `impactSound_t` | enum | 120 |
| `lerpFrame_t` | struct | 137 |
| `playerEntity_t` | struct | 168 |
| `clientInfo_t` | struct | 196 |
| `cgLoopSound_t` | struct | 324 |
| `centity_t` | struct | 333 |
| `markPoly_t` | struct | 470 |
| `leType_t` | enum | 481 |
| `leFlag_t` | enum | 498 |
| `leMarkType_t` | enum | 505 |
| `leBounceSoundType_t` | enum | 511 |
| `localEntity_t` | struct | 519 |
| `score_t` | struct | 630 |
| `weaponInfo_t` | struct | 652 |
| `itemInfo_t` | struct | 708 |
| `powerupInfo_t` | struct | 723 |
| `skulltrail_t` | struct | 730 |
| `chatBoxItem_t` | struct | 748 |
| `cg_t` | struct | 755 |
| `forceTicPos_t` | struct | 1018 |
| `cgscreffects_t` | struct | 1030 |
| `cgMedia_t` | struct | 1067 |
| `cgEffects_t` | struct | 1385 |
| `cgs_t` | struct | 1516 |
| `siegeExtended_t` | struct | 1611 |
| `q3print_t` | enum | 2373 |

#### `codemp/cgame/cg_public.h`

| Type | Kind | Line |
|------|------|------|
| `snapshot_t` | struct | 20 |
| `cgameImport_t` | enum | 56 |
| `cgameExport_t` | enum | 352 |
| `autoMapInput_t` | struct | 442 |
| `TCGPointContents` | struct | 452 |
| `TCGGetBoltData` | struct | 459 |
| `TCGImpactMark` | struct | 468 |
| `TCGVectorData` | struct | 484 |
| `TCGTrace` | struct | 491 |
| `TCGG2Mark` | struct | 499 |
| `TCGIncomingConsoleCommand` | struct | 507 |
| `TCGCameraShake` | struct | 513 |
| `TCGMiscEnt` | struct | 522 |
| `TCGPositionOnBolt` | struct | 528 |
| `ragCallbackDebugBox_t` | struct | 542 |
| `ragCallbackDebugLine_t` | struct | 550 |
| `ragCallbackBoneSnap_t` | struct | 560 |
| `ragCallbackBoneImpact_t` | struct | 567 |
| `ragCallbackBoneInSolid_t` | struct | 574 |
| `ragCallbackTraceLine_t` | struct | 582 |

#### `codemp/cgame/tr_types.h`

| Type | Kind | Line |
|------|------|------|
| `color4ub_t` | typedef (alias) | 69 |
| `polyVert_t` | struct | 71 |
| `poly_t` | struct | 77 |
| `refEntityType_t` | enum | 83 |
| `miniRefEntity_t` | struct | 100 |
| `refEntity_t` | struct | 133 |
| `refdef_t` | struct | 257 |
| `stereoFrame_t` | typedef (alias) | 283 |
| `textureCompression_t` | enum | 293 |
| `glconfig_t` | struct | 299 |

### `codemp/client`

#### `codemp/client/BinkVideo.h`

| Type | Kind | Line |
|------|------|------|
| `BinkVideo` | class (C++) | 26 |

#### `codemp/client/FxPrimitives.h`

| Type | Kind | Line |
|------|------|------|
| `EMatImpactEffect` | enum | 101 |
| `CEffect` | class (C++) | 108 |
| `CTrail` | class (C++) | 174 |
| `CLight` | class (C++) | 215 |
| `CParticle` | class (C++) | 267 |
| `CFlash` | class (C++) | 351 |
| `CLine` | class (C++) | 377 |
| `CBezier` | class (C++) | 397 |
| `CElectricity` | class (C++) | 426 |
| `COrientedParticle` | class (C++) | 450 |
| `CTail` | class (C++) | 469 |
| `CCylinder` | class (C++) | 499 |
| `CEmitter` | class (C++) | 532 |
| `CPoly` | class (C++) | 576 |

#### `codemp/client/FxScheduler.h`

| Type | Kind | Line |
|------|------|------|
| `CMediaHandles` | class (C++) | 66 |
| `CFxRange` | class (C++) | 91 |
| `EPrimType` | enum | 120 |
| `CPrimitiveTemplate` | class (C++) | 152 |
| `SEffectTemplate` | struct | 346 |
| `CFxScheduler` | class (C++) | 373 |

#### `codemp/client/FxSystem.h`

| Type | Kind | Line |
|------|------|------|
| `SFxHelper` | class (C++) | 49 |

#### `codemp/client/client.h`

| Type | Kind | Line |
|------|------|------|
| `clSnapshot_t` | struct | 25 |
| `outPacket_t` | struct | 58 |
| `clientActive_t` | struct | 75 |
| `rmAutomapSymbol_t` | struct | 143 |
| `clientConnection_t` | struct | 166 |
| `ping_t` | struct | 247 |
| `serverInfo_t` | struct | 257 |
| `serverAddress_t` | struct | 290 |
| `clientStatic_t` | struct | 295 |
| `console_t` | struct | 358 |
| `kbutton_t` | struct | 479 |

#### `codemp/client/fffx.h`

| Type | Kind | Line |
|------|------|------|
| `ffFX_e` | enum | 13 |

#### `codemp/client/keycodes.h`

| Type | Kind | Line |
|------|------|------|
| `fakeAscii_t` | enum | 6 |

#### `codemp/client/keys.h`

| Type | Kind | Line |
|------|------|------|
| `qkey_t` | struct | 3 |
| `field_t` | struct | 12 |
| `keyGlobals_t` | struct | 19 |
| `keyname_t` | struct | 36 |

#### `codemp/client/snd_ambient.h`

| Type | Kind | Line |
|------|------|------|
| `set_e` | enum | 33 |
| `setKeyword_e` | enum | 42 |
| `ambientSet_t` | struct | 60 |
| `parseFunc_t` | fn-ptr typedef | 75 |
| `CSetGroup` | class (C++) | 80 |

#### `codemp/client/snd_local.h`

| Type | Kind | Line |
|------|------|------|
| `portable_samplepair_t` | struct | 30 |
| `SoundCompressionMethod_t` | enum | 38 |
| `sfx_t` | struct | 48 |
| `dma_t` | struct | 67 |
| `STREAMINGBUFFER` | struct | 80 |
| `channel_t` | struct | 94 |
| `wavinfo_t` | struct | 137 |

#### `codemp/client/snd_local_console.h`

| Type | Kind | Line |
|------|------|------|
| `streamHandle_t` | typedef (alias) | 19 |
| `wavinfo_t` | struct | 25 |
| `sfx_t` | struct | 42 |
| `channel_t` | struct | 52 |

#### `codemp/client/snd_mp3.h`

| Type | Kind | Line |
|------|------|------|
| `id3v1_1` | struct | 15 |

#### `codemp/client/snd_music.h`

| Type | Kind | Line |
|------|------|------|
| `MusicState_e` | enum | 11 |

### `codemp/client/OpenAL`

#### `codemp/client/OpenAL/alc.h`

| Type | Kind | Line |
|------|------|------|
| `ALCdevice` | typedef (alias) | 23 |
| `ALCcontext` | typedef (alias) | 24 |

#### `codemp/client/OpenAL/alctypes.h`

| Type | Kind | Line |
|------|------|------|
| `ALCdevice` | typedef (alias) | 31 |
| `ALCcontext` | typedef (alias) | 34 |
| `ALCboolean` | typedef (alias) | 38 |
| `ALCbyte` | typedef (alias) | 41 |
| `ALCubyte` | typedef (alias) | 44 |
| `ALCshort` | typedef (alias) | 47 |
| `ALCushort` | typedef (alias) | 50 |
| `ALCuint` | typedef (alias) | 53 |
| `ALCint` | typedef (alias) | 56 |
| `ALCfloat` | typedef (alias) | 59 |
| `ALCdouble` | typedef (alias) | 62 |
| `ALCsizei` | typedef (alias) | 65 |
| `ALCvoid` | typedef (alias) | 68 |
| `ALCenum` | typedef (alias) | 71 |

#### `codemp/client/OpenAL/altypes.h`

| Type | Kind | Line |
|------|------|------|
| `ALboolean` | typedef (alias) | 30 |
| `ALbyte` | typedef (alias) | 33 |
| `ALubyte` | typedef (alias) | 36 |
| `ALshort` | typedef (alias) | 39 |
| `ALushort` | typedef (alias) | 42 |
| `ALuint` | typedef (alias) | 45 |
| `ALint` | typedef (alias) | 48 |
| `ALfloat` | typedef (alias) | 51 |
| `ALdouble` | typedef (alias) | 54 |
| `ALsizei` | typedef (alias) | 57 |
| `ALvoid` | typedef (alias) | 60 |
| `ALenum` | typedef (alias) | 63 |

### `codemp/client/eax`

#### `codemp/client/eax/EaxMan.h`

| Type | Kind | Line |
|------|------|------|
| `EMPOINT` | struct | 24 |
| `LPEMPOINT` | typedef (alias) | 29 |
| `LISTENERATTRIBUTES` | struct | 31 |
| `LPLISTENERATTRIBUTES` | typedef (alias) | 36 |
| `SOURCEATTRIBUTES` | struct | 38 |
| `LPSOURCEATTRIBUTES` | typedef (alias) | 51 |
| `MATERIALATTRIBUTES` | struct | 53 |
| `LPMATERIALATTRIBUTES` | typedef (alias) | 59 |
| `DIFFRACTIONBOX` | struct | 64 |
| `LPDIFFRACTIONBOX` | typedef (alias) | 69 |
| `LPEAXMANAGER` | typedef (alias) | 78 |

#### `codemp/client/eax/eax.h`

| Type | Kind | Line |
|------|------|------|
| `FAR` | typedef (alias) | 44 |
| `GUID` | struct | 56 |
| `EAXSet` | fn-ptr typedef | 78 |
| `EAXGet` | fn-ptr typedef | 79 |
| `EAXCONTEXTPROPERTIES` | struct | 122 |
| `LPEAXCONTEXTPROPERTIES` | struct | 122 |
| `EAXSOURCEPROPERTIES` | struct | 143 |
| `LPEAXSOURCEPROPERTIES` | struct | 143 |
| `EAXSOURCEALLSENDPROPERTIES` | struct | 168 |
| `LPEAXSOURCEALLSENDPROPERTIES` | struct | 168 |
| `EAXACTIVEFXSLOTS` | struct | 182 |
| `LPEAXACTIVEFXSLOTS` | struct | 182 |
| `EAXOBSTRUCTIONPROPERTIES` | struct | 188 |
| `LPEAXOBSTRUCTIONPROPERTIES` | struct | 188 |
| `EAXOCCLUSIONPROPERTIES` | struct | 195 |
| `LPEAXOCCLUSIONPROPERTIES` | struct | 195 |
| `EAXEXCLUSIONPROPERTIES` | struct | 204 |
| `LPEAXEXCLUSIONPROPERTIES` | struct | 204 |
| `EAXSOURCESENDPROPERTIES` | struct | 211 |
| `LPEAXSOURCESENDPROPERTIES` | struct | 211 |
| `EAXSOURCEOCCLUSIONSENDPROPERTIES` | struct | 219 |
| `LPEAXSOURCEOCCLUSIONSENDPROPERTIES` | struct | 219 |
| `EAXSOURCEEXCLUSIONSENDPROPERTIES` | struct | 229 |
| `LPEAXSOURCEEXCLUSIONSENDPROPERTIES` | struct | 229 |
| `EAXFXSLOTPROPERTIES` | struct | 248 |
| `LPEAXFXSLOTPROPERTIES` | struct | 248 |
| `EAXVECTOR` | struct | 259 |
| `EAXCONTEXT_PROPERTY` | enum | 298 |
| `EAXFXSLOT_PROPERTY` | enum | 375 |
| `EAXSOURCE_PROPERTY` | enum | 429 |
| `EAXREVERB_PROPERTY` | enum | 590 |
| `EAXREVERBPROPERTIES` | struct | 696 |
| `LPEAXREVERBPROPERTIES` | struct | 696 |
| `EAXAGCCOMPRESSOR_PROPERTY` | enum | 843 |
| `EAXAGCCOMPRESSORPROPERTIES` | struct | 857 |
| `LPEAXAGCCOMPRESSORPROPERTIES` | struct | 857 |
| `EAXAUTOWAH_PROPERTY` | enum | 882 |
| `EAXAUTOWAHPROPERTIES` | struct | 899 |
| `LPEAXAUTOWAHPROPERTIES` | struct | 899 |
| `EAXCHORUS_PROPERTY` | enum | 941 |
| `EAXCHORUSPROPERTIES` | struct | 967 |
| `LPEAXCHORUSPROPERTIES` | struct | 967 |
| `EAXDISTORTION_PROPERTY` | enum | 1018 |
| `EAXDISTORTIONPROPERTIES` | struct | 1036 |
| `LPEAXDISTORTIONPROPERTIES` | struct | 1036 |
| `EAXECHO_PROPERTY` | enum | 1082 |
| `EAXECHOPROPERTIES` | struct | 1100 |
| `LPEAXECHOPROPERTIES` | struct | 1100 |
| `EAXEQUALIZER_PROPERTY` | enum | 1147 |
| `EAXEQUALIZERPROPERTIES` | struct | 1170 |
| `LPEAXEQUALIZERPROPERTIES` | struct | 1170 |
| `EAXFLANGER_PROPERTY` | enum | 1241 |
| `EAXFLANGERPROPERTIES` | struct | 1267 |
| `LPEAXFLANGERPROPERTIES` | struct | 1267 |
| `EAXFREQUENCYSHIFTER_PROPERTY` | enum | 1318 |
| `EAXFREQUENCYSHIFTERPROPERTIES` | struct | 1342 |
| `LPEAXFREQUENCYSHIFTERPROPERTIES` | struct | 1342 |
| `EAXVOCALMORPHER_PROPERTY` | enum | 1378 |
| `EAXVOCALMORPHERPROPERTIES` | struct | 1412 |
| `LPEAXVOCALMORPHERPROPERTIES` | struct | 1412 |
| `EAXPITCHSHIFTER_PROPERTY` | enum | 1463 |
| `EAXPITCHSHIFTERPROPERTIES` | struct | 1478 |
| `LPEAXPITCHSHIFTERPROPERTIES` | struct | 1478 |
| `EAXRINGMODULATOR_PROPERTY` | enum | 1509 |
| `EAXRINGMODULATORPROPERTIES` | struct | 1533 |
| `LPEAXRINGMODULATORPROPERTIES` | struct | 1533 |

### `codemp/game`

#### `codemp/game/ai.h`

| Type | Kind | Line |
|------|------|------|
| `distance_e` | enum | 5 |
| `attack_e` | enum | 12 |
| `rank_t` | enum | 31 |
| `AIGroupMember_t` | struct | 87 |
| `AIGroupInfo_t` | struct | 97 |

#### `codemp/game/ai_main.h`

| Type | Kind | Line |
|------|------|------|
| `bot_ctf_state_t` | enum | 81 |
| `bot_siege_state_t` | enum | 92 |
| `bot_teamplay_state_t` | enum | 100 |
| `botattachment_t` | struct | 109 |
| `nodeobject_t` | struct | 115 |
| `boteventtracker_t` | struct | 130 |
| `botskills_t` | struct | 137 |
| `bot_state_t` | struct | 148 |

#### `codemp/game/anims.h`

| Type | Kind | Line |
|------|------|------|
| `animNumber_t` | enum | 6 |

#### `codemp/game/b_local.h`

| Type | Kind | Line |
|------|------|------|
| `navInfo_t` | struct | 314 |

#### `codemp/game/b_public.h`

| Type | Kind | Line |
|------|------|------|
| `visibility_t` | enum | 68 |
| `spot_t` | enum | 69 |
| `lookMode_t` | enum | 71 |
| `jumpState_t` | enum | 77 |
| `gNPCstats_t` | struct | 86 |
| `gNPC_t` | struct | 116 |

#### `codemp/game/be_aas.h`

| Type | Kind | Line |
|------|------|------|
| `solid_t` | enum | 59 |
| `aas_trace_t` | struct | 68 |
| `aas_entityinfo_t` | struct | 107 |
| `aas_areainfo_t` | struct | 135 |
| `aas_clientmove_t` | struct | 162 |
| `aas_altroutegoal_t` | struct | 180 |
| `aas_predictroute_t` | struct | 196 |

#### `codemp/game/be_ai_chat.h`

| Type | Kind | Line |
|------|------|------|
| `bot_consolemessage_t` | struct | 29 |
| `bot_matchvariable_t` | struct | 39 |
| `bot_match_t` | struct | 45 |

#### `codemp/game/be_ai_goal.h`

| Type | Kind | Line |
|------|------|------|
| `bot_goal_t` | struct | 25 |

#### `codemp/game/be_ai_move.h`

| Type | Kind | Line |
|------|------|------|
| `bot_initmove_t` | struct | 60 |
| `bot_moveresult_t` | struct | 74 |
| `bot_avoidspot_t` | struct | 89 |

#### `codemp/game/be_ai_weap.h`

| Type | Kind | Line |
|------|------|------|
| `projectileinfo_t` | struct | 27 |
| `weaponinfo_t` | struct | 45 |

#### `codemp/game/bg_lib.h`

| Type | Kind | Line |
|------|------|------|
| `size_t` | typedef (alias) | 6 |
| `va_list` | typedef (alias) | 8 |
| `void` | typedef (alias) | 30 |

#### `codemp/game/bg_local.h`

| Type | Kind | Line |
|------|------|------|
| `pml_t` | struct | 15 |

#### `codemp/game/bg_public.h`

| Type | Kind | Line |
|------|------|------|
| `g2ModelParts_t` | enum | 126 |
| `forceHandAnims_t` | enum | 149 |
| `brokenLimb_t` | enum | 172 |
| `gametype_t` | typedef (alias) | 199 |
| `gender_t` | enum | 201 |
| `animation_t` | struct | 241 |
| `footstepType_t` | enum | 258 |
| `animEventType_t` | enum | 304 |
| `animevent_t` | struct | 318 |
| `bgLoadedAnim_t` | struct | 326 |
| `bgLoadedEvents_t` | struct | 335 |
| `pmtype_t` | enum | 360 |
| `weaponstate_t` | enum | 372 |
| `bgEntity_t` | struct | 423 |
| `pmove_t` | struct | 435 |
| `statIndex_t` | enum | 520 |
| `persEnum_t` | enum | 539 |
| `effectTypes_t` | enum | 627 |
| `powerup_t` | typedef (alias) | 684 |
| `holdable_t` | typedef (alias) | 704 |
| `ctfMsg_t` | enum | 707 |
| `pdSounds_t` | enum | 734 |
| `entity_event_t` | enum | 745 |
| `global_team_sound_t` | enum | 993 |
| `team_t` | typedef (alias) | 1017 |
| `duelTeam_t` | enum | 1019 |
| `teamtask_t` | enum | 1034 |
| `meansOfDeath_t` | enum | 1046 |
| `itemType_t` | typedef (alias) | 1118 |
| `gitem_t` | struct | 1122 |
| `entityType_t` | enum | 1190 |
| `fieldtype_t` | enum | 1231 |
| `BG_field_t` | struct | 1263 |
| `saberMoveName_t` | typedef (alias) | 1482 |
| `saberQuadrant_t` | enum | 1484 |
| `saberMoveData_t` | struct | 1496 |

#### `codemp/game/bg_saga.h`

| Type | Kind | Line |
|------|------|------|
| `siegePlayerClassFlags_t` | enum | 20 |
| `siegeClassFlags_t` | enum | 31 |
| `siegeClassDesc_t` | struct | 49 |
| `siegeClass_t` | struct | 54 |
| `siegeTeam_t` | struct | 82 |

#### `codemp/game/bg_vehicles.h`

| Type | Kind | Line |
|------|------|------|
| `bgEntity_t` | typedef (alias) | 7 |
| `vehicleType_t` | enum | 9 |
| `EWeaponPose` | enum | 20 |
| `vehWeaponInfo_t` | struct | 35 |
| `turretStats_t` | struct | 89 |
| `vehWeaponStats_t` | struct | 112 |
| `vehicleInfo_t` | struct | 131 |
| `vehFlags_t` | enum | 417 |
| `vehWeaponStatus_t` | struct | 450 |
| `vehTurretStatus_t` | struct | 462 |
| `Vehicle_t` | struct | 477 |

#### `codemp/game/bg_weapons.h`

| Type | Kind | Line |
|------|------|------|
| `weapon_t` | typedef (alias) | 40 |
| `ammo_t` | enum | 45 |
| `weaponData_t` | struct | 61 |
| `ammoData_t` | struct | 87 |

#### `codemp/game/botlib.h`

| Type | Kind | Line |
|------|------|------|
| `bot_input_t` | struct | 93 |
| `bsp_surface_t` | struct | 108 |
| `bsp_trace_t` | struct | 117 |
| `bot_entitystate_t` | struct | 134 |
| `botlib_import_t` | struct | 157 |
| `aas_export_t` | struct | 195 |
| `ea_export_t` | struct | 255 |
| `ai_export_t` | struct | 289 |
| `botlib_export_t` | struct | 388 |

#### `codemp/game/g_local.h`

| Type | Kind | Line |
|------|------|------|
| `gentity_t` | typedef (alias) | 16 |
| `gclient_t` | typedef (alias) | 17 |
| `moverState_t` | enum | 89 |
| `gentity_s` | struct | 133 |
| `clientConnected_t` | typedef (alias) | 371 |
| `spectatorState_t` | enum | 373 |
| `playerTeamStateState_t` | enum | 380 |
| `playerTeamState_t` | struct | 385 |
| `clientSession_t` | struct | 412 |
| `clientPersistant_t` | struct | 443 |
| `renderInfo_t` | struct | 460 |
| `gclient_s` | struct | 536 |
| `interestPoint_t` | struct | 754 |
| `combatPoint_t` | struct | 764 |
| `alertEventType_e` | enum | 779 |
| `alertEventLevel_e` | enum | 785 |
| `alertEvent_t` | struct | 794 |
| `waypointData_t` | struct | 810 |
| `level_locals_t` | struct | 820 |
| `reference_tag_t` | struct | 1240 |
| `bot_settings_t` | struct | 1491 |

#### `codemp/game/g_public.h`

| Type | Kind | Line |
|------|------|------|
| `failedEdge_t` | struct | 52 |
| `entityShared_t` | struct | 60 |
| `gameImport_t` | enum | 102 |
| `bState_t` | enum | 584 |
| `taskID_t` | enum | 625 |
| `bSet_t` | enum | 641 |
| `parms_t` | struct | 668 |
| `Vehicle_t` | typedef (alias) | 675 |
| `sharedEntity_t` | struct | 679 |
| `gameExport_t` | enum | 734 |
| `T_G_ICARUS_PLAYSOUND` | struct | 801 |
| `T_G_ICARUS_SET` | struct | 810 |
| `T_G_ICARUS_LERP2POS` | struct | 818 |
| `T_G_ICARUS_LERP2ORIGIN` | struct | 828 |
| `T_G_ICARUS_LERP2ANGLES` | struct | 836 |
| `T_G_ICARUS_GETTAG` | struct | 844 |
| `T_G_ICARUS_LERP2START` | struct | 852 |
| `T_G_ICARUS_LERP2END` | struct | 859 |
| `T_G_ICARUS_USE` | struct | 866 |
| `T_G_ICARUS_KILL` | struct | 872 |
| `T_G_ICARUS_REMOVE` | struct | 878 |
| `T_G_ICARUS_PLAY` | struct | 884 |
| `T_G_ICARUS_GETFLOAT` | struct | 892 |
| `T_G_ICARUS_GETVECTOR` | struct | 900 |
| `T_G_ICARUS_GETSTRING` | struct | 908 |
| `T_G_ICARUS_SOUNDINDEX` | struct | 916 |
| `T_G_ICARUS_GETSETIDFORSTRING` | struct | 920 |

#### `codemp/game/q_shared.h`

| Type | Kind | Line |
|------|------|------|
| `byte` | typedef (alias) | 349 |
| `word` | typedef (alias) | 350 |
| `ulong` | typedef (alias) | 351 |
| `qboolean` | enum | 353 |
| `qhandle_t` | typedef (alias) | 358 |
| `thandle_t` | typedef (alias) | 359 |
| `fxHandle_t` | typedef (alias) | 360 |
| `sfxHandle_t` | typedef (alias) | 361 |
| `fileHandle_t` | typedef (alias) | 362 |
| `clipHandle_t` | typedef (alias) | 363 |
| `cbufExec_t` | enum | 405 |
| `WL_e` | enum | 428 |
| `printParm_t` | enum | 438 |
| `errorParm_t` | enum | 451 |
| `ha_pref` | enum | 504 |
| `vec_t` | typedef (alias) | 530 |
| `vec2_t` | typedef (alias) | 531 |
| `vec3_t` | typedef (alias) | 532 |
| `vec4_t` | typedef (alias) | 533 |
| `vec5_t` | typedef (alias) | 534 |
| `vec3pair_t` | typedef (alias) | 537 |
| `ivec3_t` | typedef (alias) | 539 |
| `ivec4_t` | typedef (alias) | 540 |
| `ivec5_t` | typedef (alias) | 541 |
| `fixed4_t` | typedef (alias) | 543 |
| `fixed8_t` | typedef (alias) | 544 |
| `fixed16_t` | typedef (alias) | 545 |
| `saberBlockType_t` | enum | 552 |
| `saberBlockedType_t` | enum | 558 |
| `saber_colors_t` | typedef (alias) | 588 |
| `forcePowers_t` | typedef (alias) | 613 |
| `saberType_t` | enum | 615 |
| `saberTrail_t` | struct | 633 |
| `bladeInfo_t` | struct | 652 |
| `saber_styles_t` | enum | 672 |
| `saberInfo_t` | struct | 735 |
| `sharedERagPhase` | enum | 856 |
| `sharedERagEffector` | enum | 867 |
| `sharedRagDollParams_t` | struct | 896 |
| `sharedRagDollUpdateParams_t` | struct | 925 |
| `sharedIKMoveParams_t` | struct | 936 |
| `sharedSetBoneIKStateParams_t` | struct | 945 |
| `sharedEIKMoveState` | enum | 960 |
| `material_t` | typedef (alias) | 990 |
| `wpneighbor_t` | struct | 1001 |
| `wpobject_t` | struct | 1007 |
| `ct_table_t` | enum | 1044 |
| `vec3struct_t` | struct | 1389 |
| `pc_token_t` | struct | 1661 |
| `fsMode_t` | enum | 1685 |
| `fsOrigin_t` | enum | 1692 |
| `qint64` | struct | 1726 |
| `cvar_t` | struct | 1804 |
| `cvarHandle_t` | typedef (alias) | 1820 |
| `vmCvar_t` | struct | 1824 |
| `cplane_t` | struct | 1860 |
| `CollisionRecord_t` | struct | 1870 |
| `MAX_G2_COLLISIONS` | typedef (alias) | 1888 |
| `trace_t` | struct | 1894 |
| `markFragment_t` | struct | 1919 |
| `orientation_t` | struct | 1926 |
| `soundChannel_t` | typedef (alias) | 1961 |
| `gameState_t` | struct | 2047 |
| `trackchan_t` | enum | 2056 |
| `forcedata_t` | struct | 2068 |
| `itemUseFail_t` | enum | 2126 |
| `playerState_t` | struct | 2169 |
| `siegePers_t` | struct | 2437 |
| `genCmds_t` | enum | 2488 |
| `usercmd_t` | struct | 2524 |
| `addpolyArgStruct_t` | struct | 2538 |
| `addbezierArgStruct_t` | struct | 2558 |
| `addspriteArgStruct_t` | struct | 2579 |
| `effectTrailVertStruct_t` | struct | 2595 |
| `effectTrailArgStruct_t` | struct | 2615 |
| `addElectricityArgStruct_t` | struct | 2622 |
| `trType_t` | enum | 2644 |
| `trajectory_t` | struct | 2654 |
| `entityState_t` | struct | 2670 |
| `connstate_t` | enum | 2991 |
| `qtime_t` | struct | 3011 |
| `e_status` | typedef (alias) | 3041 |
| `flagStatus_t` | typedef (alias) | 3050 |
| `mdxaBone_t` | struct | 3080 |
| `memtag_t` | typedef (alias) | 3107 |
| `SSkinGoreData` | struct | 3112 |
| `stringID_table_t` | struct | 3154 |
| `ForceReload_e` | enum | 3166 |

#### `codemp/game/say.h`

| Type | Kind | Line |
|------|------|------|
| `saying_t` | enum | 4 |

#### `codemp/game/teams.h`

| Type | Kind | Line |
|------|------|------|
| `npcteam_t` | typedef (alias) | 14 |
| `class_t` | enum | 17 |

#### `codemp/game/w_saber.h`

| Type | Kind | Line |
|------|------|------|
| `evasionType_t` | enum | 44 |

### `codemp/ghoul2`

#### `codemp/ghoul2/G2_gore.h`

| Type | Kind | Line |
|------|------|------|
| `GoreTextureCoordinates` | struct | 10 |
| `SGoreSurface` | struct | 44 |
| `CGoreSet` | class (C++) | 59 |
| `SRagDollEffectorCollision` | struct | 81 |
| `CRagDollUpdateParams` | class (C++) | 94 |
| `CRagDollParams` | class (C++) | 131 |

#### `codemp/ghoul2/ghoul2_shared.h`

| Type | Kind | Line |
|------|------|------|
| `surfaceInfo_t` | struct | 38 |
| `boneInfo_t` | struct | 63 |
| `boltInfo_t` | struct | 170 |
| `goreEnum_t` | enum | 185 |
| `goreEnumShader_t` | struct | 196 |
| `SSkinGoreData` | struct | 202 |
| `surfaceInfo_v` | typedef (alias) | 223 |
| `boneInfo_v` | typedef (alias) | 224 |
| `boltInfo_v` | typedef (alias) | 225 |
| `mdxaBone_v` | typedef (alias) | 226 |
| `CGhoul2Info` | class (C++) | 240 |
| `IGhoul2InfoArray` | class (C++) | 316 |
| `CGhoul2Info_v` | class (C++) | 328 |
| `EG2_Collision` | enum | 465 |

### `codemp/goblib`

#### `codemp/goblib/goblib.h`

| Type | Kind | Line |
|------|------|------|
| `int32` | typedef (alias) | 132 |
| `uint32` | typedef (alias) | 133 |
| `ulong` | typedef (alias) | 137 |
| `byte` | typedef (alias) | 138 |
| `GOBInt32` | typedef (alias) | 140 |
| `GOBUInt32` | typedef (alias) | 141 |
| `GOBChar` | typedef (alias) | 142 |
| `GOBBool` | typedef (alias) | 143 |
| `GOBError` | typedef (alias) | 144 |
| `GOBSeekType` | typedef (alias) | 145 |
| `GOBHandle` | typedef (alias) | 146 |
| `GOBAccessType` | typedef (alias) | 147 |
| `GOBFSHandle` | typedef (alias) | 148 |
| `GOBVoid` | typedef (alias) | 149 |
| `GOBFileSysOpenFunc` | fn-ptr typedef | 151 |
| `GOBFileSysCloseFunc` | fn-ptr typedef | 152 |
| `GOBFileSysReadFunc` | fn-ptr typedef | 153 |
| `GOBFileSysWriteFunc` | fn-ptr typedef | 154 |
| `GOBFileSysSeekFunc` | fn-ptr typedef | 155 |
| `GOBFileSysRenameFunc` | fn-ptr typedef | 156 |
| `GOBMemAllocFunc` | fn-ptr typedef | 158 |
| `GOBMemFreeFunc` | fn-ptr typedef | 159 |
| `GOBCompressFunc` | fn-ptr typedef | 161 |
| `GOBDecompressFunc` | fn-ptr typedef | 162 |
| `GOBCacheFileOpenFunc` | fn-ptr typedef | 164 |
| `GOBCacheFileCloseFunc` | fn-ptr typedef | 165 |
| `GOBCacheFileReadFunc` | fn-ptr typedef | 166 |
| `GOBCacheFileWriteFunc` | fn-ptr typedef | 167 |
| `GOBCacheFileSeekFunc` | fn-ptr typedef | 168 |
| `GOBBlockTableEntry` | struct | 170 |
| `GOBFileTableBasicEntry` | struct | 177 |
| `GOBFileTableExtEntry` | struct | 184 |
| `GOBMemoryFuncSet` | struct | 191 |
| `GOBSingleCodecDesc` | struct | 197 |
| `GOBCodecFuncSet` | struct | 205 |
| `GOBFileSysFuncSet` | struct | 211 |
| `GOBCacheFileFuncSet` | struct | 220 |
| `GOBReadStats` | struct | 229 |
| `GOBProfileReadFunc` | fn-ptr typedef | 283 |
| `GOBProfileFuncSet` | struct | 284 |

### `codemp/icarus`

#### `codemp/icarus/GameInterface.h`

| Type | Kind | Line |
|------|------|------|
| `pscript_t` | struct | 4 |
| `entlist_t` | typedef (alias) | 10 |
| `bufferlist_t` | typedef (alias) | 11 |

#### `codemp/icarus/Q3_Interface.h`

| Type | Kind | Line |
|------|------|------|
| `setType_t` | enum | 6 |
| `playType_t` | enum | 261 |

#### `codemp/icarus/Q3_Registers.h`

| Type | Kind | Line |
|------|------|------|
| `varString_m` | typedef (alias) | 16 |
| `varFloat_m` | typedef (alias) | 17 |

#### `codemp/icarus/blockstream.h`

| Type | Kind | Line |
|------|------|------|
| `vector_t` | typedef (alias) | 24 |
| `CBlockMember` | class (C++) | 38 |
| `CBlock` | class (C++) | 109 |
| `CBlockStream` | class (C++) | 158 |

#### `codemp/icarus/instance.h`

| Type | Kind | Line |
|------|------|------|
| `ICARUS_Instance` | class (C++) | 12 |

#### `codemp/icarus/interface.h`

| Type | Kind | Line |
|------|------|------|
| `DWORD` | typedef (alias) | 9 |
| `vec_t` | typedef (alias) | 11 |
| `vec3_t` | typedef (alias) | 12 |
| `interface_export_t` | struct | 17 |

#### `codemp/icarus/interpreter.h`

| Type | Kind | Line |
|------|------|------|
| `vector_t` | typedef (alias) | 11 |
| `variable_t` | struct | 115 |
| `variable_m` | typedef (alias) | 122 |
| `variable_v` | typedef (alias) | 123 |
| `CInterpreter` | class (C++) | 127 |

#### `codemp/icarus/sequence.h`

| Type | Kind | Line |
|------|------|------|
| `CSequence` | class (C++) | 12 |

#### `codemp/icarus/sequencer.h`

| Type | Kind | Line |
|------|------|------|
| `bstream_t` | struct | 42 |
| `CSequencer` | class (C++) | 68 |

#### `codemp/icarus/taskmanager.h`

| Type | Kind | Line |
|------|------|------|
| `CTask` | class (C++) | 33 |
| `CTaskGroup` | class (C++) | 62 |
| `CTaskManager` | class (C++) | 97 |

#### `codemp/icarus/tokenizer.h`

| Type | Kind | Line |
|------|------|------|
| `byte` | typedef (alias) | 22 |
| `word` | typedef (alias) | 23 |
| `keywordArray_t` | struct | 77 |
| `lessstr` | class (C++) | 83 |
| `CParseStream` | class (C++) | 89 |
| `CToken` | class (C++) | 112 |
| `CCharToken` | class (C++) | 134 |
| `CStringToken` | class (C++) | 148 |
| `CIntToken` | class (C++) | 162 |
| `CFloatToken` | class (C++) | 181 |
| `CIdentifierToken` | class (C++) | 199 |
| `CCommentToken` | class (C++) | 213 |
| `CUserToken` | class (C++) | 227 |
| `CUndefinedToken` | class (C++) | 243 |
| `CSymbol` | class (C++) | 257 |
| `symbolmap_t` | typedef (alias) | 273 |
| `CDirectiveSymbol` | class (C++) | 275 |
| `CIntSymbol` | class (C++) | 292 |
| `CSymbolTable` | class (C++) | 307 |
| `CSymbolLookup` | class (C++) | 326 |
| `CTokenizerState` | class (C++) | 351 |
| `CTokenizerHolderState` | class (C++) | 371 |
| `LPTokenizerErrorProc` | fn-ptr typedef | 384 |
| `CTokenizer` | class (C++) | 387 |
| `CKeywordTable` | class (C++) | 464 |
| `CParsePutBack` | class (C++) | 475 |
| `CParseMemory` | class (C++) | 496 |
| `CParseBlock` | class (C++) | 518 |
| `CParseToken` | class (C++) | 530 |
| `CParseDefine` | class (C++) | 552 |
| `CParseFile` | class (C++) | 567 |

### `codemp/jpeg-6`

#### `codemp/jpeg-6/jchuff.h`

| Type | Kind | Line |
|------|------|------|
| `c_derived_tbl` | struct | 15 |

#### `codemp/jpeg-6/jdct.h`

| Type | Kind | Line |
|------|------|------|
| `DCTELEM` | typedef (alias) | 30 |
| `data` | typedef (alias) | 35 |
| `ISLOW_MULT_TYPE` | typedef (alias) | 56 |
| `IFAST_MULT_TYPE` | typedef (alias) | 58 |
| `FLOAT_MULT_TYPE` | typedef (alias) | 64 |

#### `codemp/jpeg-6/jdhuff.h`

| Type | Kind | Line |
|------|------|------|
| `d_derived_tbl` | struct | 26 |
| `bit_buf_type` | typedef (alias) | 68 |
| `bitread_perm_state` | struct | 78 |
| `bitread_working_state` | struct | 84 |

#### `codemp/jpeg-6/jerror.h`

| Type | Kind | Line |
|------|------|------|
| `J_MESSAGE_CODE` | enum | 33 |

#### `codemp/jpeg-6/jmemsys.h`

| Type | Kind | Line |
|------|------|------|
| `XMSH` | typedef (alias) | 118 |
| `EMSH` | typedef (alias) | 119 |
| `handle_union` | union | 121 |
| `backing_store_ptr` | typedef (alias) | 129 |
| `backing_store_info` | struct | 131 |

#### `codemp/jpeg-6/jmorecfg.h`

| Type | Kind | Line |
|------|------|------|
| `JSAMPLE` | typedef (alias) | 59 |
| `JCOEF` | typedef (alias) | 99 |
| `JOCTET` | typedef (alias) | 110 |
| `UINT8` | typedef (alias) | 135 |
| `UINT16` | typedef (alias) | 147 |
| `INT16` | typedef (alias) | 158 |
| `JDIMENSION` | typedef (alias) | 174 |

#### `codemp/jpeg-6/jpegint.h`

| Type | Kind | Line |
|------|------|------|
| `J_BUF_MODE` | enum | 16 |
| `jpeg_comp_master` | struct | 45 |
| `jpeg_c_main_controller` | struct | 56 |
| `jpeg_c_prep_controller` | struct | 64 |
| `jpeg_c_coef_controller` | struct | 76 |
| `jpeg_color_converter` | struct | 83 |
| `jpeg_downsampler` | struct | 91 |
| `jpeg_forward_dct` | struct | 102 |
| `jpeg_entropy_encoder` | struct | 113 |
| `jpeg_marker_writer` | struct | 120 |
| `jpeg_decomp_master` | struct | 136 |
| `jpeg_input_controller` | struct | 145 |
| `jpeg_d_main_controller` | struct | 157 |
| `jpeg_d_coef_controller` | struct | 165 |
| `jpeg_d_post_controller` | struct | 176 |
| `jpeg_marker_reader` | struct | 188 |
| `jpeg_entropy_decoder` | struct | 211 |
| `output_col` | typedef (alias) | 218 |
| `jpeg_inverse_dct` | struct | 223 |
| `jpeg_upsampler` | struct | 230 |
| `jpeg_color_deconverter` | struct | 244 |
| `jpeg_color_quantizer` | struct | 252 |
| `jvirt_sarray_control` | struct | 385 |
| `jvirt_barray_control` | struct | 386 |

#### `codemp/jpeg-6/jpeglib.h`

| Type | Kind | Line |
|------|------|------|
| `boolean` | typedef (alias) | 20 |
| `JSAMPROW` | typedef (alias) | 83 |
| `JSAMPARRAY` | typedef (alias) | 84 |
| `JSAMPIMAGE` | typedef (alias) | 85 |
| `DCTSIZE2` | typedef (alias) | 87 |
| `JBLOCKROW` | typedef (alias) | 88 |
| `JBLOCKARRAY` | typedef (alias) | 89 |
| `JBLOCKIMAGE` | typedef (alias) | 90 |
| `JCOEFPTR` | typedef (alias) | 92 |
| `JQUANT_TBL` | struct | 100 |
| `JHUFF_TBL` | struct | 116 |
| `jpeg_component_info` | struct | 132 |
| `jpeg_scan_info` | struct | 200 |
| `J_COLOR_SPACE` | enum | 210 |
| `J_DCT_METHOD` | enum | 221 |
| `J_DITHER_MODE` | enum | 236 |
| `jpeg_common_struct` | struct | 256 |
| `j_common_ptr` | typedef (alias) | 264 |
| `j_compress_ptr` | typedef (alias) | 265 |
| `j_decompress_ptr` | typedef (alias) | 266 |
| `jpeg_compress_struct` | struct | 271 |
| `jpeg_decompress_struct` | struct | 410 |
| `jpeg_error_mgr` | struct | 634 |
| `jpeg_progress_mgr` | struct | 692 |
| `jpeg_destination_mgr` | struct | 704 |
| `jpeg_source_mgr` | struct | 716 |
| `jvirt_sarray_ptr` | typedef (alias) | 743 |
| `jvirt_barray_ptr` | typedef (alias) | 744 |
| `jpeg_memory_mgr` | struct | 747 |
| `cinfo` | typedef (alias) | 797 |
| `jvirt_sarray_control` | struct | 1017 |
| `jvirt_barray_control` | struct | 1018 |
| `jpeg_comp_master` | struct | 1019 |
| `jpeg_c_main_controller` | struct | 1020 |
| `jpeg_c_prep_controller` | struct | 1021 |
| `jpeg_c_coef_controller` | struct | 1022 |
| `jpeg_marker_writer` | struct | 1023 |
| `jpeg_color_converter` | struct | 1024 |
| `jpeg_downsampler` | struct | 1025 |
| `jpeg_forward_dct` | struct | 1026 |
| `jpeg_entropy_encoder` | struct | 1027 |
| `jpeg_decomp_master` | struct | 1028 |
| `jpeg_d_main_controller` | struct | 1029 |
| `jpeg_d_coef_controller` | struct | 1030 |
| `jpeg_d_post_controller` | struct | 1031 |
| `jpeg_input_controller` | struct | 1032 |
| `jpeg_marker_reader` | struct | 1033 |
| `jpeg_entropy_decoder` | struct | 1034 |
| `jpeg_inverse_dct` | struct | 1035 |
| `jpeg_upsampler` | struct | 1036 |
| `jpeg_color_deconverter` | struct | 1037 |
| `jpeg_color_quantizer` | struct | 1038 |

### `codemp/mp3code`

#### `codemp/mp3code/config.h`

| Type | Kind | Line |
|------|------|------|
| `socklen_t` | typedef (alias) | 53 |
| `real` | typedef (alias) | 62 |
| `uint8` | typedef (alias) | 66 |
| `int8` | typedef (alias) | 67 |
| `uint16` | typedef (alias) | 75 |
| `int16` | typedef (alias) | 76 |
| `uint32` | typedef (alias) | 90 |
| `int32` | typedef (alias) | 91 |

#### `codemp/mp3code/l3.h`

| Type | Kind | Line |
|------|------|------|
| `HUFF_ELEMENT` | union | 70 |
| `BITDAT` | struct | 103 |
| `GR` | struct | 115 |
| `SIDE_INFO` | struct | 133 |
| `SCALEFACT` | struct | 150 |
| `CB_INFO` | struct | 158 |
| `IS_SF_INFO` | struct | 171 |

#### `codemp/mp3code/mhead.h`

| Type | Kind | Line |
|------|------|------|
| `MPEG_HEAD` | struct | 33 |
| `DEC_INFO` | struct | 57 |

#### `codemp/mp3code/mp3struct.h`

| Type | Kind | Line |
|------|------|------|
| `SBT_FUNCTION` | fn-ptr typedef | 13 |
| `XFORM_FUNCTION` | fn-ptr typedef | 14 |
| `DECODE_FUNCTION` | fn-ptr typedef | 15 |
| `MP3STREAM` | struct | 17 |
| `LP_MP3STREAM` | struct | 17 |

#### `codemp/mp3code/small_header.h`

| Type | Kind | Line |
|------|------|------|
| `SAMPLE` | union | 11 |
| `IN_OUT` | struct | 18 |
| `byte` | typedef (alias) | 27 |

### `codemp/png`

#### `codemp/png/png.h`

| Type | Kind | Line |
|------|------|------|
| `byte` | typedef (alias) | 40 |
| `word` | typedef (alias) | 41 |
| `ulong` | typedef (alias) | 42 |
| `png_ihdr_t` | struct | 47 |
| `png_image_t` | struct | 60 |

### `codemp/qcommon`

#### `codemp/qcommon/GenericParser2.h`

| Type | Kind | Line |
|------|------|------|
| `CTextPool` | class (C++) | 24 |
| `CGPObject` | class (C++) | 45 |
| `CGPValue` | class (C++) | 68 |
| `CGPGroup` | class (C++) | 93 |
| `CGenericParser2` | class (C++) | 136 |
| `TGenericParser2` | typedef (alias) | 166 |
| `TGPGroup` | typedef (alias) | 167 |
| `TGPValue` | typedef (alias) | 168 |

#### `codemp/qcommon/INetProfile.h`

| Type | Kind | Line |
|------|------|------|
| `INetProfile` | class (C++) | 6 |

#### `codemp/qcommon/MiniHeap.h`

| Type | Kind | Line |
|------|------|------|
| `CMiniHeap` | class (C++) | 5 |

#### `codemp/qcommon/RoffSystem.h`

| Type | Kind | Line |
|------|------|------|
| `CROFFSystem` | class (C++) | 35 |

#### `codemp/qcommon/chash.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 11 |

#### `codemp/qcommon/cm_draw.h`

| Type | Kind | Line |
|------|------|------|
| `CPixel32` | class (C++) | 42 |
| `CDraw32` | class (C++) | 86 |

#### `codemp/qcommon/cm_landscape.h`

| Type | Kind | Line |
|------|------|------|
| `areaType_t` | enum | 29 |
| `CArea` | class (C++) | 42 |
| `areaList_t` | typedef (alias) | 72 |
| `areaIter_t` | typedef (alias) | 73 |
| `CCMHeightDetails` | class (C++) | 75 |
| `CCMPatch` | class (C++) | 90 |
| `CCMLandScape` | class (C++) | 135 |

#### `codemp/qcommon/cm_local.h`

| Type | Kind | Line |
|------|------|------|
| `cNode_t` | struct | 19 |
| `cLeaf_t` | struct | 34 |
| `cmodel_t` | struct | 45 |
| `cbrushside_t` | struct | 53 |
| `cbrush_t` | struct | 68 |
| `CCMShader` | class (C++) | 77 |
| `cPatch_t` | struct | 91 |
| `cArea_t` | struct | 99 |
| `clipMap_t` | struct | 107 |
| `sphere_t` | struct | 230 |
| `traceWork_t` | struct | 238 |
| `leafList_t` | struct | 266 |

#### `codemp/qcommon/cm_patch.h`

| Type | Kind | Line |
|------|------|------|
| `patchPlane_t` | struct | 45 |
| `facetLoad_t` | struct | 56 |
| `facet_t` | struct | 64 |
| `patchCollide_t` | struct | 93 |
| `cGrid_t` | struct | 107 |

#### `codemp/qcommon/cm_polylib.h`

| Type | Kind | Line |
|------|------|------|
| `winding_t` | struct | 4 |

#### `codemp/qcommon/cm_randomterrain.h`

| Type | Kind | Line |
|------|------|------|
| `CPathInfo` | class (C++) | 15 |
| `CRandomTerrain` | class (C++) | 52 |

#### `codemp/qcommon/cm_terrainmap.h`

| Type | Kind | Line |
|------|------|------|
| `CTerrainMap` | class (C++) | 17 |

#### `codemp/qcommon/files.h`

| Type | Kind | Line |
|------|------|------|
| `wfhandle_t` | typedef (alias) | 11 |
| `fileInPack_t` | struct | 36 |
| `pack_t` | struct | 42 |
| `directory_t` | struct | 58 |
| `searchpath_t` | struct | 63 |
| `qfile_gut` | union | 71 |
| `qfile_ut` | struct | 78 |
| `fileHandleData_t` | struct | 84 |

#### `codemp/qcommon/fixedmap.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 14 |

#### `codemp/qcommon/hstring.h`

| Type | Kind | Line |
|------|------|------|
| `hstring` | class (C++) | 14 |
| `CMapPoolLow` | class (C++) | 88 |
| `T` | class (C++) | 104 |
| `T1` | class (C++) | 188 |
| `K` | class (C++) | 201 |
| `X` | class (C++) | 217 |

#### `codemp/qcommon/platform.h`

| Type | Kind | Line |
|------|------|------|
| `LPCTSTR` | typedef (alias) | 14 |
| `LPCSTR` | typedef (alias) | 15 |
| `DWORD` | typedef (alias) | 16 |
| `UINT` | typedef (alias) | 17 |
| `HANDLE` | typedef (alias) | 18 |
| `COLORREF` | typedef (alias) | 19 |
| `BYTE` | typedef (alias) | 20 |

#### `codemp/qcommon/qcommon.h`

| Type | Kind | Line |
|------|------|------|
| `msg_t` | struct | 17 |
| `netadrtype_t` | enum | 108 |
| `netsrc_t` | enum | 118 |
| `netadr_t` | struct | 123 |
| `netchan_t` | struct | 163 |
| `svc_ops_e` | enum | 233 |
| `clc_ops_e` | enum | 256 |
| `vm_t` | typedef (alias) | 273 |
| `vmInterpret_t` | enum | 275 |
| `sharedTraps_t` | enum | 281 |
| `xcommand_t` | fn-ptr typedef | 363 |
| `joystickAxis_t` | enum | 913 |
| `sysEventType_t` | enum | 923 |
| `sysEvent_t` | struct | 934 |
| `node_t` | struct | 1047 |
| `huff_t` | struct | 1057 |
| `huffman_t` | struct | 1071 |
| `Lump` | struct | 1104 |

#### `codemp/qcommon/qfiles.h`

| Type | Kind | Line |
|------|------|------|
| `vmHeader_t` | struct | 26 |
| `pcx_t` | struct | 49 |
| `TargaHeader` | struct | 74 |
| `md3Frame_t` | struct | 107 |
| `md3Tag_t` | struct | 114 |
| `md3Surface_t` | struct | 130 |
| `md3Shader_t` | struct | 151 |
| `md3Triangle_t` | struct | 156 |
| `md3St_t` | struct | 160 |
| `md3XyzNormal_t` | struct | 164 |
| `md3Header_t` | struct | 169 |
| `dmodel_t` | struct | 250 |
| `dshader_t` | struct | 258 |
| `dplane_t` | struct | 266 |
| `dnode_t` | struct | 271 |
| `dleaf_t` | struct | 278 |
| `dbrushside_t` | struct | 292 |
| `dbrush_t` | struct | 297 |
| `dfog_t` | struct | 303 |
| `mapVert_t` | struct | 316 |
| `drawVert_t` | struct | 339 |
| `dgrid_t` | struct | 350 |
| `dface_t` | struct | 355 |
| `dpatch_t` | struct | 369 |
| `dtrisurf_t` | struct | 385 |
| `dflare_t` | struct | 396 |
| `lump_t` | struct | 410 |
| `dheader_t` | struct | 434 |
| `mapSurfaceType_t` | enum | 530 |
| `dsurface_t` | struct | 538 |
| `glyphInfo_t` | struct | 574 |
| `dfontdat_t` | struct | 591 |

#### `codemp/qcommon/sparc.h`

| Type | Kind | Line |
|------|------|------|
| `NotSoShort` | struct | 48 |
| `T` | class (C++) | 114 |

#### `codemp/qcommon/sstring.h`

| Type | Kind | Line |
|------|------|------|
| `sstring` | class (C++) | 12 |
| `sstring_t` | typedef (alias) | 115 |

#### `codemp/qcommon/stringed_ingame.h`

| Type | Kind | Line |
|------|------|------|
| `LPCSTR` | typedef (alias) | 40 |

#### `codemp/qcommon/timing.h`

| Type | Kind | Line |
|------|------|------|
| `timing_c` | class (C++) | 2 |

#### `codemp/qcommon/unzip.h`

| Type | Kind | Line |
|------|------|------|
| `unzFile__` | struct | 8 |
| `unzFile` | typedef (alias) | 9 |
| `tm_unz` | struct | 15 |
| `unz_global_info` | struct | 27 |
| `unz_file_info` | struct | 35 |
| `unz_file_info_internal` | struct | 57 |
| `file_in_zip_read_info_s` | struct | 64 |
| `unz_s` | struct | 88 |

#### `codemp/qcommon/vm_local.h`

| Type | Kind | Line |
|------|------|------|
| `opcode_t` | enum | 10 |
| `vmptr_t` | typedef (alias) | 99 |
| `vmSymbol_t` | struct | 101 |
| `vm_s` | struct | 111 |
| `symbolMap_t` | typedef (alias) | 149 |
| `symbolVMMap_t` | typedef (alias) | 150 |

### `codemp/renderer`

#### `codemp/renderer/glext.h`

| Type | Kind | Line |
|------|------|------|
| `alpha` | typedef (alias) | 1542 |
| `mode` | typedef (alias) | 1543 |
| `indices` | typedef (alias) | 1544 |
| `table` | typedef (alias) | 1545 |
| `params` | typedef (alias) | 1546 |
| `width` | typedef (alias) | 1548 |
| `data` | typedef (alias) | 1552 |
| `image` | typedef (alias) | 1554 |
| `height` | typedef (alias) | 1561 |
| `span` | typedef (alias) | 1565 |
| `column` | typedef (alias) | 1566 |
| `values` | typedef (alias) | 1567 |
| `sink` | typedef (alias) | 1573 |
| `target` | typedef (alias) | 1575 |
| `pixels` | typedef (alias) | 1577 |
| `texture` | typedef (alias) | 1620 |
| `s` | typedef (alias) | 1622 |
| `v` | typedef (alias) | 1623 |
| `t` | typedef (alias) | 1630 |
| `r` | typedef (alias) | 1638 |
| `q` | typedef (alias) | 1646 |
| `m` | typedef (alias) | 1664 |
| `invert` | typedef (alias) | 1676 |
| `pass` | typedef (alias) | 1677 |
| `img` | typedef (alias) | 1705 |
| `bias` | typedef (alias) | 1725 |
| `weights` | typedef (alias) | 1748 |
| `border` | typedef (alias) | 1771 |
| `param` | typedef (alias) | 1878 |
| `residences` | typedef (alias) | 1914 |
| `textures` | typedef (alias) | 1916 |
| `priorities` | typedef (alias) | 1919 |
| `points` | typedef (alias) | 1928 |
| `pattern` | typedef (alias) | 1957 |
| `i` | typedef (alias) | 1977 |
| `pointer` | typedef (alias) | 1978 |
| `count` | typedef (alias) | 1979 |
| `void` | typedef (alias) | 2082 |
| `buffer` | typedef (alias) | 2083 |
| `marker_p` | typedef (alias) | 2084 |
| `marker` | typedef (alias) | 2085 |
| `factor` | typedef (alias) | 2099 |
| `mask` | typedef (alias) | 2120 |
| `equation` | typedef (alias) | 2129 |
| `ref` | typedef (alias) | 2275 |
| `pname` | typedef (alias) | 2380 |
| `markerp` | typedef (alias) | 2403 |
| `range` | typedef (alias) | 2405 |
| `blue` | typedef (alias) | 2483 |
| `primcount` | typedef (alias) | 2516 |
| `coord` | typedef (alias) | 2529 |
| `tz` | typedef (alias) | 2566 |
| `bz` | typedef (alias) | 2576 |
| `code` | typedef (alias) | 2647 |
| `y` | typedef (alias) | 2700 |
| `z` | typedef (alias) | 2702 |
| `w` | typedef (alias) | 2712 |
| `dfactorAlpha` | typedef (alias) | 2747 |
| `weight` | typedef (alias) | 2793 |
| `componentUsage` | typedef (alias) | 2833 |
| `muxSum` | typedef (alias) | 2834 |
| `modestride` | typedef (alias) | 2932 |
| `ptrstride` | typedef (alias) | 2948 |

#### `codemp/renderer/mdx_format.h`

| Type | Kind | Line |
|------|------|------|
| `mdxaCompQuatBone_t` | struct | 119 |
| `mdxaBone_t` | struct | 139 |
| `mdxmHeader_t` | struct | 153 |
| `mdxmHierarchyOffsets_t` | struct | 177 |
| `mdxmSurfHierarchy_t` | struct | 187 |
| `mdxmLOD_t` | struct | 203 |
| `mdxmLODSurfOffset_t` | struct | 210 |
| `mdxmSurface_t` | struct | 219 |
| `mdxmTriangle_t` | struct | 250 |
| `mdxmVertex_t` | struct | 260 |
| `mdxmVertexTexCoord_t` | struct | 328 |
| `mdxaHeader_t` | struct | 351 |
| `mdxaSkelOffsets_t` | struct | 376 |
| `mdxaSkel_t` | struct | 388 |
| `mdxaIndex_t` | struct | 410 |

#### `codemp/renderer/qgl.h`

| Type | Kind | Line |
|------|------|------|
| `s` | typedef (alias) | 78 |
| `v` | typedef (alias) | 79 |
| `t` | typedef (alias) | 86 |
| `r` | typedef (alias) | 94 |
| `q` | typedef (alias) | 102 |
| `target` | typedef (alias) | 110 |
| `params` | typedef (alias) | 151 |
| `param` | typedef (alias) | 153 |
| `componentUsage` | typedef (alias) | 155 |
| `muxSum` | typedef (alias) | 157 |
| `piValues` | typedef (alias) | 194 |
| `pfValues` | typedef (alias) | 195 |
| `nNumFormats` | typedef (alias) | 196 |
| `piAttribList` | typedef (alias) | 219 |
| `hPbuffer` | typedef (alias) | 220 |
| `hDC` | typedef (alias) | 221 |
| `piValue` | typedef (alias) | 223 |
| `iBuffer` | typedef (alias) | 245 |
| `string` | typedef (alias) | 282 |
| `program` | typedef (alias) | 283 |
| `programs` | typedef (alias) | 284 |
| `w` | typedef (alias) | 286 |

#### `codemp/renderer/qgl_console.h`

| Type | Kind | Line |
|------|------|------|
| `GLenum` | typedef (alias) | 23 |
| `GLboolean` | typedef (alias) | 24 |
| `GLbitfield` | typedef (alias) | 25 |
| `GLbyte` | typedef (alias) | 26 |
| `GLshort` | typedef (alias) | 27 |
| `GLint` | typedef (alias) | 28 |
| `GLsizei` | typedef (alias) | 29 |
| `GLubyte` | typedef (alias) | 30 |
| `GLushort` | typedef (alias) | 31 |
| `GLuint` | typedef (alias) | 32 |
| `GLfloat` | typedef (alias) | 33 |
| `GLclampf` | typedef (alias) | 34 |
| `GLdouble` | typedef (alias) | 35 |
| `GLclampd` | typedef (alias) | 36 |
| `GLvoid` | typedef (alias) | 37 |
| `PFNGLMULTITEXCOORD1DARBPROC` | fn-ptr typedef | 775 |
| `PFNGLMULTITEXCOORD1DVARBPROC` | fn-ptr typedef | 776 |
| `PFNGLMULTITEXCOORD1FARBPROC` | fn-ptr typedef | 777 |
| `PFNGLMULTITEXCOORD1FVARBPROC` | fn-ptr typedef | 778 |
| `PFNGLMULTITEXCOORD1IARBPROC` | fn-ptr typedef | 779 |
| `PFNGLMULTITEXCOORD1IVARBPROC` | fn-ptr typedef | 780 |
| `PFNGLMULTITEXCOORD1SARBPROC` | fn-ptr typedef | 781 |
| `PFNGLMULTITEXCOORD1SVARBPROC` | fn-ptr typedef | 782 |
| `PFNGLMULTITEXCOORD2DARBPROC` | fn-ptr typedef | 783 |
| `PFNGLMULTITEXCOORD2DVARBPROC` | fn-ptr typedef | 784 |
| `PFNGLMULTITEXCOORD2FARBPROC` | fn-ptr typedef | 785 |
| `PFNGLMULTITEXCOORD2FVARBPROC` | fn-ptr typedef | 786 |
| `PFNGLMULTITEXCOORD2IARBPROC` | fn-ptr typedef | 787 |
| `PFNGLMULTITEXCOORD2IVARBPROC` | fn-ptr typedef | 788 |
| `PFNGLMULTITEXCOORD2SARBPROC` | fn-ptr typedef | 789 |
| `PFNGLMULTITEXCOORD2SVARBPROC` | fn-ptr typedef | 790 |
| `PFNGLMULTITEXCOORD3DARBPROC` | fn-ptr typedef | 791 |
| `PFNGLMULTITEXCOORD3DVARBPROC` | fn-ptr typedef | 792 |
| `PFNGLMULTITEXCOORD3FARBPROC` | fn-ptr typedef | 793 |
| `PFNGLMULTITEXCOORD3FVARBPROC` | fn-ptr typedef | 794 |
| `PFNGLMULTITEXCOORD3IARBPROC` | fn-ptr typedef | 795 |
| `PFNGLMULTITEXCOORD3IVARBPROC` | fn-ptr typedef | 796 |
| `PFNGLMULTITEXCOORD3SARBPROC` | fn-ptr typedef | 797 |
| `PFNGLMULTITEXCOORD3SVARBPROC` | fn-ptr typedef | 798 |
| `PFNGLMULTITEXCOORD4DARBPROC` | fn-ptr typedef | 799 |
| `PFNGLMULTITEXCOORD4DVARBPROC` | fn-ptr typedef | 800 |
| `PFNGLMULTITEXCOORD4FARBPROC` | fn-ptr typedef | 801 |
| `PFNGLMULTITEXCOORD4FVARBPROC` | fn-ptr typedef | 802 |
| `PFNGLMULTITEXCOORD4IARBPROC` | fn-ptr typedef | 803 |
| `PFNGLMULTITEXCOORD4IVARBPROC` | fn-ptr typedef | 804 |
| `PFNGLMULTITEXCOORD4SARBPROC` | fn-ptr typedef | 805 |
| `PFNGLMULTITEXCOORD4SVARBPROC` | fn-ptr typedef | 806 |
| `PFNGLACTIVETEXTUREARBPROC` | fn-ptr typedef | 807 |
| `PFNGLCLIENTACTIVETEXTUREARBPROC` | fn-ptr typedef | 808 |

#### `codemp/renderer/tr_WorldEffects.h`

| Type | Kind | Line |
|------|------|------|
| `SParticle` | struct | 13 |
| `CWorldEffect` | class (C++) | 22 |
| `CWorldEffectsSystem` | class (C++) | 69 |

#### `codemp/renderer/tr_landscape.h`

| Type | Kind | Line |
|------|------|------|
| `CTerVert` | class (C++) | 22 |
| `CTRHeightDetails` | class (C++) | 37 |
| `CTRPatch` | class (C++) | 52 |
| `TPatchInfo` | struct | 108 |
| `CTRLandScape` | class (C++) | 119 |

#### `codemp/renderer/tr_local.h`

| Type | Kind | Line |
|------|------|------|
| `LPCSTR` | typedef (alias) | 5 |
| `USHORT` | typedef (alias) | 6 |
| `GLuint` | typedef (alias) | 7 |
| `glIndex_t` | typedef (alias) | 24 |
| `eDLightTypes` | enum | 53 |
| `dlight_t` | struct | 59 |
| `trMiniRefEntity_t` | struct | 87 |
| `trRefEntity_t` | struct | 94 |
| `orientationr_t` | struct | 109 |
| `image_t` | struct | 118 |
| `shaderSort_t` | enum | 156 |
| `genFunc_t` | enum | 192 |
| `deform_t` | enum | 207 |
| `alphaGen_t` | enum | 226 |
| `colorGen_t` | enum | 242 |
| `texCoordGen_t` | enum | 259 |
| `acff_t` | enum | 272 |
| `EGLFogOverride` | enum | 279 |
| `waveForm_t` | struct | 287 |
| `texMod_t` | enum | 298 |
| `deformStage_t` | struct | 310 |
| `texModInfo_t` | struct | 323 |
| `surfaceSprite_t` | struct | 363 |
| `textureBundle_t` | struct | 372 |
| `shaderStage_t` | struct | 394 |
| `cullType_t` | enum | 436 |
| `fogPass_t` | enum | 442 |
| `skyParms_t` | struct | 449 |
| `fogParms_t` | struct | 454 |
| `shader_t` | struct | 459 |
| `shaderState_t` | struct | 532 |
| `hitMatReg_t` | struct | 544 |
| `trRefdef_t` | struct | 563 |
| `skinSurface_t` | struct | 604 |
| `skin_t` | struct | 609 |
| `fog_t` | struct | 616 |
| `viewParms_t` | struct | 629 |
| `surfaceType_t` | enum | 656 |
| `drawSurf_t` | struct | 680 |
| `srfPoly_t` | struct | 692 |
| `srfDisplayList_t` | struct | 700 |
| `srfFlare_t` | struct | 706 |
| `srfTerrain_t` | struct | 744 |
| `srfGridMesh_t` | struct | 750 |
| `srfSurfaceFace_t` | struct | 778 |
| `srfTriangles_t` | struct | 818 |
| `msurface_t` | struct | 872 |
| `mnode_t` | struct | 886 |
| `mleaf_s` | struct | 899 |
| `bmodel_t` | struct | 938 |
| `mgrid_t` | struct | 946 |
| `world_t` | struct | 985 |
| `modtype_t` | enum | 1103 |
| `model_t` | struct | 1117 |
| `CPBUFFER` | class (C++) | 1156 |
| `frontEndCounters_t` | struct | 1235 |
| `glstate_t` | struct | 1253 |
| `backEndCounters_t` | struct | 1263 |
| `backEndState_t` | struct | 1279 |
| `trGlobals_t` | struct | 1309 |
| `levelLightParm_t` | struct | 1810 |
| `color4ub_t` | typedef (alias) | 1830 |
| `stageVars_t` | struct | 1832 |
| `shaderCommands_s` | struct | 1844 |
| `shaderCommands_t` | typedef (alias) | 1885 |
| `CRenderableSurface` | class (C++) | 2047 |
| `renderCommandList_t` | struct | 2180 |
| `setColorCommand_t` | struct | 2185 |
| `drawBufferCommand_t` | struct | 2190 |
| `subImageCommand_t` | struct | 2195 |
| `swapBuffersCommand_t` | struct | 2203 |
| `endFrameCommand_t` | struct | 2207 |
| `stretchPicCommand_t` | struct | 2212 |
| `rotatePicCommand_t` | struct | 2221 |
| `drawSurfsCommand_t` | struct | 2231 |
| `renderCommand_t` | enum | 2239 |
| `backEndData_t` | struct | 2263 |
| `decalPoly_t` | struct | 2319 |
| `DDS_PIXELFORMAT` | struct | 2336 |
| `DDS_HEADER` | struct | 2348 |

#### `codemp/renderer/tr_public.h`

| Type | Kind | Line |
|------|------|------|
| `refexport_t` | struct | 14 |

#### `codemp/renderer/tr_quicksprite.h`

| Type | Kind | Line |
|------|------|------|
| `CQuickSpriteSystem` | class (C++) | 16 |

### `codemp/server`

#### `codemp/server/server.h`

| Type | Kind | Line |
|------|------|------|
| `svEntity_t` | struct | 27 |
| `serverState_t` | enum | 47 |
| `server_t` | struct | 53 |
| `clientSnapshot_t` | struct | 94 |
| `clientState_t` | enum | 114 |
| `client_t` | struct | 124 |
| `challenge_t` | struct | 194 |
| `serverStatic_t` | struct | 208 |

### `codemp/server/NPCNav`

#### `codemp/server/NPCNav/navigator.h`

| Type | Kind | Line |
|------|------|------|
| `EdgeMultimap` | typedef (alias) | 40 |
| `EdgeMultimapIt` | typedef (alias) | 41 |
| `CEdge` | class (C++) | 50 |
| `CNode` | class (C++) | 70 |
| `CNavigator` | class (C++) | 134 |
| `CPriorityQueue` | class (C++) | 254 |

### `codemp/ui`

#### `codemp/ui/keycodes.h`

| Type | Kind | Line |
|------|------|------|
| `fakeAscii_t` | enum | 8 |

#### `codemp/ui/ui_local.h`

| Type | Kind | Line |
|------|------|------|
| `menuframework_s` | struct | 144 |
| `menucommon_s` | struct | 160 |
| `mfield_t` | struct | 179 |
| `menufield_s` | struct | 187 |
| `menuslider_s` | struct | 193 |
| `menulist_s` | struct | 204 |
| `menuaction_s` | struct | 221 |
| `menuradiobutton_s` | struct | 226 |
| `menubitmap_s` | struct | 232 |
| `menutext_s` | struct | 244 |
| `lerpFrame_t` | struct | 461 |
| `playerInfo_t` | struct | 480 |
| `uiStatic_t` | struct | 538 |
| `characterInfo` | struct | 599 |
| `aliasInfo` | struct | 608 |
| `teamInfo` | struct | 614 |
| `gameTypeInfo` | struct | 624 |
| `mapInfo` | struct | 629 |
| `tierInfo` | struct | 642 |
| `serverFilter_t` | struct | 649 |
| `pinglist_t` | struct | 654 |
| `serverStatus_t` | struct | 660 |
| `pendingServer_t` | struct | 690 |
| `pendingServerStatus_t` | struct | 698 |
| `serverStatusInfo_t` | struct | 703 |
| `modInfo_t` | struct | 711 |
| `playerSpeciesInfo_t` | struct | 716 |
| `uiInfo_t` | struct | 729 |
| `awardType_t` | enum | 1064 |
| `postGameInfo_t` | struct | 1125 |

#### `codemp/ui/ui_public.h`

| Type | Kind | Line |
|------|------|------|
| `uiClientState_t` | struct | 8 |
| `uiImport_t` | enum | 17 |
| `uiMenuCommand_t` | typedef (alias) | 208 |
| `uiExport_t` | enum | 216 |

#### `codemp/ui/ui_shared.h`

| Type | Kind | Line |
|------|------|------|
| `scriptDef_t` | struct | 106 |
| `rectDef_t` | struct | 112 |
| `Rectangle` | typedef (alias) | 119 |
| `windowDef_t` | struct | 122 |
| `Window` | typedef (alias) | 146 |
| `colorRangeDef_t` | struct | 148 |
| `columnInfo_t` | struct | 166 |
| `listBoxDef_t` | struct | 172 |
| `editFieldDef_t` | struct | 188 |
| `multiDef_t` | struct | 200 |
| `modelDef_t` | struct | 208 |
| `textScrollDef_t` | struct | 226 |
| `itemDef_t` | struct | 258 |
| `menuDef_t` | struct | 307 |
| `cachedAssets_t` | struct | 338 |
| `commandDef_t` | struct | 394 |
| `displayContextDef_t` | struct | 400 |

### `codemp/win32`

#### `codemp/win32/glw_win.h`

| Type | Kind | Line |
|------|------|------|
| `glwstate_t` | struct | 9 |

#### `codemp/win32/glw_win_dx8.h`

| Type | Kind | Line |
|------|------|------|
| `glwstate_t` | struct | 31 |

#### `codemp/win32/snd_fx_img.h`

| Type | Kind | Line |
|------|------|------|
| `DSP_IMAGE_image_FX_INDICES` | enum | 4 |
| `GraphI3DL2_FX0_I3DL2Reverb_STATE` | struct | 15 |
| `LPGraphI3DL2_FX0_I3DL2Reverb_STATE` | struct | 15 |
| `LPCGraphI3DL2_FX0_I3DL2Reverb_STATE` | typedef (alias) | 25 |
| `GraphXTalk_FX0_XTalk_STATE` | struct | 27 |
| `LPGraphXTalk_FX0_XTalk_STATE` | struct | 27 |
| `LPCGraphXTalk_FX0_XTalk_STATE` | typedef (alias) | 37 |
| `GraphVoice_FX0_Voice_0_STATE` | struct | 39 |
| `LPGraphVoice_FX0_Voice_0_STATE` | struct | 39 |
| `LPCGraphVoice_FX0_Voice_0_STATE` | typedef (alias) | 49 |
| `GraphVoice_FX1_Voice_1_STATE` | struct | 51 |
| `LPGraphVoice_FX1_Voice_1_STATE` | struct | 51 |
| `LPCGraphVoice_FX1_Voice_1_STATE` | typedef (alias) | 61 |
| `GraphVoice_FX2_Voice_2_STATE` | struct | 63 |
| `LPGraphVoice_FX2_Voice_2_STATE` | struct | 63 |
| `LPCGraphVoice_FX2_Voice_2_STATE` | typedef (alias) | 73 |
| `GraphVoice_FX3_Voice_3_STATE` | struct | 75 |
| `LPGraphVoice_FX3_Voice_3_STATE` | struct | 75 |
| `LPCGraphVoice_FX3_Voice_3_STATE` | typedef (alias) | 85 |

#### `codemp/win32/win_file.h`

| Type | Kind | Line |
|------|------|------|
| `wfhandle_t` | typedef (alias) | 16 |

#### `codemp/win32/win_input.h`

| Type | Kind | Line |
|------|------|------|
| `JoystickInfo` | struct | 79 |
| `PadInfo` | struct | 86 |

#### `codemp/win32/win_local.h`

| Type | Kind | Line |
|------|------|------|
| `WinVars_t` | struct | 61 |

### `codemp/zlib32`

#### `codemp/zlib32/deflate.h`

| Type | Kind | Line |
|------|------|------|
| `block_state` | enum | 83 |
| `ct_data` | struct | 92 |
| `static_tree_desc` | struct | 106 |
| `tree_desc` | struct | 115 |
| `deflate_state` | struct | 123 |
| `compress_func` | fn-ptr typedef | 220 |
| `config` | struct | 222 |

#### `codemp/zlib32/inflate.h`

| Type | Kind | Line |
|------|------|------|
| `check_func` | fn-ptr typedef | 12 |
| `inflate_block_mode` | enum | 14 |
| `inflate_codes_mode` | enum | 29 |
| `inflate_mode` | enum | 43 |
| `inflate_huft_t` | struct | 56 |
| `inflate_codes_state_t` | struct | 64 |
| `inflate_blocks_state_t` | struct | 93 |
| `inflate_state` | struct | 129 |

#### `codemp/zlib32/zip.h`

| Type | Kind | Line |
|------|------|------|
| `ELevel` | enum | 64 |
| `EFlush` | enum | 79 |
| `EStatus` | enum | 89 |
| `z_stream` | struct | 129 |

---

## SP (`code/`)

### `code/RMG`

#### `code/RMG/RM_Area.h`

| Type | Kind | Line |
|------|------|------|
| `CRMArea` | class (C++) | 17 |
| `rmAreaVector_t` | typedef (alias) | 74 |
| `CRMAreaManager` | class (C++) | 76 |

#### `code/RMG/RM_Headers.h`

| Type | Kind | Line |
|------|------|------|
| `symmetry_t` | enum | 27 |

#### `code/RMG/RM_Instance.h`

| Type | Kind | Line |
|------|------|------|
| `CRMAutomapSymbol` | enum | 13 |
| `CRMInstance` | class (C++) | 25 |
| `rmInstanceIter_t` | typedef (alias) | 119 |
| `rmInstanceList_t` | typedef (alias) | 120 |

#### `code/RMG/RM_InstanceFile.h`

| Type | Kind | Line |
|------|------|------|
| `CRMInstanceFile` | class (C++) | 11 |

#### `code/RMG/RM_Instance_BSP.h`

| Type | Kind | Line |
|------|------|------|
| `CRMBSPInstance` | class (C++) | 9 |

#### `code/RMG/RM_Instance_Group.h`

| Type | Kind | Line |
|------|------|------|
| `CRMGroupInstance` | class (C++) | 9 |

#### `code/RMG/RM_Instance_Random.h`

| Type | Kind | Line |
|------|------|------|
| `CRMRandomInstance` | class (C++) | 11 |

#### `code/RMG/RM_Instance_Void.h`

| Type | Kind | Line |
|------|------|------|
| `CRMVoidInstance` | class (C++) | 9 |

#### `code/RMG/RM_Manager.h`

| Type | Kind | Line |
|------|------|------|
| `CRMManager` | class (C++) | 9 |

#### `code/RMG/RM_Mission.h`

| Type | Kind | Line |
|------|------|------|
| `rmIntVector_t` | typedef (alias) | 12 |
| `CRMMission` | class (C++) | 15 |

#### `code/RMG/RM_Objective.h`

| Type | Kind | Line |
|------|------|------|
| `CRMObjective` | class (C++) | 9 |
| `rmObjectiveIter_t` | typedef (alias) | 61 |
| `rmObjectiveList_t` | typedef (alias) | 62 |

#### `code/RMG/RM_Path.h`

| Type | Kind | Line |
|------|------|------|
| `ERMDir` | enum | 24 |
| `CRMNode` | class (C++) | 41 |
| `rmNodeVector_t` | typedef (alias) | 72 |
| `CRMLoc` | class (C++) | 75 |
| `rmLocVector_t` | typedef (alias) | 110 |
| `CRMCell` | struct | 114 |
| `rmCellVector_t` | typedef (alias) | 132 |
| `CRMPathManager` | class (C++) | 135 |

#### `code/RMG/RM_Terrain.h`

| Type | Kind | Line |
|------|------|------|
| `CRandomModel` | class (C++) | 7 |
| `CCGHeightDetails` | class (C++) | 30 |
| `CCGPatch` | class (C++) | 50 |
| `CRMLandScape` | class (C++) | 59 |

### `code/Ragl`

#### `code/Ragl/graph_region.h`

| Type | Kind | Line |
|------|------|------|
| `TNODE` | class (C++) | 39 |

#### `code/Ragl/graph_triangulate.h`

| Type | Kind | Line |
|------|------|------|
| `TNODE` | class (C++) | 103 |

#### `code/Ragl/graph_vs.h`

| Type | Kind | Line |
|------|------|------|
| `TNODE` | class (C++) | 128 |

#### `code/Ragl/kdtree_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 37 |

#### `code/Ragl/ragl_common.h`

| Type | Kind | Line |
|------|------|------|
| `CNode` | class (C++) | 98 |
| `CEdge` | class (C++) | 149 |
| `TDATA` | class (C++) | 167 |

### `code/Ratl`

#### `code/Ratl/array_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 32 |

#### `code/Ratl/bits_vs.h`

| Type | Kind | Line |
|------|------|------|
| `bits_vs` | class (C++) | 37 |

#### `code/Ratl/grid_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 42 |

#### `code/Ratl/handle_pool_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 41 |

#### `code/Ratl/hash_pool_vs.h`

| Type | Kind | Line |
|------|------|------|
| `hash_pool` | class (C++) | 39 |

#### `code/Ratl/heap_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 37 |

#### `code/Ratl/list_vs.h`

| Type | Kind | Line |
|------|------|------|
| `list_node` | class (C++) | 40 |
| `T` | class (C++) | 50 |

#### `code/Ratl/map_vs.h`

| Type | Kind | Line |
|------|------|------|
| `tree_node` | class (C++) | 35 |
| `T` | class (C++) | 97 |
| `K` | class (C++) | 1194 |

#### `code/Ratl/pool_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 39 |

#### `code/Ratl/queue_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 40 |

#### `code/Ratl/ratl_common.h`

| Type | Kind | Line |
|------|------|------|
| `alignStruct` | struct | 120 |
| `T` | class (C++) | 152 |
| `compile_assert` | class (C++) | 261 |
| `ratl_base` | class (C++) | 300 |
| `bits_base` | class (C++) | 319 |
| `ratl_compare` | struct | 463 |
| `bits_true` | class (C++) | 478 |

#### `code/Ratl/scheduler_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 42 |

#### `code/Ratl/stack_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 35 |

#### `code/Ratl/string_vs.h`

| Type | Kind | Line |
|------|------|------|
| `string_vs` | class (C++) | 35 |
| `TString_vs` | typedef (alias) | 363 |
| `TUIString_vs` | typedef (alias) | 364 |

#### `code/Ratl/vector_vs.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 37 |

### `code/Ravl`

#### `code/Ravl/CBounds.h`

| Type | Kind | Line |
|------|------|------|
| `TPlanes` | typedef (alias) | 40 |
| `CBTrace` | class (C++) | 49 |
| `CBBox` | class (C++) | 101 |

#### `code/Ravl/CMatrix.h`

| Type | Kind | Line |
|------|------|------|
| `CMatrix` | class (C++) | 39 |

#### `code/Ravl/CVec.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 51 |
| `ESide` | enum | 71 |
| `CVec4` | class (C++) | 88 |
| `CVec3` | class (C++) | 559 |

### `code/Rufl`

#### `code/Rufl/hfile.h`

| Type | Kind | Line |
|------|------|------|
| `hfile` | class (C++) | 32 |

#### `code/Rufl/hstring.h`

| Type | Kind | Line |
|------|------|------|
| `hstring` | class (C++) | 28 |

### `code/cgame`

#### `code/cgame/FxPrimitives.h`

| Type | Kind | Line |
|------|------|------|
| `CEffect` | class (C++) | 96 |
| `CTrail` | class (C++) | 151 |
| `CLight` | class (C++) | 193 |
| `CFlash` | class (C++) | 231 |
| `CParticle` | class (C++) | 252 |
| `CLine` | class (C++) | 328 |
| `CBezier` | class (C++) | 348 |
| `CElectricity` | class (C++) | 378 |
| `COrientedParticle` | class (C++) | 402 |
| `CTail` | class (C++) | 424 |
| `CCylinder` | class (C++) | 456 |
| `CEmitter` | class (C++) | 486 |
| `CPoly` | class (C++) | 536 |

#### `code/cgame/FxScheduler.h`

| Type | Kind | Line |
|------|------|------|
| `fxString_t` | typedef (alias) | 8 |
| `CMediaHandles` | class (C++) | 64 |
| `CFxRange` | class (C++) | 89 |
| `EPrimType` | enum | 128 |
| `CPrimitiveTemplate` | class (C++) | 163 |
| `SEffectTemplate` | struct | 353 |
| `SLoopedEffect` | struct | 384 |
| `CFxScheduler` | class (C++) | 394 |

#### `code/cgame/FxSystem.h`

| Type | Kind | Line |
|------|------|------|
| `SFxHelper` | struct | 38 |

#### `code/cgame/cg_camera.h`

| Type | Kind | Line |
|------|------|------|
| `camera_t` | struct | 30 |

#### `code/cgame/cg_lights.h`

| Type | Kind | Line |
|------|------|------|
| `clightstyle_t` | struct | 5 |

#### `code/cgame/cg_local.h`

| Type | Kind | Line |
|------|------|------|
| `lerpFrame_t` | struct | 93 |
| `playerEntity_t` | struct | 112 |
| `centity_s` | struct | 130 |
| `centity_t` | typedef (alias) | 176 |
| `markPoly_t` | struct | 184 |
| `leType_t` | enum | 195 |
| `leFlag_t` | enum | 208 |
| `leBounceSound_t` | enum | 215 |
| `localEntity_t` | struct | 222 |
| `itemInfo_t` | struct | 256 |
| `powerupInfo_t` | struct | 263 |
| `overrides_t` | struct | 277 |
| `cg_t` | struct | 297 |
| `screengraphics_s` | struct | 521 |

#### `code/cgame/cg_media.h`

| Type | Kind | Line |
|------|------|------|
| `footstep_t` | enum | 6 |
| `HUDMenuItem_t` | struct | 43 |
| `otherhudbits_t` | enum | 61 |
| `cgMedia_t` | struct | 96 |
| `cgEffects_t` | struct | 311 |
| `cgs_t` | struct | 370 |

#### `code/cgame/cg_public.h`

| Type | Kind | Line |
|------|------|------|
| `snapshot_s` | struct | 24 |
| `snapshot_t` | typedef (alias) | 47 |
| `cgameImport_t` | enum | 60 |

### `code/client`

#### `code/client/BinkVideo.h`

| Type | Kind | Line |
|------|------|------|
| `BinkVideo` | class (C++) | 18 |

#### `code/client/cl_input_hotswap.h`

| Type | Kind | Line |
|------|------|------|
| `HotSwapManager` | class (C++) | 13 |

#### `code/client/cl_mp3.h`

| Type | Kind | Line |
|------|------|------|
| `id3v1_1` | struct | 15 |

#### `code/client/client.h`

| Type | Kind | Line |
|------|------|------|
| `clSnapshot_t` | struct | 14 |
| `clientActive_t` | struct | 53 |
| `clientConnection_t` | struct | 127 |
| `exitTo_t` | enum | 160 |
| `ping_t` | struct | 177 |
| `serverInfoResponse_t` | struct | 183 |
| `getserversResponse_t` | struct | 188 |
| `clientStatic_t` | struct | 193 |
| `console_t` | struct | 238 |
| `kbutton_t` | struct | 332 |

#### `code/client/fffx.h`

| Type | Kind | Line |
|------|------|------|
| `ffFX_e` | enum | 13 |

#### `code/client/keycodes.h`

| Type | Kind | Line |
|------|------|------|
| `fakeAscii_t` | enum | 6 |

#### `code/client/keys.h`

| Type | Kind | Line |
|------|------|------|
| `qkey_t` | struct | 3 |
| `field_t` | struct | 12 |
| `keyGlobals_t` | struct | 19 |
| `keyname_t` | struct | 36 |

#### `code/client/snd_ambient.h`

| Type | Kind | Line |
|------|------|------|
| `set_e` | enum | 33 |
| `setKeyword_e` | enum | 42 |
| `ambientSet_t` | struct | 60 |
| `parseFunc_t` | fn-ptr typedef | 75 |
| `CSetGroup` | class (C++) | 80 |

#### `code/client/snd_local.h`

| Type | Kind | Line |
|------|------|------|
| `portable_samplepair_t` | struct | 30 |
| `SoundCompressionMethod_t` | enum | 38 |
| `sfx_t` | struct | 48 |
| `dma_t` | struct | 67 |
| `STREAMINGBUFFER` | struct | 80 |
| `channel_t` | struct | 94 |
| `wavinfo_t` | struct | 137 |

#### `code/client/snd_local_console.h`

| Type | Kind | Line |
|------|------|------|
| `streamHandle_t` | typedef (alias) | 19 |
| `wavinfo_t` | struct | 25 |
| `sfx_t` | struct | 43 |
| `channel_t` | struct | 58 |

#### `code/client/snd_music.h`

| Type | Kind | Line |
|------|------|------|
| `MusicState_e` | enum | 11 |

#### `code/client/vmachine.h`

| Type | Kind | Line |
|------|------|------|
| `cgameExport_t` | enum | 13 |
| `vm_s` | struct | 48 |
| `vm_t` | typedef (alias) | 52 |

### `code/client/OpenAL`

#### `code/client/OpenAL/alc.h`

| Type | Kind | Line |
|------|------|------|
| `ALCdevice` | typedef (alias) | 23 |
| `ALCcontext` | typedef (alias) | 24 |

#### `code/client/OpenAL/alctypes.h`

| Type | Kind | Line |
|------|------|------|
| `ALCdevice` | typedef (alias) | 31 |
| `ALCcontext` | typedef (alias) | 34 |
| `ALCboolean` | typedef (alias) | 38 |
| `ALCbyte` | typedef (alias) | 41 |
| `ALCubyte` | typedef (alias) | 44 |
| `ALCshort` | typedef (alias) | 47 |
| `ALCushort` | typedef (alias) | 50 |
| `ALCuint` | typedef (alias) | 53 |
| `ALCint` | typedef (alias) | 56 |
| `ALCfloat` | typedef (alias) | 59 |
| `ALCdouble` | typedef (alias) | 62 |
| `ALCsizei` | typedef (alias) | 65 |
| `ALCvoid` | typedef (alias) | 68 |
| `ALCenum` | typedef (alias) | 71 |

#### `code/client/OpenAL/altypes.h`

| Type | Kind | Line |
|------|------|------|
| `ALboolean` | typedef (alias) | 30 |
| `ALbyte` | typedef (alias) | 33 |
| `ALubyte` | typedef (alias) | 36 |
| `ALshort` | typedef (alias) | 39 |
| `ALushort` | typedef (alias) | 42 |
| `ALuint` | typedef (alias) | 45 |
| `ALint` | typedef (alias) | 48 |
| `ALfloat` | typedef (alias) | 51 |
| `ALdouble` | typedef (alias) | 54 |
| `ALsizei` | typedef (alias) | 57 |
| `ALvoid` | typedef (alias) | 60 |
| `ALenum` | typedef (alias) | 63 |

### `code/client/eax`

#### `code/client/eax/EaxMan.h`

| Type | Kind | Line |
|------|------|------|
| `EMPOINT` | struct | 24 |
| `LPEMPOINT` | typedef (alias) | 29 |
| `LISTENERATTRIBUTES` | struct | 31 |
| `LPLISTENERATTRIBUTES` | typedef (alias) | 36 |
| `SOURCEATTRIBUTES` | struct | 38 |
| `LPSOURCEATTRIBUTES` | typedef (alias) | 51 |
| `MATERIALATTRIBUTES` | struct | 53 |
| `LPMATERIALATTRIBUTES` | typedef (alias) | 59 |
| `DIFFRACTIONBOX` | struct | 64 |
| `LPDIFFRACTIONBOX` | typedef (alias) | 69 |
| `LPEAXMANAGER` | typedef (alias) | 78 |

#### `code/client/eax/eax.h`

| Type | Kind | Line |
|------|------|------|
| `FAR` | typedef (alias) | 44 |
| `GUID` | struct | 56 |
| `EAXSet` | fn-ptr typedef | 78 |
| `EAXGet` | fn-ptr typedef | 79 |
| `EAXCONTEXTPROPERTIES` | struct | 122 |
| `LPEAXCONTEXTPROPERTIES` | struct | 122 |
| `EAXSOURCEPROPERTIES` | struct | 143 |
| `LPEAXSOURCEPROPERTIES` | struct | 143 |
| `EAXSOURCEALLSENDPROPERTIES` | struct | 168 |
| `LPEAXSOURCEALLSENDPROPERTIES` | struct | 168 |
| `EAXACTIVEFXSLOTS` | struct | 182 |
| `LPEAXACTIVEFXSLOTS` | struct | 182 |
| `EAXOBSTRUCTIONPROPERTIES` | struct | 188 |
| `LPEAXOBSTRUCTIONPROPERTIES` | struct | 188 |
| `EAXOCCLUSIONPROPERTIES` | struct | 195 |
| `LPEAXOCCLUSIONPROPERTIES` | struct | 195 |
| `EAXEXCLUSIONPROPERTIES` | struct | 204 |
| `LPEAXEXCLUSIONPROPERTIES` | struct | 204 |
| `EAXSOURCESENDPROPERTIES` | struct | 211 |
| `LPEAXSOURCESENDPROPERTIES` | struct | 211 |
| `EAXSOURCEOCCLUSIONSENDPROPERTIES` | struct | 219 |
| `LPEAXSOURCEOCCLUSIONSENDPROPERTIES` | struct | 219 |
| `EAXSOURCEEXCLUSIONSENDPROPERTIES` | struct | 229 |
| `LPEAXSOURCEEXCLUSIONSENDPROPERTIES` | struct | 229 |
| `EAXFXSLOTPROPERTIES` | struct | 248 |
| `LPEAXFXSLOTPROPERTIES` | struct | 248 |
| `EAXVECTOR` | struct | 259 |
| `EAXCONTEXT_PROPERTY` | enum | 298 |
| `EAXFXSLOT_PROPERTY` | enum | 375 |
| `EAXSOURCE_PROPERTY` | enum | 429 |
| `EAXREVERB_PROPERTY` | enum | 590 |
| `EAXREVERBPROPERTIES` | struct | 696 |
| `LPEAXREVERBPROPERTIES` | struct | 696 |
| `EAXAGCCOMPRESSOR_PROPERTY` | enum | 843 |
| `EAXAGCCOMPRESSORPROPERTIES` | struct | 857 |
| `LPEAXAGCCOMPRESSORPROPERTIES` | struct | 857 |
| `EAXAUTOWAH_PROPERTY` | enum | 882 |
| `EAXAUTOWAHPROPERTIES` | struct | 899 |
| `LPEAXAUTOWAHPROPERTIES` | struct | 899 |
| `EAXCHORUS_PROPERTY` | enum | 941 |
| `EAXCHORUSPROPERTIES` | struct | 967 |
| `LPEAXCHORUSPROPERTIES` | struct | 967 |
| `EAXDISTORTION_PROPERTY` | enum | 1018 |
| `EAXDISTORTIONPROPERTIES` | struct | 1036 |
| `LPEAXDISTORTIONPROPERTIES` | struct | 1036 |
| `EAXECHO_PROPERTY` | enum | 1082 |
| `EAXECHOPROPERTIES` | struct | 1100 |
| `LPEAXECHOPROPERTIES` | struct | 1100 |
| `EAXEQUALIZER_PROPERTY` | enum | 1147 |
| `EAXEQUALIZERPROPERTIES` | struct | 1170 |
| `LPEAXEQUALIZERPROPERTIES` | struct | 1170 |
| `EAXFLANGER_PROPERTY` | enum | 1241 |
| `EAXFLANGERPROPERTIES` | struct | 1267 |
| `LPEAXFLANGERPROPERTIES` | struct | 1267 |
| `EAXFREQUENCYSHIFTER_PROPERTY` | enum | 1318 |
| `EAXFREQUENCYSHIFTERPROPERTIES` | struct | 1342 |
| `LPEAXFREQUENCYSHIFTERPROPERTIES` | struct | 1342 |
| `EAXVOCALMORPHER_PROPERTY` | enum | 1378 |
| `EAXVOCALMORPHERPROPERTIES` | struct | 1412 |
| `LPEAXVOCALMORPHERPROPERTIES` | struct | 1412 |
| `EAXPITCHSHIFTER_PROPERTY` | enum | 1463 |
| `EAXPITCHSHIFTERPROPERTIES` | struct | 1478 |
| `LPEAXPITCHSHIFTERPROPERTIES` | struct | 1478 |
| `EAXRINGMODULATOR_PROPERTY` | enum | 1509 |
| `EAXRINGMODULATORPROPERTIES` | struct | 1533 |
| `LPEAXRINGMODULATORPROPERTIES` | struct | 1533 |

### `code/ff`

#### `code/ff/ff.h`

| Type | Kind | Line |
|------|------|------|
| `xcommand_t` | fn-ptr typedef | 24 |

#### `code/ff/ff_ChannelCompound.h`

| Type | Kind | Line |
|------|------|------|
| `ChannelCompound` | class (C++) | 13 |
| `THandleTable` | typedef (alias) | 62 |

#### `code/ff/ff_ChannelSet.h`

| Type | Kind | Line |
|------|------|------|
| `FFChannelSet` | class (C++) | 16 |
| `ChannelIterator` | class (C++) | 51 |

#### `code/ff/ff_ConfigParser.h`

| Type | Kind | Line |
|------|------|------|
| `FFConfigParser` | class (C++) | 8 |

#### `code/ff/ff_HandleTable.h`

| Type | Kind | Line |
|------|------|------|
| `FFHandleTable` | class (C++) | 13 |

#### `code/ff/ff_MultiCompound.h`

| Type | Kind | Line |
|------|------|------|
| `MultiCompound` | class (C++) | 13 |

#### `code/ff/ff_MultiEffect.h`

| Type | Kind | Line |
|------|------|------|
| `MultiEffect` | class (C++) | 18 |

#### `code/ff/ff_MultiSet.h`

| Type | Kind | Line |
|------|------|------|
| `FFMultiSet` | class (C++) | 14 |

#### `code/ff/ff_ffset.h`

| Type | Kind | Line |
|------|------|------|
| `FFSet` | class (C++) | 14 |

#### `code/ff/ff_public.h`

| Type | Kind | Line |
|------|------|------|
| `ffHandle_t` | typedef (alias) | 8 |

#### `code/ff/ff_system.h`

| Type | Kind | Line |
|------|------|------|
| `FFSystem` | class (C++) | 23 |

#### `code/ff/ff_utils.h`

| Type | Kind | Line |
|------|------|------|
| `TNameTable` | typedef (alias) | 41 |
| `Type` | class (C++) | 48 |
| `T` | class (C++) | 76 |

### `code/ff/IFC`

#### `code/ff/IFC/FeelitAPI.h`

| Type | Kind | Line |
|------|------|------|
| `FEELIT_CONSTANTFORCE` | struct | 183 |
| `LPFEELIT_CONSTANTFORCE` | struct | 183 |
| `LPCFEELIT_CONSTANTFORCE` | typedef (alias) | 186 |
| `FEELIT_RAMPFORCE` | struct | 188 |
| `LPFEELIT_RAMPFORCE` | struct | 188 |
| `LPCFEELIT_RAMPFORCE` | typedef (alias) | 192 |
| `FEELIT_PERIODIC` | struct | 194 |
| `LPFEELIT_PERIODIC` | struct | 194 |
| `LPCFEELIT_PERIODIC` | typedef (alias) | 200 |
| `FEELIT_CONDITION` | struct | 202 |
| `LPFEELIT_CONDITION` | struct | 202 |
| `LPCFEELIT_CONDITION` | typedef (alias) | 210 |
| `FEELIT_TEXTURE` | struct | 212 |
| `LPFEELIT_TEXTURE` | struct | 212 |
| `LPCFEELIT_TEXTURE` | typedef (alias) | 222 |
| `FEELIT_CUSTOMFORCE` | struct | 224 |
| `LPFEELIT_CUSTOMFORCE` | struct | 224 |
| `LPCFEELIT_CUSTOMFORCE` | typedef (alias) | 230 |
| `FEELIT_ENVELOPE` | struct | 232 |
| `LPFEELIT_ENVELOPE` | struct | 232 |
| `LPCFEELIT_ENVELOPE` | typedef (alias) | 239 |
| `FEELIT_EFFECT` | struct | 241 |
| `LPFEELIT_EFFECT` | struct | 241 |
| `LPCFEELIT_EFFECT` | typedef (alias) | 258 |
| `FEELIT_EFFESCAPE` | struct | 324 |
| `LPFEELIT_EFFESCAPE` | struct | 324 |
| `LPIFEELIT_EFFECT` | typedef (alias) | 356 |
| `FEELIT_ENCLOSURE` | struct | 388 |
| `LPFEELIT_ENCLOSURE` | struct | 388 |
| `LPCFEELIT_ENCLOSURE` | typedef (alias) | 401 |
| `FEELIT_ELLIPSE` | struct | 404 |
| `LPFEELIT_ELLIPSE` | struct | 404 |
| `LPCFEELIT_ELLIPSE` | typedef (alias) | 414 |
| `FEELIT_DEVCAPS` | struct | 437 |
| `LPFEELIT_DEVCAPS` | struct | 437 |
| `LPCFEELIT_DEVCAPS` | typedef (alias) | 450 |
| `FEELIT_OBJECTDATAFORMAT` | struct | 497 |
| `LPFEELIT_OBJECTDATAFORMAT` | struct | 497 |
| `LPCFEELIT_OBJECTDATAFORMAT` | typedef (alias) | 503 |
| `FEELIT_DATAFORMAT` | struct | 505 |
| `LPFEELIT_DATAFORMAT` | struct | 505 |
| `LPCFEELIT_DATAFORMAT` | typedef (alias) | 513 |
| `FEELIT_DEVICEOBJECTINSTANCE` | struct | 522 |
| `LPFEELIT_DEVICEOBJECTINSTANCE` | struct | 522 |
| `LPCFEELIT_DEVICEOBJECTINSTANCE` | typedef (alias) | 539 |
| `LPVOID` | typedef (alias) | 541 |
| `FEELIT_PROPHEADER` | struct | 553 |
| `LPFEELIT_PROPHEADER` | struct | 553 |
| `LPCFEELIT_PROPHEADER` | typedef (alias) | 559 |
| `FEELIT_PROPDWORD` | struct | 566 |
| `LPFEELIT_PROPDWORD` | struct | 566 |
| `LPCFEELIT_PROPDWORD` | typedef (alias) | 570 |
| `FEELIT_PROPRANGE` | struct | 572 |
| `LPFEELIT_PROPRANGE` | struct | 572 |
| `LPCFEELIT_PROPRANGE` | typedef (alias) | 577 |
| `FEELIT_PROPBALLISTICS` | struct | 617 |
| `LPFEELIT_PROPBALLISTICS` | struct | 617 |
| `LPCFEELIT_PROPBALLISTICS` | typedef (alias) | 623 |
| `FEELIT_PROPSCREENSIZE` | struct | 625 |
| `LPFEELIT_PROPSCREENSIZE` | struct | 625 |
| `LPCFEELIT_PROPSCREENSIZE` | typedef (alias) | 630 |
| `FEELIT_PROPABSOLUTEMODE` | struct | 632 |
| `LPFEELIT_PROPABSOLUTEMODE` | struct | 632 |
| `LPCFEELIT_PROPABSOLUTEMODE` | typedef (alias) | 636 |
| `FEELIT_PROPNUMEFFECTS` | struct | 646 |
| `LPFEELIT_PROPNUMEFFECTS` | struct | 646 |
| `LPCFEELIT_PROPNUMEFFECTS` | typedef (alias) | 651 |
| `FEELIT_DEVICEOBJECTDATA` | struct | 654 |
| `LPFEELIT_DEVICEOBJECTDATA` | struct | 654 |
| `LPCFEELIT_DEVICEOBJECTDATA` | typedef (alias) | 660 |
| `FEELIT_DEVICEINSTANCE` | struct | 674 |
| `LPFEELIT_DEVICEINSTANCE` | struct | 674 |
| `LPCFEELIT_DEVICEINSTANCE` | typedef (alias) | 685 |
| `FEELIT_EFFECTINFO` | struct | 707 |
| `LPFEELIT_EFFECTINFO` | struct | 707 |
| `LPCFEELIT_EFFECTINFO` | typedef (alias) | 715 |
| `LPHFEELITEVENT` | typedef (alias) | 748 |
| `FEELIT_EVENT` | struct | 750 |
| `LPFEELIT_EVENT` | struct | 750 |
| `LPCFEELIT_EVENT` | typedef (alias) | 758 |
| `LPIFEELIT_DEVICE` | typedef (alias) | 826 |
| `FEELIT_MOUSESTATE` | struct | 898 |
| `LPFEELIT_MOUSESTATE` | struct | 898 |
| `LPIFEELIT` | typedef (alias) | 953 |

#### `code/ff/IFC/IFCErrors.h`

| Type | Kind | Line |
|------|------|------|
| `IFC_ERROR_CODE` | enum | 57 |
| `IFC_ERROR_HANDLING_FLAGS` | enum | 92 |
| `DLLIFC` | class (C++) | 134 |

#### `code/ff/IFC/ImmBox.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 77 |

#### `code/ff/IFC/ImmCompoundEffect.h`

| Type | Kind | Line |
|------|------|------|
| `IMM_FFE_FILEEFFECT` | struct | 58 |
| `LPIMM_FFE_FILEEFFECT` | struct | 58 |
| `DLLIFC` | class (C++) | 80 |

#### `code/ff/IFC/ImmCondition.h`

| Type | Kind | Line |
|------|------|------|
| `IC_ArgumentType` | enum | 65 |
| `DLLIFC` | class (C++) | 93 |

#### `code/ff/IFC/ImmConstant.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 70 |

#### `code/ff/IFC/ImmDXDevice.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 52 |

#### `code/ff/IFC/ImmDamper.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 70 |

#### `code/ff/IFC/ImmDevice.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 128 |

#### `code/ff/IFC/ImmDevices.h`

| Type | Kind | Line |
|------|------|------|
| `IMM_ENUMERATE` | enum | 48 |
| `IMM_ENUMERATE_PREFERENCE` | enum | 54 |
| `CInitializeEnum` | class (C++) | 61 |
| `IMM_DEVICE_PTR` | typedef (alias) | 74 |
| `DLLIFC` | class (C++) | 81 |

#### `code/ff/IFC/ImmEffect.h`

| Type | Kind | Line |
|------|------|------|
| `GENERIC_EFFECT_PTR` | typedef (alias) | 72 |
| `DLLIFC` | class (C++) | 83 |

#### `code/ff/IFC/ImmEffectSuite.h`

| Type | Kind | Line |
|------|------|------|
| `ECacheState` | enum | 51 |
| `DLLIFC` | class (C++) | 62 |
| `CImmEffectSuite` | class (C++) | 88 |

#### `code/ff/IFC/ImmEllipse.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 81 |

#### `code/ff/IFC/ImmEnclosure.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 81 |

#### `code/ff/IFC/ImmFriction.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 68 |

#### `code/ff/IFC/ImmGrid.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 73 |

#### `code/ff/IFC/ImmIFR.h`

| Type | Kind | Line |
|------|------|------|
| `HIFRPROJECT` | typedef (alias) | 69 |
| `IFREffect` | struct | 76 |

#### `code/ff/IFC/ImmInertia.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 70 |

#### `code/ff/IFC/ImmMouse.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 52 |

#### `code/ff/IFC/ImmPeriodic.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 77 |

#### `code/ff/IFC/ImmProjects.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 67 |

#### `code/ff/IFC/ImmRamp.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 71 |

#### `code/ff/IFC/ImmSpring.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 73 |

#### `code/ff/IFC/ImmTexture.h`

| Type | Kind | Line |
|------|------|------|
| `DLLIFC` | class (C++) | 73 |

### `code/game`

#### `code/game/Q3_Interface.h`

| Type | Kind | Line |
|------|------|------|
| `setType_t` | enum | 8 |
| `playType_t` | enum | 311 |
| `pscript_t` | struct | 521 |
| `entitylist_t` | typedef (alias) | 528 |
| `scriptlist_t` | typedef (alias) | 529 |
| `varString_m` | typedef (alias) | 532 |
| `varFloat_m` | typedef (alias) | 533 |
| `CQuake3GameInterface` | class (C++) | 538 |

#### `code/game/ai.h`

| Type | Kind | Line |
|------|------|------|
| `distance_e` | enum | 5 |
| `attack_e` | enum | 12 |
| `rank_t` | enum | 31 |
| `AIGroupMember_t` | struct | 96 |
| `AIGroupInfo_t` | struct | 106 |

#### `code/game/anims.h`

| Type | Kind | Line |
|------|------|------|
| `animNumber_t` | enum | 6 |

#### `code/game/b_local.h`

| Type | Kind | Line |
|------|------|------|
| `navInfo_t` | struct | 340 |

#### `code/game/b_public.h`

| Type | Kind | Line |
|------|------|------|
| `visibility_t` | enum | 88 |
| `spot_t` | enum | 89 |
| `lookMode_t` | enum | 91 |
| `jumpState_t` | enum | 97 |
| `sexType_t` | enum | 106 |
| `gNPCstats_t` | struct | 115 |
| `gNPC_t` | struct | 146 |

#### `code/game/bg_local.h`

| Type | Kind | Line |
|------|------|------|
| `pml_t` | struct | 11 |

#### `code/game/bg_public.h`

| Type | Kind | Line |
|------|------|------|
| `pmtype_t` | enum | 63 |
| `weaponstate_t` | enum | 72 |
| `Trace_Functor_t` | struct | 109 |
| `gentity_t` | typedef (alias) | 129 |
| `pmove_t` | struct | 130 |
| `persEnum_t` | enum | 195 |
| `powerup_t` | enum | 248 |
| `entity_event_t` | enum | 283 |
| `animation_t` | struct | 468 |
| `animEventType_t` | enum | 520 |
| `animevent_t` | struct | 537 |
| `footstepType_t` | enum | 550 |
| `meansOfDeath_t` | enum | 560 |
| `itemType_t` | enum | 623 |
| `gitem_t` | struct | 638 |
| `entityType_t` | enum | 713 |

#### `code/game/bset.h`

| Type | Kind | Line |
|------|------|------|
| `bSet_t` | enum | 1 |

#### `code/game/bstate.h`

| Type | Kind | Line |
|------|------|------|
| `bState_t` | enum | 5 |

#### `code/game/channels.h`

| Type | Kind | Line |
|------|------|------|
| `soundChannel_t` | enum | 6 |

#### `code/game/characters.h`

| Type | Kind | Line |
|------|------|------|
| `characters_t` | enum | 1 |
| `characterName_t` | struct | 47 |

#### `code/game/dmstates.h`

| Type | Kind | Line |
|------|------|------|
| `dynamicMusic_t` | enum | 5 |

#### `code/game/events.h`

| Type | Kind | Line |
|------|------|------|
| `eventType_t` | enum | 4 |

#### `code/game/fields.h`

| Type | Kind | Line |
|------|------|------|
| `fieldtypeSAVE_t` | enum | 31 |
| `save_field_t` | struct | 55 |

#### `code/game/g_functions.h`

| Type | Kind | Line |
|------|------|------|
| `thinkFunc_t` | enum | 17 |
| `clThinkFunc_t` | enum | 232 |
| `reachedFunc_t` | enum | 249 |
| `blockedFunc_t` | enum | 269 |
| `touchFunc_t` | enum | 286 |
| `useFunc_t` | enum | 333 |
| `painFunc_t` | enum | 499 |
| `dieFunc_t` | enum | 562 |

#### `code/game/g_local.h`

| Type | Kind | Line |
|------|------|------|
| `animFileSet_t` | struct | 68 |
| `interestPoint_t` | struct | 84 |
| `combatPoint_t` | struct | 94 |
| `alertEventType_e` | enum | 109 |
| `alertEventLevel_e` | enum | 115 |
| `alertEvent_t` | struct | 125 |
| `waypointData_t` | struct | 146 |
| `level_locals_t` | struct | 161 |
| `reference_tag_t` | struct | 573 |

#### `code/game/g_navigator.h`

| Type | Kind | Line |
|------|------|------|
| `TNodeHandle` | typedef (alias) | 38 |
| `TEdgeHandle` | typedef (alias) | 39 |
| `EPointType` | enum | 42 |

#### `code/game/g_public.h`

| Type | Kind | Line |
|------|------|------|
| `gentity_t` | typedef (alias) | 51 |
| `gclient_t` | typedef (alias) | 52 |
| `SavedGameJustLoaded_e` | enum | 54 |
| `gentity_s` | struct | 67 |
| `Trace_Functor_t` | struct | 115 |
| `game_import_t` | struct | 168 |
| `game_export_t` | struct | 476 |

#### `code/game/g_roff.h`

| Type | Kind | Line |
|------|------|------|
| `roff_hdr_t` | struct | 18 |
| `move_rotate_t` | struct | 31 |
| `roff_hdr2_t` | struct | 38 |
| `move_rotate2_t` | struct | 50 |
| `roff_list_t` | struct | 62 |

#### `code/game/g_shared.h`

| Type | Kind | Line |
|------|------|------|
| `taskID_t` | enum | 20 |
| `material_t` | enum | 37 |
| `clientInfo_t` | struct | 76 |
| `moverState_t` | enum | 107 |
| `targetModel_t` | enum | 118 |
| `renderInfo_t` | struct | 135 |
| `clientConnected_t` | enum | 240 |
| `playerTeamStateState_t` | enum | 246 |
| `playerTeamState_t` | struct | 275 |
| `objectives_t` | struct | 292 |
| `missionStats_t` | struct | 302 |
| `clientSession_t` | struct | 331 |
| `clientPersistant_t` | struct | 341 |
| `saberBlockType_t` | enum | 352 |
| `saberBlockedType_t` | enum | 358 |
| `movetype_t` | enum | 374 |
| `gclient_s` | struct | 387 |
| `parms_t` | struct | 492 |
| `centity_t` | typedef (alias) | 512 |
| `gentity_s` | struct | 514 |
| `weaponInfo_t` | struct | 836 |

#### `code/game/g_vehicles.h`

| Type | Kind | Line |
|------|------|------|
| `vehicleType_t` | enum | 7 |
| `EWeaponPose` | enum | 18 |
| `vehWeaponInfo_t` | struct | 33 |
| `turretStats_t` | struct | 90 |
| `vehWeaponStats_t` | struct | 113 |
| `vehicleInfo_t` | struct | 135 |
| `Muzzle` | struct | 443 |
| `vehWeaponStatus_t` | struct | 481 |
| `vehTurretStatus_t` | struct | 493 |
| `Vehicle_t` | struct | 510 |

#### `code/game/genericparser2.h`

| Type | Kind | Line |
|------|------|------|
| `CTextPool` | class (C++) | 32 |
| `CGPObject` | class (C++) | 53 |
| `CGPValue` | class (C++) | 76 |
| `CGPGroup` | class (C++) | 101 |
| `CGenericParser2` | class (C++) | 144 |
| `TGenericParser2` | typedef (alias) | 174 |
| `TGPGroup` | typedef (alias) | 175 |
| `TGPValue` | typedef (alias) | 176 |

#### `code/game/ghoul2_shared.h`

| Type | Kind | Line |
|------|------|------|
| `surfaceInfo_t` | struct | 33 |
| `boneInfo_t` | struct | 80 |
| `boltInfo_t` | struct | 185 |
| `surfaceInfo_v` | typedef (alias) | 201 |
| `boneInfo_v` | typedef (alias) | 202 |
| `boltInfo_v` | typedef (alias) | 203 |
| `mdxaBone_v` | typedef (alias) | 204 |
| `CRenderableSurface` | class (C++) | 219 |
| `CGhoul2Info` | class (C++) | 240 |
| `IGhoul2InfoArray` | class (C++) | 313 |
| `CGhoul2Info_v` | class (C++) | 326 |
| `CCollisionRecord` | class (C++) | 461 |
| `EG2_Collision` | enum | 484 |

#### `code/game/objectives.h`

| Type | Kind | Line |
|------|------|------|
| `objectiveNumber_t` | enum | 9 |
| `missionFailed_t` | enum | 118 |
| `statusText_t` | enum | 141 |

#### `code/game/q_shared.h`

| Type | Kind | Line |
|------|------|------|
| `ulong` | typedef (alias) | 173 |
| `word` | typedef (alias) | 174 |
| `byte` | typedef (alias) | 176 |
| `LPCSTR` | typedef (alias) | 178 |
| `qboolean` | enum | 180 |
| `qhandle_t` | typedef (alias) | 183 |
| `thandle_t` | typedef (alias) | 184 |
| `fxHandle_t` | typedef (alias) | 185 |
| `sfxHandle_t` | typedef (alias) | 186 |
| `fileHandle_t` | typedef (alias) | 187 |
| `clipHandle_t` | typedef (alias) | 188 |
| `cbufExec_t` | enum | 221 |
| `printParm_t` | enum | 243 |
| `errorParm_t` | enum | 251 |
| `vec_t` | typedef (alias) | 314 |
| `vec2_t` | typedef (alias) | 315 |
| `vec3_t` | typedef (alias) | 316 |
| `vec4_t` | typedef (alias) | 317 |
| `vec5_t` | typedef (alias) | 318 |
| `vec3pair_t` | typedef (alias) | 320 |
| `ivec2_t` | typedef (alias) | 322 |
| `ivec3_t` | typedef (alias) | 323 |
| `ivec4_t` | typedef (alias) | 324 |
| `ivec5_t` | typedef (alias) | 325 |
| `fixed4_t` | typedef (alias) | 327 |
| `fixed8_t` | typedef (alias) | 328 |
| `fixed16_t` | typedef (alias) | 329 |
| `ct_table_t` | enum | 355 |
| `saber_colors_t` | enum | 474 |
| `fsMode_t` | enum | 1186 |
| `fsOrigin_t` | enum | 1193 |
| `cvar_t` | struct | 1310 |
| `cvarHandle_t` | typedef (alias) | 1325 |
| `vmCvar_t` | struct | 1329 |
| `cplane_t` | struct | 1357 |
| `trace_t` | struct | 1379 |
| `markFragment_t` | struct | 1402 |
| `orientation_t` | struct | 1409 |
| `gameState_t` | struct | 1532 |
| `forcePowers_t` | enum | 1538 |
| `saberType_t` | enum | 1561 |
| `waterHeightLevel_t` | enum | 1603 |
| `saberTrail_t` | struct | 1616 |
| `bladeInfo_t` | struct | 1634 |
| `saber_styles_t` | enum | 1660 |
| `saberInfo_t` | struct | 1724 |
| `saberInfoRetail_t` | struct | 1947 |
| `playerState_t` | struct | 2077 |
| `genCmds_t` | enum | 2389 |
| `usercmd_t` | struct | 2408 |
| `trType_t` | enum | 2422 |
| `trajectory_t` | struct | 2432 |
| `entityState_t` | struct | 2448 |
| `connstate_t` | enum | 2518 |
| `SSkinGoreData` | struct | 2530 |
| `sharedRagDollUpdateParams_t` | struct | 2571 |
| `sharedIKMoveParams_t` | struct | 2581 |
| `sharedSetBoneIKStateParams_t` | struct | 2590 |
| `sharedEIKMoveState` | enum | 2604 |
| `stringID_table_t` | struct | 2617 |
| `Eorientations` | enum | 2641 |
| `parseData_t` | struct | 2656 |
| `e_status` | enum | 2671 |
| `memtag_t` | typedef (alias) | 2688 |
| `ForceReload_e` | enum | 2692 |

#### `code/game/say.h`

| Type | Kind | Line |
|------|------|------|
| `saying_t` | enum | 4 |

#### `code/game/statindex.h`

| Type | Kind | Line |
|------|------|------|
| `statIndex_t` | enum | 10 |

#### `code/game/teams.h`

| Type | Kind | Line |
|------|------|------|
| `team_t` | enum | 4 |
| `class_t` | enum | 18 |

#### `code/game/weapons.h`

| Type | Kind | Line |
|------|------|------|
| `weapon_t` | enum | 9 |
| `ammo_t` | enum | 65 |
| `weaponData_t` | struct | 81 |
| `ammoData_t` | struct | 142 |

#### `code/game/wp_saber.h`

| Type | Kind | Line |
|------|------|------|
| `saberLockResult_t` | enum | 37 |
| `sabersLockMode_t` | enum | 44 |
| `evasionType_t` | enum | 191 |
| `swingType_t` | enum | 206 |
| `saberMoveName_t` | enum | 221 |
| `saberQuadrant_t` | enum | 416 |
| `saberMoveData_t` | struct | 428 |

### `code/ghoul2`

#### `code/ghoul2/ghoul2_gore.h`

| Type | Kind | Line |
|------|------|------|
| `GoreTextureCoordinates` | struct | 4 |
| `SGoreSurface` | struct | 35 |
| `CGoreSet` | class (C++) | 50 |
| `SRagDollEffectorCollision` | struct | 69 |
| `CRagDollUpdateParams` | class (C++) | 82 |
| `CRagDollParams` | class (C++) | 120 |

### `code/goblib`

#### `code/goblib/goblib.h`

| Type | Kind | Line |
|------|------|------|
| `int32` | typedef (alias) | 132 |
| `uint32` | typedef (alias) | 133 |
| `ulong` | typedef (alias) | 137 |
| `byte` | typedef (alias) | 138 |
| `GOBInt32` | typedef (alias) | 140 |
| `GOBUInt32` | typedef (alias) | 141 |
| `GOBChar` | typedef (alias) | 142 |
| `GOBBool` | typedef (alias) | 143 |
| `GOBError` | typedef (alias) | 144 |
| `GOBSeekType` | typedef (alias) | 145 |
| `GOBHandle` | typedef (alias) | 146 |
| `GOBAccessType` | typedef (alias) | 147 |
| `GOBFSHandle` | typedef (alias) | 148 |
| `GOBVoid` | typedef (alias) | 149 |
| `GOBFileSysOpenFunc` | fn-ptr typedef | 151 |
| `GOBFileSysCloseFunc` | fn-ptr typedef | 152 |
| `GOBFileSysReadFunc` | fn-ptr typedef | 153 |
| `GOBFileSysWriteFunc` | fn-ptr typedef | 154 |
| `GOBFileSysSeekFunc` | fn-ptr typedef | 155 |
| `GOBFileSysRenameFunc` | fn-ptr typedef | 156 |
| `GOBMemAllocFunc` | fn-ptr typedef | 158 |
| `GOBMemFreeFunc` | fn-ptr typedef | 159 |
| `GOBCompressFunc` | fn-ptr typedef | 161 |
| `GOBDecompressFunc` | fn-ptr typedef | 162 |
| `GOBCacheFileOpenFunc` | fn-ptr typedef | 164 |
| `GOBCacheFileCloseFunc` | fn-ptr typedef | 165 |
| `GOBCacheFileReadFunc` | fn-ptr typedef | 166 |
| `GOBCacheFileWriteFunc` | fn-ptr typedef | 167 |
| `GOBCacheFileSeekFunc` | fn-ptr typedef | 168 |
| `GOBBlockTableEntry` | struct | 170 |
| `GOBFileTableBasicEntry` | struct | 177 |
| `GOBFileTableExtEntry` | struct | 184 |
| `GOBMemoryFuncSet` | struct | 191 |
| `GOBSingleCodecDesc` | struct | 197 |
| `GOBCodecFuncSet` | struct | 205 |
| `GOBFileSysFuncSet` | struct | 211 |
| `GOBCacheFileFuncSet` | struct | 220 |
| `GOBReadStats` | struct | 229 |
| `GOBProfileReadFunc` | fn-ptr typedef | 283 |
| `GOBProfileFuncSet` | struct | 284 |

### `code/icarus`

#### `code/icarus/IcarusImplementation.h`

| Type | Kind | Line |
|------|------|------|
| `CIcarus` | class (C++) | 28 |

#### `code/icarus/IcarusInterface.h`

| Type | Kind | Line |
|------|------|------|
| `IIcarusInterface` | class (C++) | 20 |
| `IGameInterface` | class (C++) | 48 |

#### `code/icarus/blockstream.h`

| Type | Kind | Line |
|------|------|------|
| `vec3_t` | typedef (alias) | 8 |
| `CBlockMember` | class (C++) | 15 |
| `CBlock` | class (C++) | 93 |
| `CBlockStream` | class (C++) | 163 |

#### `code/icarus/sequence.h`

| Type | Kind | Line |
|------|------|------|
| `CSequence` | class (C++) | 6 |

#### `code/icarus/sequencer.h`

| Type | Kind | Line |
|------|------|------|
| `bstream_t` | struct | 13 |
| `CSequencer` | class (C++) | 29 |

#### `code/icarus/taskmanager.h`

| Type | Kind | Line |
|------|------|------|
| `DWORD` | typedef (alias) | 6 |
| `CTask` | class (C++) | 28 |
| `CTaskGroup` | class (C++) | 69 |
| `CTaskManager` | class (C++) | 116 |

### `code/jpeg-6`

#### `code/jpeg-6/jchuff.h`

| Type | Kind | Line |
|------|------|------|
| `c_derived_tbl` | struct | 15 |

#### `code/jpeg-6/jdct.h`

| Type | Kind | Line |
|------|------|------|
| `DCTELEM` | typedef (alias) | 30 |
| `data` | typedef (alias) | 35 |
| `ISLOW_MULT_TYPE` | typedef (alias) | 56 |
| `IFAST_MULT_TYPE` | typedef (alias) | 58 |
| `FLOAT_MULT_TYPE` | typedef (alias) | 64 |

#### `code/jpeg-6/jdhuff.h`

| Type | Kind | Line |
|------|------|------|
| `d_derived_tbl` | struct | 26 |
| `bit_buf_type` | typedef (alias) | 68 |
| `bitread_perm_state` | struct | 78 |
| `bitread_working_state` | struct | 84 |

#### `code/jpeg-6/jerror.h`

| Type | Kind | Line |
|------|------|------|
| `J_MESSAGE_CODE` | enum | 33 |

#### `code/jpeg-6/jmemsys.h`

| Type | Kind | Line |
|------|------|------|
| `XMSH` | typedef (alias) | 118 |
| `EMSH` | typedef (alias) | 119 |
| `handle_union` | union | 121 |
| `backing_store_ptr` | typedef (alias) | 129 |
| `backing_store_info` | struct | 131 |

#### `code/jpeg-6/jmorecfg.h`

| Type | Kind | Line |
|------|------|------|
| `JSAMPLE` | typedef (alias) | 59 |
| `JCOEF` | typedef (alias) | 99 |
| `JOCTET` | typedef (alias) | 110 |
| `UINT8` | typedef (alias) | 135 |
| `UINT16` | typedef (alias) | 147 |
| `INT16` | typedef (alias) | 158 |
| `JDIMENSION` | typedef (alias) | 174 |

#### `code/jpeg-6/jpegint.h`

| Type | Kind | Line |
|------|------|------|
| `J_BUF_MODE` | enum | 16 |
| `jpeg_comp_master` | struct | 45 |
| `jpeg_c_main_controller` | struct | 56 |
| `jpeg_c_prep_controller` | struct | 64 |
| `jpeg_c_coef_controller` | struct | 76 |
| `jpeg_color_converter` | struct | 83 |
| `jpeg_downsampler` | struct | 91 |
| `jpeg_forward_dct` | struct | 102 |
| `jpeg_entropy_encoder` | struct | 113 |
| `jpeg_marker_writer` | struct | 120 |
| `jpeg_decomp_master` | struct | 136 |
| `jpeg_input_controller` | struct | 145 |
| `jpeg_d_main_controller` | struct | 157 |
| `jpeg_d_coef_controller` | struct | 165 |
| `jpeg_d_post_controller` | struct | 176 |
| `jpeg_marker_reader` | struct | 188 |
| `jpeg_entropy_decoder` | struct | 211 |
| `output_col` | typedef (alias) | 218 |
| `jpeg_inverse_dct` | struct | 223 |
| `jpeg_upsampler` | struct | 230 |
| `jpeg_color_deconverter` | struct | 244 |
| `jpeg_color_quantizer` | struct | 252 |
| `jvirt_sarray_control` | struct | 385 |
| `jvirt_barray_control` | struct | 386 |

#### `code/jpeg-6/jpeglib.h`

| Type | Kind | Line |
|------|------|------|
| `boolean` | typedef (alias) | 26 |
| `JSAMPROW` | typedef (alias) | 89 |
| `JSAMPARRAY` | typedef (alias) | 90 |
| `JSAMPIMAGE` | typedef (alias) | 91 |
| `DCTSIZE2` | typedef (alias) | 93 |
| `JBLOCKROW` | typedef (alias) | 94 |
| `JBLOCKARRAY` | typedef (alias) | 95 |
| `JBLOCKIMAGE` | typedef (alias) | 96 |
| `JCOEFPTR` | typedef (alias) | 98 |
| `JQUANT_TBL` | struct | 106 |
| `JHUFF_TBL` | struct | 122 |
| `jpeg_component_info` | struct | 138 |
| `jpeg_scan_info` | struct | 206 |
| `J_COLOR_SPACE` | enum | 216 |
| `J_DCT_METHOD` | enum | 227 |
| `J_DITHER_MODE` | enum | 242 |
| `jpeg_common_struct` | struct | 262 |
| `j_common_ptr` | typedef (alias) | 270 |
| `j_compress_ptr` | typedef (alias) | 271 |
| `j_decompress_ptr` | typedef (alias) | 272 |
| `jpeg_compress_struct` | struct | 277 |
| `jpeg_decompress_struct` | struct | 416 |
| `jpeg_error_mgr` | struct | 640 |
| `jpeg_progress_mgr` | struct | 698 |
| `jpeg_destination_mgr` | struct | 710 |
| `jpeg_source_mgr` | struct | 722 |
| `jvirt_sarray_ptr` | typedef (alias) | 749 |
| `jvirt_barray_ptr` | typedef (alias) | 750 |
| `jpeg_memory_mgr` | struct | 753 |
| `cinfo` | typedef (alias) | 803 |
| `jvirt_sarray_control` | struct | 1023 |
| `jvirt_barray_control` | struct | 1024 |
| `jpeg_comp_master` | struct | 1025 |
| `jpeg_c_main_controller` | struct | 1026 |
| `jpeg_c_prep_controller` | struct | 1027 |
| `jpeg_c_coef_controller` | struct | 1028 |
| `jpeg_marker_writer` | struct | 1029 |
| `jpeg_color_converter` | struct | 1030 |
| `jpeg_downsampler` | struct | 1031 |
| `jpeg_forward_dct` | struct | 1032 |
| `jpeg_entropy_encoder` | struct | 1033 |
| `jpeg_decomp_master` | struct | 1034 |
| `jpeg_d_main_controller` | struct | 1035 |
| `jpeg_d_coef_controller` | struct | 1036 |
| `jpeg_d_post_controller` | struct | 1037 |
| `jpeg_input_controller` | struct | 1038 |
| `jpeg_marker_reader` | struct | 1039 |
| `jpeg_entropy_decoder` | struct | 1040 |
| `jpeg_inverse_dct` | struct | 1041 |
| `jpeg_upsampler` | struct | 1042 |
| `jpeg_color_deconverter` | struct | 1043 |
| `jpeg_color_quantizer` | struct | 1044 |

### `code/mac`

#### `code/mac/MacGamma.h`

| Type | Kind | Line |
|------|------|------|
| `recDeviceGamma` | struct | 38 |
| `precDeviceGamma` | typedef (alias) | 43 |
| `recSystemGamma` | struct | 45 |
| `precSystemGamma` | typedef (alias) | 50 |

### `code/mp3code`

#### `code/mp3code/config.h`

| Type | Kind | Line |
|------|------|------|
| `socklen_t` | typedef (alias) | 53 |
| `real` | typedef (alias) | 62 |
| `uint8` | typedef (alias) | 66 |
| `int8` | typedef (alias) | 67 |
| `uint16` | typedef (alias) | 75 |
| `int16` | typedef (alias) | 76 |
| `uint32` | typedef (alias) | 90 |
| `int32` | typedef (alias) | 91 |

#### `code/mp3code/l3.h`

| Type | Kind | Line |
|------|------|------|
| `HUFF_ELEMENT` | union | 70 |
| `BITDAT` | struct | 103 |
| `GR` | struct | 115 |
| `SIDE_INFO` | struct | 133 |
| `SCALEFACT` | struct | 150 |
| `CB_INFO` | struct | 158 |
| `IS_SF_INFO` | struct | 171 |

#### `code/mp3code/mhead.h`

| Type | Kind | Line |
|------|------|------|
| `MPEG_HEAD` | struct | 33 |
| `DEC_INFO` | struct | 57 |

#### `code/mp3code/mp3struct.h`

| Type | Kind | Line |
|------|------|------|
| `SBT_FUNCTION` | fn-ptr typedef | 13 |
| `XFORM_FUNCTION` | fn-ptr typedef | 14 |
| `DECODE_FUNCTION` | fn-ptr typedef | 15 |
| `MP3STREAM` | struct | 17 |
| `LP_MP3STREAM` | struct | 17 |

#### `code/mp3code/small_header.h`

| Type | Kind | Line |
|------|------|------|
| `SAMPLE` | union | 11 |
| `IN_OUT` | struct | 18 |
| `byte` | typedef (alias) | 27 |

### `code/png`

#### `code/png/png.h`

| Type | Kind | Line |
|------|------|------|
| `byte` | typedef (alias) | 40 |
| `word` | typedef (alias) | 41 |
| `ulong` | typedef (alias) | 42 |
| `png_ihdr_t` | struct | 47 |
| `png_image_t` | struct | 60 |

### `code/qcommon`

#### `code/qcommon/MiniHeap.h`

| Type | Kind | Line |
|------|------|------|
| `CMiniHeap` | class (C++) | 5 |

#### `code/qcommon/chash.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 11 |

#### `code/qcommon/cm_draw.h`

| Type | Kind | Line |
|------|------|------|
| `CPixel32` | class (C++) | 37 |
| `CDraw32` | class (C++) | 81 |

#### `code/qcommon/cm_landscape.h`

| Type | Kind | Line |
|------|------|------|
| `areaType_t` | enum | 29 |
| `CArea` | class (C++) | 42 |
| `areaList_t` | typedef (alias) | 72 |
| `areaIter_t` | typedef (alias) | 73 |
| `CCMHeightDetails` | class (C++) | 75 |
| `CCMPatch` | class (C++) | 90 |
| `CCMLandScape` | class (C++) | 135 |

#### `code/qcommon/cm_local.h`

| Type | Kind | Line |
|------|------|------|
| `cNode_t` | struct | 17 |
| `cLeaf_t` | struct | 31 |
| `cmodel_t` | struct | 42 |
| `cbrushside_t` | struct | 49 |
| `cbrush_t` | struct | 64 |
| `CCMShader` | class (C++) | 73 |
| `cPatch_t` | struct | 87 |
| `cArea_t` | struct | 95 |
| `clipMap_t` | struct | 103 |
| `sphere_t` | struct | 228 |
| `traceWork_t` | struct | 236 |
| `leafList_t` | struct | 265 |

#### `code/qcommon/cm_patch.h`

| Type | Kind | Line |
|------|------|------|
| `patchPlane_t` | struct | 45 |
| `facetLoad_t` | struct | 56 |
| `facet_t` | struct | 64 |
| `patchCollide_t` | struct | 93 |
| `cGrid_t` | struct | 104 |

#### `code/qcommon/cm_polylib.h`

| Type | Kind | Line |
|------|------|------|
| `winding_t` | struct | 7 |

#### `code/qcommon/cm_randomterrain.h`

| Type | Kind | Line |
|------|------|------|
| `CPathInfo` | class (C++) | 15 |
| `CRandomTerrain` | class (C++) | 52 |

#### `code/qcommon/cm_terrainmap.h`

| Type | Kind | Line |
|------|------|------|
| `CTerrainMap` | class (C++) | 11 |

#### `code/qcommon/files.h`

| Type | Kind | Line |
|------|------|------|
| `wfhandle_t` | typedef (alias) | 15 |
| `fileInPack_t` | struct | 27 |
| `pack_t` | struct | 33 |
| `directory_t` | struct | 45 |
| `searchpath_t` | struct | 50 |
| `qfile_gut` | union | 60 |
| `qfile_ut` | struct | 67 |
| `fileHandleData_t` | struct | 72 |

#### `code/qcommon/fixedmap.h`

| Type | Kind | Line |
|------|------|------|
| `T` | class (C++) | 14 |

#### `code/qcommon/hstring.h`

| Type | Kind | Line |
|------|------|------|
| `hstring` | class (C++) | 13 |
| `CMapPoolLow` | class (C++) | 90 |
| `T` | class (C++) | 106 |
| `T1` | class (C++) | 190 |
| `K` | class (C++) | 204 |

#### `code/qcommon/qcommon.h`

| Type | Kind | Line |
|------|------|------|
| `msg_t` | struct | 26 |
| `netadrtype_t` | enum | 127 |
| `netsrc_t` | enum | 132 |
| `netadr_t` | struct | 137 |
| `netchan_t` | struct | 164 |
| `svc_ops_e` | enum | 207 |
| `clc_ops_e` | enum | 222 |
| `xcommand_t` | fn-ptr typedef | 272 |
| `joystickAxis_t` | enum | 724 |
| `sysEventType_t` | enum | 734 |
| `sysEvent_t` | struct | 744 |
| `Lump` | struct | 820 |

#### `code/qcommon/qfiles.h`

| Type | Kind | Line |
|------|------|------|
| `vmHeader_t` | struct | 26 |
| `pcx_t` | struct | 48 |
| `TargaHeader` | struct | 73 |
| `md3Frame_t` | struct | 106 |
| `md3Tag_t` | struct | 113 |
| `md3Surface_t` | struct | 129 |
| `md3Shader_t` | struct | 150 |
| `md3Triangle_t` | struct | 155 |
| `md3St_t` | struct | 159 |
| `md3XyzNormal_t` | struct | 163 |
| `md3Header_t` | struct | 168 |
| `dmodel_t` | struct | 250 |
| `dshader_t` | struct | 258 |
| `dplane_t` | struct | 266 |
| `dnode_t` | struct | 271 |
| `dleaf_t` | struct | 278 |
| `dbrushside_t` | struct | 292 |
| `dbrush_t` | struct | 297 |
| `dfog_t` | struct | 303 |
| `mapVert_t` | struct | 316 |
| `drawVert_t` | struct | 345 |
| `dgrid_t` | struct | 360 |
| `dface_t` | struct | 365 |
| `dpatch_t` | struct | 379 |
| `dtrisurf_t` | struct | 395 |
| `dflare_t` | struct | 406 |
| `lump_t` | struct | 420 |
| `dheader_t` | struct | 444 |
| `mapSurfaceType_t` | enum | 540 |
| `dsurface_t` | struct | 548 |
| `hunkAllocType_t` | enum | 573 |
| `glyphInfo_t` | struct | 600 |
| `dfontdat_t` | struct | 617 |

#### `code/qcommon/sparc.h`

| Type | Kind | Line |
|------|------|------|
| `NotSoShort` | struct | 48 |
| `T` | class (C++) | 114 |

#### `code/qcommon/sstring.h`

| Type | Kind | Line |
|------|------|------|
| `sstring` | class (C++) | 12 |
| `sstring_t` | typedef (alias) | 115 |

#### `code/qcommon/stringed_ingame.h`

| Type | Kind | Line |
|------|------|------|
| `LPCSTR` | typedef (alias) | 40 |

#### `code/qcommon/timing.h`

| Type | Kind | Line |
|------|------|------|
| `timing_c` | class (C++) | 2 |

#### `code/qcommon/unzip.h`

| Type | Kind | Line |
|------|------|------|
| `unzFile__` | struct | 5 |
| `unzFile` | typedef (alias) | 6 |
| `tm_unz` | struct | 14 |
| `unz_global_info` | struct | 26 |
| `unz_file_info` | struct | 34 |
| `unz_file_info_internal` | struct | 56 |
| `file_in_zip_read_info_s` | struct | 63 |
| `unz_s` | struct | 87 |

### `code/renderer`

#### `code/renderer/glext.h`

| Type | Kind | Line |
|------|------|------|
| `alpha` | typedef (alias) | 1472 |
| `mode` | typedef (alias) | 1473 |
| `indices` | typedef (alias) | 1474 |
| `table` | typedef (alias) | 1475 |
| `params` | typedef (alias) | 1476 |
| `width` | typedef (alias) | 1478 |
| `data` | typedef (alias) | 1482 |
| `image` | typedef (alias) | 1484 |
| `height` | typedef (alias) | 1491 |
| `span` | typedef (alias) | 1495 |
| `column` | typedef (alias) | 1496 |
| `values` | typedef (alias) | 1497 |
| `sink` | typedef (alias) | 1503 |
| `target` | typedef (alias) | 1505 |
| `pixels` | typedef (alias) | 1507 |
| `texture` | typedef (alias) | 1550 |
| `s` | typedef (alias) | 1552 |
| `v` | typedef (alias) | 1553 |
| `t` | typedef (alias) | 1560 |
| `r` | typedef (alias) | 1568 |
| `q` | typedef (alias) | 1576 |
| `m` | typedef (alias) | 1594 |
| `invert` | typedef (alias) | 1606 |
| `pass` | typedef (alias) | 1607 |
| `img` | typedef (alias) | 1635 |
| `bias` | typedef (alias) | 1655 |
| `weights` | typedef (alias) | 1678 |
| `border` | typedef (alias) | 1701 |
| `param` | typedef (alias) | 1808 |
| `residences` | typedef (alias) | 1844 |
| `textures` | typedef (alias) | 1846 |
| `priorities` | typedef (alias) | 1849 |
| `points` | typedef (alias) | 1858 |
| `pattern` | typedef (alias) | 1887 |
| `i` | typedef (alias) | 1907 |
| `pointer` | typedef (alias) | 1908 |
| `count` | typedef (alias) | 1909 |
| `void` | typedef (alias) | 2012 |
| `buffer` | typedef (alias) | 2013 |
| `marker_p` | typedef (alias) | 2014 |
| `marker` | typedef (alias) | 2015 |
| `factor` | typedef (alias) | 2029 |
| `equation` | typedef (alias) | 2045 |
| `ref` | typedef (alias) | 2191 |
| `pname` | typedef (alias) | 2296 |
| `blue` | typedef (alias) | 2373 |
| `primcount` | typedef (alias) | 2406 |
| `coord` | typedef (alias) | 2419 |
| `tz` | typedef (alias) | 2456 |
| `bz` | typedef (alias) | 2466 |
| `code` | typedef (alias) | 2537 |
| `y` | typedef (alias) | 2590 |
| `z` | typedef (alias) | 2592 |
| `w` | typedef (alias) | 2602 |
| `dfactorAlpha` | typedef (alias) | 2637 |
| `weight` | typedef (alias) | 2683 |
| `componentUsage` | typedef (alias) | 2723 |
| `muxSum` | typedef (alias) | 2724 |
| `modestride` | typedef (alias) | 2822 |
| `ptrstride` | typedef (alias) | 2838 |
| `mask` | typedef (alias) | 2877 |

#### `code/renderer/mdx_format.h`

| Type | Kind | Line |
|------|------|------|
| `mdxaCompQuatBone_t` | struct | 119 |
| `mdxaBone_t` | struct | 139 |
| `mdxmHeader_t` | struct | 153 |
| `mdxmHierarchyOffsets_t` | struct | 177 |
| `mdxmSurfHierarchy_t` | struct | 187 |
| `mdxmLOD_t` | struct | 203 |
| `mdxmLODSurfOffset_t` | struct | 210 |
| `mdxmSurface_t` | struct | 219 |
| `mdxmTriangle_t` | struct | 250 |
| `mdxmVertex_t` | struct | 260 |
| `mdxmVertexTexCoord_t` | struct | 328 |
| `mdxaHeader_t` | struct | 351 |
| `mdxaSkelOffsets_t` | struct | 376 |
| `mdxaSkel_t` | struct | 388 |
| `mdxaIndex_t` | struct | 410 |

#### `code/renderer/qgl.h`

| Type | Kind | Line |
|------|------|------|
| `s` | typedef (alias) | 63 |
| `v` | typedef (alias) | 64 |
| `t` | typedef (alias) | 71 |
| `r` | typedef (alias) | 79 |
| `q` | typedef (alias) | 87 |
| `target` | typedef (alias) | 95 |
| `params` | typedef (alias) | 135 |
| `param` | typedef (alias) | 137 |
| `componentUsage` | typedef (alias) | 139 |
| `muxSum` | typedef (alias) | 141 |
| `piValues` | typedef (alias) | 178 |
| `pfValues` | typedef (alias) | 179 |
| `nNumFormats` | typedef (alias) | 180 |
| `piAttribList` | typedef (alias) | 203 |
| `hPbuffer` | typedef (alias) | 204 |
| `hDC` | typedef (alias) | 205 |
| `piValue` | typedef (alias) | 207 |
| `iBuffer` | typedef (alias) | 229 |
| `string` | typedef (alias) | 266 |
| `program` | typedef (alias) | 267 |
| `programs` | typedef (alias) | 268 |
| `w` | typedef (alias) | 270 |

#### `code/renderer/qgl_console.h`

| Type | Kind | Line |
|------|------|------|
| `GLenum` | typedef (alias) | 23 |
| `GLboolean` | typedef (alias) | 24 |
| `GLbitfield` | typedef (alias) | 25 |
| `GLbyte` | typedef (alias) | 26 |
| `GLshort` | typedef (alias) | 27 |
| `GLint` | typedef (alias) | 28 |
| `GLsizei` | typedef (alias) | 29 |
| `GLubyte` | typedef (alias) | 30 |
| `GLushort` | typedef (alias) | 31 |
| `GLuint` | typedef (alias) | 32 |
| `GLfloat` | typedef (alias) | 33 |
| `GLclampf` | typedef (alias) | 34 |
| `GLdouble` | typedef (alias) | 35 |
| `GLclampd` | typedef (alias) | 36 |
| `GLvoid` | typedef (alias) | 37 |
| `PFNGLMULTITEXCOORD1DARBPROC` | fn-ptr typedef | 776 |
| `PFNGLMULTITEXCOORD1DVARBPROC` | fn-ptr typedef | 777 |
| `PFNGLMULTITEXCOORD1FARBPROC` | fn-ptr typedef | 778 |
| `PFNGLMULTITEXCOORD1FVARBPROC` | fn-ptr typedef | 779 |
| `PFNGLMULTITEXCOORD1IARBPROC` | fn-ptr typedef | 780 |
| `PFNGLMULTITEXCOORD1IVARBPROC` | fn-ptr typedef | 781 |
| `PFNGLMULTITEXCOORD1SARBPROC` | fn-ptr typedef | 782 |
| `PFNGLMULTITEXCOORD1SVARBPROC` | fn-ptr typedef | 783 |
| `PFNGLMULTITEXCOORD2DARBPROC` | fn-ptr typedef | 784 |
| `PFNGLMULTITEXCOORD2DVARBPROC` | fn-ptr typedef | 785 |
| `PFNGLMULTITEXCOORD2FARBPROC` | fn-ptr typedef | 786 |
| `PFNGLMULTITEXCOORD2FVARBPROC` | fn-ptr typedef | 787 |
| `PFNGLMULTITEXCOORD2IARBPROC` | fn-ptr typedef | 788 |
| `PFNGLMULTITEXCOORD2IVARBPROC` | fn-ptr typedef | 789 |
| `PFNGLMULTITEXCOORD2SARBPROC` | fn-ptr typedef | 790 |
| `PFNGLMULTITEXCOORD2SVARBPROC` | fn-ptr typedef | 791 |
| `PFNGLMULTITEXCOORD3DARBPROC` | fn-ptr typedef | 792 |
| `PFNGLMULTITEXCOORD3DVARBPROC` | fn-ptr typedef | 793 |
| `PFNGLMULTITEXCOORD3FARBPROC` | fn-ptr typedef | 794 |
| `PFNGLMULTITEXCOORD3FVARBPROC` | fn-ptr typedef | 795 |
| `PFNGLMULTITEXCOORD3IARBPROC` | fn-ptr typedef | 796 |
| `PFNGLMULTITEXCOORD3IVARBPROC` | fn-ptr typedef | 797 |
| `PFNGLMULTITEXCOORD3SARBPROC` | fn-ptr typedef | 798 |
| `PFNGLMULTITEXCOORD3SVARBPROC` | fn-ptr typedef | 799 |
| `PFNGLMULTITEXCOORD4DARBPROC` | fn-ptr typedef | 800 |
| `PFNGLMULTITEXCOORD4DVARBPROC` | fn-ptr typedef | 801 |
| `PFNGLMULTITEXCOORD4FARBPROC` | fn-ptr typedef | 802 |
| `PFNGLMULTITEXCOORD4FVARBPROC` | fn-ptr typedef | 803 |
| `PFNGLMULTITEXCOORD4IARBPROC` | fn-ptr typedef | 804 |
| `PFNGLMULTITEXCOORD4IVARBPROC` | fn-ptr typedef | 805 |
| `PFNGLMULTITEXCOORD4SARBPROC` | fn-ptr typedef | 806 |
| `PFNGLMULTITEXCOORD4SVARBPROC` | fn-ptr typedef | 807 |
| `PFNGLACTIVETEXTUREARBPROC` | fn-ptr typedef | 808 |
| `PFNGLCLIENTACTIVETEXTUREARBPROC` | fn-ptr typedef | 809 |

#### `code/renderer/tr_jpeg_interface.h`

| Type | Kind | Line |
|------|------|------|
| `LPCSTR` | typedef (alias) | 18 |

#### `code/renderer/tr_landscape.h`

| Type | Kind | Line |
|------|------|------|
| `CTerVert` | class (C++) | 24 |
| `CTRHeightDetails` | class (C++) | 39 |
| `CTRPatch` | class (C++) | 54 |
| `TPatchInfo` | struct | 110 |
| `CTRLandScape` | class (C++) | 121 |

#### `code/renderer/tr_local.h`

| Type | Kind | Line |
|------|------|------|
| `glIndex_t` | typedef (alias) | 19 |
| `dlight_t` | struct | 43 |
| `trRefEntity_t` | struct | 54 |
| `trRefdef_t` | struct | 71 |
| `orientationr_t` | struct | 108 |
| `image_t` | struct | 115 |
| `shaderSort_t` | enum | 144 |
| `genFunc_t` | enum | 180 |
| `deform_t` | enum | 195 |
| `alphaGen_t` | enum | 214 |
| `colorGen_t` | enum | 230 |
| `texCoordGen_t` | enum | 248 |
| `acff_t` | enum | 261 |
| `EGLFogOverride` | enum | 268 |
| `waveForm_t` | struct | 276 |
| `texMod_t` | enum | 287 |
| `deformStage_t` | struct | 299 |
| `texModInfo_t` | struct | 312 |
| `surfaceSprite_t` | struct | 350 |
| `textureBundle_t` | struct | 359 |
| `shaderStage_t` | struct | 380 |
| `cullType_t` | enum | 422 |
| `fogPass_t` | enum | 428 |
| `skyParms_t` | struct | 434 |
| `fogParms_t` | struct | 440 |
| `shader_t` | struct | 446 |
| `skinSurface_t` | struct | 531 |
| `skin_t` | struct | 536 |
| `fog_t` | struct | 543 |
| `viewParms_t` | struct | 556 |
| `surfaceType_t` | enum | 583 |
| `drawSurf_t` | struct | 608 |
| `srfPoly_t` | struct | 620 |
| `srfDisplayList_t` | struct | 628 |
| `srfFlare_t` | struct | 634 |
| `srfGridMesh_t` | struct | 652 |
| `srfSurfaceFace_t` | struct | 707 |
| `srfTriangles_t` | struct | 745 |
| `msurface_t` | struct | 784 |
| `mnode_t` | struct | 799 |
| `mleaf_s` | struct | 812 |
| `bmodel_t` | struct | 851 |
| `mgrid_t` | struct | 859 |
| `world_t` | struct | 896 |
| `modtype_t` | enum | 955 |
| `model_t` | struct | 970 |
| `frontEndCounters_t` | struct | 1047 |
| `glstate_t` | struct | 1065 |
| `backEndCounters_t` | struct | 1075 |
| `backEndState_t` | struct | 1091 |
| `srfTerrain_t` | struct | 1106 |
| `trGlobals_t` | struct | 1126 |
| `levelLightParm_t` | struct | 1607 |
| `color4ub_t` | typedef (alias) | 1628 |
| `stageVars_t` | struct | 1630 |
| `shaderCommands_s` | struct | 1642 |
| `shaderCommands_t` | typedef (alias) | 1682 |
| `CRenderableSurface` | class (C++) | 1840 |
| `renderCommandList_t` | struct | 1977 |
| `setColorCommand_t` | struct | 1982 |
| `drawBufferCommand_t` | struct | 1987 |
| `subImageCommand_t` | struct | 1992 |
| `swapBuffersCommand_t` | struct | 2000 |
| `endFrameCommand_t` | struct | 2004 |
| `stretchPicCommand_t` | struct | 2009 |
| `rotatePicCommand_t` | struct | 2018 |
| `setModeCommand_t` | struct | 2028 |
| `scissorCommand_t` | struct | 2033 |
| `drawSurfsCommand_t` | struct | 2040 |
| `renderCommand_t` | enum | 2048 |
| `backEndData_t` | struct | 2075 |
| `DDS_PIXELFORMAT` | struct | 2141 |
| `DDS_HEADER` | struct | 2153 |

#### `code/renderer/tr_public.h`

| Type | Kind | Line |
|------|------|------|
| `refexport_t` | struct | 19 |

#### `code/renderer/tr_quicksprite.h`

| Type | Kind | Line |
|------|------|------|
| `CQuickSpriteSystem` | class (C++) | 16 |

#### `code/renderer/tr_types.h`

| Type | Kind | Line |
|------|------|------|
| `color4ub_t` | typedef (alias) | 68 |
| `polyVert_t` | struct | 70 |
| `poly_t` | struct | 76 |
| `refEntityType_t` | enum | 82 |
| `refEntity_t` | struct | 100 |
| `refdef_t` | struct | 159 |
| `stereoFrame_t` | enum | 179 |
| `textureCompression_t` | enum | 193 |
| `glconfig_t` | struct | 199 |

### `code/server`

#### `code/server/server.h`

| Type | Kind | Line |
|------|------|------|
| `svEntity_t` | struct | 22 |
| `serverState_t` | enum | 42 |
| `server_t` | struct | 48 |
| `clientSnapshot_t` | struct | 76 |
| `clientState_t` | enum | 89 |
| `client_t` | struct | 99 |
| `challenge_t` | struct | 135 |
| `serverStatic_t` | struct | 142 |

### `code/ui`

#### `code/ui/gameinfo.h`

| Type | Kind | Line |
|------|------|------|
| `gameinfo_import_t` | struct | 9 |

#### `code/ui/ui_local.h`

| Type | Kind | Line |
|------|------|------|
| `uifield_t` | struct | 21 |
| `uiStatic_t` | struct | 66 |
| `modInfo_t` | struct | 98 |
| `playerSpeciesInfo_t` | struct | 103 |
| `uiInfo_t` | struct | 119 |

#### `code/ui/ui_public.h`

| Type | Kind | Line |
|------|------|------|
| `uiimport_t` | struct | 11 |
| `dpTypes_t` | enum | 143 |
| `uiImport_t` | enum | 151 |

#### `code/ui/ui_shared.h`

| Type | Kind | Line |
|------|------|------|
| `pc_token_t` | struct | 25 |
| `columnInfo_t` | struct | 48 |
| `listBoxDef_t` | struct | 54 |
| `editFieldDef_t` | struct | 71 |
| `multiDef_t` | struct | 83 |
| `cachedAssets_t` | struct | 113 |
| `displayContextDef_t` | struct | 169 |
| `rectDef_t` | struct | 306 |
| `UIRectangle` | typedef (alias) | 313 |
| `windowDef_t` | struct | 316 |
| `Window` | typedef (alias) | 342 |
| `colorRangeDef_t` | struct | 344 |
| `modelDef_t` | struct | 350 |
| `itemDef_t` | struct | 374 |
| `menuDef_t` | struct | 427 |
| `textScrollDef_t` | struct | 461 |
| `commandDef_t` | struct | 477 |

### `code/unix`

#### `code/unix/unix_glw.h`

| Type | Kind | Line |
|------|------|------|
| `glwstate_t` | struct | 8 |

### `code/win32`

#### `code/win32/glw_win.h`

| Type | Kind | Line |
|------|------|------|
| `glwstate_t` | struct | 8 |

#### `code/win32/glw_win_dx8.h`

| Type | Kind | Line |
|------|------|------|
| `glwstate_t` | struct | 31 |

#### `code/win32/rad.h`

| Type | Kind | Line |
|------|------|------|
| `RADPCHAR` | typedef (alias) | 524 |
| `bytes` | typedef (alias) | 846 |
| `ptr` | typedef (alias) | 847 |
| `RADTimerSetupType` | fn-ptr typedef | 868 |
| `RADTimerReadType` | fn-ptr typedef | 869 |
| `RADTimerDoneType` | fn-ptr typedef | 870 |

#### `code/win32/snd_fx_img.h`

| Type | Kind | Line |
|------|------|------|
| `DSP_IMAGE_image_FX_INDICES` | enum | 4 |
| `GraphI3DL2_FX0_I3DL2Reverb_STATE` | struct | 15 |
| `LPGraphI3DL2_FX0_I3DL2Reverb_STATE` | struct | 15 |
| `LPCGraphI3DL2_FX0_I3DL2Reverb_STATE` | typedef (alias) | 25 |
| `GraphXTalk_FX0_XTalk_STATE` | struct | 27 |
| `LPGraphXTalk_FX0_XTalk_STATE` | struct | 27 |
| `LPCGraphXTalk_FX0_XTalk_STATE` | typedef (alias) | 37 |
| `GraphVoice_FX0_Voice_0_STATE` | struct | 39 |
| `LPGraphVoice_FX0_Voice_0_STATE` | struct | 39 |
| `LPCGraphVoice_FX0_Voice_0_STATE` | typedef (alias) | 49 |
| `GraphVoice_FX1_Voice_1_STATE` | struct | 51 |
| `LPGraphVoice_FX1_Voice_1_STATE` | struct | 51 |
| `LPCGraphVoice_FX1_Voice_1_STATE` | typedef (alias) | 61 |
| `GraphVoice_FX2_Voice_2_STATE` | struct | 63 |
| `LPGraphVoice_FX2_Voice_2_STATE` | struct | 63 |
| `LPCGraphVoice_FX2_Voice_2_STATE` | typedef (alias) | 73 |
| `GraphVoice_FX3_Voice_3_STATE` | struct | 75 |
| `LPGraphVoice_FX3_Voice_3_STATE` | struct | 75 |
| `LPCGraphVoice_FX3_Voice_3_STATE` | typedef (alias) | 85 |

#### `code/win32/win_file.h`

| Type | Kind | Line |
|------|------|------|
| `wfhandle_t` | typedef (alias) | 16 |

#### `code/win32/win_input.h`

| Type | Kind | Line |
|------|------|------|
| `JoystickInfo` | struct | 79 |
| `PadInfo` | struct | 86 |

#### `code/win32/win_local.h`

| Type | Kind | Line |
|------|------|------|
| `WinVars_t` | struct | 57 |

#### `code/win32/win_stencilshadow.h`

| Type | Kind | Line |
|------|------|------|
| `edgeDef_t` | struct | 12 |
| `StencilShadow` | class (C++) | 24 |

### `code/zlib32`

#### `code/zlib32/deflate.h`

| Type | Kind | Line |
|------|------|------|
| `block_state` | enum | 83 |
| `ct_data` | struct | 92 |
| `static_tree_desc` | struct | 106 |
| `tree_desc` | struct | 115 |
| `deflate_state` | struct | 123 |
| `compress_func` | fn-ptr typedef | 220 |
| `config` | struct | 222 |

#### `code/zlib32/inflate.h`

| Type | Kind | Line |
|------|------|------|
| `check_func` | fn-ptr typedef | 12 |
| `inflate_block_mode` | enum | 14 |
| `inflate_codes_mode` | enum | 29 |
| `inflate_mode` | enum | 43 |
| `inflate_huft_t` | struct | 56 |
| `inflate_codes_state_t` | struct | 64 |
| `inflate_blocks_state_t` | struct | 93 |
| `inflate_state` | struct | 129 |

#### `code/zlib32/zip.h`

| Type | Kind | Line |
|------|------|------|
| `ELevel` | enum | 64 |
| `EFlush` | enum | 79 |
| `EStatus` | enum | 89 |
| `z_stream` | struct | 129 |

