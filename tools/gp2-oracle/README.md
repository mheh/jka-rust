# gp2-oracle — differential harness for the GP2 C++-track port

Compiles the **unmodified** Raven GP2 sources (MP `codemp/qcommon/GenericParser2.cpp`,
SP `code/game/genericparser2.cpp`) into standalone dumper binaries, runs them over
`fixtures/*.gp2`, and stores the canonical dumps under `golden/`. The Rust ports
(`crates/mp/engine/qcommon/src/gp2/`, `crates/sp/qshared/src/common/sp/game/gp2/`)
must reproduce the goldens byte-for-byte via `tests/gp2_parity.rs` in each crate.

## Usage

```sh
sh run.sh           # build dumpers, diff current output against golden/
sh run.sh --regen   # rebuild golden/ (after adding fixtures)
cargo test -p mp_engine_qcommon -p sp_qshared --test gp2_parity
```

`run.sh` copies the oracle sources into `build/` next to the stub headers in
`stubs/` so their relative `#include`s resolve to the stubs; `oracle/` is never
edited. The goldens are committed, so `cargo test` needs no C++ toolchain —
`run.sh` is only needed to regenerate or spot-check.

## What the dump covers

Per group (recursively): name, pairs with all values in insertion order (`G`/`P`
lines), `FindPairValue` probes for every pair name plus fixed keys exercising
missing keys, case-insensitivity, and MP's `"||"` multi-key search (`F` lines);
then the sorted `InOrder` chains (`IG`/`IP`), and finally `Write` output.

Fixture highlights: `truncated.gp2` proves the MP/SP divergence (unclosed group =
parse error in MP, success in SP, because SP's `AddGroup` never sets `mParent`);
`eol.gp2` pins rest-of-line value semantics (trailing space before a `//` comment
is kept, trailing tabs trimmed); `bigtoken.gp2` pins the `MAX_TOKEN_SIZE` discard.

## Normalizations (documented Rust divergences)

- `FindPairValue` returning NULL for a found-but-valueless pair (`key []`) is
  printed as the default `<DEF>` — the Rust port folds that case into `None`.
- Fixtures end with a newline and avoid unterminated quotes at EOF: Raven's
  tokenizer reads past the buffer there (UB); the Rust port stops at end of data.

## Stub fidelity

`Q_stricmp`/`Q_stricmpn`/`stricmp`/`strcmpi` are stubbed as `strcasecmp`, which
matches Raven's ASCII-range behavior; fixtures keep sorted names ASCII.
