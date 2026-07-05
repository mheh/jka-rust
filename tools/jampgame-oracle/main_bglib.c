// bg_lib differential-oracle dumper. Compiled against the UNMODIFIED Raven
// bg_lib.c (copied into build/ by run.sh) plus raven_atoi.c (Raven's Q3_VM
// atoi extracted verbatim and renamed raven_atoi so the native, no-Q3_VM
// build does not bind libc's atoi — see run.sh / README). Prints a canonical
// bit-exact dump the Rust parity test reproduces via crate::bg_lib +
// bg_channel::rng::Rng.
#include "q_shared.h"
#include "dumpcommon.h"

// Raven bg_lib functions not visible without bg_lib.h (Q3_VM-only header).
double _atof(const char **stringPtr);
int    raven_atoi(const char *string);
// atof/qsort/memmove/srand/rand come from bg_lib.c (declared by <stdlib.h>/
// <string.h> which the shim pulls in; signatures match Raven's).

static void dump_rand(void) {
	printf("== rand ==\n");
	static const unsigned seeds[] = { 0, 1, 12345, 0x7fffffff, 0xdeadbeef };
	for (unsigned s = 0; s < sizeof(seeds)/sizeof(seeds[0]); s++) {
		printf("seed %08x\n", seeds[s]);
		srand(seeds[s]);
		for (int i = 0; i < 64; i++) printf("r %d\n", rand());
	}
}

static void dump_atox(const char *dir) {
	char path[1024];
	snprintf(path, sizeof(path), "%s/strings.txt", dir);
	long len;
	char *buf = slurp(path, &len);
	printf("== atox ==\n");
	long off = 0;
	int idx = 0;
	while (off < len) {
		const char *s = buf + off;
		size_t l = strlen(s);
		// atoi (Raven's) / atof / _atof over the same record.
		printf("a %d atoi %d\n", idx, raven_atoi(s));
		printf("a %d atof %016llx\n", idx, d2b(atof(s)));
		const char *p = s;
		double v = _atof(&p);
		printf("a %d _atof %016llx adv %ld\n", idx, d2b(v), (long)(p - s));
		off += l + 1;
		idx++;
	}
	printf("atox count %d\n", idx);
	free(buf);
}

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

static void print_buf(const char *tag, const unsigned char *b, int n) {
	printf("%s", tag);
	for (int i = 0; i < n; i++) printf(" %02x", b[i]);
	printf("\n");
}

static void dump_memmove(void) {
	printf("== memmove ==\n");
	unsigned char b[32];
	// forward overlap (dest > src): copy backwards
	for (int i = 0; i < 32; i++) b[i] = (unsigned char)i;
	memmove(b + 4, b, 16); print_buf("fwd", b, 32);
	// backward overlap (dest < src): copy forwards
	for (int i = 0; i < 32; i++) b[i] = (unsigned char)i;
	memmove(b, b + 4, 16); print_buf("back", b, 32);
	// non-overlap
	for (int i = 0; i < 32; i++) b[i] = (unsigned char)i;
	memmove(b + 20, b, 8); print_buf("nonov", b, 32);
	// full overlap dest==src
	for (int i = 0; i < 32; i++) b[i] = (unsigned char)i;
	memmove(b, b, 12); print_buf("same", b, 32);
	// count 0
	for (int i = 0; i < 32; i++) b[i] = (unsigned char)i;
	memmove(b + 1, b, 0); print_buf("zero", b, 32);
}

int main(int argc, char **argv) {
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture-dir>\n", argv[0]); return 2; }
	dump_rand();
	dump_atox(argv[1]);
	dump_qsort(argv[1]);
	dump_memmove();
	printf("== end ==\n");
	return 0;
}
