// BlockStream reader golden dumper (icarus.md § Verification strategy, unit 1:
// "BlockStream (read-only)"). Compiled against the UNMODIFIED oracle
// codemp/icarus/BlockStream.cpp (reader half). Loads a committed .IBI blob and
// dumps the parsed (block id, flags) + per-member (id, size, bytes) record
// stream produced by Open -> BlockAvailable -> ReadBlock -> ReadMember. The Rust
// port's blockstream reader must reproduce this byte-for-byte.
#include "exe_headers.h"   // q_shared.h + qcommon.h, exactly as every oracle icarus TU opens
#include "icarus.h"   // pulls the whole icarus header chain (CBlockStream, CBlock, CBlockMember)

#include <cstdio>
#include <cstdlib>

int main(int argc, char **argv)
{
	if (argc != 2) { fprintf(stderr, "usage: %s <fixture.IBI>\n", argv[0]); return 2; }

	FILE *f = fopen(argv[1], "rb");
	if (!f) { fprintf(stderr, "cannot open %s\n", argv[1]); return 2; }
	fseek(f, 0, SEEK_END);
	long size = ftell(f);
	fseek(f, 0, SEEK_SET);
	char *buf = (char *)malloc(size);
	if (fread(buf, 1, size, f) != (size_t)size) { fprintf(stderr, "short read\n"); return 2; }
	fclose(f);

	printf("== blockstream %ld bytes ==\n", size);

	CBlockStream stream;
	if (stream.Open(buf, size) == 0)
	{
		printf("open error\n== end ==\n");
		return 0;
	}
	printf("open ok\n");

	int blockIdx = 0;
	while (stream.BlockAvailable())
	{
		CBlock block;
		if (stream.ReadBlock(&block) == 0)
		{
			printf("readblock error\n");
			break;
		}
		int nMembers = block.GetNumMembers();
		printf("block %d id=%d flags=%u members=%d\n",
		       blockIdx, block.GetBlockID(), (unsigned)block.GetFlags(), nMembers);
		for (int j = 0; j < nMembers; j++)
		{
			CBlockMember *m = block.GetMember(j);
			int mid = 0, msize = 0; void *data = 0;
			m->GetInfo(&mid, &msize, &data);
			printf("  m%d id=%d size=%d bytes=", j, mid, msize);
			unsigned char *p = (unsigned char *)data;
			for (int k = 0; k < msize; k++) printf("%02x", p ? p[k] : 0);
			printf("\n");
		}
		block.Free();
		blockIdx++;
	}
	printf("== end ==\n");
	return 0;
}
