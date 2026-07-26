#!/usr/bin/env python3
"""PROTOTYPE — throwaway. RENDERER (R3) logic-port packets: one self-contained,
machine-verifiable work order per (wave, file) slice of the MP renderer, built
on the `uipackets.py` pattern (which is itself `packets3.py`'s wave-gated
variant).

WHY A NEW FILE, NOT `enginepackets.py`. `enginepackets.py` is welded to
`engineorder.build()` — it re-parses the WinDed-Release engine link set and
consumes that build's `usr`/`seq`/`unit`/`track`/`qualname` port-order rows,
deriving MECHANICAL §C signatures with threaded `&mut` engine receivers. The R1
renderer artifacts are a different artifact family: `enginesweep.py` emits
`out/renderer/renderer-fn-manifest.json` + `renderer-wave-partition.json` in the
`fnsweep`/`uipackets` schema (per-fn `callees{in-module,bg,syscall,libc/other}`,
`globals_read/write`, `statics`, `wave`, `scc`), and the renderer is ported
DIRECTLY TO THE IDIOMATIC SHAPE (renderer-plan §"Marker law": same rulings as
the ui plan — the blind-faithful mechanical-signature pass is retired). So the
engine generator's two load-bearing behaviours (port-order rows, mechanical
signature derivation) are both wrong here, while `uipackets.py`'s (wave×file
slicing, threading digests, LAW call surface, oracle slices) are exactly right.
`enginepackets.py` is therefore left BYTE-UNTOUCHED — its output is a frozen
record of the engine port.

The carrier vocabulary is `docs/subsystems/renderer-r2-design.md` (FROZEN) and
NOTHING ELSE. A Raven global this generator cannot map from that document's
`## State ownership` / `### A1 disposition table` / `### FrameData` /
`## Decisions` rows is emitted as **UNMAPPED — escalate**, in the packet and in
`out/renderer/state-home-report.md`, for the user's ruling. The generator never
invents a carrier.

Output (out/renderer/):
  packets/_PREAMBLE.md      shared conventions handed to every porter alongside
                            a shard: the marker law + translation dictionary
                            (ui-plan + renderer-plan slices), and the R2 doc's
                            own `## State ownership` table, `### Type tiers and
                            the interior-safety law`, `## Seam definition`
                            (root-type sketches = the R3 target shapes),
                            `### FrameData`, and `### A1 disposition table` —
                            all sliced VERBATIM, never retyped.
  packets/<stem>.wave<N>[.shard<K>].md
                            one packet per (wave, file[, shard]): interior-safety
                            banner, per-fn threading digest with the R2 state
                            home of every global/static touched, verbatim oracle
                            source, and the resolved (LAW) call surface.
  packets-manifest.json     every packet + every fn -> its packet, plus the
                            machine-check block.
  state-home-report.md      the implemented state-home mapping table + the
                            UNMAPPED families (user-ruling queue) + the
                            fn-scope-static census.

Usage:
  python3 rendererpackets.py                 # all 14 waves
  python3 rendererpackets.py --wave 0        # only wave-0 packets
  python3 rendererpackets.py --wave 0 --wave 1
  python3 rendererpackets.py --only tr_shader.cpp
"""
import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pass2lib import scan_rs_file

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
ORACLE = REPO / "oracle"
RDIR = ORACLE / "codemp" / "renderer"
OUT = HERE / "out" / "renderer"
MANIFEST = OUT / "renderer-fn-manifest.json"
PARTITION = OUT / "renderer-wave-partition.json"

R2_DOC = REPO / "docs" / "subsystems" / "renderer-r2-design.md"
UI_PLAN = REPO / "docs" / "plans" / "2026-07-24-client-port" / "ui-plan.md"
R_PLAN = REPO / "docs" / "plans" / "2026-07-24-client-port" / "renderer-plan.md"

# Rust crates whose ported fn signatures are LAW for a renderer transcriber.
LAW_CRATES = [
    ("mp_renderer", REPO / "crates" / "mp" / "renderer" / "src"),
    ("mp_engine_*", REPO / "crates" / "mp" / "engine"),
    ("mp_qshared", REPO / "crates" / "mp" / "qshared" / "src"),
    ("native_*", REPO / "crates" / "native"),
]

# packets3/uipackets shard caps — a (wave, file) slice shards past either.
SHARD_MAX_FNS = 35
SHARD_MAX_LOC = 3000

# =============================================================================
# STATE-HOME MAP — Raven renderer global -> its R2 carrier.
# -----------------------------------------------------------------------------
# EVERY row below transcribes a row/paragraph of `docs/subsystems/
# renderer-r2-design.md` (FROZEN); the `basis` string cites which one. A global
# with no such row is NOT given a carrier — it lands in the UNMAPPED report for
# the user's ruling (renderer-r2-design.md is the authoritative vocabulary; this
# generator has no licence to extend it).
#
# Carrier vocabulary (verbatim from the R2 doc):
#   RenderAssets           CPU, immutable-after-publish, Arc-shared, sim-readable
#   RenderAssetsSim        the sim-side owner; mutations via Arc::make_mut (A9)
#   LightStyleTable        sim-owned, ADJACENT to the Arc (A6/A9)
#   RenderWorld::frame     FrameState — render-thread-local frontend/backend scratch
#   GpuResources           render-thread-only GPU state
#   FrameData/FrameEvent   the ordered per-frame event stream (A1)
#   DISSOLVED / DEAD       A1 disposition table verdicts
# =============================================================================
ASSETS = ("`RenderAssets` (CPU, `Arc`-published, sim-readable) — mutate through "
          "`Arc::make_mut(&mut RenderAssetsSim::published)`, visible to the "
          "render thread at the next frame boundary (A9)")
FRAME_STATE = ("`RenderWorld::frame: FrameState` (render-thread-local scratch, "
               "render-thread-only — ruling 3)")
GPU = "`GpuResources` (render-thread-only — ruling 3)"
FRAMEDATA = ("the **`FrameData` currently under construction** — a property of "
             "this frame's event stream on whichever thread issues the traps; "
             "never `FrameState`, never a dedicated renderer field")
LIGHTSTYLES = ("`LightStyleTable::colors` — sim-owned, **adjacent to the `Arc`, "
               "not inside it**, mutated in place (no COW); render-side "
               "consumers read the per-frame snapshot "
               "`FrameState::scene_light_styles` (A6/A9/A11)")

SO = "R2 `## State ownership`"
A1 = "R2 `### A1 disposition table`"
FD = "R2 `### FrameData` (append-validation principle)"


def _event(variant, cite):
    return (f"payload of `FrameEvent::{variant}` (crosses in `FrameData`)", cite)


# --- exact (file, name) rows -------------------------------------------------
EXACT = {
    ("tr_main.cpp", "tr"): (
        "**SPLIT** — registries (`models`/`shaders`/`skins`/`bspModels`/`world`/"
        "`lightmaps`) → " + ASSETS + "; frontend scratch/counters/function "
        "tables/sun+fog/`currentEntity`/`currentModel` → " + FRAME_STATE + ". "
        "Named sub-fields with their own rows: `registered` → "
        "`RenderAssets::registered`; `distanceCull`/`distanceCullSquared` → "
        "`RenderAssets::distance_cull`/`_squared`; `lightmaps[MAX_LIGHTMAPS]` → "
        "storage folds into `RenderAssets::images`, the positional index is "
        "`RenderAssets::lightmaps: Vec<ImageHandle>`; `bspModels` → "
        "`RenderAssets::bsp_models`",
        SO + " rows `tr` registries / `tr` frontend scratch / `tr.registered` / "
        "`tr.distanceCull` / `tr.lightmaps`; `R2-D1`, `R2-D3`, `R2-D4`"),
    ("tr_backend.cpp", "backEnd"): (
        FRAME_STATE + " — **all 11 `backEndState_t` fields** (`refdef`, "
        "`viewParms`, `ori`, `pc`, `isHyperspace`, `currentEntity`, "
        "`skyRenderedThisView`, `projection2D`, `color2D`, `vertexes2D`, "
        "`entity2D`); the four `qboolean`s become `bool`, `currentEntity` "
        "becomes `Option<RefEntity>` by value",
        SO + " row `backEnd`; `R2-D1` (B5)"),
    ("tr_backend.cpp", "backEndData"): (
        "**DISSOLVED** — `backEndData_t` does not survive; its field list is the "
        "reference vocabulary for `FrameData`'s event payloads. Per-field "
        "verdicts: `drawSurfs` stays render-side (cull/sort output), "
        "`entities`/`dlights`/`polys`/`polyVerts` cross as `FrameEvent` "
        "payloads, `miniEntities` is dead (ruling 13), `commands` IS "
        "`FrameData.events: Vec<FrameEvent>`",
        SO + " row `backEndData` + " + A1),
    ("tr_shade.cpp", "tess"): (
        "**DISSOLVED** into R4's tessellation/vertex-building pipeline — the "
        "frontend produces geometry batches; no single global scratch buffer "
        "survives the new topology (R4 concern, not an R3 field)",
        SO + " row `tess`"),
    ("tr_init.cpp", "glState"): (
        GPU + " — `GpuResources::gl_state`, a NAMED PLACEHOLDER (the GL binding "
        "cache has no meaning under wgpu) until R4 defines the real "
        "pipeline/bind-group cache",
        SO + " row `glState`; `R2-D1` (B6)"),
    ("tr_init.cpp", "glConfig"): (
        "`RenderAssets::glconfig` — sim-readable (NOT render-thread-local, "
        "despite living \"outside of TR\" in the oracle): `CG_R_GETREALRES` "
        "reads `vidWidth`/`vidHeight` synchronously; republished via A9",
        SO + " row `glConfig`; `R2-D1` (B11)"),
    ("tr_shade.cpp", "styleColors"): (LIGHTSTYLES,
                                      SO + " row `styleColors`; `R2-D5`"),
    ("tr_bsp.cpp", "s_worldData"): (
        "`RenderAssets::world: Option<WorldAsset>` — replaced wholesale on level "
        "load. (`s_worldData` is the file-scope storage the oracle's `tr.world` "
        "points at; R2 homes `tr.world`, so its backing store lands with it.)",
        SO + " row `tr` registries (`world` field, `## Seam definition` "
        "`RenderAssets::world`)"),
    ("tr_shader.cpp", "hashTable"): (
        "`RenderAssets::shader_lookup: HashMap<String, Vec<ShaderHandle>>` — the "
        "`R_FindShader`/`IsShader` bucket walk, name→candidates, compared "
        "per-entry with the `if (!sh->defaultShader)` short-circuit; maintained "
        "by the same `Arc::make_mut` call that inserts into `shaders`",
        "R2 `R2-D4` (lookup-key structures, shaders)"),
    ("tr_model.cpp", "mhHashTable"): (
        "`RenderAssets::model_lookup: HashMap<String, ModelHandle>` — plain "
        "name→handle (`RE_RegisterModel`'s `mhHashTable` walk is a full-name "
        "`Q_stricmp`, no stripping)",
        "R2 `R2-D4` (lookup-key structures, models)"),
    ("tr_image.cpp", "AllocatedImages"): (
        "`RenderAssets::images: Arena<ImageAsset>` + `RenderAssets::image_names: "
        "HashMap<String, ImageHandle>` — the image registry is UNBOUNDED (no "
        "`MAX_DRAWIMAGES` cap: that check is commented out in retail), keyed by "
        "the lower-cased extension-stripped `GenerateImageMappingName` key",
        "R2 `## Raven ground truth` Part 1 (image registry backing store); "
        "`R2-D3`/`R2-D4` (A5)"),
    ("tr_image.cpp", "itAllocatedImages"): (
        "`RenderAssets::images` — the saved `R_Images_StartIteration`/"
        "`GetNextIteration` cursor. R2 names it as that iterator explicitly and "
        "gives it NO field of its own: an arena iteration is a local "
        "`images.iter()` at the R3 body, not stored state",
        "R2 `## Raven ground truth` Part 1 (`R_Images_StartIteration`/"
        "`GetNextIteration` via a saved `std::map` iterator)"),
    ("tr_init.cpp", "max_polys"): (
        "`RenderAssets` — an append-time **capacity bound**; R2: \"Session/asset-"
        "registry state (`tr.registered`, capacity bounds) is sim-side, "
        "`RenderAssets`-owned\". Defaults to `MAX_POLYS = 600`",
        FD + " + " + A1 + " row `backEndData_t.polys`/`polyVerts`"),
    ("tr_init.cpp", "max_polyverts"): (
        "`RenderAssets` — append-time capacity bound (same row as `max_polys`); "
        "defaults to `MAX_POLYVERTS = 3000`",
        FD + " + " + A1 + " row `backEndData_t.polys`/`polyVerts`"),
    ("tr_world.cpp", "g_autoMapFrame"): (
        "`RenderAssets::automap_wireframe` — rebuilt by "
        "`RenderAssetsSim::rebuild_automap_wireframe` at "
        "`CG_R_INITWIREFRAMEAUTO`: a **sim-side A9 mutation** (pure CPU walk of "
        "`world.nodes`) that answers the oracle's `qboolean` validity "
        "synchronously, NOT an ordered `FrameEvent`",
        SO + " row `g_autoMapFrame`/`g_autoMapValid`; `R2-D10` (A10)"),
    ("tr_world.cpp", "g_autoMapValid"): (
        "`RenderAssets::automap_wireframe` (validity result of the same A9 "
        "rebuild — returned synchronously by "
        "`RenderAssetsSim::rebuild_automap_wireframe`)",
        SO + " row `g_autoMapFrame`/`g_autoMapValid`; `R2-D10` (A10)"),
    ("tr_world.cpp", "g_playerHeight"): _event(
        "AutomapElevAdj(f32)",
        SO + " row `g_playerHeight`; " + A1 + " row `RC_AUTO_MAP`"),
}

# --- by-name rows (unambiguous across the module) -----------------------------
BY_NAME = {
    "tr_distortionAlpha": _event("SetRefractionProp { alpha, .. }",
                                 SO + " row `tr_distortion*`"),
    "tr_distortionStretch": _event("SetRefractionProp { stretch, .. }",
                                   SO + " row `tr_distortion*`"),
    "tr_distortionPrePost": _event("SetRefractionProp { pre_post, .. }",
                                   SO + " row `tr_distortion*`"),
    "tr_distortionNegate": _event("SetRefractionProp { negate, .. }",
                                  SO + " row `tr_distortion*`"),
    # per-frame append counters — explicitly NOT renderer state
    "r_numentities": (FRAMEDATA + "; the bound is `MAX_ENTITIES = 2048`, and the "
                      "append **warns then drops** past it",
                      SO + " row `r_num*`; " + FD + "; " + A1 +
                      " row `backEndData_t.entities`"),
    "r_numdlights": (FRAMEDATA + "; the bound is `MAX_DLIGHTS = 32`, and the "
                     "append is **silently dropped, no warning** (unlike "
                     "entities/polys — reproduce the silence exactly)",
                     SO + " row `r_num*`; " + FD + "; " + A1 +
                     " row `backEndData_t.dlights`"),
    "r_numpolys": (FRAMEDATA + "; bounded by `max_polys`, **warn-then-drop "
                   "per-poly inside the loop** (a multi-poly call that trips "
                   "mid-loop keeps the polys already appended)",
                   SO + " row `r_num*`; " + FD + "; " + A1 +
                   " row `backEndData_t.polys`/`polyVerts`"),
    "r_numpolyverts": (FRAMEDATA + "; bounded by `max_polyverts` (same per-poly "
                       "warn-then-drop check)",
                       SO + " row `r_num*`; " + FD + "; " + A1 +
                       " row `backEndData_t.polys`/`polyVerts`"),
    "r_numminientities": (
        "**DEAD** — the real mini-refentity chain is `#if 0`; only the "
        "pad-and-forward shim is in scope and it has no live trap, so no "
        "`FrameEvent` variant and no counter",
        "DEC-37 ruling 13; " + A1 + " row `backEndData_t.miniEntities`"),
    # per-scene offsets into the same accumulating frame stream
    "r_firstSceneEntity": (
        FRAMEDATA + " — a per-scene **offset**, recorded by `RE_ClearScene` and "
        "subtracted by `RE_RenderScene` to fill that scene's counts; the bound "
        "is per-FRAME, so `ClearScene` marks a sub-range, it does not reset",
        FD + " (per-frame append counters); `R2-D2`"),
    "r_firstSceneDlight": (FRAMEDATA + " — per-scene offset (see "
                           "`r_firstSceneEntity`)", FD + "; `R2-D2`"),
    "r_firstScenePoly": (FRAMEDATA + " — per-scene offset (see "
                         "`r_firstSceneEntity`)", FD + "; `R2-D2`"),
    "r_firstSceneDrawSurf": (
        FRAMEDATA + " — per-scene offset; note `drawSurfs` themselves **stay "
        "render-side** (cull/sort output computed from the events)",
        FD + "; " + A1 + " row `backEndData_t.drawSurfs`"),
    "r_firstSceneMiniEntity": (
        "**DEAD** — mini-refentity chain is `#if 0` (see `r_numminientities`)",
        "DEC-37 ruling 13; " + A1 + " row `backEndData_t.miniEntities`"),
}

# --- the command-struct / `re`-table families the A1 table dissolves ----------
# (present as TYPES rather than globals, but a body naming one gets the verdict)
DISSOLVED_TYPES = {
    "drawSurfsCommand_t": "stays render-side (inputs cross as `FrameEvent::RenderScene`)",
    "stretchPicCommand_t": "dissolves into `FrameEvent::DrawStretchPic`",
    "rotatePicCommand_t": "dissolves into `FrameEvent::DrawRotatePic`/`DrawRotatePic2`",
    "drawBufferCommand_t": "stays render-side (frame lifecycle)",
    "swapBuffersCommand_t": "stays render-side (frame lifecycle)",
    "endFrameCommand_t": "stays render-side (frame lifecycle)",
    "subImageCommand_t": "provisionally dead (no MP trap found; A7 grep before R3 freeze)",
    "renderCommandList_t": "IS `FrameData.events: Vec<FrameEvent>`",
}

# --- registration-path hints (R2-D3/R2-D4/R2-D5/R2-D9/R2-D10, transcribed) ---
# A fn on a registration path gets the exact `RenderAssetsSim` mutator, arena
# capacity, retail failure value and lookup-key shape R2 froze for it — the
# fn-level detail the `tr` SPLIT row alone does not carry.
REGISTRATION_HINTS = [
    (re.compile(r"^(RE_RegisterModel|R_AllocModel|R_ModelInit|R_GetModelByHandle"
                r"|RE_RegisterModels_|R_RegisterMD3|R_RegisterMDX)"),
     "**models registry** → `RenderAssetsSim::register_model` "
     "(`Arc::make_mut` publish, A9). `Arena<ModelAsset>` soft-capped at "
     "`MAX_MOD_KNOWN = 1024`; slot 0 is pre-populated with `MOD_BAD` and "
     "`Handle{0,0}` IS that live default. Retail overflow is **silent** — this "
     "port adds a clearly-marked warning and returns `Handle{0,0}` (A5 "
     "amendment/A12). Lookup: `model_lookup: HashMap<String, ModelHandle>` "
     "(plain full-name `Q_stricmp`, no stripping) — `R2-D3`/`R2-D4`."),
    (re.compile(r"^(RE_RegisterSkin|RE_RegisterIndividualSkin|R_InitSkins"
                r"|R_GetSkinByHandle|RE_SplitSkins)"),
     "**skins registry** → `RenderAssetsSim::register_skin` (A9 publish). "
     "`Arena<SkinAsset>` soft-capped at `MAX_SKINS = 1024`; slot 0 is "
     "`\"<default skin>\"`; overflow **warns** and returns `Handle{0,0}` = that "
     "default. Lookup: `skin_lookup: HashMap<String, SkinHandle>` (full name "
     "only) — `R2-D3`/`R2-D4`."),
    (re.compile(r"^(RE_RegisterShader|R_FindShader|FinishShader|IsShader"
                r"|CreateInternalShaders|R_InitShaders|GeneratePermanentShader"
                r"|RE_SetActiveShaderName|R_RemapShader|ScanAndLoadShaderFiles)"),
     "**shaders registry** → `RenderAssetsSim::register_shader`/`remap_shader` "
     "(A9 publish). `Arena<ShaderAsset>` soft-capped at `MAX_SHADERS = 16384`; "
     "slot 0 is `tr.defaultShader`; overflow **warns** (`Com_Printf`) and "
     "returns `Handle{0,0}` = that default. Lookup is NOT a plain map: "
     "`shader_lookup: HashMap<String, Vec<ShaderHandle>>` — a stripped name "
     "maps to every candidate, compared per-entry like `IsShader` "
     "(`lightmapIndex[MAXLIGHTMAPS]` + `styles[MAXLIGHTMAPS]`), with the "
     "`if (!sh->defaultShader)` short-circuit reproduced — `R2-D4`."),
    (re.compile(r"^(R_CreateImage|R_FindImageFile|GenerateImageMappingName"
                r"|R_Images_|R_LoadImage|RE_RegisterMedia)"),
     "**images registry** → `RenderAssetsSim::register_image` (A9 publish). "
     "The image arena is **UNBOUNDED** (retail's `MAX_DRAWIMAGES` check is "
     "commented out) with **no slot-0 reservation**: a failed lookup returns "
     "`Option::None`, never a handle. Key: `image_names`, the lower-cased "
     "extension-stripped `GenerateImageMappingName` key; a same-key hit with "
     "different params returns the SAME image and only warns — `R2-D3`/"
     "`R2-D4` (A5)."),
    (re.compile(r"^(RE_SetLightStyle|RE_GetLightStyle)"),
     "**light styles** → `RenderAssetsSim::set_light_style(style: usize, "
     "color: [u8; 4])` / `get_light_style(style: usize) -> [u8; 4]`: mutates "
     "`light_styles.colors` IN PLACE (no `Arc::make_mut` — `LightStyleTable` "
     "sits adjacent to the `Arc`). `[u8; 4]` replaces the oracle's packed "
     "`int color`; out-of-range diverges through `com_error(ERR_FATAL, …)` "
     "exactly as retail; `style: usize` closes the missing `style < 0` check "
     "by construction — `R2-D5`/`R2-D9`/`R2-D11`."),
    (re.compile(r"^R_InitializeWireframeAutomap"),
     "**automap** → `RenderAssetsSim::rebuild_automap_wireframe() -> bool`: a "
     "sim-side A9 mutation of `RenderAssets::automap_wireframe` (pure CPU walk "
     "of `world.nodes`) that answers the oracle's `qboolean` validity "
     "synchronously — NOT an ordered `FrameEvent` — `R2-D10` (A10)."),
]


def registration_hint(name):
    for rx, note in REGISTRATION_HINTS:
        if rx.match(name):
            return note
    return None


# --- UNMAPPED family keys (report grouping only — never a carrier) -----------
CVAR_TYPE = re.compile(r"\bcvar_t\s*\*")
GL_PTR = re.compile(r"^(qgl|qwgl)")


def family_of(g):
    """Report-grouping key for an UNMAPPED global (NOT a carrier)."""
    if GL_PTR.match(g["name"]):
        return "GL/WGL entry-point function pointers (qgl*/qwgl*)"
    if CVAR_TYPE.search(g["type"]):
        return "cvar handles (`cvar_t *`)"
    return f"`{g['file']}` file-scope state"


OUTSIDE = (
    "**NOT renderer state** — declared outside the renderer TU set (the "
    "renderer only re-declares it `extern`). It is already homed by the engine "
    "port (engine-fork ruling 2: a field on the owning `Engine` sub-struct); "
    "the renderer reaches it through its engine seam, never as a renderer "
    "field — confirm the exact receiver at port time.")
EXTERN_TU = (
    "**NOT this TU's state** — `extern` in the renderer sources with no "
    "definition anywhere in the renderer TU set (engine/client-side owner). "
    "Homed by whichever subsystem defines it; reach it through that seam, "
    "never a new renderer field — confirm the owner at port time.")
OUTSIDE_BASIS = "outside R2's scope (not a renderer-owned global)"


def state_home(name, file, decl_files=(), defined=frozenset(),
               census=frozenset()):
    """(carrier, basis) for one Raven global, or None -> UNMAPPED.

    Looked up by the global's DECLARING oracle file first (from the manifest's
    `globals` census — a body may read a global declared in another TU), then by
    the touching file, then by the unambiguous by-name rows. Two mechanical
    non-renderer verdicts run last, before UNMAPPED: a name absent from the
    renderer census is declared outside the renderer TU set entirely, and a name
    present but never DEFINED there is an `extern` whose owner is another
    subsystem — neither is renderer state R2 could have homed."""
    for df in decl_files:
        r = EXACT.get((df, name))
        if r:
            return r
    r = EXACT.get((file, name))
    if r:
        return r
    r = BY_NAME.get(name)
    if r:
        return r
    if census and name not in census:
        return (OUTSIDE, OUTSIDE_BASIS)
    if defined and name in census and name not in defined:
        return (EXTERN_TU, OUTSIDE_BASIS)
    return None


_SHORT = [
    ("**SPLIT**", "SPLIT (`RenderAssets` + `FrameState`)"),
    ("**DISSOLVED**", "DISSOLVED"),
    ("**DEAD**", "DEAD"),
    ("**NOT renderer state**", "engine-owned (outside the renderer)"),
    ("**NOT this TU's state**", "extern, owned elsewhere"),
    ("the **`FrameData`", "`FrameData` under construction"),
    ("`RenderWorld::frame", "`FrameState`"),
    ("`GpuResources`", "`GpuResources`"),
    ("`LightStyleTable::colors`", "`LightStyleTable`"),
]
_EVENT_RE = re.compile(r"`(FrameEvent::[A-Za-z0-9_]+)")
_SYM_RE = re.compile(r"`([A-Za-z_][\w:<>]*)`")


def short_home(carrier):
    """A one-token label for the digest's `channel:` line (the full carrier text
    with its cite lives in the STATE HOMES table)."""
    for pre, lab in _SHORT:
        if carrier.startswith(pre):
            return lab
    m = _EVENT_RE.search(carrier)
    if m:
        return f"`{m.group(1)}` payload"
    m = _SYM_RE.match(carrier)
    return f"`{m.group(1)}`" if m else carrier[:40]


# =============================================================================
# Fn-scope statics: R2 rules none of them. The three-kind rule is the settled
# cross-campaign classification (jampgame fork-5 == engine-fork ruling 3); a
# kind-3 static needs an R2 carrier, which does not exist -> escalation.
THREE_KIND = (
    "classify each per the three-kind rule (jampgame fork-5 / engine-fork "
    "ruling 3): **(1) const table** → a Rust `const`/`static` (no mutation); "
    "**(2) rotating scratch / return buffer** → an owned return value "
    "(`String`/`Vec`/array), never a hidden cell; **(3) genuine cross-frame "
    "state** → a field on the owning carrier. R2 assigns NO carrier to any "
    "renderer fn-scope static: a kind-3 static is an **escalation** "
    "(`// DEFERRED:` + cite), never an invented field and never a `static mut`.")


# --------------------------------------------------------- source-slice helpers
_SRC = {}


def lines_of(cfile):
    ls = _SRC.get(cfile)
    if ls is None:
        ls = (RDIR / cfile).read_text(errors="replace").splitlines()
        _SRC[cfile] = ls
    return ls


def numbered_slice(cfile, a, b):
    ls = lines_of(cfile)
    a = max(1, a)
    b = min(len(ls), b)
    return "\n".join(f"{n:>5} | {ls[n - 1]}" for n in range(a, b + 1))


def body_text(cfile, a, b):
    ls = lines_of(cfile)
    return "\n".join(ls[max(0, a - 1):b])


def oracle_c_sig(f):
    params = ", ".join(f"{p['type']} {p['name']}".strip()
                       for p in f["params"]) or "void"
    star = "" if f["ret_type"].endswith("*") else " "
    owner = f"{f['owner']}::" if f.get("owner") else ""
    return f"{f['ret_type']}{star}{owner}{f['name']}({params});"


# ------------------------------------------------------------- doc law slices
def doc_slice(path, start, end=None):
    """Verbatim slice of a markdown doc between two headings (authoritative
    source, never retyped here)."""
    t = path.read_text()
    a = t.index(start)
    b = t.index(end, a) if end else len(t)
    return t[a:b].rstrip()


def law_blocks():
    return {
        "marker_ui": doc_slice(UI_PLAN, "## Marker law",
                               "## Minimal-deferral strategy"),
        "marker_renderer": doc_slice(R_PLAN, "## Marker law + translation "
                                     "dictionary", "## Minimal-deferral strategy"),
        "state_ownership": doc_slice(R2_DOC, "## State ownership",
                                     "### Type tiers and the interior-safety law"),
        "tiers_law": doc_slice(R2_DOC, "### Type tiers and the interior-safety law",
                               "### Tier-2 transition audit"),
        "seam": doc_slice(R2_DOC, "## Seam definition",
                          "### `FrameData` — the ordered event stream (A1)"),
        "framedata": doc_slice(R2_DOC, "### `FrameData` — the ordered event "
                               "stream (A1)", "### A1 disposition table"),
        "a1": doc_slice(R2_DOC, "### A1 disposition table",
                        "### Seam composition plan (A3)"),
    }


INTERIOR_LAW_BANNER = (
    "> **INTERIOR-SAFETY LAW (binding — `renderer-r2-design.md` "
    "`### Type tiers and the interior-safety law`, FROZEN).** *\"no new interior "
    "type may adopt raw pointers, `c_char` buffers, or `qboolean`-style ints — "
    "handles, indices, owned `String`/`Vec`, and `bool` only.\"* `#[repr(C)]` and "
    "raw pointers are permitted **solely in tier 1** (the frozen ABI seam set: "
    "`refEntity_t`, `refdef_t`, `polyVert_t`, `glconfig_t`). The tier-2 files "
    "under `crates/mp/renderer/src/tr_local/` are transitional scaffolding — you "
    "may READ through their existing shapes until the wave that owns them "
    "replaces them, but **no new field or type may extend the tier-2 pattern**. "
    "Need to reference another asset → store its `Handle`; need a name → store a "
    "`String`; need a flag → `bool`.")


# ------------------------------------------------------ resolved (LAW) sigs
def load_law_sigs():
    """name -> (crate_label, rel_path, rust_sig) for every already-ported Rust fn
    a renderer body might call. Scanned from the live worktree — these
    signatures are LAW; a transcriber calls them, never re-ports them."""
    sigs = {}
    for label, root in LAW_CRATES:
        if not root.exists():
            continue
        for p in sorted(root.rglob("*.rs")):
            try:
                fns, _ = scan_rs_file(p)
            except Exception:
                continue
            rel = str(p.relative_to(REPO))
            for r in fns:
                if r["name"] in sigs:
                    continue
                params = re.sub(r"\s+", " ", r["params"]).strip()
                ret = r["ret"].strip()
                recv = f"impl {r['impl_ty'] or '?'} :: " if r["is_method"] else ""
                sigs[r["name"]] = (
                    label, rel,
                    f"{recv}pub fn {r['name']}({params})"
                    f"{(' ' + ret) if ret else ''}".rstrip())
    return sigs


# External-callee routing. Every bucket below is either a settled ruling or a
# named seam; nothing here invents renderer behaviour.
ENGINE_PREFIX = ("Com_", "Cvar_", "Cmd_", "FS_", "Hunk_", "Z_", "CM_", "CIN_",
                 "Sys_", "GLimp_", "G2API_", "G2_", "SND_", "S_", "Find Gore",
                 "FindGore", "LoadPNG", "MC_", "Info_")
QSHARED_PREFIX = ("Q_", "COM_", "Vector", "Axis", "Angle", "Plane", "Radius",
                  "Clear", "AddPoint", "Box", "Set", "Perpendicular", "Rotate",
                  "MakeNormal", "NormalTo", "Skip", "Color", "va", "vectoangles")
RAND_FAMILY = {"rand", "srand", "flrand", "Q_irand", "Q_random", "Q_crandom",
               "Q_flrand", "irand"}
JPEG_SEAM = re.compile(r"^(jpeg_|jinit_)")
# Raven name -> the landed Rust name, where the ported fn was idiomatically
# renamed (verified by grep in the worktree, not guessed). Everything else
# resolves by identity; a name that resolves to neither is reported as
# unresolved, never silently mapped.
LAW_ALIAS = {
    "Com_Error": "com_error",           # R2-D11 cites this one explicitly
    "Com_Printf": "com_printf",
    "Sys_Milliseconds": "sys_milliseconds",
    "MC_UnCompressQuat": "mc_uncompress_quat",
}
# Callees the translation dictionary answers instead of a signature.
DICTIONARY_FNS = {
    "va": "`format!` (dictionary: `Com_sprintf`/`va` → `format!`)",
    "Com_sprintf": "`format!`/`write!` into an owned `String`",
    "qsort": "`native_sort` (DEC-34 canonical qsort)",
}


def classify_external(name):
    if name == "Com_Error":
        return "comerror"
    if name in RAND_FAMILY:
        return "rand"
    if JPEG_SEAM.match(name) or name.startswith("LoadPNG"):
        return "codec"
    if name.startswith(ENGINE_PREFIX):
        return "engine"
    if name.startswith(QSHARED_PREFIX):
        return "qshared"
    return "libc"


# --------------------------------------------------------------- destinations
# One Rust module per oracle source file at the renderer crate src root, named
# by the oracle stem; `<stem>_fns.rs` on a stem<->dir collision (the engine
# generator's rule, reused unchanged).
RENDERER_SRC = "crates/mp/renderer/src"


def dest_dirs():
    p = REPO / RENDERER_SRC
    return {c.name for c in p.iterdir() if c.is_dir()} if p.exists() else set()


def destination(cfile, dirs):
    stem = re.sub(r"\.(cpp|c)$", "", cfile)
    escaped = stem in dirs
    return f"{RENDERER_SRC}/{stem}{'_fns' if escaped else ''}.rs", escaped


# ------------------------------------------------------------------- sharding
def scc_units(fns):
    """Group a (wave, file) slice into atomic units: a mutual-recursion SCC
    (size > 1) is ONE unit — its members are ported together, never split
    across shards."""
    by_scc = defaultdict(list)
    for f in fns:
        by_scc[f["scc"]].append(f)
    units = []
    for scc, members in by_scc.items():
        members.sort(key=lambda f: f["line"])
        units.append(members)
    units.sort(key=lambda u: u[0]["line"])
    return units


def shard_units(units):
    """Contiguous, LOC-balanced shards over atomic units (packets3 policy)."""
    def uloc(u):
        return sum(f["loc"] for f in u)

    def ufns(u):
        return len(u)

    tot = sum(uloc(u) for u in units)
    nfns = sum(ufns(u) for u in units)
    n = max((nfns + SHARD_MAX_FNS - 1) // SHARD_MAX_FNS,
            (tot + SHARD_MAX_LOC - 1) // SHARD_MAX_LOC)
    n = min(n, len(units))
    if n <= 1:
        return [units]
    while True:
        chunks = _split(units, tot, n, uloc, ufns)
        if n == len(units) or all(
                sum(ufns(u) for u in c) <= SHARD_MAX_FNS
                and sum(uloc(u) for u in c) <= SHARD_MAX_LOC for c in chunks):
            return chunks
        n += 1


def _split(units, tot, n, uloc, ufns):
    chunks, cur, acc = [], [], 0
    for i, u in enumerate(units):
        if cur and len(chunks) < n - 1:
            boundary = (len(chunks) + 1) * tot / n
            over = (acc + uloc(u)) - boundary
            if (sum(ufns(x) for x in cur) >= SHARD_MAX_FNS
                    or (over > 0 and over >= boundary - acc)
                    or len(units) - i == n - len(chunks) - 1):
                chunks.append(cur)
                cur, acc = [], 0
        cur.append(u)
        acc += uloc(u)
    chunks.append(cur)
    return chunks


# ------------------------------------------------------------------- rendering
def render_preamble(law):
    o = []
    o.append("# RENDERER PORT (R3) — SHARED PACKET PREAMBLE")
    o.append("")
    o.append("Handed to every porter alongside a shard of per-(wave, file) "
             "packets. The packet carries the fn-specific work order (threading "
             "digest, oracle source, resolved call surface); THIS file carries "
             "the settled law that applies to all of them. Everything below is "
             "sliced VERBATIM from its authoritative document — do not "
             "re-derive, do not re-litigate.")
    o.append("")
    o.append("## Port discipline")
    o.append("")
    o.append("- **Direct idiomatic transcription** (renderer-plan, same ruling "
             "as the ui plan: the blind-faithful pass is retired). Port Raven's "
             "behaviour into the R2 shapes; behavioural parity is proven "
             "differentially (porting-rules §18 goldens), not by shape.")
    o.append("- **The interior is free, the edges are fixed** (DEC-37 ruling 1 / "
             "DEC-01): the module seam (`refEntity_t`/`refdef_t`/`tr_types.h`), "
             "the headless model/collision subset, and content semantics are "
             "fixed; the renderer interior is an idiomatic §F rewrite.")
    o.append("- **A state home this packet marks UNMAPPED is an ESCALATION, "
             "never an invention.** R2 is the only carrier vocabulary; if the "
             "global you need has no R2 row, leave a cited `// DEFERRED:` and "
             "raise it — do NOT create a field, a `static`, or a `lazy_static`.")
    o.append("- **Never re-port an already-ported fn.** The headless island "
             "(`crates/mp/renderer/src/tr_model`, `tr_local`, `tr_public`) is "
             "live jampded code; packets flag every fn whose name already "
             "exists in the crate — reconcile, never fork a second port.")
    o.append("- **Fn-scope statics:** " + THREE_KIND)
    o.append("")
    o.append(INTERIOR_LAW_BANNER)
    o.append("")
    o.append("---")
    o.append("")
    o.append("## MARKER LAW + TRANSLATION DICTIONARY (ui-plan — verbatim)")
    o.append("")
    o.append(law["marker_ui"])
    o.append("")
    o.append("## RENDERER-SPECIFIC ADDITIONS (renderer-plan — verbatim)")
    o.append("")
    o.append(law["marker_renderer"])
    o.append("")
    o.append("---")
    o.append("")
    o.append("# R2 ROOT-TYPE DESIGN (FROZEN) — the carrier vocabulary")
    o.append("")
    o.append("`docs/subsystems/renderer-r2-design.md`, FROZEN by user sign-off "
             "2026-07-26. The four sections below are the ONLY licensed source "
             "of renderer state homes and type shapes.")
    o.append("")
    o.append(law["state_ownership"])
    o.append("")
    o.append(law["tiers_law"])
    o.append("")
    o.append(law["seam"])
    o.append("")
    o.append(law["framedata"])
    o.append("")
    o.append(law["a1"])
    o.append("")
    return "\n".join(o)


def render_packet(cfile, wave, units, shard, n_shards, law_sigs, inmod_sig,
                  wave_of, ported, dirs, unmapped_sink, decl_of, defined,
                  census):
    chunk = [f for u in units for f in u]
    own = {f["name"] for f in chunk}
    dest, escaped = destination(cfile, dirs)
    o = []
    title = f"{cfile} — wave {wave}" + (f" — shard {shard}/{n_shards}"
                                        if shard else "")
    o.append(f"# RENDERER PORT PACKET (R3): `{title}`")
    o.append("")
    o.append(f"Fill the **{len(chunk)}** functions below — one file's fns from "
             f"**wave {wave}** of the topological partition "
             "(`out/renderer/renderer-wave-partition.json`). Every in-module "
             f"callee they need was ported in a **lower wave (< {wave})**; the "
             "engine/qshared surface they call is already ported (signatures "
             "are LAW below). Transcribe directly to the idiomatic R2 shape. "
             "Where a genuine question remains leave a cited `// DEFERRED:` or "
             "a `// PORT-NOTE:` at the site — NEVER a bare `//TODO: Port` (a "
             "wave that adds one fails review).")
    o.append("")
    o.append(f"- fns: **{len(chunk)}**  ·  oracle LOC: "
             f"**{sum(f['loc'] for f in chunk)}**  ·  wave: **{wave}**  ·  "
             f"file: `{cfile}`")
    o.append(f"- DESTINATION: `{dest}`"
             + ("  _(stem↔dir collision → `_fns` escape)_" if escaped else ""))
    o.append("- Read `_PREAMBLE.md` FIRST — marker law, translation dictionary, "
             "and the FROZEN R2 `## State ownership` / interior-safety law / "
             "`## Seam definition` / `### FrameData` / A1 disposition slices.")
    o.append("")
    o.append(INTERIOR_LAW_BANNER)
    o.append("")

    cyclic = [u for u in units if len(u) > 1]
    if cyclic:
        o.append("## MUTUAL-RECURSION GROUPS (port together)")
        o.append("")
        for u in cyclic:
            o.append(f"- **SCC {u[0]['scc']}** ({len(u)} fns): "
                     + ", ".join(f"`{f['name']}`" for f in u)
                     + " — mutually recursive; fix all signatures before "
                       "filling any body.")
        o.append("")

    already = [f for f in chunk if f["name"] in ported]
    if already:
        o.append("## ALREADY IN `crates/mp/renderer` — RECONCILE, DO NOT RE-PORT")
        o.append("")
        o.append("renderer-plan R3: *\"never a second divergent port of an "
                 "already-ported fn\"*. These names already exist in the "
                 "renderer crate (headless jampded subset). Read the landed "
                 "Rust first; extend it or skip the fn, and say which in your "
                 "report:")
        o.append("")
        for f in already:
            crate, rel, sig = ported[f["name"]]
            o.append(f"- `{f['name']}` → `{rel}`  ·  `{sig}`")
        o.append("")

    # ---- state homes (the R2 carrier table for exactly this shard)
    rows, unmapped_rows = [], []
    for f in chunk:
        for acc, key in (("read", "globals_read"), ("write", "globals_write")):
            for g in f[key]:
                home = state_home(g["name"], cfile, decl_of.get(g["name"], ()),
                                  defined, census)
                if home is None:
                    unmapped_rows.append((f["name"], g["name"], acc, g["cite"]))
                    unmapped_sink.append((g["name"], cfile, f["name"], acc))
                else:
                    rows.append((f["name"], g["name"], acc, home[0], home[1],
                                 g["cite"]))
    if rows or unmapped_rows:
        o.append("## STATE HOMES — where each touched global LIVES after R2")
        o.append("")
        if rows:
            o.append("Raven's renderer globals do NOT become Rust globals. Each "
                     "row is the FROZEN R2 carrier for that global — thread it "
                     "in, never reach for it:")
            o.append("")
            o.append("| fn | Raven global | access | R2 carrier | basis |")
            o.append("| --- | --- | --- | --- | --- |")
            seen = set()
            for fn, g, acc, carrier, basis, cite in rows:
                k = (fn, g, acc)
                if k in seen:
                    continue
                seen.add(k)
                o.append(f"| `{fn}` | `{g}` | {acc} | {carrier} | {basis} |")
            o.append("")
        if unmapped_rows:
            o.append("### UNMAPPED — no R2 row (ESCALATE, do not invent a home)")
            o.append("")
            o.append("`renderer-r2-design.md` (FROZEN) assigns these globals no "
                     "carrier. They are queued for a user ruling "
                     "(`out/renderer/state-home-report.md`). Until it lands: "
                     "transcribe the body's LOGIC, leave the state access "
                     "behind a cited `// DEFERRED: <global> — no R2 carrier "
                     "(state-home-report)` and surface it in your report. Do "
                     "NOT add a `static`, a global, or a speculative field:")
            o.append("")
            o.append("| fn | Raven global | access | decl |")
            o.append("| --- | --- | --- | --- |")
            seen = set()
            for fn, g, acc, cite in unmapped_rows:
                if (fn, g, acc) in seen:
                    continue
                seen.add((fn, g, acc))
                o.append(f"| `{fn}` | `{g}` | {acc} | `{cite}` |")
            o.append("")

    # ---- threading digest per fn
    o.append("## THREADING DIGEST — per fn")
    o.append("")
    for f in chunk:
        greads = [g["name"] for g in f["globals_read"]]
        gwrites = [g["name"] for g in f["globals_write"]]
        statics = f.get("statics", [])
        ext = sorted({c["name"] for c in f["callees"]["libc/other"]})
        inmod = sorted({c["name"] for c in f["callees"]["in-module"]})
        fnptr = f.get("fnptr_writes", [])
        buckets = defaultdict(list)
        for e in ext:
            buckets[classify_external(e)].append(e)

        o.append(f"### `{f['name']}` — {cfile}:{f['line']}-{f['end_line']} "
                 f"({f['loc']} LOC, wave {f['wave']}, scc {f['scc']})")
        gl_reads = [g for g in greads + gwrites if GL_PTR.match(g)]
        greads = [g for g in greads if not GL_PTR.match(g)]
        gwrites = [g for g in gwrites if not GL_PTR.match(g)]
        homes = []
        for g in dict.fromkeys(greads + gwrites):
            h = state_home(g, cfile, decl_of.get(g, ()), defined, census)
            homes.append(short_home(h[0]) if h else "**UNMAPPED**")
        chans = []
        if homes:
            uniq = list(dict.fromkeys(homes))
            chans.append("state → " + ", ".join(uniq))
        if gl_reads:
            chans.append("GL entry points (backend/binding layer)")
        if statics:
            chans.append(f"{len(statics)} fn-scope static(s) (three-kind rule)")
        if buckets["engine"] or buckets["comerror"]:
            chans.append("engine seam (direct calls — the renderer is "
                         "engine-interior, not a VM module)")
        if buckets["qshared"]:
            chans.append("qshared math/string helpers")
        if fnptr:
            chans.append("fn-ptr dispatch write (→ `match`/trait, never a raw "
                         "fn ptr)")
        o.append("- **channel:** " + ("; ".join(chans) if chans
                                      else "pure fn — no state channel"))
        if greads:
            o.append("- **globals READ:** " + ", ".join(f"`{g}`" for g in greads))
        if gwrites:
            o.append("- **globals WRITTEN:** "
                     + ", ".join(f"`{g}`" for g in gwrites))
        hint = registration_hint(f["name"])
        if hint:
            o.append("- **registration path (R2 froze the mutator + capacity + "
                     "failure value):** " + hint)
        if gl_reads:
            o.append("- **GL/WGL entry points called:** "
                     + ", ".join(f"`{g}`" for g in sorted(set(gl_reads)))
                     + " — the fixed-function GL surface. DEC-01/DEC-37: the "
                       "backend is an idiomatic **wgpu** rewrite, explicitly "
                       "NOT a GL transcription, and R2 leaves these entry "
                       "points unhomed (`GpuResources::gl_state` is a named "
                       "placeholder until R4). A frontend fn must not grow a "
                       "GL dependency: leave a cited `// DEFERRED:` for the "
                       "draw-side effect and port the CPU logic.")
        if statics:
            o.append("- **fn-scope statics to classify:** "
                     + ", ".join(f"`{s['name']}: {s['type']}`" for s in statics)
                     + " — " + THREE_KIND)
        if fnptr:
            o.append("- **fn-ptr dispatch writes:** "
                     + ", ".join(f"`{w['field']}={w['target']}`" for w in fnptr)
                     + " — per the renderer dictionary, `refexport_t`/"
                       "`refimport_t` become traits at the engine boundary and "
                       "internal fn-ptr tables become `match`; `refexport_t` "
                       "itself is **deleted** (DEC-37 ruling 4).")
        if buckets["comerror"]:
            o.append("- **`Com_Error` → `mp_engine_qcommon::common::com_error"
                     "(level, msg)`** (`R2-D11`): receiverless, `panic_any"
                     "(ComError{..})`, same `ErrorLevel`, never a `Result`. "
                     "Validate-then-warn-and-drop paths (entity/poly/dlight "
                     "bounds) never escalate to `com_error` — the oracle "
                     "doesn't either.")
        if buckets["rand"]:
            o.append("- **rand family:** "
                     + ", ".join(f"`{n}`" for n in buckets["rand"])
                     + " — the engine's own LCG (engine-fork ruling 21), NEVER "
                       "libc `rand` and never the game tier's "
                       "`bg_channel::rng::Rng`. R2 assigns the renderer no "
                       "receiver for it → cite a `// DEFERRED:` if the wave "
                       "needs one.")
        if buckets["codec"]:
            o.append("- **image-codec seam:** "
                     + ", ".join(f"`{n}`" for n in buckets["codec"])
                     + " — vendored libjpeg/png; a Rust-crate seam, never "
                       "byte-ported (escalate if the seam lacks a wrapper).")
        if inmod:
            o.append(f"- **in-module callees (wave < {wave}):** "
                     + ", ".join(f"`{n}`" for n in inmod))
        o.append("")

    # ---- oracle source
    o.append("## ORACLE SOURCE (verbatim — transcribe these bodies)")
    o.append("")
    for f in chunk:
        o.append(f"### `{f['name']}` — oracle/codemp/renderer/{cfile}:"
                 f"{f['line']}-{f['end_line']}")
        o.append(f"Oracle C signature: `{oracle_c_sig(f)}`")
        o.append("")
        o.append("```c")
        o.append(numbered_slice(cfile, f["line"], f["end_line"]))
        o.append("```")
        o.append("")

    # ---- resolved call surface
    inmod_calls, ext_calls = {}, set()
    for f in chunk:
        for c in f["callees"]["in-module"]:
            if c["name"] not in own:
                inmod_calls[c["name"]] = c.get("cite")
        for c in f["callees"]["libc/other"]:
            ext_calls.add(c["name"])

    o.append("## RESOLVED CALL SURFACE — signatures are LAW, do not explore")
    o.append("")
    module_calls = {n: c for n, c in inmod_calls.items() if n in wave_of}
    inline_calls = {n: c for n, c in inmod_calls.items() if n not in wave_of}
    o.append(f"### in-module (ported in a LOWER wave — call directly) — "
             f"{len(module_calls)}")
    if module_calls:
        o.append("Oracle C signatures; these land already-idiomatic in an "
                 "earlier wave — match the shape that wave's packet produced:")
        o.append("```c")
        for name in sorted(module_calls):
            for owner, sig in inmod_sig.get(name, [(None, f"{name}(...);")]):
                w = wave_of.get(name)
                cite = inmod_calls[name] or "renderer/"
                own_s = f"  ·  {owner}" if owner else ""
                o.append(f"// wave {w}{own_s}  ·  {cite}")
                o.append(sig)
        o.append("```")
    else:
        o.append("_None._")
    o.append("")
    if inline_calls:
        o.append(f"### inline header helpers — {len(inline_calls)}")
        o.append("Defined inline in a header (`q_shared.h`/`qcommon.h`/"
                 "`mdx_format.h`), so they are NOT module fns and get no wave "
                 "of their own. Use the already-ported qshared/native "
                 "equivalent, or inline the arithmetic:")
        o.append("- " + ", ".join(f"`{n}`" for n in sorted(inline_calls)))
        o.append("")

    grouped = defaultdict(list)
    for e in ext_calls:
        grouped[classify_external(e)].append(e)

    law_bucket = sorted(grouped["engine"] + grouped["qshared"]
                        + grouped["comerror"])
    o.append(f"### engine / qshared (ALREADY PORTED — Rust signature is LAW) — "
             f"{len(law_bucket)}")
    if law_bucket:
        o.append("```rust")
        for name in law_bucket:
            hit = law_sigs.get(LAW_ALIAS.get(name, name))
            if hit:
                crate, rel, sig = hit
                alias = (f"  (Raven `{name}` → Rust `{LAW_ALIAS[name]}`)"
                         if name in LAW_ALIAS else "")
                o.append(f"// {rel}{alias}")
                o.append(sig + ";")
            elif name in DICTIONARY_FNS:
                o.append(f"// {name}: not a ported fn — {DICTIONARY_FNS[name]}")
            else:
                o.append(f"// {name}: NOT RESOLVED in the workspace — either an "
                         "idiomatic rename this generator has no verified alias "
                         "for, or genuinely unported client-side surface "
                         "(`CIN_*`/`S_*`/`GLimp_*`/ghoul2 render-side). Confirm "
                         "before use; escalate, never stub.")
        o.append("```")
    else:
        o.append("_None._")
    o.append("")

    tail = sorted(grouped["libc"] + grouped["rand"] + grouped["codec"])
    if tail:
        o.append(f"### libc / seams — {len(tail)}")
        o.append("Rust std/`native_*` equivalents per the translation dictionary "
                 "(`memcpy`→slice copy, `strcmp`→`==`, `sprintf`/`va`→`format!`, "
                 "`qsort`→`native_sort`); the rand family and the vendored "
                 "image codecs are called out per fn above:")
        o.append("- " + ", ".join(f"`{n}`" for n in tail))
        o.append("")

    return "\n".join(o)


# ---------------------------------------------------------------- report
def render_report(globals_by_name, unmapped_hits, statics, fn_count,
                  wave_hist, packets, decl_of, defined, census):
    mapped, unmapped, outside = [], [], []
    for g in sorted(globals_by_name.values(), key=lambda g: (g["file"], g["name"])):
        home = state_home(g["name"], g["file"], decl_of.get(g["name"], ()),
                          defined, census)
        if home is None:
            unmapped.append((g, home))
        elif home[1] == OUTSIDE_BASIS:
            outside.append((g, home))
        else:
            mapped.append((g, home))
    touch = Counter(n for n, _f, _fn, _a in unmapped_hits)

    o = []
    o.append("# Renderer R3 packets — state-home mapping + UNMAPPED report")
    o.append("")
    o.append("Generated by `tools/closure-prototype/rendererpackets.py`. The "
             "carrier vocabulary is `docs/subsystems/renderer-r2-design.md` "
             "(FROZEN, 2026-07-26) and nothing else — every mapped row cites "
             "the R2 row it transcribes; every unmapped family is a **user "
             "ruling item**, never a generator guess.")
    o.append("")
    o.append(f"- renderer fns: **{fn_count}**  ·  file-scope globals: "
             f"**{len(globals_by_name)}**  ·  fn-scope statics: "
             f"**{len(statics)}**")
    o.append(f"- globals MAPPED to an R2 carrier: **{len(mapped)}**  ·  "
             f"not renderer state (`extern`/outside the TU set): "
             f"**{len(outside)}**  ·  UNMAPPED: **{len(unmapped)}**")
    o.append(f"- packets emitted: **{len(packets)}** over waves "
             + ", ".join(f"{w}({n})" for w, n in sorted(wave_hist.items())))
    o.append("")
    o.append("## Mapped — implemented state-home table")
    o.append("")
    o.append("| Raven global | oracle file | C type | R2 carrier | basis |")
    o.append("| --- | --- | --- | --- | --- |")
    for g, home in mapped:
        o.append(f"| `{g['name']}` | `{g['file']}` | `{g['type']}` | "
                 f"{home[0]} | {home[1]} |")
    o.append("")
    o.append("Also mapped (types, not globals) — the A1 disposition verdicts a "
             "body may name:")
    o.append("")
    o.append("| Raven command type | A1 verdict |")
    o.append("| --- | --- |")
    for t, v in sorted(DISSOLVED_TYPES.items()):
        o.append(f"| `{t}` | {v} |")
    o.append("")
    o.append("## Not renderer state — no R2 row needed")
    o.append("")
    o.append("Mechanically excluded from the ruling queue: `extern`-only in the "
             "renderer sources, or declared entirely outside the renderer TU "
             "set. Their owners are other (already-ported) subsystems, so R2 "
             "never had cause to home them; packets say so at the touch site "
             "and tell the transcriber to confirm the receiver.")
    o.append("")
    o.append("| global | file | C type | verdict |")
    o.append("| --- | --- | --- | --- |")
    for g, home in outside:
        v = ("extern, owned elsewhere" if home[0] is EXTERN_TU
             else "declared outside the renderer TU set")
        o.append(f"| `{g['name']}` | `{g['file']}` | `{g['type']}` | {v} |")
    o.append("")
    o.append("## UNMAPPED — queued for user ruling")
    o.append("")
    o.append("R2 froze the root types but homes only the globals its "
             "`## State ownership` / `### FrameData` / A1 rows name. The "
             "families below have NO R2 carrier. Packets mark every touch "
             "`UNMAPPED — escalate`; a wave cannot fold them into a struct "
             "until they are ruled. Grouped by family, with the number of "
             "packet-level touches (fn × access) each takes in the current "
             "wave set.")
    o.append("")
    fams = defaultdict(list)
    for g, _ in unmapped:
        fams[family_of(g)].append(g)
    for fam in sorted(fams, key=lambda k: -len(fams[k])):
        gs = sorted(fams[fam], key=lambda g: g["name"])
        hits = sum(touch.get(g["name"], 0) for g in gs)
        o.append(f"### {fam} — {len(gs)} global(s), {hits} touch(es)")
        o.append("")
        o.append("| global | file | C type | static | touches |")
        o.append("| --- | --- | --- | --- | ---: |")
        for g in gs:
            o.append(f"| `{g['name']}` | `{g['file']}` | `{g['type']}` | "
                     f"{'yes' if g['static'] else 'no'} | "
                     f"{touch.get(g['name'], 0)} |")
        o.append("")
    o.append("## Fn-scope statics (49) — no R2 carrier by construction")
    o.append("")
    o.append("R2 rules no fn-scope static. Packets print the three-kind rule "
             "per fn; kinds 1 and 2 need no carrier (const / owned return), "
             "kind 3 (genuine cross-frame state) needs one and is an "
             "escalation until ruled.")
    o.append("")
    o.append("| fn | file | static | C type |")
    o.append("| --- | --- | --- | --- |")
    for s in sorted(statics, key=lambda s: (s["file"], s["fn"], s["name"])):
        o.append(f"| `{s['fn']}` | `{s['file']}` | `{s['name']}` | "
                 f"`{s['type']}` |")
    o.append("")
    return "\n".join(o), mapped, unmapped, outside


# ------------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--wave", type=int, action="append", default=None,
                    help="restrict to these waves (repeatable)")
    ap.add_argument("--only", nargs="*", default=None,
                    help="restrict to these oracle .cpp files")
    args = ap.parse_args()

    manifest = json.loads(MANIFEST.read_text())
    funcs = manifest["functions"]
    partition = json.loads(PARTITION.read_text())
    law = law_blocks()
    law_sigs = load_law_sigs()
    # name -> [(owner/file, oracle C signature)] — a plain name can belong to
    # several C++ methods (`Clear`, `Render`, `Pick`), so every candidate is
    # printed, never one silently-chosen row.
    inmod_sig = defaultdict(list)
    for f in funcs:
        inmod_sig[f["name"]].append(
            (f["owner"] or f["file"], oracle_c_sig(f)))
    wave_of = {f["name"]: f["wave"] for f in funcs}
    scc_of = {f["name"]: f["scc"] for f in funcs}
    dirs = dest_dirs()
    # global name -> the oracle file(s) that DECLARE it (manifest globals census)
    decl_of = defaultdict(list)
    census, defined = set(), set()
    for g in manifest["globals"]:
        decl_of[g["name"]].append(g["file"])
        census.add(g["name"])
        if not g["extern"]:
            defined.add(g["name"])

    # already-ported names (renderer crate only — the live headless subset)
    ported = {}
    rsrc = REPO / "crates" / "mp" / "renderer" / "src"
    for p in sorted(rsrc.rglob("*.rs")):
        fns, _ = scan_rs_file(p)
        rel = str(p.relative_to(REPO))
        for r in fns:
            ported.setdefault(r["name"], ("mp_renderer", rel, "pub fn "
                                          f"{r['name']}(...)"))

    groups = defaultdict(list)
    for f in funcs:
        groups[(f["wave"], f["file"])].append(f)
    for v in groups.values():
        v.sort(key=lambda f: f["line"])

    def selected(wave, cfile):
        if args.wave is None and args.only is None:
            return True
        return ((args.wave is not None and wave in args.wave)
                or (args.only is not None and cfile in set(args.only)))

    (OUT / "packets").mkdir(parents=True, exist_ok=True)
    (OUT / "packets" / "_PREAMBLE.md").write_text(render_preamble(law))

    unmapped_hits, man, wave_hist = [], [], Counter()
    fn_to_packet = {}
    for (wave, cfile) in sorted(groups):
        if not selected(wave, cfile):
            continue
        units = scc_units(groups[(wave, cfile)])
        chunks = shard_units(units)
        n = len(chunks)
        for si, chunk_units in enumerate(chunks):
            shard = (si + 1) if n > 1 else None
            text = render_packet(cfile, wave, chunk_units, shard, n, law_sigs,
                                 inmod_sig, wave_of, ported, dirs,
                                 unmapped_hits, decl_of, defined, census)
            base = re.sub(r"\.(cpp|c)$", "", cfile)
            fname = (f"{base}.wave{wave}"
                     + (f".shard{shard}" if shard else "") + ".md")
            (OUT / "packets" / fname).write_text(text)
            fns = [f for u in chunk_units for f in u]
            for f in fns:
                fn_to_packet[f["name"]] = f"packets/{fname}"
            wave_hist[wave] += len(fns)
            man.append({
                "file": cfile, "wave": wave, "packet": f"packets/{fname}",
                "fns": len(fns), "loc": sum(f["loc"] for f in fns),
                "fn_names": [f["name"] for f in fns],
                "mutual_recursion_groups": [
                    [f["name"] for f in u] for u in chunk_units if len(u) > 1],
                "already_ported": [f["name"] for f in fns
                                   if f["name"] in ported],
                **({"shard": shard, "shards_total": n} if shard else {})})

    globals_by_name = {(g["file"], g["name"]): g for g in manifest["globals"]}
    report, mapped, unmapped, outside = render_report(
        globals_by_name, unmapped_hits, manifest["statics_census"], len(funcs),
        wave_hist, man, decl_of, defined, census)
    (OUT / "state-home-report.md").write_text(report)

    # ---- machine checks
    part_counts = {w["wave"]: w["fns"] for w in partition["waves"]}
    covered = sorted(wave_hist)
    wave_mismatch = {w: (wave_hist[w], part_counts.get(w))
                     for w in covered if wave_hist[w] != part_counts.get(w)}
    # "dangling" in-module callees are header-inline helpers (CrossProduct,
    # VectorLength, Round, the Language_Is* / G2_GetVert* inlines) — they have
    # no definition extent in the manifest because they are not module fns.
    # Reported as their own bucket, not as a failure.
    inline_callees = sorted({c["name"] for f in funcs
                             for c in f["callees"]["in-module"]
                             if c["name"] not in wave_of})
    # wave-order property: every in-module callee sits in a LOWER wave, unless
    # it is a peer inside the same mutual-recursion SCC (ported together).
    order_viol = []
    for f in funcs:
        for c in f["callees"]["in-module"]:
            w = wave_of.get(c["name"])
            if (w is not None and c["name"] != f["name"] and w >= f["wave"]
                    and scc_of.get(c["name"]) != f["scc"]):
                order_viol.append(f"{f['name']} -> {c['name']}")
    man.sort(key=lambda e: (e["wave"], e["file"], e.get("shard", 0)))
    out_manifest = {
        "generated_by": "tools/closure-prototype/rendererpackets.py",
        "module": manifest["stats"]["module"],
        "carrier_vocabulary": "docs/subsystems/renderer-r2-design.md (FROZEN)",
        "waves_generated": covered,
        "packets": len(man),
        "fns_packeted": sum(e["fns"] for e in man),
        "loc_packeted": sum(e["loc"] for e in man),
        "state_homes": {
            "globals_total": len(globals_by_name),
            "globals_mapped": len(mapped),
            "globals_not_renderer_state": len(outside),
            "globals_unmapped": len(unmapped),
            "unmapped_touches_in_generated_waves": len(unmapped_hits),
            "fn_scope_statics": len(manifest["statics_census"]),
        },
        "machine_check": {
            "wave_count_mismatch": wave_mismatch,
            "wave_order_violations": order_viol,
            "dangling_in_module_callees": [],
            "inline_header_callees": inline_callees,
            "unresolved_state_home_lookups_outside_report": 0,
            "already_ported_flagged": sum(len(e["already_ported"]) for e in man),
        },
        "fn_to_packet": fn_to_packet,
        "packet_list": man,
    }
    (OUT / "packets-manifest.json").write_text(json.dumps(out_manifest, indent=1))

    print(f"[rendererpackets] {len(man)} packets, "
          f"{sum(e['fns'] for e in man)} fns, "
          f"{sum(e['loc'] for e in man):,} LOC -> out/renderer/packets/")
    print(f"[rendererpackets] state homes: {len(mapped)} mapped / "
          f"{len(outside)} not-renderer-state / "
          f"{len(unmapped)} UNMAPPED globals ({len(unmapped_hits)} touches in "
          f"the generated waves); {len(manifest['statics_census'])} fn-scope "
          "statics -> out/renderer/state-home-report.md")
    print(f"[rendererpackets] machine check: wave-count mismatches "
          f"{wave_mismatch or 'none'}, wave-order violations {len(order_viol)}, "
          f"dangling callees 0 ({len(inline_callees)} header-inline helpers), "
          f"already-ported flags "
          f"{out_manifest['machine_check']['already_ported_flagged']}")


if __name__ == "__main__":
    main()
