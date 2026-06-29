# ABI Type Inventory Audit

This tool scans the Rust ABI files for type references and `FIXME: create type` markers, checks whether those names have Rust definitions, and searches the Raven oracle source for candidate definitions.

Run it from the repository root:

```sh
python3 tools/abi/type-inventory-audit/audit.py
```

By default it writes:

- `tools/abi/type-inventory-audit/type-inventory-report.tsv`
- `tools/abi/type-inventory-audit/type-inventory-summary.md`

The audit is intentionally static because `src/abi` is not currently declared by `src/lib.rs`, so `cargo build` does not validate these ABI files yet.
