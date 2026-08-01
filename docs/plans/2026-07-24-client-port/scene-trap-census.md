# Scene-trap census (wayfinder #17, DEC-54)

This census counts every renderer-facing submission across the four cgame replay traces (swoop1, sabers1, spectator, ffa1 - all four replay both referee gates at 0 findings). It is the empirical live-play renderer gate and the wave-plan seed per DEC-54. The census complement (portals, mirrors, weather, flares, dynamic glow, videoMap, and every unlisted refEntity type and renderfx flag) stays fog with `//TODO: Port` markers.

Generated 2026-08-01 by `crates/cgame/tests/scene_census.rs` over the local traces (DEC-48.4 keeps traces out of git). Regenerate: `JKA_CENSUS_OUT=<path> cargo test -p cgame --release --test scene_census -- --ignored`.

## trace-swoop1

| submission | count |
|---|---:|
| 2d/DrawRotatePic | 547 |
| 2d/DrawStretchPic | 57603 |
| 2d/Font_DrawString | 12067 |
| 2d/SetColor | 49313 |
| dlight/calls | 81579 |
| fx/AddPrimitive | 24185 |
| fx/AddSprite | 370 |
| fx/PlayEffect(id) | 5943 |
| marks/MarkFragments | 59248 |
| poly/calls | 8139 |
| poly/verts | 32545 |
| refent/RT_LINE | 72742 |
| refent/RT_MODEL | 111582 |
| refent/RT_MODEL/ghoul2 | 73617 |
| refent/RT_MODEL/hModel | 37965 |
| refent/RT_SABER_GLOW | 72742 |
| refent/RT_SPRITE | 945 |
| renderfx/RF_DISTORTION | 1255 |
| renderfx/RF_FORCE_ENT_ALPHA | 1315 |
| renderfx/RF_LIGHTING_ORIGIN | 69916 |
| renderfx/RF_NOSHADOW | 24024 |
| renderfx/RF_RGB_TINT | 60 |
| scene/ClearScene | 2005 |
| scene/RenderScene | 2002 |
| scene/rdflags/0x10 | 2002 |

## trace-sabers1

| submission | count |
|---|---:|
| 2d/DrawStretchPic | 70278 |
| 2d/Font_DrawString | 13101 |
| 2d/SetColor | 57864 |
| dlight/calls | 11863 |
| fx/AddLine | 28 |
| fx/AddPrimitive | 4347 |
| fx/AddSprite | 68 |
| fx/PlayEffect(id) | 17187 |
| marks/MarkFragments | 13342 |
| poly/calls | 256131 |
| poly/verts | 1020179 |
| refent/RT_LINE | 9531 |
| refent/RT_MODEL | 110492 |
| refent/RT_MODEL/ghoul2 | 38750 |
| refent/RT_MODEL/hModel | 88271 |
| refent/RT_SABER_GLOW | 9531 |
| refent/RT_SPRITE | 27566 |
| renderfx/RF_DISINTEGRATE1 | 62 |
| renderfx/RF_DISINTEGRATE2 | 689 |
| renderfx/RF_DISTORTION | 695 |
| renderfx/RF_FORCE_ENT_ALPHA | 3690 |
| renderfx/RF_LIGHTING_ORIGIN | 16471 |
| renderfx/RF_MINLIGHT | 16529 |
| renderfx/RF_NOSHADOW | 10548 |
| renderfx/RF_RGB_TINT | 1364 |
| scene/ClearScene | 3311 |
| scene/RenderScene | 3308 |
| scene/rdflags/0x10 | 3308 |

## trace-spectator

| submission | count |
|---|---:|
| 2d/DrawStretchPic | 11600 |
| 2d/Font_DrawString | 6327 |
| 2d/SetColor | 10631 |
| dlight/calls | 4187 |
| fx/AddLine | 2 |
| fx/AddPrimitive | 1388 |
| fx/PlayEffect(id) | 10287 |
| marks/MarkFragments | 5158 |
| poly/calls | 99393 |
| poly/verts | 394436 |
| refent/RT_LINE | 2008 |
| refent/RT_MODEL | 44443 |
| refent/RT_MODEL/ghoul2 | 11365 |
| refent/RT_MODEL/hModel | 39182 |
| refent/RT_SABER_GLOW | 2008 |
| refent/RT_SPRITE | 117 |
| renderfx/RF_DEPTHHACK | 188 |
| renderfx/RF_FIRST_PERSON | 188 |
| renderfx/RF_FORCE_ENT_ALPHA | 365 |
| renderfx/RF_LIGHTING_ORIGIN | 5261 |
| renderfx/RF_MINLIGHT | 6104 |
| renderfx/RF_NOSHADOW | 7868 |
| renderfx/RF_RGB_TINT | 126 |
| renderfx/RF_THIRD_PERSON | 188 |
| scene/ClearScene | 1317 |
| scene/RenderScene | 1314 |
| scene/rdflags/0x10 | 1314 |

## trace-ffa1

| submission | count |
|---|---:|
| 2d/DrawStretchPic | 145779 |
| 2d/Font_DrawString | 17017 |
| 2d/SetColor | 117605 |
| dlight/calls | 14885 |
| fx/AddElectricity | 87 |
| fx/AddLine | 22 |
| fx/AddPrimitive | 2736 |
| fx/AddSprite | 4490 |
| fx/PlayEffect(id) | 33223 |
| marks/MarkFragments | 14574 |
| poly/calls | 79852 |
| poly/verts | 314948 |
| refent/RT_LINE | 10065 |
| refent/RT_MODEL | 168908 |
| refent/RT_MODEL/ghoul2 | 52733 |
| refent/RT_MODEL/hModel | 148106 |
| refent/RT_SABER_GLOW | 10065 |
| refent/RT_SPRITE | 121 |
| renderfx/RF_DISINTEGRATE1 | 63 |
| renderfx/RF_DISINTEGRATE2 | 1030 |
| renderfx/RF_DISTORTION | 175 |
| renderfx/RF_FORCE_ENT_ALPHA | 1216 |
| renderfx/RF_LIGHTING_ORIGIN | 16511 |
| renderfx/RF_MINLIGHT | 29287 |
| renderfx/RF_NOSHADOW | 22909 |
| renderfx/RF_RGB_TINT | 7129 |
| renderfx/RF_THIRD_PERSON | 6 |
| renderfx/RF_VOLUMETRIC | 50 |
| scene/ClearScene | 4779 |
| scene/RenderScene | 4776 |
| scene/rdflags/0x10 | 4776 |

## aggregate (all traces)

| submission | count |
|---|---:|
| 2d/DrawRotatePic | 547 |
| 2d/DrawStretchPic | 285260 |
| 2d/Font_DrawString | 48512 |
| 2d/SetColor | 235413 |
| dlight/calls | 112514 |
| fx/AddElectricity | 87 |
| fx/AddLine | 52 |
| fx/AddPrimitive | 32656 |
| fx/AddSprite | 4928 |
| fx/PlayEffect(id) | 66640 |
| marks/MarkFragments | 92322 |
| poly/calls | 443515 |
| poly/verts | 1762108 |
| refent/RT_LINE | 94346 |
| refent/RT_MODEL | 435425 |
| refent/RT_MODEL/ghoul2 | 176465 |
| refent/RT_MODEL/hModel | 313524 |
| refent/RT_SABER_GLOW | 94346 |
| refent/RT_SPRITE | 28749 |
| renderfx/RF_DEPTHHACK | 188 |
| renderfx/RF_DISINTEGRATE1 | 125 |
| renderfx/RF_DISINTEGRATE2 | 1719 |
| renderfx/RF_DISTORTION | 2125 |
| renderfx/RF_FIRST_PERSON | 188 |
| renderfx/RF_FORCE_ENT_ALPHA | 6586 |
| renderfx/RF_LIGHTING_ORIGIN | 108159 |
| renderfx/RF_MINLIGHT | 51920 |
| renderfx/RF_NOSHADOW | 65349 |
| renderfx/RF_RGB_TINT | 8679 |
| renderfx/RF_THIRD_PERSON | 194 |
| renderfx/RF_VOLUMETRIC | 50 |
| scene/ClearScene | 11412 |
| scene/RenderScene | 11400 |
| scene/rdflags/0x10 | 11400 |

### shaders via 2d (62 distinct)

- `gfx/2d/crosshairb` x11328
- `gfx/2d/numbers/t_eight` x2315
- `gfx/2d/numbers/t_five` x3881
- `gfx/2d/numbers/t_four` x1081
- `gfx/2d/numbers/t_nine` x6783
- `gfx/2d/numbers/t_one` x25631
- `gfx/2d/numbers/t_seven` x1931
- `gfx/2d/numbers/t_six` x1888
- `gfx/2d/numbers/t_three` x1044
- `gfx/2d/numbers/t_two` x7695
- `gfx/2d/numbers/t_zero` x43832
- `gfx/effects/saberFlare` x21
- `gfx/hud/ammo_tic_1` x4517
- `gfx/hud/ammo_tic_2` x4517
- `gfx/hud/ammo_tic_3` x4517
- `gfx/hud/ammo_tic_4` x4517
- `gfx/hud/armor_tic_1` x4776
- `gfx/hud/armor_tic_2` x4776
- `gfx/hud/armor_tic_3` x5081
- `gfx/hud/armor_tic_4` x8871
- `gfx/hud/force_tic_1` x8446
- `gfx/hud/force_tic_2` x9866
- `gfx/hud/force_tic_3` x10444
- `gfx/hud/force_tic_4` x10444
- `gfx/hud/health_tic_1` x10338
- `gfx/hud/health_tic_2` x10338
- `gfx/hud/health_tic_3` x10444
- `gfx/hud/health_tic_4` x10444
- `gfx/hud/hudleft` x20888
- `gfx/hud/load_tick` x132
- `gfx/hud/load_tick_cap` x264
- `gfx/hud/mp_levelload` x132
- `gfx/hud/saber_fast` x1998
- `gfx/hud/saber_med` x1021
- `gfx/hud/saber_strong` x2908
- `gfx/hud/vehicle_ammo_tick` x2735
- `gfx/hud/vehicle_frame` x1094
- `gfx/hud/vehicle_grid` x547
- `gfx/hud/vehicle_grid2` x1641
- `gfx/hud/vehicle_health_tick` x5718
- `gfx/hud/vehicle_turbo_tick` x1801
- `gfx/hud/w_icon_blaster_pistol` x350
- `gfx/hud/w_icon_c_rifle` x89
- `gfx/hud/w_icon_demp2` x89
- `gfx/hud/w_icon_flechette` x89
- `gfx/hud/w_icon_lightsaber` x350
- `gfx/hud/w_icon_merrsonn` x89
- `gfx/hud/w_icon_repeater` x89
- `gfx/hud/w_icon_thermal` x89
- `gfx/hud/w_icon_tripmine` x89
- `gfx/menus/menu_buttonback.tga` x72
- `gfx/menus/radar/arrow_w` x547
- `gfx/menus/radar/radar.png` x547
- `gfx/menus/radar/swoop` x497
- `gfx/mp/small_shield` x169
- `levelshots/mp/ffa1` x93
- `levelshots/t2_trip` x39
- `models/players/cultist/icon_default` x1178
- `models/players/desann/icon_default` x1314
- `models/players/kyle/icon_default` x5183
- `models/players/swamptrooper/icon_default` x3653
- `white` x547

### shaders via poly (3 distinct)

- `gfx/damage/rivetmark` x292999
- `gfx/effects/saberDamageGlow` x70591
- `markShadow` x79925

### shaders via refEntity.customShader (27 distinct)

- `effects/refract_2` x1671
- `effects/refraction` x454
- `gfx/effects/burn` x125
- `gfx/effects/demp2shell` x50
- `gfx/effects/forcePush` x23376
- `gfx/effects/sabers/blue_glow` x6195
- `gfx/effects/sabers/blue_line` x6195
- `gfx/effects/sabers/green_glow` x2002
- `gfx/effects/sabers/green_line` x2002
- `gfx/effects/sabers/orange_glow` x8008
- `gfx/effects/sabers/orange_line` x8008
- `gfx/effects/sabers/purple_glow` x1118
- `gfx/effects/sabers/purple_line` x1118
- `gfx/effects/sabers/red_glow` x76190
- `gfx/effects/sabers/red_line` x71017
- `gfx/effects/sabers/yellow_glow` x6006
- `gfx/effects/sabers/yellow_line` x6006
- `gfx/effects/solidWhite_cull` x256
- `gfx/misc/electric` x413
- `gfx/misc/fullbodyelectric2` x382
- `gfx/misc/personalshield` x785
- `gfx/mp/chat_icon` x1325
- `halfShieldShell` x1112
- `powerups/invulnerabilityshell` x1671
- `powerups/placeholder` x4461
- `powerups/rezout` x756
- `powerups/ysalimarishell` x501

### effects played (59 distinct)

- `blaster/deflect` x2
- `blaster/flesh_impact` x1
- `blaster/muzzle_flash` x6
- `blaster/shot` x20
- `blaster/wall_impact` x2
- `bowcaster/explosion` x1
- `bowcaster/muzzle_flash` x2
- `bowcaster/shot` x2
- `bryar/crackleShot` x169
- `bryar/muzzle_flash` x43
- `bryar/shot` x437
- `bryar/wall_impact` x17
- `bryar/wall_impact2` x3
- `bryar/wall_impact3` x4
- `concussion/explosion` x5
- `concussion/muzzle_flash` x10
- `concussion/shot` x46
- `demp2/altDetonate.efx` x1
- `demp2/flesh_impact` x1
- `demp2/muzzle_flash` x4
- `demp2/projectile` x13
- `detpack/explosion.efx` x8
- `disruptor/death_smoke` x179
- `disruptor/flesh_impact` x4
- `disruptor/muzzle_flash` x106
- `disruptor/wall_impact` x48
- `effects/force/lightningwide.efx` x73
- `effects/mp/drainwide.efx` x83
- `effects/ships/dest_burning.efx` x346
- `flechette/alt_blow.efx` x2
- `flechette/alt_shot` x298
- `flechette/flesh_impact` x16
- `flechette/muzzle_flash` x38
- `flechette/shot` x6581
- `materials/gravel` x33
- `materials/gravel_large` x413
- `mp/itemcone.efx` x51605
- `mp/spawn.efx` x12
- `repeater/muzzle_flash` x10
- `repeater/projectile` x150
- `repeater/wall_impact` x5
- `rocket/explosion` x3
- `rocket/muzzle_flash` x6
- `rocket/shot` x44
- `saber/blood_sparks_25_mp.efx` x15
- `saber/blood_sparks_50_mp.efx` x41
- `saber/blood_sparks_mp.efx` x156
- `saber/saber_block.efx` x75
- `ships/burner` x142
- `ships/jet` x134
- `ships/ship_explosion_mark` x2
- `ships/swoop_explosion` x2
- `ships/swoop_turbo_start` x2
- `sparks/spark_nosnd.efx` x4872
- `thermal/explosion` x7
- `thermal/shockwave` x7
- `tripMine/explosion` x3
- `tripMine/glowbit.efx` x207
- `tripMine/laserMP.efx` x123

### models registered (189)

- `*1`
- `*10`
- `*11`
- `*12`
- `*13`
- `*14`
- `*15`
- `*16`
- `*17`
- `*18`
- `*19`
- `*2`
- `*20`
- `*21`
- `*22`
- `*23`
- `*24`
- `*25`
- `*26`
- `*27`
- `*28`
- `*29`
- `*3`
- `*30`
- `*31`
- `*32`
- `*33`
- `*34`
- `*35`
- `*36`
- `*37`
- `*38`
- `*39`
- `*4`
- `*40`
- `*41`
- `*42`
- `*43`
- `*44`
- `*45`
- `*46`
- `*47`
- `*48`
- `*49`
- `*5`
- `*50`
- `*51`
- `*52`
- `*53`
- `*54`
- `*55`
- `*56`
- `*57`
- `*58`
- `*59`
- `*6`
- `*60`
- `*61`
- `*62`
- `*63`
- `*64`
- `*65`
- `*66`
- `*67`
- `*68`
- `*69`
- `*7`
- `*70`
- `*71`
- `*72`
- `*73`
- `*74`
- `*75`
- `*76`
- `*77`
- `*78`
- `*79`
- `*8`
- `*9`
- `/models/items/a_pwr_converter.md3`
- `models/chunks/crate/crate1_1.md3`
- `models/chunks/crate/crate1_2.md3`
- `models/chunks/crate/crate1_3.md3`
- `models/chunks/crate/crate1_4.md3`
- `models/chunks/crate/crate2_1.md3`
- `models/chunks/crate/crate2_2.md3`
- `models/chunks/crate/crate2_3.md3`
- `models/chunks/crate/crate2_4.md3`
- `models/chunks/metal/metal1_1.md3`
- `models/chunks/metal/metal1_2.md3`
- `models/chunks/metal/metal1_3.md3`
- `models/chunks/metal/metal1_4.md3`
- `models/chunks/metal/metal2_1.md3`
- `models/chunks/metal/metal2_2.md3`
- `models/chunks/metal/metal2_3.md3`
- `models/chunks/metal/metal2_4.md3`
- `models/chunks/metal/wmetal1_1.md3`
- `models/chunks/metal/wmetal1_2.md3`
- `models/chunks/metal/wmetal1_3.md3`
- `models/chunks/metal/wmetal1_4.md3`
- `models/chunks/rock/rock1_1.md3`
- `models/chunks/rock/rock1_2.md3`
- `models/chunks/rock/rock1_3.md3`
- `models/chunks/rock/rock1_4.md3`
- `models/chunks/rock/rock2_1.md3`
- `models/chunks/rock/rock2_2.md3`
- `models/chunks/rock/rock2_3.md3`
- `models/chunks/rock/rock2_4.md3`
- `models/chunks/rock/rock3_1.md3`
- `models/chunks/rock/rock3_2.md3`
- `models/chunks/rock/rock3_3.md3`
- `models/chunks/rock/rock3_4.md3`
- `models/items/energy_cell.md3`
- `models/items/metallic_bolts.md3`
- `models/items/power_cell.md3`
- `models/items/remote.md3`
- `models/items/rockets.md3`
- `models/items/sphere.md3`
- `models/map_objects/desert/crawler_junk2.md3`
- `models/map_objects/desert/crawler_junk3.md3`
- `models/map_objects/desert/evaporator.md3`
- `models/map_objects/desert/wall_generator.md3`
- `models/map_objects/hoth/bed.md3`
- `models/map_objects/mp/holo.md3`
- `models/map_objects/mp/medpac.md3`
- `models/map_objects/mp/psd.md3`
- `models/map_objects/mp/psd_sm.md3`
- `models/map_objects/mp/sphere.md3`
- `models/map_objects/quicktrip/rib_single_new.md3`
- `models/map_objects/roof_top/crate3.md3`
- `models/players/human_merc/model.glm`
- `models/players/swoop/model.glm`
- `models/weaphits/testboom.md3`
- `models/weapons2/blaster_pistol/blaster_pistol.md3`
- `models/weapons2/blaster_pistol/blaster_pistol_hand.md3`
- `models/weapons2/blaster_pistol/blaster_pistol_w.glm`
- `models/weapons2/blaster_r/blaster.md3`
- `models/weapons2/blaster_r/blaster_hand.md3`
- `models/weapons2/blaster_r/blaster_w.glm`
- `models/weapons2/bowcaster/bowcaster.md3`
- `models/weapons2/bowcaster/bowcaster_hand.md3`
- `models/weapons2/bowcaster/bowcaster_w.glm`
- `models/weapons2/concussion/c_rifle.md3`
- `models/weapons2/concussion/c_rifle_hand.md3`
- `models/weapons2/concussion/c_rifle_w.glm`
- `models/weapons2/demp2/demp2.md3`
- `models/weapons2/demp2/demp2_hand.md3`
- `models/weapons2/demp2/demp2_w.glm`
- `models/weapons2/detpack/det_pack.md3`
- `models/weapons2/detpack/det_pack_hand.md3`
- `models/weapons2/detpack/det_pack_proj.glm`
- `models/weapons2/detpack/det_pack_pu.md3`
- `models/weapons2/disruptor/disruptor.md3`
- `models/weapons2/disruptor/disruptor_barrel.md3`
- `models/weapons2/disruptor/disruptor_hand.md3`
- `models/weapons2/disruptor/disruptor_w.glm`
- `models/weapons2/golan_arms/golan_arms.md3`
- `models/weapons2/golan_arms/golan_arms_barrel.md3`
- `models/weapons2/golan_arms/golan_arms_hand.md3`
- `models/weapons2/golan_arms/golan_arms_w.glm`
- `models/weapons2/golan_arms/projectile.md3`
- `models/weapons2/golan_arms/projectileMain.md3`
- `models/weapons2/heavy_repeater/heavy_repeater.md3`
- `models/weapons2/heavy_repeater/heavy_repeater_barrel.md3`
- `models/weapons2/heavy_repeater/heavy_repeater_hand.md3`
- `models/weapons2/heavy_repeater/heavy_repeater_w.glm`
- `models/weapons2/laser_trap/laser_trap.md3`
- `models/weapons2/laser_trap/laser_trap_hand.md3`
- `models/weapons2/laser_trap/laser_trap_pu.md3`
- `models/weapons2/laser_trap/laser_trap_w.glm`
- `models/weapons2/merr_sonn/merr_sonn.md3`
- `models/weapons2/merr_sonn/merr_sonn_barrel.md3`
- `models/weapons2/merr_sonn/merr_sonn_hand.md3`
- `models/weapons2/merr_sonn/merr_sonn_w.glm`
- `models/weapons2/merr_sonn/projectile.md3`
- `models/weapons2/saber/saber_w.glm`
- `models/weapons2/saber/saber_w.md3`
- `models/weapons2/saber_8/saber_8.glm`
- `models/weapons2/stun_baton/baton.md3`
- `models/weapons2/stun_baton/baton_barrel.md3`
- `models/weapons2/stun_baton/baton_barrel2.md3`
- `models/weapons2/stun_baton/baton_barrel3.md3`
- `models/weapons2/stun_baton/baton_hand.md3`
- `models/weapons2/stun_baton/baton_w.glm`
- `models/weapons2/thermal/thermal.md3`
- `models/weapons2/thermal/thermal_hand.md3`
- `models/weapons2/thermal/thermal_proj.md3`
- `models/weapons2/thermal/thermal_pu.md3`
- `models/weapons2/thermal/thermal_w.glm`

### skins registered (31)

- `models/players/alora/model_default.skin`
- `models/players/bespin_cop/model_default.skin`
- `models/players/boba_fett/model_default.skin`
- `models/players/chewbacca/model_default.skin`
- `models/players/chiss/model_default.skin`
- `models/players/cultist/model_default.skin`
- `models/players/desann/model_default.skin`
- `models/players/galak/model_default.skin`
- `models/players/gran/model_default.skin`
- `models/players/human_merc/model_default.skin`
- `models/players/imperial/model_default.skin`
- `models/players/imperial_worker/model_default.skin`
- `models/players/jeditrainer/model_default.skin`
- `models/players/kyle/model_default.skin`
- `models/players/luke/model_default.skin`
- `models/players/morgan/model_default.skin`
- `models/players/rax_joris/model_default.skin`
- `models/players/rebel/model_default.skin`
- `models/players/reborn_new/model_default.skin`
- `models/players/reborn_twin/model_default.skin`
- `models/players/reelo/model_default.skin`
- `models/players/rodian/model_default.skin`
- `models/players/saboteur/model_default.skin`
- `models/players/shadowtrooper/model_default.skin`
- `models/players/stormpilot/model_default.skin`
- `models/players/stormtrooper/model_default.skin`
- `models/players/swamptrooper/model_default.skin`
- `models/players/swoop/model_black|blue|gold|green|purple|default|silver.skin`
- `models/players/swoop/model_default.skin`
- `models/players/tusken/model_default.skin`
- `models/players/weequay/model_default.skin`

### fonts registered (4)

- `arialnb`
- `ergoec`
- `fonts/reallybigfont`
- `ocr_a`
