// bg_lib qsort differential-oracle dumper. Compiled against the UNMODIFIED
// Raven bg_lib.c (copied into build/ by run.sh); its BSD Bentley-McIlroy
// qsort is the C ground truth the Rust parity test reproduces via
// native_sort::qsort — including the tie permutation of equal keys. The
// other bg_lib sections (rand/srand 2026-07-14; atoi/atof/memmove
// 2026-07-19) were retired: retail's JK2_game.vcproj excludes bg_lib.c from
// the native DLL, so those bodies never linked — their canonical homes are
// native_string/bg_channel::rng, pinned by their own tests.
#include "q_shared.h"
#include "dumpcommon.h"

static int cmp_int(const void *a, const void *b) {
	int x = *(const int *)a, y = *(const int *)b;
	return (x > y) - (x < y);
}

typedef struct { int key; int payload; } kv_t;
static int cmp_kv(const void *a, const void *b) {
	int x = ((const kv_t *)a)->key, y = ((const kv_t *)b)->key;
	return (x > y) - (x < y);
}

static void dump_qsort(const char *dir) {
	printf("== qsort ==\n");
	char path[1024];
	snprintf(path, sizeof(path), "%s/ints.txt", dir);
	FILE *f = fopen(path, "rb");
	if (!f) { fprintf(stderr, "cannot open %s\n", path); exit(2); }
	static int arr[512];
	int n = 0, v;
	while (fscanf(f, "%d", &v) == 1) arr[n++] = v;
	fclose(f);
	qsort(arr, n, sizeof(int), cmp_int);
	printf("ints %d\n", n);
	for (int i = 0; i < n; i++) printf("%d\n", arr[i]);

	// struct array: sort by key, dump (key,payload) to observe qsort's
	// (unstable) ordering of equal keys deterministically.
	static kv_t kv[] = {
		{5,0},{3,1},{5,2},{1,3},{3,4},{9,5},{1,6},{7,7},{3,8},{5,9},
		{0,10},{8,11},{2,12},{8,13},{4,14},{6,15},{3,16},{9,17},{1,18},{5,19},
	};
	int kn = (int)(sizeof(kv)/sizeof(kv[0]));
	qsort(kv, kn, sizeof(kv_t), cmp_kv);
	printf("kv %d\n", kn);
	for (int i = 0; i < kn; i++) printf("%d %d\n", kv[i].key, kv[i].payload);
}

int main(int argc, char **argv) {
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-dir>\n", argv[0]); return 2; }
	dump_qsort(argv[1]);
	printf("== end ==\n");
	return 0;
}
