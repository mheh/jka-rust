# Idiom slice 3 — bg item-grab surface (DEC-31)

> **Status: EXECUTED — merged to master (idiom era, DEC-31; slice 3 branch `idiom/bg-item-grab`).**

Third slice: the bg_misc grab/inventory surface — the code shared with
future cgame prediction — rewritten on ItemKind, retiring the bridge
accessors from bg. Branch: `idiom/bg-item-grab`. Gates: referee 8/8
(the ffa1 tape runs BG_CanItemBeGrabbed on every item touch).

Scope + shapes (established rulings apply; no new user decisions):

- `BG_CanItemBeGrabbed`: the giType() match and the trueJedi /
  trueNonJedi / JediMaster prune ladders become kind matches with
  payload binding. The two De Morgan corners preserved exactly:
  trueJedi allows `Powerup(t) if t != PW_YSALAMIRI` (not only
  ysalamiri — the original conjunct reads inverted), and the IT_TEAM
  cubes (Team(0)) still fall through to qfalse.
- `BG_GetItemIndexByTag(tag, type)`: keeps Raven's (tag, type) int
  params; the comparison goes through a new `ItemKind::from_gi(type,
  tag) -> Option<ItemKind>` inverse of the giType()/giTag() bridge
  (Rust-named, no Raven counterpart).
- The "selected holdable tag" raw read
  (`bg_itemlist[ps.stats[STAT_HOLDABLE_ITEM]].giTag()`) appears in
  BG_CycleInven + two bg_pmove sites; it centralizes into one helper
  (`bg_misc::selected_holdable_tag`) wrapping the read verbatim —
  the slot may hold sentinel 0, so the raw tag read is the honest
  shape. bg_pmove internals otherwise untouched (their slice later).
- After this slice the only bridge-accessor reads left in the
  workspace are LaunchItem's four documented type-puns, the helper's
  one central read, and the un-idiomized game files (ai_*, g_team,
  g_cmds — retire with their slices).
