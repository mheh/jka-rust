// trmodel-oracle — matcomp golden (docs/subsystems/tr-model.md § Verification
// strategy, "MC_UnCompressQuat goldens"). Dumps the quantized-quaternion -> 3x4
// matrix table over a spread of packed inputs (the sole live matcomp path,
// UnCompressBone) plus MC_Compress/MC_UnCompress round-trips (they link even
// without a live caller). Floats are dumped as raw IEEE-754 bits for bit-exact,
// deterministic parity.
#include <cstdio>
#include <cstdint>
#include <cstring>

extern "C" {
	void MC_Compress(const float mat[3][4], unsigned char *comp);
	void MC_UnCompress(float mat[3][4], const unsigned char *comp);
	void MC_UnCompressQuat(float mat[3][4], const unsigned char *comp);
}

static uint32_t bits(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }

static void dump_mat(const char *label, const float mat[3][4]) {
	printf("%s\n", label);
	for (int r = 0; r < 3; r++)
		printf("  [%08x %08x %08x %08x]  (%.6f %.6f %.6f %.6f)\n",
			bits(mat[r][0]), bits(mat[r][1]), bits(mat[r][2]), bits(mat[r][3]),
			mat[r][0], mat[r][1], mat[r][2], mat[r][3]);
}

int main() {
	printf("=== MC_UnCompressQuat (7 x u16: quat wxyz + xlat) ===\n");
	// A spread of packed inputs. MC_UnCompressQuat reads w,x,y,z as u16/16383-2
	// then 3 xlat u16/64-512.
	const uint16_t inputs[][7] = {
		{ 16383*2, 16383*2, 16383*2, 16383*2, 512*64, 512*64, 512*64 }, // identity-ish
		{ 16383*3, 16383*2, 16383*2, 16383*2, 0,       0,       0      },
		{ 20000,   10000,   30000,   5000,    40000,   20000,   1000   },
		{ 0,       0,       0,       0,       65535,   0,       32768  },
		{ 65535,   65535,   65535,   65535,   1,       2,       3      },
	};
	for (size_t i = 0; i < sizeof(inputs)/sizeof(inputs[0]); i++) {
		unsigned char comp[24]; memset(comp, 0, sizeof(comp));
		memcpy(comp, inputs[i], sizeof(inputs[i]));
		float mat[3][4];
		MC_UnCompressQuat(mat, comp);
		char lbl[64]; snprintf(lbl, sizeof(lbl), "quat[%zu]", i);
		dump_mat(lbl, mat);
	}

	printf("\n=== MC_Compress -> MC_UnCompress round-trip ===\n");
	const float mats[][3][4] = {
		{ {1,0,0, 10}, {0,1,0, -20}, {0,0,1, 30} },
		{ {0.5f,-0.5f,0.25f, 100}, {-1,1,-1, -100}, {0.1f,0.2f,0.3f, 0} },
	};
	for (size_t i = 0; i < sizeof(mats)/sizeof(mats[0]); i++) {
		unsigned char comp[24]; memset(comp, 0, sizeof(comp));
		MC_Compress(mats[i], comp);
		printf("comp[%zu]:", i);
		for (int b = 0; b < 24; b++) printf(" %02x", comp[b]);
		printf("\n");
		float mat[3][4];
		MC_UnCompress(mat, comp);
		char lbl[64]; snprintf(lbl, sizeof(lbl), "uncompress[%zu]", i);
		dump_mat(lbl, mat);
	}
	return 0;
}
