// roff-oracle — fixture generator (RULING 14 / ROFF-D4: hand-authored minimal
// .rof binaries, NO retail data). Emits the on-disk v1/v2 ROFF layouts exactly as
// the 32-bit-Windows retail exporter wrote them: char[4] mHeader (NO NUL), a
// fixed 4-byte mVersion, then counts and the move/rotate entries. Every field and
// offset is spelled out below with its RoffSystem.h cite. The unmodified
// IsROFF/InitROFF/InitROFF2 consume these through the stubbed FS.
//
// On-disk widths (RoffSystem.h:54-89): v1 header = 12 bytes (char[4] + long(4) +
// float(4)); v1 entry = 24 bytes (6 floats). v2 header = 20 bytes (char[4] +
// long(4) + 3*int); v2 entry = 32 bytes (6 floats + 2 ints). This host is
// little-endian (as was the exporter), so ints/floats are written raw.
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>
#include <array>

struct Buf {
	std::vector<uint8_t> b;
	void ensure(size_t n) { if (b.size() < n) b.resize(n, 0); }
	void raw(size_t off, const void *p, size_t n) { ensure(off + n); memcpy(&b[off], p, n); }
	void i32(size_t off, int32_t v)  { raw(off, &v, 4); }
	void f32(size_t off, float v)    { raw(off, &v, 4); }
	// mHeader: exactly 4 bytes, NO NUL terminator (ROFF-V1 relies on this).
	void hdr4(size_t off, const char *s) { ensure(off + 4); memcpy(&b[off], s, 4); }
	void write(const std::string &path) {
		FILE *f = fopen(path.c_str(), "wb");
		if (!f) { fprintf(stderr, "roffgen: cannot write %s\n", path.c_str()); exit(1); }
		fwrite(b.data(), 1, b.size(), f);
		fclose(f);
		printf("  %-28s %zu bytes\n", path.c_str(), b.size());
	}
};

// --- version-1 .rof (RoffSystem.h:54-67) -------------------------------------
// entries: {ox,oy,oz, rx,ry,rz} triples, count of them.
static void gen_v1(const std::string &path, int version, float count,
                   const std::vector<std::array<float,6>> &entries) {
	Buf b;
	b.hdr4(0, "ROFF");            // mHeader (h:58)
	b.i32(4, version);            // mVersion — fixed 4-byte on disk (h:59, ROFF-D4)
	b.f32(8, count);              // mCount   — float (h:60)
	size_t off = 12;              // entries start after the 12-byte header
	for (auto &e : entries) {
		for (int k = 0; k < 6; k++) b.f32(off + k * 4, e[k]);
		off += 24;                // TROFFEntry = 24 bytes (h:63-66)
	}
	b.write(path);
}

// --- version-2 .rof (RoffSystem.h:69-89) -------------------------------------
// entry adds mStartNote/mNumNotes; note tracks are packed NUL-terminated strings
// after the entries (InitROFF2 :214-237).
struct V2Entry { float o[3]; float r[3]; int startNote; int numNotes; };
static void gen_v2(const std::string &path, int count, int frameRate, int numNotes,
                   const std::vector<V2Entry> &entries,
                   const std::vector<std::string> &notes) {
	Buf b;
	b.hdr4(0, "ROFF");            // mHeader (h:72)
	b.i32(4, 2);                  // mVersion = ROFF_NEW_VERSION, 4-byte (h:73)
	b.i32(8, count);              // mCount int (h:74)
	b.i32(12, frameRate);         // mFrameRate (h:75)
	b.i32(16, numNotes);          // mNumNotes  (h:76)
	size_t off = 20;              // v2 header = 20 bytes
	for (auto &e : entries) {
		for (int k = 0; k < 3; k++) b.f32(off + k * 4, e.o[k]);
		for (int k = 0; k < 3; k++) b.f32(off + 12 + k * 4, e.r[k]);
		b.i32(off + 24, e.startNote);
		b.i32(off + 28, e.numNotes);
		off += 32;                // TROFF2Entry = 32 bytes (h:82-86)
	}
	// packed NUL-terminated note-track strings
	for (auto &s : notes) { b.raw(off, s.c_str(), s.size() + 1); off += s.size() + 1; }
	b.write(path);
}

int main() {
	using A6 = std::array<float,6>;

	// 1) v1_basic: 3 entries, plain small offsets/rotations (Golden A parse +
	//    Golden B non-translated / translated playback).
	gen_v1("fixtures/v1_basic.rof", 1, 3.0f, {
		A6{ 10.0f, 0.0f,  0.0f,   0.0f,  0.0f,  0.0f },
		A6{ 20.0f, 5.0f,  0.0f,   0.0f, 45.0f,  0.0f },
		A6{ 30.0f, 5.0f, 15.0f,  10.0f, 90.0f, 20.0f },
	});

	// 2) v1_badangle: rotate components outside [-180,180] to exercise
	//    FixBadAngles (>180 -= 360; < -180 += 360).
	gen_v1("fixtures/v1_badangle.rof", 1, 2.0f, {
		A6{ 0.0f, 0.0f, 0.0f,   270.0f, -200.0f,  45.0f },   // -> -90, 160, 45
		A6{ 1.0f, 2.0f, 3.0f,   181.0f, -181.0f, 179.0f },   // -> -179, 179, 179
	});

	// 3) v2_notes: 2 entries, frameRate 50 (frameTime=50, lerp=1000/50=20), one
	//    note track fired by entry 0.
	gen_v2("fixtures/v2_notes.rof", 2, 50, 1, {
		V2Entry{ { 5.0f, 0.0f, 0.0f }, { 0.0f, 10.0f, 0.0f }, 0, 1 },
		V2Entry{ { 6.0f, 0.0f, 0.0f }, { 0.0f, 20.0f, 0.0f }, -1, 0 },
	}, { "hello" });

	// 4) fallback: a valid v1 roff placed at scripts/<name>.rof so Cache's
	//    FS_ReadFile miss -> va("scripts/%s.rof", stripped) fallback path fires.
	gen_v1("fixtures/scripts/fallbackcase.rof", 1, 1.0f, {
		A6{ 7.0f, 8.0f, 9.0f,   1.0f, 2.0f, 3.0f },
	});

	// 5) bad_version: header "ROFF" ok but version 99 -> IsROFF version reject.
	gen_v1("fixtures/bad_version.rof", 99, 1.0f, {
		A6{ 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f },
	});

	// 6) bad_count: v1 with mCount = 0 -> IsROFF count reject (version 1, count<=0).
	gen_v1("fixtures/bad_count.rof", 1, 0.0f, {});

	return 0;
}
