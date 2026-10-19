/* Freestanding entry for tests/cross.sh. The binary is not run. */
typedef long long i64;
typedef unsigned long long u64;

static unsigned char buf[256];

void exit(int code) {
  (void)code;
  for (;;) {
  }
}

i64 write(int fd, const void *p, u64 n) {
  (void)fd;
  (void)p;
  return (i64)n;
}

void *malloc(u64 n) {
  (void)n;
  return buf;
}

void free(void *p) { (void)p; }

void *memcpy(void *d, const void *s, u64 n) {
  unsigned char *dd = d;
  const unsigned char *ss = s;
  u64 i;
  for (i = 0; i < n; i++) dd[i] = ss[i];
  return d;
}

void *memset(void *d, int c, u64 n) {
  unsigned char *dd = d;
  u64 i;
  for (i = 0; i < n; i++) dd[i] = (unsigned char)c;
  return d;
}

void *memmove(void *d, const void *s, u64 n) { return memcpy(d, s, n); }

int main(int argc, char **argv);

void _start(void) {
  main(0, 0);
  exit(0);
}
