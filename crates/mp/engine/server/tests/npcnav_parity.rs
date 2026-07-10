//! Differential parity: the Rust `mp_engine_server` NPCNav port
//! ([`Navigator`](mp_engine_server::npcnav::Navigator)) must reproduce, byte
//! for byte, the fixtures + goldens produced by the UNMODIFIED Raven C++
//! `CNavigator` compiled by `tools/npcnav-oracle/build.sh` (goldens under
//! `tools/npcnav-oracle/goldens/`, binary `.nav` fixtures under
//! `tools/npcnav-oracle/fixtures/`).
//!
//! Each test drives the REAL build path exactly as `tools/npcnav-oracle/main.cpp`
//! does — `Init` → `AddRawPoint` → `HardConnect` → `CalculatePaths` → `Save` —
//! reaching the world through the fixture-backed
//! [`MockHost`](mp_host_interface::mock::MockHost) (ruling 32: no test-only
//! constructor; the port's frozen host-taking signatures reach the world through
//! the mock's front door). The dump format mirrors `main.cpp`'s `fprintf`s
//! character for character:
//! * [`matches_oracle_goldens`] — build via the API, `Save`, then emit the
//!   `dumpAll` surface (`== graph ==` + `== rank tables ==` parsed straight out
//!   of the just-`Save`d bytes + the pure-graph query surface) and assert it
//!   equals the committed golden.
//! * [`save_matches_nav_fixtures`] — the `Save`d bytes equal the committed
//!   retail-shaped `.nav` fixture (the 4-byte-`long` shim witness, NAV-D1 /
//!   RULING 44).
//! * [`load_roundtrip_matches_goldens`] — `Load` the committed `.nav` fixture
//!   back through the mock's FS and re-emit the full dump, the oracle's own
//!   Save/Load round-trip witness (`main.cpp:333-343`).
//!
//! Fixtures/goldens are read from `tools/npcnav-oracle/` and are never edited;
//! `d_altRoutes`/`d_patched` read `0` from the mock (missing cvar → `0`), so the
//! `GetBestNodeAltRoute` surface is the pure-graph `!d_altRoutes->integer`
//! short-circuit exactly as the dumper pins it.

use std::fmt::Write as _;
use std::path::PathBuf;

use mp_engine_server::npcnav::{Navigator, NODE_NONE};
use mp_host_interface::mock::MockHost;
use mp_qshared::shared::qfalse;

/// Repo-relative `tools/npcnav-oracle` root (this crate is
/// `crates/mp/engine/server`).
fn oracle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/npcnav-oracle")
}

// ---------------------------------------------------------------------------
// Layout parsing — mirrors `main.cpp`'s `parseLayout` (main.cpp:141-163)
// ---------------------------------------------------------------------------

/// One `node <x> <y> <z> <flags> <radius>` row; flags/radius default `0` when
/// the row omits them (`main.cpp:154`).
struct LNode {
    x: f32,
    y: f32,
    z: f32,
    flags: i32,
    radius: i32,
}

/// A parsed `layouts/*.layout` file: `checksum`, `node` rows (id == file order),
/// `connect <a> <b>` pairs. Mirror of `main.cpp`'s `Layout`.
struct Layout {
    checksum: i32,
    nodes: Vec<LNode>,
    conns: Vec<(i32, i32)>,
}

/// Parse a `.layout` file the same way `main.cpp:141-163` does: skip blank /
/// `#`-comment lines, keyword-dispatch on the first token, and read the fixed
/// field count off each row (trailing `#` comments are ignored because only the
/// leading tokens are consumed, matching the C `sscanf`).
fn parse_layout(path: &std::path::Path) -> Layout {
    let text = std::fs::read_to_string(path).expect("read layout");
    let mut lay = Layout {
        checksum: 0,
        nodes: Vec::new(),
        conns: Vec::new(),
    };
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let tok: Vec<&str> = trimmed.split_whitespace().collect();
        match tok[0] {
            "checksum" => lay.checksum = tok[1].parse().unwrap(),
            "node" => {
                // `sscanf(p, "%*s %f %f %f %d %d", ...)` — flags/radius stay 0
                // if absent (main.cpp:154-155).
                let x = tok[1].parse().unwrap();
                let y = tok[2].parse().unwrap();
                let z = tok[3].parse().unwrap();
                let flags = tok.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
                let radius = tok.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
                lay.nodes.push(LNode {
                    x,
                    y,
                    z,
                    flags,
                    radius,
                });
            }
            "connect" => {
                // Only pushed when both ids parse (main.cpp:157-158).
                if let (Some(a), Some(b)) = (tok.get(1), tok.get(2)) {
                    if let (Ok(a), Ok(b)) = (a.parse::<i32>(), b.parse::<i32>()) {
                        lay.conns.push((a, b));
                    }
                }
            }
            _ => {}
        }
    }
    lay
}

// ---------------------------------------------------------------------------
// Build path — mirrors `main.cpp:303-314`
// ---------------------------------------------------------------------------

/// Drive the REAL build path on a fresh [`Navigator`] and [`MockHost`]:
/// `Init` → per-node `AddRawPoint` → per-connect `HardConnect` →
/// `CalculatePaths` → `Save`. Returns the navigator (graph intact for querying),
/// the host, the qpath `Save` wrote to, and the emitted `.nav` bytes (there is
/// exactly one `fs_write_file` per `Save`, NAV-D3 / RULING 36).
fn build_and_save(lay: &Layout, name: &str) -> (Navigator, MockHost, String, Vec<u8>) {
    let mut host = MockHost::new();
    let mut nav = Navigator::default();
    nav.init();
    for n in &lay.nodes {
        nav.add_raw_point(&mut host, [n.x, n.y, n.z], n.flags, n.radius);
    }
    for &(a, b) in &lay.conns {
        nav.hard_connect(&mut host, a, b);
    }
    nav.calculate_paths(&mut host, qfalse);
    assert!(nav.save(&mut host, name, lay.checksum), "Save failed for {name}");

    let (key, bytes) = host
        .written_files
        .iter()
        .next()
        .map(|(k, v)| (k.clone(), v.clone()))
        .expect("Save wrote exactly one .nav file");
    (nav, host, key, bytes)
}

// ---------------------------------------------------------------------------
// Golden dump — mirrors `main.cpp`'s `dumpAll` (main.cpp:170-249)
// ---------------------------------------------------------------------------

/// Little-endian `int32` read off the `.nav` bytes at `off` (the fixture is
/// retail 4-byte-`long`-shaped, NAV-D1 / RULING 44).
fn rd_i32(b: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

/// `dumpGraph` (main.cpp:170-182): `== graph ==`, `numNodes`, then per node its
/// position/radius/edge count and each `edge N -> node M`.
fn dump_graph(nav: &Navigator, out: &mut String) {
    let n = nav.get_num_nodes();
    writeln!(out, "== graph ==").unwrap();
    writeln!(out, "numNodes {n}").unwrap();
    for i in 0..n {
        let mut pos = [0.0f32; 3];
        nav.get_node_position(i, &mut pos);
        let ne = nav.get_node_num_edges(i);
        writeln!(
            out,
            "node {i} pos {:.3} {:.3} {:.3} radius {} numEdges {ne}",
            pos[0],
            pos[1],
            pos[2],
            nav.get_node_radius(i),
        )
        .unwrap();
        for e in 0..ne {
            writeln!(out, "  edge {e} -> node {}", nav.get_node_edge(i, e)).unwrap();
        }
    }
}

/// `dumpRanksFromFile` (main.cpp:225-241): re-parse each node's raw rank array
/// straight out of the `.nav` bytes, in `curRank++` pop order — the direct
/// human-readable witness of the libstdc++ heap-sift / equal-cost tie order
/// (NAV-D2 / RULING 45). The byte walk matches the dumper's `fseek` arithmetic:
/// skip `navID + checksum + numNodes` (12), then per node skip
/// `NODE id + pos + flags + ID + radius` (28), read `numEdges`, skip the
/// `edge_t[numEdges]` (12 bytes each), read `numRanks`, then the rank `int32`s.
fn dump_ranks_from_bytes(nav: &Navigator, bytes: &[u8], out: &mut String) {
    let n = nav.get_num_nodes();
    writeln!(out, "== rank tables (per node, pop-order; the heap-sift gate) ==").unwrap();
    let mut off = 12usize; // navID + checksum + numNodes
    for i in 0..n {
        off += 4 + 12 + 4 + 4 + 4; // NODE id, pos, flags, ID, radius
        let num_edges = rd_i32(bytes, off);
        off += 4;
        off += num_edges as usize * 12; // edge_t[numEdges]
        let num_ranks = rd_i32(bytes, off);
        off += 4;
        write!(out, "node {i} ranks[{num_ranks}]:").unwrap();
        for _ in 0..num_ranks {
            let v = rd_i32(bytes, off);
            off += 4;
            write!(out, " {v}").unwrap();
        }
        out.push('\n');
    }
}

/// `dumpQueries` (main.cpp:184-218): the full pure-graph query surface over all
/// node pairs — `GetPathCost`, `GetBestNode`, `GetBestNodeAltRoute`
/// (`d_altRoutes = 0`), `Connected`/`NodesAreNeighbors`, and `GetProjectedNode`
/// with each node's own position as the projection origin.
fn dump_queries(nav: &mut Navigator, host: &mut MockHost, out: &mut String) {
    let n = nav.get_num_nodes();

    writeln!(out, "== ranks (GetPathCost s->e, all pairs) ==").unwrap();
    for s in 0..n {
        for e in 0..n {
            writeln!(out, "pathcost {s} {e} = {}", nav.get_path_cost(s, e)).unwrap();
        }
    }

    writeln!(out, "== GetBestNode (s,e,reject=NONE) ==").unwrap();
    for s in 0..n {
        for e in 0..n {
            writeln!(out, "bestnode {s} {e} = {}", nav.get_best_node(s, e, NODE_NONE)).unwrap();
        }
    }

    writeln!(out, "== GetBestNodeAltRoute (d_altRoutes=0; s,e,reject=NONE) ==").unwrap();
    for s in 0..n {
        for e in 0..n {
            let mut pc = 0i32;
            let bn = nav.get_best_node_alt_route(host, s, e, &mut pc, NODE_NONE);
            writeln!(out, "altroute {s} {e} = {bn} cost {pc}").unwrap();
        }
    }

    writeln!(out, "== Connected / NodesAreNeighbors ==").unwrap();
    for s in 0..n {
        for e in 0..n {
            writeln!(
                out,
                "conn {s} {e} = {} neigh {}",
                if nav.connected(s, e) { 1 } else { 0 },
                nav.nodes_are_neighbors(s, e),
            )
            .unwrap();
        }
    }

    writeln!(out, "== GetProjectedNode (origin = each node pos, from each node) ==").unwrap();
    for from in 0..n {
        for o in 0..n {
            let mut origin = [0.0f32; 3];
            nav.get_node_position(o, &mut origin);
            writeln!(
                out,
                "proj from {from} origin@{o} = {}",
                nav.get_projected_node(origin, from)
            )
            .unwrap();
        }
    }
}

/// `dumpAll` (main.cpp:245-249): graph, then the rank tables parsed out of the
/// supplied `.nav` bytes, then the query surface.
fn dump_all(nav: &mut Navigator, host: &mut MockHost, nav_bytes: &[u8]) -> String {
    let mut out = String::new();
    dump_graph(nav, &mut out);
    dump_ranks_from_bytes(nav, nav_bytes, &mut out);
    dump_queries(nav, host, &mut out);
    out
}

/// The four committed fixtures (`layouts/*.layout` ↔ `fixtures/*.nav` ↔
/// `goldens/*.txt`), in the order the README documents them.
const FIXTURES: [&str; 4] = ["line3", "diamond", "star6", "grid9"];

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Build → `Save` → `dumpAll` reproduces every committed golden byte-for-byte.
#[test]
fn matches_oracle_goldens() {
    let root = oracle_root();
    let mut checked = 0;
    for name in FIXTURES {
        let lay = parse_layout(&root.join("layouts").join(format!("{name}.layout")));
        let golden_path = root.join("goldens").join(format!("{name}.txt"));
        let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
            panic!("missing golden {golden_path:?} — run tools/npcnav-oracle/build.sh --regen")
        });

        let (mut nav, mut host, _key, bytes) = build_and_save(&lay, name);
        assert_eq!(
            dump_all(&mut nav, &mut host, &bytes),
            golden,
            "fixture {name} diverges from the C++ npcnav oracle golden"
        );
        checked += 1;
    }
    assert_eq!(checked, FIXTURES.len(), "expected the full fixture set");
}

/// `Save` emits bytes identical to the committed retail-shaped `.nav` fixture —
/// the 4-byte-`long` shim / deterministic-padding witness (NAV-D1 / RULING 44).
#[test]
fn save_matches_nav_fixtures() {
    let root = oracle_root();
    for name in FIXTURES {
        let lay = parse_layout(&root.join("layouts").join(format!("{name}.layout")));
        let fixture_path = root.join("fixtures").join(format!("{name}.nav"));
        let fixture = std::fs::read(&fixture_path)
            .unwrap_or_else(|_| panic!("missing .nav fixture {fixture_path:?}"));

        let (_nav, _host, _key, bytes) = build_and_save(&lay, name);
        assert_eq!(
            bytes, fixture,
            "fixture {name}: Save bytes diverge from the committed .nav (long-width / padding)"
        );
    }
}

/// `Load` the committed `.nav` fixture back through the mock's FS and re-emit
/// the full dump — the oracle's own Save/Load round-trip witness
/// (`main.cpp:333-343`): the query surface + rank tables reconstructed from the
/// retail bytes match the golden exactly.
#[test]
fn load_roundtrip_matches_goldens() {
    let root = oracle_root();
    for name in FIXTURES {
        let lay = parse_layout(&root.join("layouts").join(format!("{name}.layout")));
        let golden_path = root.join("goldens").join(format!("{name}.txt"));
        let golden = std::fs::read_to_string(&golden_path)
            .unwrap_or_else(|_| panic!("missing golden {golden_path:?}"));
        let fixture = std::fs::read(root.join("fixtures").join(format!("{name}.nav")))
            .unwrap_or_else(|_| panic!("missing .nav fixture for {name}"));

        // Discover the qpath `Save` uses (Load reads the same "maps/<name>.nav"),
        // then serve the committed fixture from the mock's FS under that key.
        let (_nav, _host, key, _bytes) = build_and_save(&lay, name);
        let mut host = MockHost::new();
        host.files.insert(key, fixture.clone());

        let mut nav = Navigator::default();
        nav.init();
        assert!(
            nav.load(&mut host, name, lay.checksum),
            "Load failed for {name}"
        );
        assert_eq!(
            dump_all(&mut nav, &mut host, &fixture),
            golden,
            "fixture {name}: Load round-trip diverges from the golden"
        );
    }
}
