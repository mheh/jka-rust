# Logic audit - DEC-65 ruling 3, the ghoul2 token conversion (packet gh#31 step-001)

Read-only Fable audit, 2026-08-03. Subject: the plan to convert the module-visible ghoul2 `void*` slot from a `Box<CGhoul2Info_v>` heap pointer to the `Ghoul2Handle + 1` token, ahead of wave-3 renderer entity drawing. The packet under audit: `.claude/packets/31/step-001/packet.md` (draft commit `8829335a`). The follow-up amendment draft lands as the second part of this file.

## A. mItem mutation reachability - DEFECT

The packet claims only init, duplicate, and clean mutate `mItem`, and that all three are slot-address arms. The call graph refutes this in three places.

The complete mutator set on `CGhoul2Info_v` is `alloc`, `free`, `clear`, `deep_copy`, `assign`, `assign_item`, plus lazy allocs inside `resize` and `push_back` when `mItem == 0` (crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs:49-186). An exhaustive grep of the ghoul2 crate finds every trap-reachable call: api_models.rs:310,354 (init), 348-355 (clean), 447,497,549 (remove family), 653 (copy instance), 697 (copy specific). The ragdoll and IK family (api_ragdoll.rs, ragdoll.rs), bones, bolts, surfaces, collision, and gore never call a mutator on the module cell. ragdoll.rs:833 builds a local wrapper from a handle and never writes it back. `g2api_set_ghoul2_model_indexes` (SETMODELS) is a compiled no-op (api_models.rs:582-589). The saveload free (api_saveload.rs:144) has no trap arm in any of the three dispatch files.

The three refutations:

1. **The remove family frees.** `g2api_remove_ghoul2_model` and `g2api_remove_ghoul2_models` call `ghoul2.free(g2)` when the vector empties (api_models.rs:497, 549), which zeroes `mItem`. The arms hold the slot address (`vma(...) as *mut *mut CGhoul2Info_v`: sv_game.rs:3389, 3395, cl_cgame.rs CG_G2_REMOVEGHOUL2MODEL, cl_ui.rs UI_G2_REMOVEGHOUL2MODEL), yet the packet lists them under the class-2 guard-asymmetry note with no write-back. Without write-back the module keeps a stale token. The concrete crash: remove-to-empty, then re-init through the same slot. `from_token` yields a stale nonzero `mItem`, `push_back` skips `alloc` (cghoul2_info_v.rs:182-185), the push lands in the `null_slot` scratch (info_array.rs:156-162), and `g2api_init_ghoul2_model`'s final read `ghoul2.get(g2, model)` indexes an empty slice and panics (api_models.rs:335, cghoul2_info_v.rs:154). Today the shared cell carries `mItem 0` forward and re-init works.
2. **`g2api_copy_ghoul2_instance` mutates the value-passed destination.** COPYGHOUL2INSTANCE passes both args by value (sv_game.rs:3342-3343, cl_cgame.rs:3138-3139, cl_ui.rs:2213-2214). The callee runs `g2_to.deep_copy` (api_models.rs:653), which frees the old handle and allocs a fresh one (cghoul2_info_v.rs:99-101). Under tokens the new `mItem` dies in the discarded stack cell, the module keeps a stale token, and the fresh arena slot leaks with no live holder. Mitigation: the arm is module-dead. No Rust module calls the trap wrapper, and the oracle's game, cgame, and ui sources contain no `trap_G2API_CopyGhoul2Instance` call site either.
3. **`g2api_copy_specific_g2_model` can alloc through the value-passed destination.** `ghoul2_to.resize(g2, model_to + 1)` (api_models.rs:697) allocs when the destination cell is `mItem 0`, a state that exists today after remove-to-empty (cell kept, `mItem` zeroed). Live callers: g_client.rs:1714, 3924, cg_weapons.rs:2526, 2541, 2597, cg_servercmds.rs:666, cg_players.rs:10880, 12746. At every site the destination is a live player or weapon instance, so the alloc path needs the remove-to-empty precondition first. See C for the Raven comparison.

The packet's own pre-flight audit (packet.md:34-36) would trip on items 2 and 3 and STOP. The classification error on the remove arms would not trip it, because the packet pre-sorted those arms into class 2 by hand.

## B. Aliasing semantics - CONFIRMED-SAFE

All instance content (bones, bolts, surfaces, gore) lives in the arena keyed by `mItem`, not in the 4-byte cell. Two slots that alias today share content through the same `mItem`, and two independent token snapshots decode to the same `mItem` and reach the same arena vector. Content aliasing is therefore preserved exactly. Only mutation of `mItem` itself breaks under snapshots, and that set is precisely the A-defect list.

The copy sites checked: cg_ents.rs:969 (item slot into refEntity), cg_ents.rs:415, cg_view.rs:878, cg_weapons.rs:141, cg_draw.rs:667 (cent slot into refEntity), cg_predict.rs:1024, 1041, 1500 (pmove), cg_players.rs:13123-13174 (frame_hold), g_client.rs:3579 (`precachedKyle` as duplicate source), cg_weapons.rs:2532-2543 (`ci.ghoul2Weapons` as copy source). All are value reads into value-consuming traps or into refEntity, none mutates `mItem` through the copy.

The one identity comparison, `frame_hold != centGhoul2` (cg_players.rs:13127), keeps its truth table: aliased copies compare equal as tokens, independently created instances get distinct handles and compare unequal. Clean through one alias then use through the other improves under tokens: today a dangling deref or a stale reused Box address, under tokens a generation-checked handle that `is_valid` (info_array.rs:132-137) rejects into empty-vector no-ops, which is also Raven's own semantics for a gone handle (the function-static null vector, mirrored at info_array.rs:139-151).

## C. Null-state round trip - CONFIRMED-SAFE, with one NEEDS-RULING edge

The unguarded sv trio (sv_game.rs:3383-3397): Raven binds `**ghlRemove` and calls `.size()`, which reads `mItem` through a null `this` and crashes (G2_API.cpp:783-885). The `mItem 0` stack cell answers `size 0`, so has returns qfalse and both removes return qfalse. That is the one defined behavior for Raven's crash, a clean porting-rules §19 divergence, and the packet already notes it. The client-side arms carry explicit null guards with the same answer (cl_cgame.rs CG_G2_HASGHOUL2MODELONINDEX, cl_ui.rs:2252-2275).

Clean and init round-trip correctly. Raven's clean nulls the slot, the plan's `mItem 0 -> to_token -> null` matches. A failed init still pushes a `-1` entry into a real arena slot (api_models.rs:298-323), so the token is non-null, which matches Raven's non-null empty-ish object.

The edge that needs a ruling: Raven's removes null the module slot (`delete *ghlRemove; *ghlRemove = NULL;`, G2_API.cpp:868-869, 956-957). Today's Rust arms do NOT null the slot and leave a live empty cell, an existing divergence. Amendment 1 below (write-back at remove) restores Raven's exact observable state. But it changes one today-behavior: today, copy-specific into a post-remove empty cell allocs and revives the instance (api_models.rs:695-697), while under write-back the slot is null and the arm no-ops on the null guard (sv_game.rs:3354-3356). Raven itself nulls the slot at remove, so Raven also no-ops at that follow-on copy. The write-back version therefore matches Raven and today's Rust is the outlier. This deserves one sentence of ruling in the packet, not a design change. The same reasoning covers duplicate-with-invalid-source, where Raven leaves a non-null empty object and the plan leaves null: distinguishable only through the same copy-specific path.

## D. Long-term retention - CONFIRMED-SAFE

- The FX system is the only engine subsystem that retains a ghoul2 reference across trap returns, and it already retains the `i32` `mItem`, not a pointer. The CG_FX_PLAY_BOLTED_EFFECT_ID arm decodes the slot value and passes `g2v.mItem` (cl_cgame.rs:2656-2667). Storage is `i_ghoul2: i32` (fx_scheduler.rs:482, clight.rs:77, cparticle.rs:108), and every later use runs the generation check (`Ghoul2IsValid` -> `info_array.is_valid`, fx_host.rs:644-651). Token conversion changes nothing here.
- `g2api_attach_instance_to_ent_num`, `clear_attached_instance`, and `clean_ent_attachments` are compiled no-ops (api_bolts.rs:552-579).
- sv_world.rs:785, 797 reads the gentity slot at trace time and holds nothing across calls.
- `CG_GET_GHOUL2` (cg_main.rs:5623-5634) returns the raw slot value to the engine, but no engine consumer exists in the Rust tree or the oracle client. Dead path.
- fx_host.rs:693 builds its stack cell from the retained `i32` handle, not from a module slot. It stays consistent, per packet.md:32.

## E. Scheme coherence end to end - CONFIRMED-SAFE

The refEntity copy sites are cg_ents.rs:415, 969, cg_view.rs:878, cg_weapons.rs:141, cg_draw.rs:667, and cg_players.rs:13135, 13160, 13174. The renderer already decodes that field as a token: tr_scene.rs:427 (`ghoul2_token_decode(ent.ghoul2)`), tr_main.rs:1715 and the encode at 1792, and the render-side struct is already `ghoul2: Option<Ghoul2Handle>` (render_state/placeholders.rs:175). No consumer assumes pointer-ness beyond null tests and the one equality at cg_players.rs:13127. No hashing, no ordering, no arithmetic exists in modules or engine. ui_saber.rs passes either the slot address (line 757, 1348) or the slot value (789, 1374) straight into traps, and the display-context `ghoul2Ptr` (ui_display_context.rs:649-701) is a pass-through parameter, not stored state. tr_scene.rs:379 only null-tests. The one raw-value export, `CG_GET_GHOUL2`, has no consumer (see D).

## F. Clean-through-value - DEFECT, folded into A

Exactly one trap path frees through a value argument: the COPYGHOUL2INSTANCE destination, whose `deep_copy` frees the old instance with no reachable slot to update (A item 2). Raven at the same site mutates the pointee, so the module slot stays correct. The token plan changes observable behavior there. The arm is dead in both trees, so the exposure is latent, but the arm body must still be ruled, since a converted decode-call-discard body silently leaks an arena slot if the trap is ever exercised. Clean and the remove family all receive the slot address, so no other free escapes write-back reach.

## Required packet amendments

1. Reclassify REMOVEGHOUL2MODEL and REMOVEGHOUL2MODELS (sv_game.rs:3386-3397, cl_cgame.rs, cl_ui.rs remove arms, 4 arms total) as write-back arms: decode, call, `to_token` back through the existing `pp`. This closes the stale-token re-init panic and restores Raven's `*ghlRemove = NULL` (G2_API.cpp:868-869, 956-957) that today's Rust omits. HASGHOUL2MODELONINDEX stays read-only, no write-back.
2. Rule the three COPYGHOUL2INSTANCE arms. They are value-passed `mItem` mutators, the pre-flight STOP condition, and they cannot write back. Evidence for the ruling: no module caller in the Rust tree, no `trap_G2API_CopyGhoul2Instance` caller in oracle game, cgame, or ui. Options are a documented dead-arm conversion with a leak note, or an error arm.
3. Add a §19-style note for the three COPYSPECIFICGHOUL2MODEL arms: the empty-destination alloc path (api_models.rs:695-697) becomes a null no-op after amendment 1. Raven no-ops there too because Raven nulls at remove, so today's Rust is the outlier, and the note should say so.
4. Reword the pre-flight audit so the has/remove family is audited as slot-address, not value-passed. The current guard-asymmetry sentence (packet.md:30) pre-sorts them wrongly.
5. Minor: at sv_world.rs:785 the `from_token` cell feeds `.get(g2, 0)`, which panics on an empty instance when `sv_showghoultraces` is on. Add a size guard note at the site.

## Why, and whether it is worth it

The collision is real and already latched. The renderer decodes `refEntity_t.ghoul2` as `handle + 1` today (tr_scene.rs:427, tr_main.rs:1715), while cgame fills that field with the Box pointer at six live copy sites. The moment wave 3 draws ghoul2 entities, decode of a pointer gives `pointer - 1` truncated into the masked `is_valid` lookup: almost always a silent no-draw, and on an index-plus-generation coincidence, a draw of the wrong instance. Doing nothing fails wave 3.

The two cheap outs both lose on inspection. Deref-decode (renderer reads `mItem` through the pointer at scene-add) is same-thread and one line, but the CLEANMODELS arm frees the Box (`Box::from_raw`, sv_game.rs:3407), so any refEntity copy that outlives a clean derefs freed memory, and the raw pointer becomes permanent module ABI against DEC-65's explicit ruling. Tokens-on-cgame-only splits one `void*` convention across three dispatch files while sv_world, FX, and ui still touch the same scheme, and saves little, because cgame alone is 46 of the 115 arms. The ruled fallback 3b (pointer-to-handle map at the render trap) keeps every stale-pointer hazard and adds a map that must be invalidated at every free, forever.

The evidence favors 3a strongly. The Box cell holds nothing but the 4-byte handle (cghoul2_info_v.rs:11-15), so the heap indirection buys only mutable-cell aliasing, and this audit shows that aliasing carries real semantics in exactly the arms that hold the slot address anyway, once the remove family is classified correctly. The g2api crate needs zero change. The FX and sv_world paths already speak `mItem`. Raven's own remove semantics null the slot, so write-back is the native shape of the API, not an imposition. The defects found reclassify four arms, add two short rulings, and add one site note. They raise the cost by hours, not by architecture, and amendment 1 makes the port more Raven-faithful than the current code. Verdict: the conversion is worth doing, and it should not start until the five amendments are in the packet, because the plan as written ships a reachable panic (remove-then-reinit) and a silent-loss arm.

## Post-audit ruling

The user ruled amendment 2 as option (c), 2026-08-03: in-place `deep_copy`. The destination keeps its handle when one exists and the arena contents are replaced in place, which matches Raven's `g2To.DeepCopy(g2From)` object-identity-preserving assignment (`G2_API.cpp:2239-2259`, verified against the oracle first-hand). The alloc path remains only for a `mItem: 0` destination.

## Follow-up: the amendment draft (applied to the packet verbatim)

The same auditor drafted the packet fold-in after the ruling. The replacement sections below now live in `.claude/packets/31/step-001/packet.md` - this is the frozen record of the draft as delivered. The class arithmetic: 6 (init/duplicate) + 3 (clean) + 4 (remove family) + 3 (has, read-only through the slot address) + 99 value-passed = 115 arms, consistent with the survey count after the seven slot-address arms leave the value-passed set. The `deep_copy` in-place change sits in commit 1 because it is a `mp_engine_ghoul2` crate change every dispatch commit depends on.

Key deltas the draft introduced over the original packet:

- A new write-back class 4 for the remove family, restoring Raven's `*ghlRemove = NULL` (`G2_API.cpp:868-869,956-957`).
- `CGhoul2Info_v::deep_copy` named in the surface contract as a behavior change: in-place, handle-preserving on a live destination, alloc only on `mItem: 0`, doc comment records the divergence from the old free-then-alloc.
- `COPYGHOUL2INSTANCE` x3: null-destination guard returning -1 without calling (§19 - Raven crashes on the null reference), module-dead note at the site.
- `COPYSPECIFICGHOUL2MODEL` x3: §19 site note that the post-remove no-op matches Raven and the current Rust realloc-through-kept-cell is the outlier.
- The pre-flight audit names the full mutator inventory (`alloc`, `free`, `clear`, `deep_copy`, `assign`, `assign_item`, lazy allocs in `resize`/`push_back`, `cghoul2_info_v.rs:49-186`) and audits the has/remove family as slot-address.
- `sv_world.rs:785` diagnostic print guarded with `size(g2) > 0`.
- Commit 1 gains the `deep_copy` change plus a unit test that a live destination keeps its handle across a copy.
