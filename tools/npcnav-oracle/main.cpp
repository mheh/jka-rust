// npcnav-oracle — fixture generator + differential-golden dumper for the
// CNavigator (server/NPCNav) port. NAV-D3 / RULING 42.
//
// Compiles the UNMODIFIED oracle navigator.cpp TU (copied into build/ next to
// the stub headers by run.sh; oracle/ is never edited, §18). For each
// hand-authored layout under layouts/*.layout it drives the REAL build path —
// AddRawPoint -> HardConnect -> CalculatePaths -> Save — emits the binary
// `.nav` bytes to fixtures/<name>.nav, and dumps the query/rank goldens the
// same in-memory run produces to goldens/<name>.txt. The Rust port
// (crates/mp/engine-server npcnav) must reproduce both byte-for-byte.
//
// NAV-D1 / RULING 44: the TU is built with the 4-byte-`long` shim (see
// stubs/game/q_shared.h). main verifies the emitted width from the bytes.
// NAV-D2 / RULING 45: CalculatePath's priority queue is libstdc++
// std::push_heap/pop_heap; building with g++-16 makes the equal-cost tie order
// (baked into every node's rank table via curRank++ pop order) the reference.

#include "server/NPCNav/navigator.h"

// navigator.h armed `#define long int` (via the q_shared.h stub). Undo it
// IMMEDIATELY — before any of main's own includes — so <string>/<vector> and
// the file-length arithmetic below use real 64-bit longs. (The nav TU is
// already fully parsed at this point, so its 4-byte width is locked in.)
#undef long

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdarg.h>
#include <string>
#include <vector>

extern CNavigator navigator;

// svs frame clock (declared in the q_shared.h stub). Fixed; only the unexercised
// recheck-timer arms read it.
struct serverStatic_stub svs = { 0 };

// ===========================================================================
// Engine-service stubs (the harness runtime). None perturb the emitted bytes:
// the generation path uses only FS_*, va, Cvar_Get, Distance/vector math, and
// a clear SV_Trace. Trace/PVS/gentity/VM paths are compiled but not executed.
// ===========================================================================

// --- backing file table ----------------------------------------------------
static const int   MAX_FH = 8;
static FILE       *g_fh[MAX_FH + 1] = {0}; // index 0 reserved = invalid handle
static std::string g_outdir = "fixtures";

// Redirect Raven's "maps/<name>.nav" path into g_outdir/<name>.nav.
static std::string redirect(const char *qpath) {
    const char *slash = strrchr(qpath, '/');
    const char *base  = slash ? slash + 1 : qpath;
    return g_outdir + "/" + base;
}

extern "C" int FS_FOpenFileByMode(const char *qpath, fileHandle_t *f, fsMode_t mode) {
    std::string path = redirect(qpath);
    FILE *fp = fopen(path.c_str(), mode == FS_READ ? "rb" : "wb");
    if (!fp) { if (f) *f = 0; return -1; }
    int h = 0;
    for (int i = 1; i <= MAX_FH; i++) if (!g_fh[i]) { h = i; break; }
    if (!h) { fclose(fp); if (f) *f = 0; return -1; }
    g_fh[h] = fp;
    if (f) *f = h;
    long len = 0;
    if (mode == FS_READ) { fseek(fp, 0, SEEK_END); len = ftell(fp); fseek(fp, 0, SEEK_SET); }
    return (int)len;
}
extern "C" int  FS_Read (void *buffer, int len, fileHandle_t f) {
    if (f < 1 || f > MAX_FH || !g_fh[f]) return 0;
    return (int)fread(buffer, 1, len, g_fh[f]);
}
extern "C" int  FS_Write(const void *buffer, int len, fileHandle_t f) {
    if (f < 1 || f > MAX_FH || !g_fh[f]) return 0;
    return (int)fwrite(buffer, 1, len, g_fh[f]);
}
extern "C" void FS_FCloseFile(fileHandle_t f) {
    if (f >= 1 && f <= MAX_FH && g_fh[f]) { fclose(g_fh[f]); g_fh[f] = 0; }
}

// --- cvars: d_altRoutes / d_patched forced to 0 (pure-graph query surface) --
extern "C" cvar_t *Cvar_Get(const char *name, const char *value, int flags) {
    static cvar_t cv_altRoutes = {0, 0.0f, (char *)"0"};
    static cvar_t cv_patched   = {0, 0.0f, (char *)"0"};
    static cvar_t cv_default   = {0, 0.0f, (char *)"0"};
    if (!strcmp(name, "d_altRoutes")) return &cv_altRoutes;
    if (!strcmp(name, "d_patched"))   return &cv_patched;
    return &cv_default;
}

extern "C" void Com_Printf(const char *fmt, ...) {
    va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
extern "C" void Com_Error(int level, const char *fmt, ...) {
    va_list ap; va_start(ap, fmt);
    fprintf(stderr, "Com_Error(%d): ", level); vfprintf(stderr, fmt, ap); va_end(ap);
    exit(3);
}
extern "C" char *va(const char *format, ...) {
    static char buf[4][1024]; static int idx = 0;
    idx = (idx + 1) & 3; char *out = buf[idx];
    va_list ap; va_start(ap, format); vsnprintf(out, sizeof(buf[0]), format, ap); va_end(ap);
    return out;
}
extern "C" int Q_irand(int minVal, int maxVal) { return minVal; } // unused in gen path

// --- SV_* : clear trace (open map) so HardConnect cost == Euclidean distance
// and edge flags == EFLAG_NONE. PVS/gentity paths are 3c-surface, not executed.
void SV_Trace(trace_t *r, const vec3_t, const vec3_t, const vec3_t, const vec3_t,
              int, int, int, int, int) {
    if (r) { memset(r, 0, sizeof(*r)); r->fraction = 1.0f; r->entityNum = ENTITYNUM_NONE; }
}
qboolean SV_inPVS(const vec3_t, const vec3_t) { return qtrue; }
static sharedEntity_t g_player = {};
sharedEntity_t *SV_GentityNum(int) { return &g_player; }

// --- VM dispatch + game callbacks : no-ops (never touch the emitted bytes) --
vm_t *gvm = 0;
int VM_Call(vm_t *, int, ...) { return 0; }

// C++ linkage — the oracle declares these as plain `extern` (navigator.cpp:21-29).
qboolean GNavCallback_NAV_ClearPathToPoint(sharedEntity_t *, vec3_t, vec3_t, vec3_t, int, int) { return qtrue; }
qboolean GNavCallback_NPC_ClearLOS(sharedEntity_t *, const vec3_t) { return qtrue; }
int      GNavCallback_NAVNEW_ClearPathBetweenPoints(vec3_t, vec3_t, vec3_t, vec3_t, int, int) { return 1; }
qboolean GNavCallback_NAV_CheckNodeFailedForEnt(sharedEntity_t *, int) { return qfalse; }
qboolean GNavCallback_G_EntIsUnlockedDoor(int) { return qfalse; }
qboolean GNavCallback_G_EntIsDoor(int) { return qfalse; }
qboolean GNavCallback_G_EntIsBreakable(int) { return qfalse; }
qboolean GNavCallback_G_EntIsRemovableUsable(int) { return qfalse; }
void     GNavCallback_CP_FindCombatPointWaypoints(void) {}

// ===========================================================================
// Layout parsing
// ===========================================================================
struct LNode { float x, y, z; int flags, radius; };
struct LConn { int a, b; };
struct Layout { int checksum; std::vector<LNode> nodes; std::vector<LConn> conns; };

static bool parseLayout(const char *path, Layout &lay) {
    FILE *f = fopen(path, "r");
    if (!f) { fprintf(stderr, "cannot open layout %s\n", path); return false; }
    lay.checksum = 0;
    char line[512];
    while (fgets(line, sizeof(line), f)) {
        char *p = line; while (*p == ' ' || *p == '\t') p++;
        if (*p == '#' || *p == '\n' || *p == '\r' || *p == 0) continue;
        char kw[32];
        if (sscanf(p, "%31s", kw) != 1) continue;
        if (!strcmp(kw, "checksum")) {
            sscanf(p, "%*s %d", &lay.checksum);
        } else if (!strcmp(kw, "node")) {
            LNode n; n.flags = 0; n.radius = 0;
            sscanf(p, "%*s %f %f %f %d %d", &n.x, &n.y, &n.z, &n.flags, &n.radius);
            lay.nodes.push_back(n);
        } else if (!strcmp(kw, "connect")) {
            LConn c; if (sscanf(p, "%*s %d %d", &c.a, &c.b) == 2) lay.conns.push_back(c);
        }
    }
    fclose(f);
    return true;
}

// ===========================================================================
// Golden dump — everything the pure-graph query surface answers post-Save.
// ===========================================================================
static void dumpAll(FILE *out); // fwd

static void dumpGraph(FILE *out) {
    int n = navigator.GetNumNodes();
    fprintf(out, "== graph ==\n");
    fprintf(out, "numNodes %d\n", n);
    for (int i = 0; i < n; i++) {
        vec3_t pos; navigator.GetNodePosition(i, pos);
        fprintf(out, "node %d pos %.3f %.3f %.3f radius %d numEdges %d\n",
                i, pos[0], pos[1], pos[2], navigator.GetNodeRadius(i), navigator.GetNodeNumEdges(i));
        int ne = navigator.GetNodeNumEdges(i);
        for (int e = 0; e < ne; e++)
            fprintf(out, "  edge %d -> node %d\n", e, navigator.GetNodeEdge(i, e));
    }
}

static void dumpQueries(FILE *out) {
    int n = navigator.GetNumNodes();
    fprintf(out, "== ranks (GetPathCost s->e, all pairs) ==\n");
    for (int s = 0; s < n; s++)
        for (int e = 0; e < n; e++)
            fprintf(out, "pathcost %d %d = %u\n", s, e, navigator.GetPathCost(s, e));

    fprintf(out, "== GetBestNode (s,e,reject=NONE) ==\n");
    for (int s = 0; s < n; s++)
        for (int e = 0; e < n; e++)
            fprintf(out, "bestnode %d %d = %d\n", s, e, navigator.GetBestNode(s, e, NODE_NONE));

    fprintf(out, "== GetBestNodeAltRoute (d_altRoutes=0; s,e,reject=NONE) ==\n");
    for (int s = 0; s < n; s++)
        for (int e = 0; e < n; e++) {
            int pc = 0;
            int bn = navigator.GetBestNodeAltRoute(s, e, &pc, NODE_NONE);
            fprintf(out, "altroute %d %d = %d cost %d\n", s, e, bn, pc);
        }

    fprintf(out, "== Connected / NodesAreNeighbors ==\n");
    for (int s = 0; s < n; s++)
        for (int e = 0; e < n; e++)
            fprintf(out, "conn %d %d = %d neigh %d\n", s, e,
                    navigator.Connected(s, e) ? 1 : 0,
                    navigator.NodesAreNeighbors(s, e));

    fprintf(out, "== GetProjectedNode (origin = each node pos, from each node) ==\n");
    for (int from = 0; from < n; from++)
        for (int o = 0; o < n; o++) {
            vec3_t origin; navigator.GetNodePosition(o, origin);
            fprintf(out, "proj from %d origin@%d = %d\n", from, o,
                    navigator.GetProjectedNode(origin, from));
        }
}

// Parse the just-written .nav and print each node's raw rank array. The ranks
// are assigned in CalculatePath pop order (curRank++), so this row IS the
// direct human-readable witness of the libstdc++ heap sift / equal-cost tie
// order (NAV-D2 / RULING 45) — the binary fixture already contains it; this
// surfaces it as text so a sift-order mistake is legible, not just a byte diff.
static void dumpRanksFromFile(FILE *out, const char *navpath, int n) {
    FILE *f = fopen(navpath, "rb");
    if (!f) { fprintf(out, "== ranks == (missing %s)\n", navpath); return; }
    int32_t tmp;
    fseek(f, 12, SEEK_SET); // skip navID + checksum + numNodes
    fprintf(out, "== rank tables (per node, pop-order; the heap-sift gate) ==\n");
    for (int i = 0; i < n; i++) {
        fseek(f, 4 + 12 + 4 + 4 + 4, SEEK_CUR); // NODE id, pos, flags, ID, radius
        int32_t numEdges = 0; fread(&numEdges, 4, 1, f);
        fseek(f, (long)numEdges * 12, SEEK_CUR); // edge_t[numEdges]
        int32_t numRanks = 0; fread(&numRanks, 4, 1, f);
        fprintf(out, "node %d ranks[%d]:", i, numRanks);
        for (int r = 0; r < numRanks; r++) { fread(&tmp, 4, 1, f); fprintf(out, " %d", tmp); }
        fprintf(out, "\n");
    }
    fclose(f);
}

static const char *g_navpath_for_dump = 0;

static void dumpAll(FILE *out) {
    dumpGraph(out);
    if (g_navpath_for_dump) dumpRanksFromFile(out, g_navpath_for_dump, navigator.GetNumNodes());
    dumpQueries(out);
}

// ===========================================================================
// 4-byte-long verification: recompute the expected file size under the RETAIL
// (4-byte) width and confirm the emitted bytes match, plus that the NAV id is
// 4 bytes wide (byte[4..8] == checksum, not zero padding from an 8-byte long).
// ===========================================================================
static bool verifyLongWidth(const char *navpath, const Layout &lay, FILE *log) {
    FILE *f = fopen(navpath, "rb");
    if (!f) { fprintf(log, "  MISSING %s\n", navpath); return false; }
    fseek(f, 0, SEEK_END); long fsz = ftell(f); fseek(f, 0, SEEK_SET);
    unsigned char head[8] = {0};
    fread(head, 1, 8, f);

    // Read numEdges per node back out of the file to size the per-node blocks
    // exactly. Simpler: recompute from the layout's connectivity (bidirectional
    // HardConnect: each connect adds one edge to each endpoint).
    int n = (int)lay.nodes.size();
    std::vector<int> deg(n, 0);
    for (size_t i = 0; i < lay.conns.size(); i++) {
        // HardConnect adds an edge to both endpoints; duplicates collapse in
        // AddEdge, but hand-authored layouts avoid duplicate connects.
        deg[lay.conns[i].a]++; deg[lay.conns[i].b]++;
    }
    // failedEdge_t is 16 bytes; assert here so the 512-byte tail is exact.
    long expect = 4 /*navID*/ + 4 /*checksum*/ + 4 /*numNodes*/;
    for (int i = 0; i < n; i++) {
        expect += 4  /*NODE id*/ + 12 /*pos*/ + 4 /*flags*/ + 4 /*ID*/ + 4 /*radius*/
                + 4  /*numEdges*/ + (long)deg[i] * 12 /*edge_t*/
                + 4  /*numNodes*/ + (long)n * 4 /*ranks*/;
    }
    expect += (long)MAX_FAILED_EDGES * (long)sizeof(failedEdge_t); // 32 * 16 = 512

    unsigned int navid = head[0] | (head[1]<<8) | (head[2]<<16) | ((unsigned)head[3]<<24);
    unsigned int word1 = head[4] | (head[5]<<8) | (head[6]<<16) | ((unsigned)head[7]<<24);
    // 'JNV5' little-endian == 0x4A4E5635 ('J'<<24|'N'<<16|'V'<<8|'5').
    unsigned int JNV5 = ('J'<<24)|('N'<<16)|('V'<<8)|'5';

    bool ok = (fsz == expect) && (navid == JNV5) && ((int)word1 == lay.checksum);
    fprintf(log, "  navid=0x%08X (expect JNV5=0x%08X) word@4=%d (checksum=%d) size=%ld expect=%ld sizeof(failedEdge_t)=%zu -> %s\n",
            navid, JNV5, (int)word1, lay.checksum, fsz, expect, sizeof(failedEdge_t), ok ? "OK (4-byte long)" : "MISMATCH");
    fclose(f);
    return ok;
}

int main(int argc, char **argv) {
    if (argc < 3) { fprintf(stderr, "usage: %s <name> <layout.path> [outdir]\n", argv[0]); return 2; }
    const char *name   = argv[1];
    const char *lpath  = argv[2];
    if (argc >= 4) g_outdir = argv[3];

    Layout lay;
    if (!parseLayout(lpath, lay)) return 2;

    // Drive the REAL build path on the file-scope global `navigator`.
    navigator.Init();
    for (size_t i = 0; i < lay.nodes.size(); i++) {
        vec3_t p = { lay.nodes[i].x, lay.nodes[i].y, lay.nodes[i].z };
        navigator.AddRawPoint(p, lay.nodes[i].flags, lay.nodes[i].radius);
    }
    for (size_t i = 0; i < lay.conns.size(); i++)
        navigator.HardConnect(lay.conns[i].a, lay.conns[i].b);
    navigator.CalculatePaths();

    // Emit the binary fixture through the real Save path (FS_Write, 4-byte long).
    if (!navigator.Save(name, lay.checksum)) { fprintf(stderr, "Save failed\n"); return 3; }
    std::string navpath = g_outdir + "/" + name + ".nav";
    g_navpath_for_dump = navpath.c_str();

    // Verify the 4-byte-long property from the emitted bytes.
    fprintf(stderr, "[%s] long-width check:\n", name);
    if (!verifyLongWidth(navpath.c_str(), lay, stderr)) {
        fprintf(stderr, "FATAL: 4-byte-long property violated\n"); return 4;
    }

    // Dump the goldens from the in-memory graph (ranks == what Save wrote ==
    // what Rust Load reads back) into a memory buffer.
    char  *bufA = 0; size_t szA = 0;
    FILE  *msA  = open_memstream(&bufA, &szA);
    dumpAll(msA); fclose(msA);

    // Self-check: reload the just-written fixture and confirm the query surface
    // is identical — proves Save wrote and Load read the same 4-byte-shaped
    // layout in-process (an independent second witness to the long-width shim).
    navigator.Init();
    if (!navigator.Load(name, lay.checksum)) { fprintf(stderr, "reload Load failed\n"); return 5; }
    char  *bufB = 0; size_t szB = 0;
    FILE  *msB  = open_memstream(&bufB, &szB);
    dumpAll(msB); fclose(msB);

    if (szA != szB || memcmp(bufA, bufB, szA) != 0) {
        fprintf(stderr, "FATAL: Save/Load round-trip query mismatch for %s\n", name);
        return 6;
    }
    fprintf(stderr, "[%s] Save/Load round-trip: OK\n", name);

    fwrite(bufA, 1, szA, stdout);
    free(bufA); free(bufB);
    return 0;
}
