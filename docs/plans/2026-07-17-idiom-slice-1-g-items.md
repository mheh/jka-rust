# Idiom slice 1 — g_items (DEC-31)

First idiom-era slice: the item subsystem (`g_items.rs` + the bg item
surface it consumes) rewritten into idiomatic Rust, referee-gated by the
duel1 scenarios plus the item-heavy `real-ffa1-items` gate (8685e955).
Branch: `idiom/g-items`; master sees the finished slice.

All shapes below are user-settled (sit-down 2026-07-17).

## Settled type design

`gitem_t` never crosses the engine seam — only the table index does
(`s.modelindex`, CS_ITEMS length). The repr(C) struct, its eight
`*mut c_char` fields, the `unsafe impl Sync`, and the layout asserts all
retire. New types (one per file, `crates/mp/bg/src/public/`):

```rust
/// Raven `gitem_t` — one master-item-table entry.
/// Type definition source: `oracle/codemp/game/bg_public.h:1122-1138`
pub struct GItem {
    /// Spawning name.
    pub classname: &'static str,
    pub pickup_sound: Option<&'static str>,
    /// Raven `world_model[MAX_ITEM_MODELS]` — null-padded `[*;4]` becomes a slice.
    pub world_model: &'static [&'static str],
    pub view_model: Option<&'static str>,
    pub icon: Option<&'static str>,
    /// For ammo how much, or duration of powerup.
    pub quantity: i32,
    /// Replaces the Raven `giType` + `giTag` pair (manual tagged union → real one).
    pub kind: ItemKind,
    pub precaches: &'static str,
    pub sounds: &'static str,
    pub description: Option<&'static str>,
}

pub enum ItemKind {
    /// Raven `IT_BAD` — the index-0 sentinel only.
    Bad,
    /// giTag 1|2 (small/large shield).
    Armor { rating: i32 },
    Health,
    Holdable(holdable_t),
    Powerup(powerup_t),
    Weapon(weapon_t),
    /// `ammo_all` carries Raven's `giTag -1` (give-all dispenser refill).
    Ammo(ammo_t),
    /// CTF flags (PW_*FLAG); the red/blue cubes carry 0.
    Team(powerup_t),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ItemId(NonZeroU16); // index into bg_itemlist == wire modelindex
```

- Payloads stay the existing `c_int` aliases (`weapon_t` etc.); upgrading
  those to real enums belongs to later slices.
- `IT_BATTERY`/`IT_HOLOCRON`: zero callers in oracle game code and in the
  port — no variants (porting-rules §20; dropped surface noted here).
- `ItemId(NonZeroU16)`: slot 0 unrepresentable, `Option<ItemId>` is 2 bytes
  with `None` in the 0 niche. `from_modelindex(c_int) -> Option<ItemId>`,
  `modelindex() -> c_int`, `item() -> &'static GItem`.

## Table + wire invariants

- `bg_itemlist` becomes `[GItem; 51]`: slot 0 stays physically present
  (sentinel, `ItemKind::Bad`, `classname: ""`) so wire indexes align; the
  C-only `{NULL}` terminator entry is dropped.
- `bg_numItems == bg_itemlist.len() == 51` — same value as the oracle's
  `ARRAY_LEN - 1`, keeping the CS_ITEMS registered-items configstring
  length byte-identical.
- Wire-visible values (`s.modelindex`, CS_ITEMS, event numbers, pickup
  timing) must produce identical bytes; everything else is free.

## Naming + consumers

- Raven fn names stay (`BG_FindItemForWeapon(weapon: weapon_t) -> ItemId`,
  `BG_FindItem(&str) -> Option<ItemId>`, `RegisterItem(ctx, ItemId)`);
  panic-on-miss vs null-return per function follows the oracle exactly.
- `gentity_t.item: *mut gitem_t` flips to `item: Option<ItemId>` — follows
  the established `FnId<EntThink>` precedent (wire prefix `s`/`r` pinned,
  private fields free). The ~37 `.item` sites outside g_items.rs
  (g_client, g_team, ai_wpnav, …) get mechanical touch-point adaptation
  only; their full idiomization waits for their own slices.
- bg idiomizes call-surface-by-call-surface (DEC-31): this slice touches
  `bg_misc`'s item fns and the table; `bg_pmove` internals untouched
  (`BG_CanItemBeGrabbed` adapts at its `bg_itemlist[modelindex]` lookup).

## Landing plan

Staged commits on `idiom/g-items`, each referee-gated (suite: 7 scenarios
incl. `real-ffa1-items`):

1. Foundation: new types + regenerated table + `BG_FindItem*` signatures +
   the `gentity.item` flip with mechanical adaptation everywhere.
2. `Touch_Item` + pickup family, idiomatic rewrite.
3. `G_SpawnItem`/`RespawnItem`/`FinishSpawningItem` family.
4. `Drop_Item`/tosses + dispensers.
5. Sweep leftovers; delete the MP `gitem_t` (`mp_qshared` copy; SP's stays).
