// q_math differential-oracle dumper. Compiled against the UNMODIFIED Raven
// q_math.c (copied into build/ by run.sh) plus raven_rng.c (Raven's LCG
// extracted verbatim, holdrand width normalized to the 32-bit i686 ship
// target — see run.sh / README). Prints a canonical bit-exact dump the Rust
// parity test (crates/mp/game/tests/jampgame_parity.rs) reproduces via the
// ported crate::q_math + bg_channel::rng::Rng.
#include "q_shared.h"
#include "dumpcommon.h"

// Raven's LCG, extracted+renamed by run.sh (r_ prefix, 32-bit holdrand).
void  r_Rand_Init(int seed);
float r_flrand(float min, float max);
float r_Q_flrand(float min, float max);
int   r_irand(int min, int max);
int   r_Q_irand(int value1, int value2);
float r_Q_rsqrt(float number); // 32-bit-normalized (see run.sh)

// q_math.c functions not prototyped in q_shared.h (declared in g_local.h).
float    DotProductNormalize(const vec3_t inVec1, const vec3_t inVec2);
qboolean G_FindClosestPointOnLineSegment(const vec3_t start, const vec3_t end, const vec3_t from, vec3_t result);
float    G_PointDistFromLineSegment(const vec3_t start, const vec3_t end, const vec3_t from);

static float bf(unsigned u) { union { unsigned u; float f; } x; x.u = u; return x.f; }

// ProjectPointOnPlane / PerpendicularVector / RotatePointAroundVector divide by
// dot(normal,normal) and carry a debug assert against a zero divisor (mirrored
// by the port's debug_assert). Skip degenerate normals (zero, or denormals that
// underflow the squared sum to 0) so neither the oracle nor the debug-mode Rust
// test aborts — the divide-by-zero path is UB (porting-rules S19).
static int dot_nz(const float *v) { return (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]) != 0.0f; }

// -------- fixture: vec3 table (hex bit patterns, one vec per line) --------
#define MAXV 256
static vec3_t g_vec[MAXV];
static int    g_nvec;

static void load_vectors(const char *dir) {
	char path[1024];
	snprintf(path, sizeof(path), "%s/vectors.txt", dir);
	FILE *f = fopen(path, "rb");
	if (!f) { fprintf(stderr, "cannot open %s\n", path); exit(2); }
	unsigned x, y, z;
	while (fscanf(f, "%x %x %x", &x, &y, &z) == 3) {
		g_vec[g_nvec][0] = bf(x);
		g_vec[g_nvec][1] = bf(y);
		g_vec[g_nvec][2] = bf(z);
		g_nvec++;
	}
	fclose(f);
}

static void dump_rng(void) {
	static const int seeds[] = { (int)0x89abcdef, 0, 1, (int)0xdeadbeef };
	printf("== rng ==\n");
	for (int s = 0; s < 4; s++) {
		printf("seed %08x\n", (unsigned)seeds[s]);
		r_Rand_Init(seeds[s]);
		for (int i = 0; i < 100; i++) {
			printf("fl %08x\n", f2b(r_flrand(0.0f, 1.0f)));
			printf("fl %08x\n", f2b(r_flrand(-1.0f, 1.0f)));
			printf("fl %08x\n", f2b(r_flrand(-100.0f, 100.0f)));
			printf("qf %08x\n", f2b(r_Q_flrand(0.0f, 1000.0f)));
			printf("ir %d\n", r_irand(0, 1));
			printf("ir %d\n", r_irand(0, 100));
			printf("ir %d\n", r_irand(-50, 50));
			printf("qi %d\n", r_Q_irand(0, 32767));
		}
	}
}

static void dump_qrand(void) {
	printf("== qrand ==\n");
	int seed;
	seed = 12345;
	for (int i = 0; i < 30; i++) printf("qr %d\n", Q_rand(&seed));
	seed = 12345;
	for (int i = 0; i < 30; i++) printf("qrn %08x\n", f2b(Q_random(&seed)));
	seed = 12345;
	for (int i = 0; i < 30; i++) printf("qcr %08x\n", f2b(Q_crandom(&seed)));
	seed = -1;
	for (int i = 0; i < 20; i++) printf("qr %d\n", Q_rand(&seed));
}

static void dump_scalars(void) {
	printf("== scalars ==\n");
	static const int ci[] = { -300, -128, -1, 0, 1, 127, 128, 200, -32769, -32768,
	                          32767, 32768, 100000, -100000, 1, 2, 3, 255, 256, 1023,
	                          1024, 0x7fffffff };
	for (unsigned i = 0; i < sizeof(ci)/sizeof(ci[0]); i++)
		printf("cc %d cs %d log2 %d\n", ClampChar(ci[i]), ClampShort(ci[i]), Q_log2(ci[i] < 0 ? -ci[i] : ci[i]));

	// Q_rsqrt / Q_fabs / powf over a bit-pattern float table.
	static const unsigned cf[] = { 0x3f800000, 0x40490fdb, 0x00000001, 0x80000000,
	                               0x7f7fffff, 0x3dcccccd, 0xc0000000, 0x42c80000,
	                               0x00800000, 0xbf000000 };
	for (unsigned i = 0; i < sizeof(cf)/sizeof(cf[0]); i++) {
		float v = bf(cf[i]);
		printf("rsqrt %08x fabs %08x\n", f2b(r_Q_rsqrt(v < 0 ? -v : v)), f2b(Q_fabs(v)));
	}
	for (int y = 1; y <= 6; y++)
		printf("powf %08x\n", f2b(powf(1.5f, y)));

	// ByteToDir over valid + out-of-range indices.
	for (int b = 0; b < 162; b += 17) {
		vec3_t d; ByteToDir(b, d); printf("b2d %d ", b); PV3(d);
	}
	{ vec3_t d; ByteToDir(-1, d); printf("b2d -1 "); PV3(d); }
	{ vec3_t d; ByteToDir(500, d); printf("b2d 500 "); PV3(d); }

	// ColorBytes / NormalizeColor over a few bit-pattern colors.
	// ColorBytes r/g/b are kept in [0,1]: float->byte of an out-of-range value
	// (r*255 < 0 or > 255) is C UB and diverges from the port's saturating cast.
	static const unsigned cc[][3] = {
		{0x3f000000,0x3e800000,0x3f800000}, {0x00000000,0x00000000,0x00000000},
		{0x3f800000,0x3f800000,0x3f800000}, {0x3e4ccccd,0x3f19999a,0x3f733333},
	};
	for (unsigned i = 0; i < sizeof(cc)/sizeof(cc[0]); i++) {
		float r = bf(cc[i][0]), g = bf(cc[i][1]), b = bf(cc[i][2]);
		// ColorBytes3 writes only bytes [0..2] of an uninitialized `unsigned i`;
		// byte [3] is indeterminate stack garbage (Raven UB). Mask it off so the
		// golden is deterministic and matches the port (which zeroes byte 3).
		printf("cb3 %08x cb4 %08x\n", ColorBytes3(r,g,b) & 0x00ffffffu, ColorBytes4(r,g,b,0.5f));
		vec3_t in = {r,g,b}, out; float m = NormalizeColor(in, out);
		printf("ncol %08x ", f2b(m)); PV3(out);
	}
}

// A handful of hand-picked SAFE unit-ish normals for NormalToLatLong so
// acos()'s argument stays in [-1,1] (acos of a non-normalized z is NaN, and
// C's (int)NaN vs Rust's `as i32` diverge — kept out of the shared fixtures
// per porting-rules S19).
static void dump_normaltolatlong(void) {
	printf("== n2ll ==\n");
	static const unsigned N[][3] = {
		{0x3f800000,0x00000000,0x00000000}, // (1,0,0)
		{0x00000000,0x3f800000,0x00000000}, // (0,1,0)
		{0x00000000,0x00000000,0x3f800000}, // (0,0,1) singularity
		{0x00000000,0x00000000,0xbf800000}, // (0,0,-1) singularity
		{0x3f0f5c29,0x3f0f5c29,0x3ef1a9fc}, // ~(0.56,0.56,0.47) normalized-ish
		{0xbf13cd36,0x3f13cd36,0x00000000}, // (-0.577,0.577,0)-ish
	};
	for (unsigned i = 0; i < sizeof(N)/sizeof(N[0]); i++) {
		vec3_t n = { bf(N[i][0]), bf(N[i][1]), bf(N[i][2]) };
		VectorNormalize(n); // keep |z|<=1
		byte b[2] = {0,0};
		NormalToLatLong(n, b);
		printf("n2ll %02x%02x\n", b[0], b[1]);
	}
}

static void dump_planes(void) {
	printf("== planes ==\n");
	for (int i = 0; i < g_nvec; i++) {
		float *n = g_vec[i];
		for (int ty = 0; ty <= 3; ty++) {
			cplane_t p;
			memset(&p, 0, sizeof(p));
			p.normal[0] = n[0]; p.normal[1] = n[1]; p.normal[2] = n[2];
			p.dist = n[0];
			p.type = ty;
			SetPlaneSignbits(&p);
			int j = (i + 1) % g_nvec, k = (i + 2) % g_nvec;
			int side = BoxOnPlaneSide(g_vec[j], g_vec[k], &p);
			printf("plane %d ty %d sb %d side %d\n", i, ty, p.signbits, side);
		}
	}
}

static void dump_vecmath(void) {
	printf("== vecmath ==\n");
	for (int i = 0; i < g_nvec; i++) {
		int j = (i + 1) % g_nvec, k = (i + 2) % g_nvec;
		float *a = g_vec[i], *b = g_vec[j], *c = g_vec[k];
		vec3_t t, u, w;

		printf("i %d\n", i);

		{ vec3_t f, r, up; AngleVectors(a, f, r, up);
		  printf("av "); printf("%08x %08x %08x %08x %08x %08x %08x %08x %08x\n",
		    f2b(f[0]),f2b(f[1]),f2b(f[2]),f2b(r[0]),f2b(r[1]),f2b(r[2]),f2b(up[0]),f2b(up[1]),f2b(up[2])); }

		{ vec3_t ang; vectoangles(a, ang); printf("va "); PV3(ang); }

		{ VectorCopy(a, t); float len = VectorNormalize(t);
		  printf("vn %08x ", f2b(len)); PV3(t); }
		{ float len = VectorNormalize2(a, t); printf("vn2 %08x ", f2b(len)); PV3(t); }

		printf("vl %08x vls %08x\n", f2b(VectorLength(a)), f2b(VectorLengthSquared(a)));
		printf("dist %08x dsq %08x\n", f2b(Distance(a,b)), f2b(DistanceSquared(a,b)));
		printf("dh %08x dhs %08x\n", f2b(DistanceHorizontal(a,b)), f2b(DistanceHorizontalSquared(a,b)));
		printf("vcmp %d %d\n", VectorCompare(a,b), VectorCompare(a,a));

		CrossProduct(a, b, t); printf("cross "); PV3(t);
		printf("dot %08x _dot %08x\n", f2b(DotProduct(a,b)), f2b(_DotProduct(a,b)));
		printf("dpn %08x\n", f2b(DotProductNormalize(a,b)));

		VectorCopy(a, t); VectorInverse(t); printf("vinv "); PV3(t);
		_VectorMA(a, 2.5f, b, t); printf("vma "); PV3(t);
		_VectorAdd(a, b, t); printf("vadd "); PV3(t);
		_VectorSubtract(a, b, t); printf("vsub "); PV3(t);
		_VectorScale(a, 1.5f, t); printf("vscale "); PV3(t);
		{ vec4_t q = {a[0],a[1],a[2],b[0]}, o; Vector4Scale(q, 2.0f, o); printf("v4s "); PV4(o); }

		// ProjectPointOnPlane / PerpendicularVector / RotatePointAroundVector
		// need a non-degenerate normal/dir (else /0).
		if (dot_nz(b)) { ProjectPointOnPlane(t, a, b); printf("proj "); PV3(t); }
		else printf("proj SKIP\n");

		MakeNormalVectors(a, t, u); printf("mnv "); printf("%08x %08x %08x %08x %08x %08x\n",
		  f2b(t[0]),f2b(t[1]),f2b(t[2]),f2b(u[0]),f2b(u[1]),f2b(u[2]));
		if (dot_nz(a)) { PerpendicularVector(t, a); printf("perp "); PV3(t); }
		else printf("perp SKIP\n");

		if (dot_nz(a)) { RotatePointAroundVector(t, a, b, a[0]); printf("rot "); PV3(t); }
		else printf("rot SKIP\n");

		// Zero-init: PlaneFromPoints leaves plane[3] untouched on the degenerate
		// path (as does the port), so the caller's initial value must match.
		{ vec4_t pl = {0.0f, 0.0f, 0.0f, 0.0f}; qboolean ok = PlaneFromPoints(pl, a, b, c);
		  printf("pfp %d ", ok); PV4(pl); }

		printf("angsub %08x\n", f2b(AngleSubtract(a[0], a[1])));
		AnglesSubtract(a, b, t); printf("angssub "); PV3(t);
		printf("lerp %08x\n", f2b(LerpAngle(a[0], a[1], a[2])));
		printf("an360 %08x an180 %08x amod %08x adel %08x\n",
		  f2b(AngleNormalize360(a[0])), f2b(AngleNormalize180(a[0])),
		  f2b(AngleMod(a[0])), f2b(AngleDelta(a[0], a[1])));

		printf("rfb %08x\n", f2b(RadiusFromBounds(a, b)));
		printf("d2b %d\n", DirToByte(a));

		{ vec3_t res; qboolean ok = G_FindClosestPointOnLineSegment(a, b, c, res);
		  printf("gclose %d ", ok); PV3(res); }
		printf("gdist %08x\n", f2b(G_PointDistFromLineSegment(a, b, c)));

		// Matrix / axis ops.
		{ vec3_t axis[3]; AnglesToAxis(a, axis);
		  printf("a2a %08x %08x %08x %08x %08x %08x %08x %08x %08x\n",
		    f2b(axis[0][0]),f2b(axis[0][1]),f2b(axis[0][2]),
		    f2b(axis[1][0]),f2b(axis[1][1]),f2b(axis[1][2]),
		    f2b(axis[2][0]),f2b(axis[2][1]),f2b(axis[2][2]));
		  VectorRotate(b, axis, t); printf("vrot "); PV3(t);
		  vec3_t cp[3]; AxisCopy(axis, cp); printf("acopy "); PV3(cp[1]);
		  RotateAroundDirection(axis, a[1]);
		  printf("rad %08x %08x %08x\n", f2b(axis[1][0]), f2b(axis[1][1]), f2b(axis[1][2])); }
		{ vec3_t ax[3]; AxisClear(ax); printf("aclear %08x %08x %08x\n", f2b(ax[0][0]), f2b(ax[1][1]), f2b(ax[2][2])); }
		{ float m1[3][3] = {{a[0],a[1],a[2]},{b[0],b[1],b[2]},{c[0],c[1],c[2]}};
		  float m2[3][3] = {{c[0],b[1],a[2]},{a[0],c[1],b[2]},{b[0],a[1],c[2]}};
		  float o[3][3]; MatrixMultiply(m1, m2, o);
		  printf("mm %08x %08x %08x\n", f2b(o[0][0]), f2b(o[1][1]), f2b(o[2][2])); }

		{ vec3_t mn, mx; ClearBounds(mn, mx); AddPointToBounds(a, mn, mx); AddPointToBounds(b, mn, mx);
		  printf("bounds "); printf("%08x %08x %08x %08x %08x %08x\n",
		    f2b(mn[0]),f2b(mn[1]),f2b(mn[2]),f2b(mx[0]),f2b(mx[1]),f2b(mx[2])); }
	}
}

int main(int argc, char **argv) {
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-dir>\n", argv[0]); return 2; }
	load_vectors(argv[1]);
	dump_rng();
	dump_qrand();
	dump_scalars();
	dump_normaltolatlong();
	dump_planes();
	dump_vecmath();
	printf("== end ==\n");
	return 0;
}
