# Quirk Ledger

This ledger collects the Raven behavioral quirks that the port preserves on purpose, plus the sites where the port diverges because Raven is undefined behavior. The entries come from a sweep of the `PORT-NOTE` and quirk comments in `crates/`, the referee masks in `crates/cgame/tests/replay_support/mod.rs`, and the quirk rulings in `docs/decisions.md`. One entry covers one quirk, and an entry lists every known site of that quirk. Every verdict is PROPOSED until the user ratifies it. Rules of record: `docs/porting-rules.md` §A2 (no speculative behavior), §19 (diverge only on Raven UB), §20 (preserve emergent quirks, drop dead surface). Plain §20 dead-surface drops are out of scope here. Classes: preserved-bug, preserved-dead-code, UB-divergence, wire-quirk, per-mode-quirk.

## Class: preserved-bug

## Q-1 msvcrt qsort tie order rejected

**What.** Retail win32 DLLs bound msvcrt's qsort, whose tie permutation differs from Raven's own `bg_lib.c` body. The port makes the `bg_lib` body canonical everywhere, so msvcrt's tie order is never reproduced.
**Where.** `crates/native/sort/src/lib.rs:1-7`. Oracle: `oracle/codemp/game/bg_lib.c` (qsort body). Ruling: DEC-34, `docs/decisions.md:580`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. DEC-34 already closed this, and no tie is compat-observable at any call site.

## Q-2 M_PI expands as the math.h double

**What.** Retail Windows compiled Raven's f32 `M_PI` fallback, while every buildable oracle gets the math.h double. The port takes the double per DEC-41, a known ~1 ulp divergence from retail Windows.
**Where.** `crates/native/math/src/qmath.rs:564,619-620,1071`. Oracle: `oracle/codemp/game/q_shared.h:547-549`. Ruling: DEC-41, `docs/decisions.md:1045`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. DEC-41 is user-ratified and the divergence is unmeasurable in play.

## Q-3 holdrand LCG stays 32-bit wide

**What.** Raven's `holdrand` generator shifts by 17 and assumes a 32-bit `unsigned long`. The LP64 oracle build makes `irand(1,2)` return ±32k garbage, the level-1 lightning "instagib" live finding, so the port pins the retail 32-bit width.
**Where.** `crates/native/math/src/rng.rs:1-23`. Oracle: `oracle/codemp/game/q_math.c:1432-1474`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. The 32-bit width is the shipped retail behavior, and the LP64 garbage is a build artifact.

## Q-4 Bot AI stays oracle-faithful

**What.** Retail-SDK bots aim poorly and drop targets, and OpenJK improved that behavior. DEC-30 keeps the oracle behavior and rejects the OpenJK bot changes.
**Where.** `crates/mp/game/src/ai_main.rs` (subsystem-wide). Oracle: `oracle/codemp/game/ai_main.c` and siblings. Ruling: DEC-30, `docs/decisions.md:486`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. DEC-30 is user-ratified and the referee proves parity with the oracle.

## Q-5 Saber jump-attack branch reads the wrong saber

**What.** The `saber2` jump-attack branch tests `saber2` and then reads `saber1`. The port copies that swap.
**Where.** `crates/mp/bg/src/bg_saber.rs:1580-1583`. Oracle: `oracle/codemp/game/bg_saber.c:2297`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. A fix changes live dual-saber move selection and breaks parity.

## Q-6 Vehicle VEC3 parse stores through the wrong field table

**What.** The vehicle VEC3 parse arm takes its store offset from `vehWeaponFields[i]` instead of `vehicleFields[i]`. The port copies the wrong-table read.
**Where.** `crates/mp/bg/src/bg_vehicleLoad.rs:438-444`. Oracle: `oracle/codemp/game/bg_vehicleLoad.c:885-887`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Vehicle files load with this behavior today, and a fix would change parsed vehicle data.

## Q-7 Meditate taunt sets legsTimer inside the torso guard

**What.** Inside the `torsoTimer < 100` guard the code sets `legsTimer`, so the torso timer is never floored. The port copies it.
**Where.** `crates/mp/bg/src/bg_pmove.rs:1155-1160`. Oracle: `oracle/codemp/game/bg_pmove.c:10349-10352`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Pmove is referee-gated and must match byte-for-byte.

## Q-8 Rocket-lock cloak test masks with the raw PW_CLOAKED value

**What.** One cloak test masks `powerups` with the raw value 11 instead of `1 << PW_CLOAKED`, so it tests bits 0, 1, and 3. It is the only shift-less cloak test in the tree.
**Where.** `crates/mp/bg/src/bg_pmove.rs:6458-6462`. Oracle: `oracle/codemp/game/bg_pmove.c:5925`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Rocket-lock behavior is gameplay-visible and referee-gated.

## Q-9 Fighter pitch snaps to -60 from above

**What.** The nose-down clamp tests `pitch > -60` and then assigns -60, so any pitch above -60 snaps straight to -60. The port copies Raven's own quirk.
**Where.** `crates/mp/bg/src/vehicles/fighter_npc.rs:399-406`. Oracle: `oracle/codemp/game/FighterNPC.c:963-968`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Vehicle prediction parity was validated live and depends on this arm.

## Q-10 Class spawn selection indexes the wrong array

**What.** After collecting class-legal spots into `classSpots`, the oracle returns `spots[selection]` with a `classCount`-bounded index. The port copies the wrong-array read.
**Where.** `crates/mp/game/src/g_team.rs:1276-1281`. Oracle: `oracle/codemp/game/g_team.c:1034`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Siege spawn placement is gameplay-visible and the wrong read stays in bounds.

## Q-11 NPC playerTeam parses from the key, never the value

**What.** The `playerTeam` parse arm formats `NPC%s` from `token`, which still holds the key "playerTeam", instead of the parsed value. The port copies it.
**Where.** `crates/mp/game/src/NPC_stats.rs:642-646`. Oracle: `oracle/codemp/game/NPC_stats.c:743`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. NPC team assignment in shipped .npc files depends on the lookup failing the same way.

## Q-12 Waypoint save drops waypoint 0's neighbor list

**What.** The waypoint saver builds `storeString` for waypoint 0's neighbors and never concatenates it into `fileString`. The port copies the dropped concatenation.
**Where.** `crates/mp/game/src/ai_wpnav.rs:2424-2433`. Oracle: `oracle/codemp/game/ai_wpnav.c:2418-2447`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Saved .wnt files must stay byte-compatible with oracle output.

## Q-13 Teamkill auto-ban guard is always true

**What.** The auto-ban path guards on `sess.IPstring`, which is an array whose address is never null, so the ban always runs. The port copies the unconditional ban.
**Where.** `crates/mp/game/src/g_cmds.rs:613-620` (second site near `:697`). Oracle: `oracle/codemp/game/g_cmds.c:541-543,608-610`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Server admin behavior matches retail, and the guard never gated anything.

## Q-14 Weapon log name table is short and index-shifted

**What.** `weaponNameFromIndex` holds 16 initializers in a 19-slot array, so three slots print as `(null)`, and the names predate `WP_MELEE`, so they are index-shifted. The port copies both.
**Where.** `crates/mp/game/src/g_log.rs:19-26`. Oracle: `oracle/codemp/game/g_log.c:79-97`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. The quirk only affects log text and log parity is cheap to keep.

## Q-15 SetTeamNumbers only ever checks entity 0

**What.** The outer loop bound is `i < 1`, so the function only inspects `g_entities[0]`. The zero-team average-health division is UB in C, and the port saturates the NaN result to 0.
**Where.** `crates/mp/game/src/NPC_utils.rs:671-681`. Oracle: `oracle/codemp/game/NPC_utils.c:818-847`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. The dead bound is Raven's shipped behavior and callers expect it.

## Q-16 Jedi speed-counter chance falls through to 1

**What.** The `g_spskill` switch has no breaks on cases 0 and 1, so `chance` ends at 1 for every skill level. The port writes the three assignments in sequence to copy the fall-through.
**Where.** `crates/mp/game/src/NPC_AI_Jedi.rs:7515-7523`. Oracle: `oracle/codemp/game/NPC_AI_Jedi.c:6148-6158`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. NPC force-speed response rate is gameplay-visible.

## Q-17 GetPairedValue mutates the buffer and can read backward

**What.** Raven's key finder overwrites `//` comment lines with `/` characters inside the input buffer, and its `startletter` guard checks 0 instead of -1, an inherited backward-read UB envelope. The port preserves the mutation and the envelope.
**Where.** `crates/mp/game/src/ai_util.rs:153-165`. Oracle: `oracle/codemp/game/ai_util.c:236-326`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Bot personality parsing was spot-checked for parity under DEC-30.

## Q-18 Master-server strstr has its arguments reversed

**What.** The master-address check calls `strstr(":", masterString)` with needle and haystack swapped, so the port-defaulting branch triggers on the reversed test. The port copies it verbatim.
**Where.** `crates/mp/engine/server/src/sv_main.rs:344-346`. Oracle: `oracle/codemp/server/sv_main.cpp:265`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Heartbeat addressing matches retail and the emergent result is correct for default configs.

## Q-19 Nav trap SETCHECKEDNODE falls through

**What.** The `G_NAV_SETCHECKEDNODE` case has no return, so C fell through toward the next nav cases. The port transcribes the fall-through per ruling NAV-Q3 instead of adding the return.
**Where.** `crates/mp/engine/server/src/sv_game.rs:1693-1695`. Oracle: `oracle/codemp/server/sv_game.cpp:928-933`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. The nav trap surface is referee-covered and the ruling is settled.

## Q-20 Projectile model row uses the weapon offset macro

**What.** The `projectileinfo_t` field table's `model` row takes its offset with `WEAPON_OFS`, and every neighbor uses `PROJECTILE_OFS`. The port reproduces the mixed macro.
**Where.** `crates/mp/engine/botlib/src/be_ai_weap_fns.rs:254-258`. Oracle: `oracle/codemp/botlib/be_ai_weap.cpp:69`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Bot weapon config parsing must match the oracle's struct stamping.

## Q-21 Bone-anim reset is asymmetric between found and added bones

**What.** `G2_Set_Bone_Anim_No_BS` resets `blendStart` to 0 on the found-bone branch and skips the reset on the added-bone branch. The port preserves the asymmetry.
**Where.** `crates/mp/engine/ghoul2/src/api_ragdoll.rs:655-656`. Oracle: `oracle/codemp/ghoul2/G2_bones.cpp:1522-1567`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Animation blending is visually sensitive and golden-verified.

## Q-22 Ghoul2 instance copy aliases the bone cache

**What.** Raven's plain struct assignment copy shares the raw `mBoneCache` pointer between source and destination. The port copies `bone_cache` by value to reproduce the aliasing.
**Where.** `crates/mp/engine/ghoul2/src/api_models.rs:96-102`. Oracle: `oracle/codemp/ghoul2/G2_API.cpp:2315`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. The aliasing is defined behavior in C and consumers depend on the shared cache.

## Q-23 Q3_SetFloatVariable returns VTYPE_FLOAT for not-found

**What.** The not-found branch returns `VTYPE_FLOAT` (1) instead of false (0), a copy/paste slip that still reads truthy in C. The port keeps it and pins it with a regression test.
**Where.** `crates/mp/engine/icarus/src/q3_registers/mod.rs:320-327`. Oracle: `oracle/codemp/icarus/Q3_Registers.cpp:210-216`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. ICARUS scripts observe the truthy result and the test guards against a cleanup.

## Q-24 ROFF header check is inverted

**What.** `IsROFF` rejects a file when `strcmp` against "ROFF" matches and accepts every other magic, so a wrong magic passes and only an exact `ROFF\0` header fails. The port reproduces the inverted gate.
**Where.** `crates/mp/engine/qcommon/src/roff/roff_system.rs:303,940-965`. Oracle: `oracle/codemp/qcommon/RoffSystem.cpp:101`, `oracle/codemp/qcommon/RoffSystem.h:26`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Shipped .rof files load through this gate and version checks catch real garbage downstream.

## Q-25 Weather ranges write mMin twice and never mMax

**What.** `SFloatRange::Clear` and `SIntRange::Clear` assign `mMin` twice and never touch `mMax`, and the rain constructor stamps `mRotationChangeTimer.mMin` twice the same way. The port copies all three.
**Where.** `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:206-209,240-243,1114-1116`. Oracle: `oracle/codemp/renderer/tr_WorldEffects.cpp:209-213,229-233,999-1000`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Weather visuals tuned themselves around the stale `mMax`, so a fix changes the look.

## Q-26 LoadTGA keeps the x-direction quirk

**What.** The TGA decoder's `iXStart`/`iXStep` derivation carries Raven's own x-direction quirk in the RGB and greyscale branches. The port transcribes it verbatim.
**Where.** `crates/mp/renderer/src/tr_image.rs:585-586,637`. Oracle: `oracle/codemp/renderer/tr_image.cpp:1421-1743`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Shipped TGAs decode correctly through the quirk and pixel parity matters.

## Q-27 Shader files concatenate in reverse list order

**What.** Shader text concatenates in reverse file-list order, so on duplicate shader names the last-listed file wins the first-match scan. The port preserves the emergent per-file precedence per §20.
**Where.** `crates/mp/renderer/src/tr_shader.rs:2838-2845`. Oracle: `oracle/codemp/renderer/tr_shader.cpp:3940-3944`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Mod content depends on which duplicate definition wins.

## Q-28 Duelist configstring scan steps past the terminator

**What.** The `|`-delimited duelist parse steps the index past the end when a field has no `|`, and later fields read as empty. The port keeps the same scan shape over a bounded slice.
**Where.** `crates/mp/cgame/src/cg_servercmds.rs:126-136,1245-1251`. Oracle: `oracle/codemp/cgame/cg_servercmds.c:612`.
**Class.** preserved-bug
**Verdict.** PROPOSED keep. Duel HUD parsing matches the oracle over every recorded trace.

## Class: preserved-dead-code

## Q-29 G_RemapTeamShaders is an empty function

**What.** Raven's whole body is `#if 0` dead code, so the live function does nothing. The port ships the faithful no-op.
**Where.** `crates/mp/game/src/g_main.rs:133-140`. Oracle: `oracle/codemp/game/g_main.c:783-795`.
**Class.** preserved-dead-code
**Verdict.** PROPOSED keep. Reviving the remap would invent behavior no retail build had.

## Q-30 Missile-explode parent credit test never fires

**What.** The splash-damage block tests `eType == ET_MISSILE` right after the code set `eType` to `ET_GENERAL`, so the vehicle-credit branch never runs. The port copies the dead test.
**Where.** `crates/mp/game/src/g_missile.rs:273-280`. Oracle: `oracle/codemp/game/g_missile.c:233-240`.
**Class.** preserved-dead-code
**Verdict.** PROPOSED keep. Enabling the branch would change damage credit versus retail.

## Q-31 Seven ui DisplayContext slots are dead surface

**What.** Seven function slots `_UI_Init` fills are never reached by any caller, so the port makes them panic with their subject per §20 instead of wiring live trampolines.
**Where.** `crates/mp/ui/src/ui_display_context.rs:18-21,73,196-226,245,509`. Oracle: `oracle/codemp/ui/ui_main.c:10731` (the `_UI_Init` fill block).
**Class.** preserved-dead-code
**Verdict.** PROPOSED keep. A panic marks any future caller loudly, which beats a silent fake.

## Class: UB-divergence

## Q-32 gc_orders bound check is off by one

**What.** C's guard uses `>` against the element count, so `order == 7` indexes a 7-element array, a UB read. The port bounds with `>=`, so the out-of-range case is a deterministic no-op.
**Where.** `crates/mp/game/src/g_cmds.rs:2009-2013`. Oracle: `oracle/codemp/game/g_cmds.c:1835`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. §19 requires one defined behavior and the no-op is it.

## Q-33 CalculateRanks zero loop overruns a 2-slot array

**What.** C zeroes `numteamVotingClients[2]` with a 4-iteration loop, a 2-int overrun that silently clobbers neighbors. The port clamps the loop to the array length because only slots 0 and 1 are ever read.
**Where.** `crates/mp/game/src/g_main.rs:1025-1031`. Oracle: `oracle/codemp/game/g_main.c:1768`, `oracle/codemp/game/g_local.h:879`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. The clobbered neighbors were rewritten before any read, so behavior is identical.

## Q-34 va truncates at the 32000-byte buffer

**What.** Raven's `va` overruns its 32000-byte rotating buffer on huge strings, a UB path Raven's own FIXME flags. The port truncates instead.
**Where.** `crates/mp/game/src/q_shared.rs:881-884`. Oracle: `oracle/codemp/game/q_shared.c:1017-1031`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. No live format call reaches 32000 bytes, so truncation is unobservable.

## Q-35 Empty forcepowers string segfaults the oracle

**What.** With an empty `forcepowers` userinfo string, `BG_LegalizedForcePowers` leaves its `final_Powers` stack array uninitialized, and the oracle then indexes out of bounds and segfaults in `ClientBegin`. The port zero-fills the array, and the referee harness always supplies a full forcepowers string so both sides stay comparable.
**Where.** `crates/mp/bg/src/bg_misc.rs:402-407`, `crates/jampgame/tests/common/mod.rs:164-176`. Oracle: `oracle/codemp/game/bg_misc.c:439-470`, `oracle/codemp/game/w_force.c:277`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Surviving a hostile empty string is strictly safer than the oracle's crash.

## Q-36 Uninitialized game locals read as zero

**What.** Across mp_game, C leaves stack locals uninitialized on some path and later reads them, which is UB. The port zero-initializes each one, picking "reads as zero" as the single defined behavior per §19.
**Where.** `crates/mp/game/src/g_combat.rs:210-212` (oracle `g_combat.c:50-63`), `crates/mp/game/src/g_session.rs:104-109` (oracle `g_session.c:118`), `crates/mp/game/src/g_team.rs:351-353` (oracle `g_team.c:308-314`), `crates/mp/game/src/g_weapon.rs:3349-3351` (oracle `g_weapon.c:2742-2750`), `crates/mp/game/src/w_saber.rs:987,10981-10985` (oracle `w_saber.c:8906`), plus `crates/mp/game/src/NPC_AI_Jedi.rs:358`, `crates/mp/game/src/NPC_reactions.rs:420`, `crates/mp/game/src/g_navnew.rs:867`, `crates/mp/game/src/g_spawn.rs:863`, `crates/mp/game/src/g_ICARUScb.rs:4132-4134`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Every site is a §19 pick and the referee stayed green through all of them.

## Q-37 Enemy-choice array panics past 128 candidates

**What.** C silently overruns its `choice[128]` stack array on a 129th valid enemy candidate, which is UB. The port panics on the same index instead, unreachable with realistic entity counts.
**Where.** `crates/mp/game/src/NPC_combat.rs:1338-1341`. Oracle: `oracle/codemp/game/NPC_combat.c:1508`.
**Class.** UB-divergence
**Verdict.** PROPOSED fix. A hostile map with 129+ candidates can panic a live server, so the pick should become "ignore extra candidates" instead of a panic.

## Q-38 Jump-route scan reads gWPArray[-1]

**What.** The oracle reads `gWPArray[i-1]` and `gWPArray[i+1]` unguarded, out of bounds at both ends. The port treats the out-of-range neighbors as null and skips the branch.
**Where.** `crates/mp/game/src/ai_wpnav.rs:1972-1976`. Oracle: `oracle/codemp/game/ai_wpnav.c:1953-2005`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. The skip is the natural defined reading of a missing neighbor.

## Q-39 Ban-file parse session gets a substituted name

**What.** The oracle passes an uninitialized `banIPFile` buffer to `COM_BeginParseSession`, a UB read used only in parse-error text. The port substitutes the literal "banip.txt".
**Where.** `crates/mp/game/src/g_svcmds.rs:408-411`. Oracle: `oracle/codemp/game/g_svcmds.c:339,352`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Only error-message text differs and the substituted name is more useful.

## Q-40 sscanf leaves unmatched outputs untouched

**What.** In Raven, an unmatched sscanf component leaves the destination as stack garbage, which is UB when read. The port's scanner leaves the Rust-side destination unmodified as the one defined behavior.
**Where.** `crates/native/string/src/sscanf.rs:23-27`. Oracle example caller: `oracle/codemp/game/bg_vehicleLoad.c:885-887`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Callers pre-zero or pre-seed their destinations, so the pick is deterministic.

## Q-41 ColorBytes3 zeroes the fourth packed byte

**What.** The oracle packs three color bytes and leaves the fourth byte of the int uninitialized, which is UB. The port zeroes it.
**Where.** `crates/native/math/src/qmath.rs:404-410`. Oracle: `oracle/codemp/game/q_math.c:333-341`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. No caller reads the fourth byte, so zero is free.

## Q-42 Fixed-buffer overruns become owned strings

**What.** Raven copies unbounded text into fixed stack buffers at many sites, an overrun UB for long inputs. The port's owned `String`s carry the full text and cannot overrun, the §19 pick at every site.
**Where.** `crates/mp/ui/src/ui_main.rs:709-710,5020,13584,13776-13777` (oracle `ui_main.c:954-976,11215`), `crates/mp/renderer/src/tr_font.rs:530-532`, `crates/mp/renderer/src/tr_model/server_skins.rs:335`, `crates/mp/game/src/NPC_AI_Utils.rs:262`, `crates/mp/game/src/g_utils.rs:81`, `crates/native/string/src/filter.rs:77`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Untruncated text only diverges on inputs the oracle would corrupt or crash on.

## Q-43 ui uninitialized reads read as zero or empty

**What.** The ui module reads several values C never initialized: a configstring buffer before the fill, `forcePowerDisable` before any assignment, stale map and player indexes, and the empty-bodied `UI_GetTeamColor` out-param. The port picks zero or empty for each.
**Where.** `crates/mp/ui/src/ui_main.rs:1204-1208` (oracle `ui_main.c:2234-2239`), `crates/mp/ui/src/ui_main.rs:12726,12917-12922` (oracle `ui_main.c:6886-6893`), `crates/mp/ui/src/ui_atoms.rs:632-633` (oracle `ui_atoms.c:236`), `crates/mp/uishared/src/ui_shared.rs:1528-1530` (oracle `ui_main.c:7509`).
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Zero matches a zeroed C automatic, the least surprising defined value.

## Q-44 Ghoul2 null-model paths get guards

**What.** Several ghoul2 debug and save-load paths dereference a null `mod_m`/`mdxm` in the oracle, which is UB. The port prints nothing, leaves values unclamped, or returns 0 at those sites and documents each divergence.
**Where.** `crates/mp/engine/ghoul2/src/misc.rs:284-286,487,1541`, `crates/mp/engine/ghoul2/src/api_saveload.rs:185,205,254`. Oracle: `oracle/codemp/ghoul2/G2_misc.cpp:279-304` and siblings.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. A null model never occurs in live play, so the guards are parity-neutral.

## Q-45 Uninitialized ghoul2 matrices are zeroed

**What.** Raven pushes bolt and bone structs whose `position` matrix is uninitialized C++ memory, and one bone path reads an uninitialized stack local. The port zeroes them, and the transform chain overwrites them before any real read.
**Where.** `crates/mp/engine/ghoul2/src/bolts.rs:134-141,210-212`, `crates/mp/engine/ghoul2/src/bones.rs:264`. Oracle: `oracle/codemp/ghoul2/ghoul2_shared.h:170-182`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. The zeroed bytes are never observable through the golden fixtures.

## Q-46 Botlib trace stacks are zero-initialized

**What.** Raven leaves `tracestack` and `linkstack` arrays uninitialized before their first writes, so early exits could read garbage frames. The port zero-initializes the stacks.
**Where.** `crates/mp/engine/botlib/src/be_aas_sample_fns.rs:287,772,1050`. Oracle: `oracle/codemp/botlib/be_aas_sample.cpp:438`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. The stack pointer discipline means the extra zeroes are never read on live paths.

## Q-47 Renderer uninitialized locals get defined stand-ins

**What.** The renderer reads several uninitialized oracle values: `texModInfo_t` fields no path wrote, a `MAC_STATIC` vertex scratch array, caller-stack fog values, a hunk slot at zero points, and an indeterminate `CFontInfo` point size on the failure path. The port gives each a defined stand-in.
**Where.** `crates/mp/renderer/src/tr_shade_calc.rs:965,1291` (oracle `tr_shade_calc.cpp:81-100`), `crates/mp/renderer/src/tr_curve.rs:458`, `crates/mp/renderer/src/tr_main.rs:538`, `crates/mp/renderer/src/tr_bsp.rs:2710`, `crates/mp/renderer/src/tr_font.rs:534-537,870` (oracle `tr_font.cpp:1628-1630`).
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Each stand-in is dead or overwritten on live paths, so pixels do not change.

## Q-48 Nightvision fog copy wrote one past the allocation

**What.** The oracle writes `fogs[numfogs]` into a `count + 1`-entry allocation, one slot past its end, a heap overrun. The port's `Vec::push` reaches the same logical index legally.
**Where.** `crates/mp/renderer/src/tr_bsp.rs:2007-2012`. Oracle: `oracle/codemp/renderer/tr_bsp.cpp:1683-1696`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. The push preserves the fog data the overrun happened to produce.

## Q-49 Patch-grid heightLodError copies onto itself

**What.** The oracle's hunk-move copies `grid->heightLodError` onto itself, so the hunk copy stays uninitialized memory. The port keeps the real values because owned `Vec`s have no uninitialized-hunk concept.
**Where.** `crates/mp/renderer/src/tr_bsp.rs:921-929`. Oracle: `oracle/codemp/renderer/tr_bsp.cpp:1333`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Real LOD errors are the only defined reading, and terrain LOD behaves at least as well.

## Q-50 Mark fragments drop the malformed index tail

**What.** The oracle's `k += 3` loop over-reads past `numIndices` on a non-multiple-of-3 surface and emits a garbage fragment. The port's `chunks_exact(3)` drops the malformed tail.
**Where.** `crates/mp/renderer/src/tr_marks.rs:578-582`. Oracle: `oracle/codemp/renderer/tr_marks.cpp:413`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. A garbage fragment is not reproducible content, and well-formed surfaces are unaffected.

## Q-51 Siege-state parse skips the 1024-byte scratch buffer

**What.** Raven copies the siege configstring through a fixed `char b[1024]` with no length check, an overrun for long strings. The port slices the input directly, and it also keeps the oracle's commented-out `prevState` guard dead.
**Where.** `crates/mp/cgame/src/cg_main.rs:1564-1573`. Oracle: `oracle/codemp/cgame/cg_main.c:1403-1443`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. Live configstrings never near 1024 bytes, so the slice is parity-neutral.

## Q-52 Referee mask: SaberClashFlare passes a 3-float color

**What.** `CG_SaberClashFlare` hands `trap_R_SetColor` a `vec3_t`, so the fourth float is stack padding that even the oracle self-check cannot reproduce. The replay referee masks the fourth float, keyed by the unique (0.8, 0.8, 0.8) triple.
**Where.** `crates/cgame/tests/replay_support/mod.rs:297-325`. Oracle: `oracle/codemp/cgame/cg_draw.c:5327` (fn), the SetColor call near `:5380-5381`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. §19 says normalize UB in the referee, never in fixtures, and this mask does exactly that.

## Q-53 Referee mask: FX_AddPrimitive verts compare only where meaningful

**What.** `effectTrailArgStruct_t` verts carry `mSetFlags`-gated fields, and the ungated bytes are uninitialized caller stack that differs run to run. The replay referee compares only each vert's origin plus the unconditional tail.
**Where.** `crates/cgame/tests/replay_support/mod.rs:279-295`. Oracle: `oracle/codemp/game/q_shared.h:2615-2620`.
**Class.** UB-divergence
**Verdict.** PROPOSED keep. The masked bytes can never be part of a byte-identical bar.

## Q-54 Mock engine serves deterministic trace results

**What.** `trap_Trace` is an out-param syscall, and a permissive mock that returns 0 leaves the `trace_t` uninitialized, making pmove nondeterministic across processes. The jampgame mock engine serves a deterministic empty-space result for all three trace traps.
**Where.** `crates/jampgame/tests/common/mod.rs:772-781`. Oracle: `oracle/codemp/game/g_public.h` (trap_Trace contract).
**Class.** UB-divergence
**Verdict.** PROPOSED keep. The oracle-vs-oracle self-test caught exactly this, and the fix is harness-side only.

## Class: wire-quirk

## Q-55 Retail netfield tables differ from the source drop

**What.** The oracle source drop's delta tables (152, 152, 80 rows) postdate the shipped 1.01 build, and the retail wire uses 137, 140, and 69 rows. The port transcribes the retail sets, verified against a retail-compatible client's compiled tables.
**Where.** `crates/mp/engine/qcommon/src/msg.rs:503-516,954-958,1398-1400`. Oracle: `oracle/codemp/qcommon/msg.cpp:1410-1568,1570-1734` (minus the never-shipped vehicle rows).
**Class.** wire-quirk
**Verdict.** PROPOSED keep. Regenerating from the oracle source drops real clients with parse errors.

## Q-56 Pilot playerstate codes only the first 58 rows

**What.** Raven computes the pilot field count as the table length minus 82, so only the first 58 entries are ever coded. The port transcribes the full 140-entry table and applies the same subtraction.
**Where.** `crates/mp/engine/qcommon/src/msg.rs:948-952`. Oracle: `oracle/codemp/qcommon/msg.cpp:2253,2503`.
**Class.** wire-quirk
**Verdict.** PROPOSED keep. The subtraction is wire-visible and both peers must agree on it.

## Class: per-mode-quirk

## Q-57 SP GP2 accepts truncated group files

**What.** SP's `AddGroup` never sets `mParent`, so end-of-data inside a nested group takes the top-level break instead of the error return, and truncated files parse. MP sets the parent and rejects them. The port duplicates both behaviors per §20.
**Where.** `crates/sp/qshared/src/common/sp/game/gp2/generic_parser2.rs:313-323`. Oracle: `oracle/code/game/genericparser2.cpp:648-664,699`.
**Class.** per-mode-quirk
**Verdict.** PROPOSED keep. §20 names this quirk as its own exemplar.

## Q-58 SP saberInfo_t diverges sharply from MP

**What.** SP's `saberInfo_t` uses heap `char *` names, string mark shaders, and adds `fallSound[3]`, and it keeps `brokenSaber1/2` live where MP treats them as dead. The port keeps both shapes separate with both source refs.
**Where.** `crates/sp/qshared/src/common/sp/qcommon/saber/saber_info.rs:16-25`. Oracle: `oracle/code/game/q_shared.h:1724-1944`.
**Class.** per-mode-quirk
**Verdict.** PROPOSED keep. SP serializes this struct into savegames, so its exact shape is load-bearing.

## Q-59 Math deviations stay per-mode pairs

**What.** A handful of math functions differ between the SP and MP trees, and `native_math` keeps them as `<Name>MP`/`<Name>SP` variant pairs re-exported per mode. The shared functions have one definition.
**Where.** `crates/native/math/src/lib.rs:4-5` and the `deviations` module it names. Oracle: `oracle/codemp/game/q_math.c` versus `oracle/code/game/q_math.c`.
**Class.** per-mode-quirk
**Verdict.** PROPOSED keep. DEC-04 rules duplicate-do-not-unify during porting, and the pairs record real behavioral splits.
