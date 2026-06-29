# Intended Folder Hierarchy

This is the intended folder hierarchy to be Rust module based. This is based on these observations: Raven has separate SP/MP source roots; `game`, `cgame`, and `ui` are peer module surfaces in both roots; `qcommon` overlaps heavily but is not byte-identical; engine/library systems sit outside the module surfaces.

| Folder | Description |
| --- | --- |
| `src/shared/` | Rust-native shared primitives that are not tied to a Raven source subsystem. |
| `src/common/` | Raven-derived common strata, split by proven sharing scope. |
| `src/common/qcommon/` | Intentionally unified Rust home for Raven `code/qcommon` and `codemp/qcommon` concepts. |
| `src/common/mp/` | Common support shared within MP modules, not yet proven shared with SP. |
| `src/common/mp/bg/` | MP `bg_*` gameplay support. |
| `src/common/mp/game/` | MP game-module common support. |
| `src/common/mp/cgame/` | MP cgame-module common support. |
| `src/common/mp/ui/` | MP ui-module common support. |
| `src/common/sp/` | Common support shared within SP modules, not yet proven shared with MP. |
| `src/common/sp/bg/` | SP `bg_*` gameplay support. |
| `src/common/sp/game/` | SP game-module common support. |
| `src/common/sp/cgame/` | SP cgame-module common support. |
| `src/common/sp/ui/` | SP ui-module common support. |
| `src/modules/` | Runtime module implementations, following Raven's module surfaces. |
| `src/modules/mp/game/` | MP game module implementation matching Raven `codemp/game`. |
| `src/modules/mp/cgame/` | MP cgame module implementation matching Raven `codemp/cgame`. |
| `src/modules/mp/ui/` | MP ui module implementation matching Raven `codemp/ui`. |
| `src/modules/sp/game/` | SP game module implementation matching Raven `code/game`. |
| `src/modules/sp/cgame/` | SP cgame module implementation matching Raven `code/cgame`. |
| `src/modules/sp/ui/` | SP ui module implementation matching Raven `code/ui`. |
| `src/boundary/` | Typed ABI boundary between engine and runtime modules. |
| `src/boundary/generic/` | Shared transport/message shapes for syscall and `vmMain`. |
| `src/boundary/mp/` | MP ABI surfaces for `game`, `cgame`, and `ui`. |
| `src/boundary/sp/` | SP ABI surfaces for `game`, `cgame`, and `ui`. |
| `src/engine/` | Rust engine-side backend code. |
| `src/ffi/` | Low-level FFI definitions that are not owned by a narrower boundary module. |
| `src/bg/` | Legacy/current Rust `bg` location; intended to be reconciled with `src/common/{mp,sp}/bg`. |
| `src/game/` | Legacy/current Rust game location; intended to be reconciled with `src/modules/{mp,sp}/game`. |

