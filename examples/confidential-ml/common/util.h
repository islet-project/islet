#include <stdio.h>
#include <stdlib.h>

void save_as_file(char *name, unsigned char *data, unsigned int size) {
  FILE* fp = fopen(name, "wb");
  if (fp == NULL) {
    printf("file open error: %s\n", name);
    return;
  }
  size_t len = fwrite(data, 1, size, fp);
  if (len != size) {
    printf("fwrite fail\n");
    fclose(fp);
    return;
  }
  fclose(fp);
}

size_t read_file(char *name, unsigned char *buffer, unsigned int size) {
  FILE *ptr;
  ptr = fopen(name, "rb");
  if (ptr == NULL) {
    printf("file open error: %s\n", strerror(errno));
    return 0;
  }

  size_t len = fread(buffer, 1, size, ptr);
  printf("read done, size: %d\n", len);
  if (len == 0) {
    printf("read fail\n");
    return 0;
  }
  return len;
}