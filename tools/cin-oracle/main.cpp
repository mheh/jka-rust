// cin-oracle: the scenario driver over the unmodified Raven RoQ decode core.
//
//   cin_dump <scenario>
//
// Every gated function is `static` in `cl_cin.cpp`, so the driver compiles the
// oracle text into its own translation unit and calls the statics directly.
// The driver writes the canonical text dump to stdout. README.md holds the
// scenario list and the coverage boundary.
//
// `RoQInterrupt` itself is outside the byte gate: its body is file I/O, console
// printing, loop and EOF control, and the `S_RawSamples` hand-off. The driver
// replicates its chunk-dispatch switch over an in-memory fixture instead, and
// `crates/mp/engine/client/tests/cin_oracle_goldens.rs` replicates the same
// switch on the Rust side, so the two drivers stay symmetric.
//
// No address is ever printed. `cin.qStatus` entries dump as signed byte offsets
// from `cin.linbuf`, and `cin.mcomp` entries dump as `(int)`, the 32-bit
// reinterpretation that Raven's pointer arithmetic actually means.
#include "codemp/client/cl_cin.cpp"
#include "host.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>

// --- digests ----------------------------------------------------------------

// FNV-1a, fed value-derived little-endian bytes rather than raw memory, so the
// digest never depends on the host's word width or padding.
static unsigned int fnv_init(void) { return 2166136261u; }

static void fnv_byte(unsigned int *h, unsigned char b)
{
	*h ^= b;
	*h *= 16777619u;
}

static void fnv_u32(unsigned int *h, unsigned int v)
{
	fnv_byte(h, (unsigned char)(v & 0xff));
	fnv_byte(h, (unsigned char)((v >> 8) & 0xff));
	fnv_byte(h, (unsigned char)((v >> 16) & 0xff));
	fnv_byte(h, (unsigned char)((v >> 24) & 0xff));
}

static void fnv_u16(unsigned int *h, unsigned short v)
{
	fnv_byte(h, (unsigned char)(v & 0xff));
	fnv_byte(h, (unsigned char)((v >> 8) & 0xff));
}

static unsigned int fnv_bytes(const unsigned char *p, size_t n)
{
	unsigned int h = fnv_init();
	for (size_t i = 0; i < n; i++) {
		fnv_byte(&h, p[i]);
	}
	return h;
}

// Raven's `long` tables. The dump narrows each entry to 32 bits, which is the
// width the shipped build stored.
static unsigned int fnv_longs(const long *p, size_t n)
{
	unsigned int h = fnv_init();
	for (size_t i = 0; i < n; i++) {
		fnv_u32(&h, (unsigned int)(int)p[i]);
	}
	return h;
}

static unsigned int fnv_shorts(const short *p, size_t n)
{
	unsigned int h = fnv_init();
	for (size_t i = 0; i < n; i++) {
		fnv_u16(&h, (unsigned short)p[i]);
	}
	return h;
}

static unsigned int fnv_ints(const int *p, size_t n)
{
	unsigned int h = fnv_init();
	for (size_t i = 0; i < n; i++) {
		fnv_u32(&h, (unsigned int)p[i]);
	}
	return h;
}

// One codebook texel: the vq books are `unsigned short` arrays holding 32-bit
// RGBA, low half first.
static unsigned int vq_texel(const unsigned short *v, size_t i)
{
	return ((unsigned int)v[i * 2 + 1] << 16) | (unsigned int)v[i * 2];
}

static unsigned int fnv_vq(const unsigned short *v, size_t nshorts)
{
	unsigned int h = fnv_init();
	for (size_t i = 0; i < nshorts / 2; i++) {
		fnv_u32(&h, vq_texel(v, i));
	}
	return h;
}

// --- driver state -----------------------------------------------------------

#define CIN_ORACLE_MAX_STREAM (1 << 20)
static unsigned char s_stream[CIN_ORACLE_MAX_STREAM];
static int s_streamLen = 0;

// Raven's `RoQInterrupt` local `short sbuf[32768]`.
// Source: `oracle/codemp/client/cl_cin.cpp:925`
static short s_sbuf[32768];

// The RLL byte sweep the `rll_direct` scenario decodes. `RllDecodeStereoToMono`
// reads two bytes per output sample, so the buffer holds four sweeps.
static unsigned char s_rllFrom[1024];

static int s_frameIndex = 0;

// Sample indices the raw-entry lines print.
static const int kTabIdx[8] = {0, 1, 64, 85, 128, 170, 254, 255};
static const int kSqrIdx[8] = {0, 1, 64, 127, 128, 129, 192, 255};
static const int kMcompIdx[8] = {0, 1, 8, 15, 16, 136, 240, 255};
static const int kQuadIdx[8] = {0, 1, 2, 3, 4, 5, 19, 20};

// The `yuv_to_rgb24` sweep. The digest covers the whole 6x6x6 grid; the raw
// lines name the corners plus a few interior colours.
static const long kYuvGrid[6] = {0, 51, 102, 153, 204, 255};
static const long kYuvShow[10][3] = {
	{0, 0, 0}, {255, 255, 255}, {0, 255, 0}, {255, 0, 255}, {128, 128, 128},
	{16, 128, 128}, {235, 128, 128}, {81, 90, 240}, {145, 54, 34}, {41, 240, 110},
};

// --- dumps ------------------------------------------------------------------

static void dump_tables(void)
{
	printf("TABLES\n");
	printf("  yy crc %08x\n", fnv_longs(ROQ_YY_tab, 256));
	printf("  ub crc %08x\n", fnv_longs(ROQ_UB_tab, 256));
	printf("  ug crc %08x\n", fnv_longs(ROQ_UG_tab, 256));
	printf("  vg crc %08x\n", fnv_longs(ROQ_VG_tab, 256));
	printf("  vr crc %08x\n", fnv_longs(ROQ_VR_tab, 256));

	const long *tabs[5] = {ROQ_YY_tab, ROQ_UB_tab, ROQ_UG_tab, ROQ_VG_tab, ROQ_VR_tab};
	const char *names[5] = {"yy", "ub", "ug", "vg", "vr"};
	for (int t = 0; t < 5; t++) {
		printf("  %s", names[t]);
		for (int k = 0; k < 8; k++) {
			printf(" %d", (int)tabs[t][kTabIdx[k]]);
		}
		printf("\n");
	}

	printf("SQR crc %08x\n", fnv_shorts(cin.sqrTable, 256));
	printf("  sqr");
	for (int k = 0; k < 8; k++) {
		printf(" %d", (int)cin.sqrTable[kSqrIdx[k]]);
	}
	printf("\n");

	unsigned int h = fnv_init();
	for (int a = 0; a < 6; a++) {
		for (int b = 0; b < 6; b++) {
			for (int c = 0; c < 6; c++) {
				fnv_u32(&h, yuv_to_rgb24(kYuvGrid[a], kYuvGrid[b], kYuvGrid[c]));
			}
		}
	}
	printf("YUV crc %08x\n", h);
	for (int k = 0; k < 10; k++) {
		printf("  yuv %d %d %d %08x\n", (int)kYuvShow[k][0], (int)kYuvShow[k][1], (int)kYuvShow[k][2],
			yuv_to_rgb24(kYuvShow[k][0], kYuvShow[k][1], kYuvShow[k][2]));
	}
}

static void dump_quadinfo(const char *tag)
{
	const cin_cache *c = &cinTable[currentHandle];
	printf("QUADINFO %s\n", tag);
	printf("  xsize %u ysize %u maxsize %u minsize %u\n", c->xsize, c->ysize, c->maxsize, c->minsize);
	printf("  cinw %d cinh %d spl %d screendelta %d\n",
		c->CIN_WIDTH, c->CIN_HEIGHT, (int)c->samplesPerLine, (int)c->screenDelta);
	// `t[0]` and `t[1]` are address algebra that cancels to +/- screenDelta. The
	// dump narrows both to 32 bits so the value cannot depend on the compile
	// width. See README.md.
	printf("  t0 %d t1 %d drawx %d drawy %d vq0 %d vq1 %d\n",
		(int)c->t[0], (int)c->t[1], (int)c->drawX, (int)c->drawY,
		c->VQ0 != NULL, c->VQ1 != NULL);
}

// The number of `qStatus` cels `setupQuad` lays out for the current size.
// Source: `oracle/codemp/client/cl_cin.cpp:762-764`
static long quad_cel_count(void)
{
	const cin_cache *c = &cinTable[currentHandle];
	long n = ((long)c->xsize * (long)c->ysize) / 16;
	n += n / 4;
	n += 64;
	return n;
}

// A `qStatus` entry as a signed byte offset from `cin.linbuf`, never an address.
// The end-of-quad NULLs dump as -1.
static int quad_offset(int bank, long i)
{
	byte *p = cin.qStatus[bank][i];
	if (p == NULL) {
		return -1;
	}
	return (int)(p - cin.linbuf);
}

static void dump_quads(const char *tag)
{
	long cels = quad_cel_count();
	printf("QUADS %s onquad %d cels %d\n", tag, (int)cinTable[currentHandle].onQuad, (int)cels);

	for (int bank = 0; bank < 2; bank++) {
		unsigned int h = fnv_init();
		for (long i = 0; i < cels; i++) {
			fnv_u32(&h, (unsigned int)quad_offset(bank, i));
		}
		printf("  q%d crc %08x\n", bank, h);
	}
	for (int bank = 0; bank < 2; bank++) {
		printf("  q%d", bank);
		for (int k = 0; k < 8; k++) {
			printf(" %d", quad_offset(bank, kQuadIdx[k]));
		}
		printf("\n");
	}
	long last = cinTable[currentHandle].onQuad;
	printf("  qend %d %d %d %d\n",
		quad_offset(0, last - 1), quad_offset(0, last),
		quad_offset(1, last - 1), quad_offset(1, last));
}

static void dump_mcomp(const char *tag)
{
	unsigned int h = fnv_init();
	int vals[256];
	for (int i = 0; i < 256; i++) {
		// Raven stores a signed delta in an `unsigned int` and adds it to a
		// 32-bit `byte *`. The dump takes the 32-bit reinterpretation, which is
		// what the pointer arithmetic means.
		vals[i] = (int)cin.mcomp[i];
	}
	h = fnv_ints(vals, 256);
	printf("MCOMP %s crc %08x\n", tag, h);
	printf("  mcomp");
	for (int k = 0; k < 8; k++) {
		printf(" %d", vals[kMcompIdx[k]]);
	}
	printf("\n");
}

static void dump_codebook(const char *tag, unsigned short flags)
{
	long two, four;
	if (!flags) {
		two = four = 256;
	} else {
		two = flags >> 8;
		if (!two) {
			two = 256;
		}
		four = flags & 0xff;
	}
	four *= 2;

	printf("CODEBOOK %s flags %04x two %d four %d\n", tag, flags, (int)two, (int)four);
	printf("  vq2 crc %08x vq4 crc %08x vq8 crc %08x\n",
		fnv_vq(vq2, 256 * 16 * 4), fnv_vq(vq4, 256 * 64 * 4), fnv_vq(vq8, 256 * 256 * 4));

	static const size_t idx2[6] = {0, 1, 2, 3, 4, 1023};
	static const size_t idx4[6] = {0, 1, 15, 16, 17, 4095};
	static const size_t idx8[6] = {0, 1, 63, 64, 65, 16383};
	printf("  vq2");
	for (int k = 0; k < 6; k++) {
		printf(" %08x", vq_texel(vq2, idx2[k]));
	}
	printf("\n  vq4");
	for (int k = 0; k < 6; k++) {
		printf(" %08x", vq_texel(vq4, idx4[k]));
	}
	printf("\n  vq8");
	for (int k = 0; k < 6; k++) {
		printf(" %08x", vq_texel(vq8, idx8[k]));
	}
	printf("\n");
}

static void dump_frame(void)
{
	const cin_cache *c = &cinTable[currentHandle];
	size_t half = (size_t)c->screenDelta;

	printf("FRAME %d numquads %d roqf0 %d roqf1 %d nbuf0 %d bufhalf %d\n",
		s_frameIndex, (int)c->numQuads, (int)c->roqF0, (int)c->roqF1, (int)c->normalBuffer0,
		(int)((c->buf - cin.linbuf) / (long)half));
	printf("  live crc %08x\n", fnv_bytes(cin.linbuf, half * 2));
	printf("  half0 crc %08x half1 crc %08x\n",
		fnv_bytes(cin.linbuf, half), fnv_bytes(cin.linbuf + half, half));

	// An 8x8 texel grid over the half this frame decoded into, so a mismatch
	// localises to a block instead of only tripping the whole-surface digest.
	long xstep = (long)c->xsize / 8;
	long ystep = (long)c->ysize / 8;
	if (xstep < 1) {
		xstep = 1;
	}
	if (ystep < 1) {
		ystep = 1;
	}
	for (int gy = 0; gy < 8; gy++) {
		printf("  grid");
		for (int gx = 0; gx < 8; gx++) {
			const byte *p = c->buf + (gy * ystep) * c->samplesPerLine + (gx * xstep) * 4;
			unsigned int texel = (unsigned int)p[0] | ((unsigned int)p[1] << 8) |
				((unsigned int)p[2] << 16) | ((unsigned int)p[3] << 24);
			printf(" %08x", texel);
		}
		printf("\n");
	}
	s_frameIndex++;
}

static void dump_audio(const char *mode, unsigned int size, int signedOutput, unsigned short flag,
	long ret, int outShorts)
{
	printf("AUDIO %s size %u signed %d flag %04x ret %d out %d\n",
		mode, size, signedOutput, flag, (int)ret, outShorts);
	printf("  crc %08x\n", fnv_shorts(s_sbuf, (size_t)outShorts));

	int idx[8];
	idx[0] = 0;
	idx[1] = 1;
	idx[2] = 2;
	idx[3] = 3;
	idx[4] = outShorts / 2;
	idx[5] = outShorts - 3;
	idx[6] = outShorts - 2;
	idx[7] = outShorts - 1;
	printf("  s");
	for (int k = 0; k < 8; k++) {
		int i = idx[k];
		if (i < 0) {
			i = 0;
		}
		if (i >= outShorts) {
			i = outShorts - 1;
		}
		printf(" %d", (int)s_sbuf[i]);
	}
	printf("\n");
}

// --- setup ------------------------------------------------------------------

// Puts the decoder back at the state `CIN_PlayCinematic` leaves it in, minus the
// file system: a zeroed `cin` and `cinTable[0]`, zeroed codebooks and colour
// tables, and `initRoQ` run.
// Source: `oracle/codemp/client/cl_cin.cpp:1259-1293`
static void cin_oracle_reset(void)
{
	memset(&cin, 0, sizeof(cin));
	memset(cinTable, 0, sizeof(cinTable));
	memset(vq2, 0, sizeof(vq2));
	memset(vq4, 0, sizeof(vq4));
	memset(vq8, 0, sizeof(vq8));
	memset(ROQ_YY_tab, 0, sizeof(ROQ_YY_tab));
	memset(ROQ_UB_tab, 0, sizeof(ROQ_UB_tab));
	memset(ROQ_UG_tab, 0, sizeof(ROQ_UG_tab));
	memset(ROQ_VG_tab, 0, sizeof(ROQ_VG_tab));
	memset(ROQ_VR_tab, 0, sizeof(ROQ_VR_tab));
	memset(s_sbuf, 0, sizeof(s_sbuf));

	// A card that reports 2048 keeps `readQuadInfo` off the Rage Pro clamp, so
	// its `Com_Printf` never runs and the driver needs no console.
	memset(&glConfig, 0, sizeof(glConfig));
	glConfig.maxTextureSize = 2048;

	currentHandle = 0;
	cinTable[0].CIN_WIDTH = DEFAULT_CIN_WIDTH;
	cinTable[0].CIN_HEIGHT = DEFAULT_CIN_HEIGHT;
	cinTable[0].playonwalls = 1;
	s_frameIndex = 0;

	initRoQ();
}

// --- the chunk dispatch -----------------------------------------------------

// `RoQInterrupt`'s switch, minus the file I/O, the console, the loop control and
// the `S_RawSamples` hand-off. The Rust driver runs the identical switch.
// Source: `oracle/codemp/client/cl_cin.cpp:949-1008`
static void cin_oracle_dispatch(byte *framedata)
{
	cin_cache *c = &cinTable[currentHandle];

	switch (c->roq_id) {
	case ROQ_QUAD_INFO:
		if (c->numQuads == -1) {
			readQuadInfo(framedata);
			dump_quadinfo("readquadinfo");
			setupQuad(0, 0);
			dump_quads("setupquad");
		}
		if (c->numQuads != 1) {
			c->numQuads = 0;
		}
		break;

	case ROQ_CODEBOOK:
		decodeCodeBook(framedata, (unsigned short)c->roq_flags);
		dump_codebook("decodecodebook", (unsigned short)c->roq_flags);
		break;

	case ROQ_QUAD_VQ:
		if ((c->numQuads & 1)) {
			c->normalBuffer0 = c->t[1];
			RoQPrepMcomp(c->roqF0, c->roqF1);
			dump_mcomp("bank1");
			blitVQQuad32fs(cin.qStatus[1], framedata);
			c->buf = cin.linbuf + c->screenDelta;
		} else {
			c->normalBuffer0 = c->t[0];
			RoQPrepMcomp(c->roqF0, c->roqF1);
			dump_mcomp("bank0");
			blitVQQuad32fs(cin.qStatus[0], framedata);
			c->buf = cin.linbuf;
		}
		if (c->numQuads == 0) {
			memcpy(cin.linbuf + c->screenDelta, cin.linbuf, (size_t)(c->samplesPerLine * (long)c->ysize));
		}
		c->numQuads++;
		dump_frame();
		break;

	case ZA_SOUND_MONO: {
		memset(s_sbuf, 0, sizeof(s_sbuf));
		long ssize = RllDecodeMonoToStereo(framedata, s_sbuf, c->RoQFrameSize, 0, (unsigned short)c->roq_flags);
		dump_audio("mono2stereo", c->RoQFrameSize, 0, (unsigned short)c->roq_flags, ssize, (int)c->RoQFrameSize * 2);
		break;
	}

	case ZA_SOUND_STEREO: {
		memset(s_sbuf, 0, sizeof(s_sbuf));
		long ssize = RllDecodeStereoToStereo(framedata, s_sbuf, c->RoQFrameSize, 0, (unsigned short)c->roq_flags);
		dump_audio("stereo2stereo", c->RoQFrameSize, 0, (unsigned short)c->roq_flags, ssize, (int)c->RoQFrameSize);
		break;
	}

	default:
		fprintf(stderr, "cin-oracle: the fixture holds unhandled chunk id %04x\n", c->roq_id);
		exit(1);
	}
}

// Walks one fixture stream. The header parse mirrors `RoQ_init`, which reads the
// file header and the first chunk header out of `cin.file` and leaves
// `roqF0`/`roqF1` at zero. Every later chunk header is parsed the way
// `RoQInterrupt`'s tail does, out of the eight bytes that trail the payload.
// Source: `oracle/codemp/client/cl_cin.cpp:1026-1030,1062-1083`
static void cin_oracle_run_stream(const char *name)
{
	cin_cache *c = &cinTable[currentHandle];

	c->roqFPS = s_stream[6] + s_stream[7] * 256;
	if (!c->roqFPS) {
		c->roqFPS = 30;
	}
	c->numQuads = -1;
	c->roq_id = s_stream[8] + s_stream[9] * 256;
	c->RoQFrameSize = s_stream[10] + s_stream[11] * 256 + s_stream[12] * 65536;
	c->roq_flags = s_stream[14] + s_stream[15] * 256;

	printf("STREAM %s bytes %d fps %d roqid %04x\n", name, s_streamLen, (int)c->roqFPS, c->roq_id);

	int pos = 16;
	int chunk = 0;
	while (1) {
		// `Sys_StreamedRead(cin.file, RoQFrameSize+8, 1, iFile)`: the payload
		// plus the next chunk header land in `cin.file` together.
		memset(cin.file, 0, sizeof(cin.file));
		int want = (int)c->RoQFrameSize + 8;
		if (pos + want > s_streamLen) {
			fprintf(stderr, "cin-oracle: %s runs past the end of the stream at chunk %d\n", name, chunk);
			exit(1);
		}
		memcpy(cin.file, s_stream + pos, (size_t)want);
		byte *framedata = cin.file;

		printf("CHUNK %d id %04x size %u flags %04x f0 %d f1 %d\n",
			chunk, c->roq_id, c->RoQFrameSize, (unsigned short)c->roq_flags, (int)c->roqF0, (int)c->roqF1);
		cin_oracle_dispatch(framedata);

		pos += want;
		byte *hdr = framedata + c->RoQFrameSize;
		c->roq_id = hdr[0] + hdr[1] * 256;
		c->RoQFrameSize = hdr[2] + hdr[3] * 256 + hdr[4] * 65536;
		c->roq_flags = hdr[6] + hdr[7] * 256;
		c->roqF0 = (char)hdr[7];
		c->roqF1 = (char)hdr[6];
		chunk++;

		// The generator ends every fixture with an all-zero terminator chunk.
		if (c->roq_id == 0) {
			break;
		}
	}
	printf("STREAMEND chunks %d frames %d\n", chunk, s_frameIndex);
}

// --- the RLL scenario -------------------------------------------------------

// The four `RllDecode*` entry points over a deterministic byte sweep.
// `RoQInterrupt` never reaches `RllDecodeMonoToMono` or `RllDecodeStereoToMono`,
// so this scenario is the only cover they get.
// Source: `oracle/codemp/client/cl_cin.cpp:184-305`
static void cin_oracle_run_rll(void)
{
	static const unsigned short kFlags[5] = {0x0000, 0x8000, 0x1234, 0xff00, 0x00ff};

	for (int i = 0; i < 1024; i++) {
		s_rllFrom[i] = (unsigned char)(i & 0xff);
	}
	printf("RLL sweep crc %08x\n", fnv_bytes(s_rllFrom, sizeof(s_rllFrom)));

	for (int f = 0; f < 5; f++) {
		for (int sgn = 0; sgn < 2; sgn++) {
			unsigned short flag = kFlags[f];
			char signedOutput = (char)sgn;
			long ret;

			memset(s_sbuf, 0, sizeof(s_sbuf));
			ret = RllDecodeMonoToMono(s_rllFrom, s_sbuf, 256, signedOutput, flag);
			dump_audio("mono2mono", 256, sgn, flag, ret, 256);

			memset(s_sbuf, 0, sizeof(s_sbuf));
			ret = RllDecodeMonoToStereo(s_rllFrom, s_sbuf, 256, signedOutput, flag);
			dump_audio("mono2stereo", 256, sgn, flag, ret, 512);

			memset(s_sbuf, 0, sizeof(s_sbuf));
			ret = RllDecodeStereoToStereo(s_rllFrom, s_sbuf, 256, signedOutput, flag);
			dump_audio("stereo2stereo", 256, sgn, flag, ret, 256);

			memset(s_sbuf, 0, sizeof(s_sbuf));
			ret = RllDecodeStereoToMono(s_rllFrom, s_sbuf, 128, signedOutput, flag);
			dump_audio("stereo2mono", 128, sgn, flag, ret, 128);
		}
	}
}

// --- main -------------------------------------------------------------------

static void cin_oracle_load(const char *name)
{
	char path[512];
	snprintf(path, sizeof(path), "fixtures/%s.roq", name);
	FILE *f = fopen(path, "rb");
	if (!f) {
		fprintf(stderr, "cin-oracle: cannot open %s\n", path);
		exit(2);
	}
	s_streamLen = (int)fread(s_stream, 1, sizeof(s_stream), f);
	fclose(f);
	if (s_streamLen < 16) {
		fprintf(stderr, "cin-oracle: %s is shorter than a RoQ header\n", path);
		exit(2);
	}
}

int main(int argc, char **argv)
{
	if (argc != 2) {
		fprintf(stderr, "usage: %s <scenario>\n", argv[0]);
		return 2;
	}
	const char *name = argv[1];

	printf("== cin-oracle %s ==\n", name);
	cin_oracle_reset();
	dump_tables();

	if (strcmp(name, "rll_direct") == 0) {
		cin_oracle_run_rll();
	} else {
		cin_oracle_load(name);
		cin_oracle_run_stream(name);
	}

	printf("== end ==\n");
	return 0;
}
