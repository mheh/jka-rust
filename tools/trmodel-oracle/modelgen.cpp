// trmodel-oracle — fixture generator (ruling 14: hand-authored minimal binaries,
// NO retail data). Emits minimal-but-valid mdxm/mdxa byte images that the
// unmodified ServerLoadMDXM/ServerLoadMDXA consume through the stubbed FS. Every
// header field, ident/version, and offset chain is spelled out below with its
// mdx_format.h cite. ofsEnd == exact file length on every image, so the morph'd
// disk buffer's zone size equals iAllocSize (Z_MemSize parity, TRM-D3).
//
// Under the shipped x86 dedicated build (_M_IX86 defined, TRM-D3) the per-frame /
// per-vertex / per-triangle / per-bone swap walks compile OUT, so the glm needs
// no vertex/triangle/bone bodies — only the header, surf-hierarchy, LOD, and
// surface-header structs the intel-live middle section walks (:880-991).
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

#define MDXM_IDENT (('M'<<24)+('G'<<16)+('L'<<8)+'2')   // mdx_format.h:20
#define MDXA_IDENT (('A'<<24)+('G'<<16)+('L'<<8)+'2')   // mdx_format.h:21
#define VERSION_OK 6                                     // MDX?_VERSION, mdx_format.h:28-29
#define MAX_QPATH  64

struct Buf {
	std::vector<uint8_t> b;
	void ensure(size_t n) { if (b.size() < n) b.resize(n, 0); }
	void i32(size_t off, int32_t v)  { ensure(off + 4); memcpy(&b[off], &v, 4); }
	void u32(size_t off, uint32_t v) { ensure(off + 4); memcpy(&b[off], &v, 4); }
	void f32(size_t off, float v)    { ensure(off + 4); memcpy(&b[off], &v, 4); }
	void str(size_t off, const char *s, size_t cap) { ensure(off + cap); memset(&b[off], 0, cap); strncpy((char*)&b[off], s, cap - 1); }
	void write(const std::string &path) {
		FILE *f = fopen(path.c_str(), "wb");
		if (!f) { fprintf(stderr, "modelgen: cannot write %s\n", path.c_str()); exit(1); }
		fwrite(b.data(), 1, b.size(), f);
		fclose(f);
		printf("  %-28s %zu bytes\n", path.c_str(), b.size());
	}
};

// --- mdxaHeader_t (100 bytes) — mdx_format.h:351-371 --------------------------
static void gen_gla(const std::string &path, const char *name, int version) {
	Buf m;
	m.i32(0,  MDXA_IDENT);        // ident
	m.i32(4,  version);           // version
	m.str(8,  name, MAX_QPATH);   // name[64]        @8
	m.f32(72, 1.0f);              // fScale          @72
	m.i32(76, 1);                 // numFrames (>=1) @76
	m.i32(80, 100);               // ofsFrames       @80
	m.i32(84, 1);                 // numBones        @84
	m.i32(88, 100);               // ofsCompBonePool @88
	m.i32(92, 100);               // ofsSkel         @92
	m.i32(96, 100);               // ofsEnd == length@96
	m.ensure(100);
	m.write(path);
}

// --- mdxmHeader_t (164) + hierarchy + 1 LOD + 1 surface — mdx_format.h:153-243 -
static void gen_glm(const std::string &path, const char *name, const char *animName,
                    int version, bool full_body) {
	Buf m;
	m.i32(0,   MDXM_IDENT);          // ident
	m.i32(4,   version);             // version
	m.str(8,   name, MAX_QPATH);     // name[64]            @8
	m.str(72,  animName, MAX_QPATH); // animName[64]        @72  (-> "%s.gla")
	m.i32(136, 0);                   // animIndex (game-filled) @136
	m.i32(140, 1);                   // numBones            @140
	m.i32(144, 1);                   // numLODs             @144
	m.i32(148, 312);                 // ofsLODs             @148
	m.i32(152, 1);                   // numSurfaces         @152
	m.i32(156, 168);                 // ofsSurfHierarchy    @156
	if (!full_body) {                // version-reject fixture: header only
		m.i32(160, 164);             // ofsEnd == length
		m.ensure(164);
		m.write(path);
		return;
	}
	m.i32(160, 360);                 // ofsEnd == length    @160

	// mdxmHierarchyOffsets_t @164 (offsets[numSurfaces]; loader ignores it, but a
	// real carcass file carries it — realism only).
	m.i32(164, 168);

	// mdxmSurfHierarchy_t[0] @168 (stride 144 = childIndexes[numChildren=0]).
	m.str(168, "test_surf", MAX_QPATH);        // name[64]     @168
	m.u32(232, 0);                             // flags        @232
	m.str(236, "models/test_shader", MAX_QPATH);// shader[64]  @236 (StoreShaderRequest name)
	m.i32(300, 0);                             // shaderIndex  @300 (StoreShaderRequest poke)
	m.i32(304, -1);                            // parentIndex (root)
	m.i32(308, 0);                             // numChildren

	// mdxmLOD_t @312 + mdxmLODSurfOffset_t[1] @316 + mdxmSurface_t @320.
	m.i32(312, 48);                            // lod.ofsEnd = 360-312
	m.i32(316, 8);                             // lodSurfOffset[0] (loader recomputes)
	// mdxmSurface_t (40 bytes) @320:
	m.i32(320, 0);                             // ident (-> SF_MDX after load)
	m.i32(324, 0);                             // thisSurfaceIndex
	m.i32(328, -320);                          // ofsHeader (negative, back to header)
	m.i32(332, 3);                             // numVerts (<= SHADER_MAX_VERTEXES)
	m.i32(336, 40);                            // ofsVerts (nominal; not walked on x86)
	m.i32(340, 1);                             // numTriangles (*3 <= SHADER_MAX_INDEXES)
	m.i32(344, 40);                            // ofsTriangles (nominal)
	m.i32(348, 1);                             // numBoneReferences
	m.i32(352, 40);                            // ofsBoneReferences (nominal)
	m.i32(356, 40);                            // ofsEnd (next surface = end)
	m.ensure(360);
	m.write(path);
}

int main() {
	// The live pair: test.glm references skeletons/test via animName; loading the
	// glm recurses RE_RegisterServerModel("skeletons/test.gla") (:867) — the
	// glm<->gla animIndex cross-reference.
	gen_glm("fixtures/models/test.glm", "models/test.glm", "skeletons/test", VERSION_OK, true);
	gen_gla("fixtures/skeletons/test.gla", "skeletons/test", VERSION_OK);

	// A second live pair (distinct gla) for the cache eviction / DumpNonPure
	// goldens — lets survivors form a clean level-keyed / pure/non-pure split.
	gen_glm("fixtures/models/modelb.glm", "models/modelb.glm", "skeletons/test2", VERSION_OK, true);
	gen_gla("fixtures/skeletons/test2.gla", "skeletons/test2", VERSION_OK);

	// Failure fixtures: unknown ident (switch default -> fail), bad version
	// (ServerLoadMDXM version-reject).
	Buf bad;
	bad.u32(0, 0xDEADBEEFu); bad.i32(4, VERSION_OK); bad.i32(160, 164); bad.ensure(164);
	bad.write("fixtures/badident.glm");
	gen_glm("fixtures/badversion.glm", "models/badversion.glm", "skeletons/test", 99, false);
	return 0;
}
