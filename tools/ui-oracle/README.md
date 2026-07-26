# ui-oracle — differential harness for the menu-parse pipeline

Compiles the **unmodified** Raven `codemp/ui/ui_shared.c` (the parse half —
`Menu_New` → `Menu_Parse` → `dispatch_menu_keyword` → `MenuParse_*` →
`MenuParse_itemDef` → `Item_Parse` → `dispatch_item_keyword` → `ItemParse_*`)
into a standalone dumper, runs it over `fixtures/*.menu`, and stores the
canonical dumps under `golden/`. The Rust port
(`crates/mp/uishared/src/ui_shared.rs`) must reproduce the goldens
byte-for-byte via `crates/mp/uishared/tests/menu_parse_goldens.rs`.

## Usage

```sh
sh run.sh           # build the dumper, diff current output against golden/
sh run.sh --regen   # rebuild golden/ (after adding/changing fixtures)
cargo test -p mp_uishared --test menu_parse_goldens
```

`run.sh` copies `ui_shared.c` and its full header closure (`ui/`, `game/`,
`qcommon/`, `ghoul2/`, `cgame/`, `icarus/`, `botlib/`) into `build/` next to
this directory's `main.cpp`/`pc_bridge.{h,cpp}`/`stubs.c`, mirroring
`tools/jampgame-oracle/run_gcombat.sh`'s "full-TU compile" pattern rather
than `tools/gp2-oracle`'s from-scratch stub headers (`ui_shared.c` is too
large and too structurally entangled with the game/ui header closure for
hand-stubbing to be less work). `oracle/` is never edited. The goldens are
committed, so `cargo test` needs no C++ toolchain — `run.sh` is only needed
to regenerate or spot-check.

## PC_* strategy

`ui_shared.c`'s `trap_PC_ReadToken`/`trap_PC_SourceFileAndLine` route, in
retail, through the engine to botlib's real preprocessor
(`codemp/botlib/l_precomp.cpp`/`l_script.cpp`). Two options were on the
table: (a) link those UNMODIFIED sources straight into the dumper and call
their handle-table API (`LoadSourceMemory`/`PC_ReadTokenHandle`/
`PC_SourceFileAndLine`) directly, bypassing the retail VM/syscall
marshaling — the same kind of direct-call bypass `gp2-oracle`'s dumper uses
for `CGenericParser2`'s methods; or (b) stub tokenization at the PC_* level
with a hand-rolled tokenizer. (a) was chosen: it's the REAL tokenizer (same
one the live Rust engine's `mp_engine_botlib` port targets), so the goldens
exercise genuine end-to-end parity — including the tokenizer — rather than
parity against a hand-written approximation of it, and it needed less new
surface (no reimplementation of comment/quote/number lexing, `#define`
handling, etc).

`l_precomp.cpp`/`l_script.cpp`/`l_memory.cpp` keep their real C++ dialect
(`c++`); `ui_shared.c`/`q_shared.c`/`main.cpp`/`stubs.c` compile as plain C
(`cc`) so `stubs.c`'s untyped K&R stub trick works (the C linker binds by
name alone, no C++ mangling — same trick `jampgame-oracle`'s
`stubs_gcombat.c` uses). `pc_bridge.cpp` is the one small C++ TU that
bridges the two: it calls the real (C++-mangled) botlib entry points and
re-exports three plain-C-linkage wrapper functions
(`ui_oracle_install_source`/`ui_oracle_PC_ReadTokenHandle`/
`ui_oracle_PC_SourceFileAndLine`) that `main.cpp` calls by plain name.

The Rust side matches this symmetrically: the test drives the SAME real,
already-ported botlib tokenizer (`mp_engine_botlib`'s `LoadSourceMemory`/
`PC_ReadTokenHandle`), not a reimplementation — see
`crates/mp/uishared/tests/menu_parse_goldens.rs`.

## Deterministic stand-ins

Documented once, here, since both dumpers (`main.cpp` and the Rust test's
`TestDisplayContext`) must reproduce the SAME values for the golden compare
to be meaningful rather than vacuous:

- `DC->registerShaderNoMip` / `registerModel` / `registerSound` and
  `trap_R_RegisterSkin` each hand out a monotonically increasing counter
  from their own base (1000/2000/3000/4000 respectively), one call = one
  increment, regardless of the name argument. `DC->RegisterFont` hands out
  a counter from base 5000.
- `DC->getCVarString` and `trap_Cvar_VariableStringBuffer` always return ""
  (no cvar system is live in either harness — every cvar reads as unset).
- `DC->textWidth(text, scale, font)` = `strlen(text) * 8 * scale` (used only
  by the TEXTSCROLL line-breaker, reached via `Menu_PostParse` for any item
  left at `type textscroll`).
- `trap_AnyLanguage_ReadCharFromString` decodes one raw byte per call
  (advance = 1, not-trailing-punctuation) — correct for the Latin-1,
  non-Asian fixture content.
- `trap_Language_UsesSpaces` always returns true.
- `trap_G2API_InitGhoul2Model` always "succeeds" (returns 0) and hands back
  a fixed non-NULL sentinel (never dereferenced or dumped — only the
  resulting `ITF_G2VALID` flag bit is observable); `trap_G2API_SetSkin`/
  `SetBoneAnim` return true; `trap_G2API_GetGLAName` always returns "" (so
  the animation-index branch behind it — and the large `bgAllAnims`/
  `animTable` data dependency it would pull in — is never reached, out of
  scope for this harness); `trap_G2API_CleanGhoul2Models` nulls the
  pointer; `trap_G2_HaveWeGhoul2Models` returns false.
- Every other `DC->` vtable entry and every other `trap_*` this TU never
  calls on the fixtures' executed path aborts loudly (C side) / panics
  (Rust side) if it IS ever reached — a hard signal a fixture strayed onto
  unmodeled surface, not a silently wrong answer.

Fixtures deliberately never exercise the botlib global-`#define` mechanism
(`trap_PC_LoadGlobalDefines("ui/jamp/menudef.h")`, called once by
`UI_LoadMenus` before any `Menu_New`, that lets retail `.menu` files use
symbolic names like `ITEM_TYPE_LISTBOX`): reproducing it would mean loading
`oracle/ui/menudef.h`'s ~200 `#define`s as botlib defines identically on
both the C and Rust sides, which is orthogonal to what this harness tests
(the keyword-dispatch pipeline, not the preprocessor's `#define` layer).
Every fixture spells item/window enum values as plain integers instead.

## Fixtures

- `retail.menu` — a hand-authored, retail-shaped menu (window chrome, a
  title/button/cvar-editfield trio of items) modeled on shipped
  `jampmenus.txt`-style structure, not copied from any shipped asset.
- `all_menu_keywords.menu` — every one of the 35 `menuParseKeywords[]`
  entries, exactly once, in table order.
- `broad_item_keywords.menu` — 80 of the 83 `itemParseKeywords[]` entries
  (`font`/`group` uncovered; `xoffset` structurally unreachable — see the
  fixture header), spread
  across six item-type families (EDITFIELD, MULTI, MODEL, LISTBOX,
  TEXTSCROLL, OWNERDRAW/shader) so each keyword lands on a `typeData` shape
  it's valid for.
- `edge_cases.menu` — unknown-keyword recovery, the permanently-dead
  `"xoffset\t\t"` keyword-table typo, `PC_Script_Parse`'s non-brace-counting
  truncation (and the multi-attempt stream desync it causes — see the
  fixture's own header comment), a quoted string with high-byte Latin-1
  characters, and a truncated/missing-closing-brace EOF.

## Divergences and bugs found (real, not harness artifacts)

- **`LoadSourceMemory` vs `LoadSourceFile` asymmetry (genuine Raven bug,
  worked around here, §19).** `LoadSourceFile` lazily allocates the
  `DEFINEHASHING`-mode `globaldefines` bucket table before calling
  `PC_AddGlobalDefinesToSource`; `LoadSourceMemory` does not, so calling it
  standalone (this harness's whole approach) null-derefs inside
  `globaldefines[i]`. `pc_bridge.cpp`'s `ui_oracle_install_source`
  replicates `LoadSourceFile`'s exact lazy allocation (a zeroed
  `DEFINEHASHSIZE`-bucket table) before calling `LoadSourceMemory`,
  matching what any caller that already did one `LoadSourceFile` first
  would see — same observable behavior (zero global defines), no crash.
- **`ItemParse_flag`'s table-size mismatch is exploitable UB.** Its loop
  guard is `while (styles[i])` (a 6-entry table) but it indexes
  `itemFlags[i]` (a 2-entry table: one real flag string plus a NULL
  sentinel) — at `i=1` it calls `Q_stricmp(token.string, NULL)`. Only
  `"WINDOW_INACTIVE"` (the one real entry, matched at `i=0`) is safe to put
  in a fixture; any other value crashes. Documented at the fixture site
  (`broad_item_keywords.menu`), not exercised past the safe value.
- **`xoffset` is permanently dead.** `itemParseKeywords[]`'s registration
  string for it is `"xoffset\t\t"` — two trailing tab characters baked into
  the C source — so `KeywordHash_Find` can never match a real `xoffset`
  token against it. `ItemParse_xoffset`'s own body separately has an
  inverted-condition bug (`if (PC_Int_Parse(...)) return qfalse;`, failing
  on parse *success*) that would abort the enclosing itemDef/menu if it
  ever ran — but the registration typo means it never does. See
  `edge_cases.menu` block B.
- **`PC_ReadToken` auto-concatenates adjacent quoted strings** (like C's
  adjacent string-literal concatenation) — `"Off" "0" "On" "1"` collapses
  into the single token `"Off0On1"`. This is a real, intentional botlib
  preprocessor feature (`l_precomp.cpp`'s `PC_ReadToken`, "recursively
  concatenate strings that are behind each other"), not a bug, but it means
  `ItemParse_cvarStrList`-style paired-string lists need non-adjacent
  (or unquoted) tokens in real `.menu` files to avoid accidental merging.
  `broad_item_keywords.menu`'s `cvarStrList` uses bare words for this
  reason (documented at the call site).
- **`PC_Script_Parse` truncates at the first `}`, not a balanced one**, and
  the resulting desync isn't confined to the one script/menu it happens in
  — the dangling tokens (everything after the real closing brace it should
  have consumed) get replayed against WHATEVER comes next in the token
  stream, including subsequent independent top-level menu blocks. See
  `edge_cases.menu`'s header comment for the full 3-extra-`Menu_New`-call
  cascade one instance of this causes.
