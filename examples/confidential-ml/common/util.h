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

/*------ Base64 Encoding Table ------*/
static const char MimeBase64[] = {
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
    'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f',
    'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
    'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
    'w', 'x', 'y', 'z', '0', '1', '2', '3',
    '4', '5', '6', '7', '8', '9', '+', '/'
};

int base64_encode(char *text, int numBytes, char **encodedText)
{
    unsigned char input[3] = { 0,0,0 };
    unsigned char output[4] = { 0,0,0,0 };
    int   index, i, j, size;
    char *p, *plen;
    plen = text + numBytes - 1;
    size = (4 * (numBytes / 3)) + (numBytes % 3 ? 4 : 0) + 1;
    (*encodedText) = (char*)malloc(size);
    j = 0;
    for (i = 0, p = text; p <= plen; i++, p++) {
        index = i % 3;
        input[index] = *p;
        if (index == 2 || p == plen) {
            output[0] = ((input[0] & 0xFC) >> 2);
            output[1] = ((input[0] & 0x3) << 4) | ((input[1] & 0xF0) >> 4);
            output[2] = ((input[1] & 0xF) << 2) | ((input[2] & 0xC0) >> 6);
            output[3] = (input[2] & 0x3F);
            (*encodedText)[j++] = MimeBase64[output[0]];
            (*encodedText)[j++] = MimeBase64[output[1]];
            (*encodedText)[j++] = index == 0 ? '=' : MimeBase64[output[2]];
            (*encodedText)[j++] = index <  2 ? '=' : MimeBase64[output[3]];
            input[0] = input[1] = input[2] = 0;
        }
    }
    (*encodedText)[j] = '\0';
    return size;
}

/*------ Base64 Decoding Table ------*/
static int DecodeMimeBase64[256] = {
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* 00-0F */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* 10-1F */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,62,-1,-1,-1,63,  /* 20-2F */
    52,53,54,55,56,57,58,59,60,61,-1,-1,-1,-1,-1,-1,  /* 30-3F */
    -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13,14,  /* 40-4F */
    15,16,17,18,19,20,21,22,23,24,25,-1,-1,-1,-1,-1,  /* 50-5F */
    -1,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,  /* 60-6F */
    41,42,43,44,45,46,47,48,49,50,51,-1,-1,-1,-1,-1,  /* 70-7F */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* 80-8F */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* 90-9F */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* A0-AF */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* B0-BF */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* C0-CF */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* D0-DF */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,  /* E0-EF */
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1   /* F0-FF */
};


int base64_decode(char * text, unsigned char * dst, int numBytes)
{
    const char* cp;
    int space_idx = 0, phase;
    int d, prev_d = 0;
    unsigned char c;
    space_idx = 0;
    phase = 0;
    for (cp = text; *cp != '\0'; ++cp) {
        d = DecodeMimeBase64[(int)*cp];
        if (d != -1) {
            switch (phase) {
            case 0:
                ++phase;
                break;
            case 1:
                c = ((prev_d << 2) | ((d & 0x30) >> 4));
                if (space_idx < numBytes)
                    dst[space_idx++] = c;
                ++phase;
                break;
            case 2:
                c = (((prev_d & 0xf) << 4) | ((d & 0x3c) >> 2));
                if (space_idx < numBytes)
                    dst[space_idx++] = c;
                ++phase;
                break;
            case 3:
                c = (((prev_d & 0x03) << 6) | d);
                if (space_idx < numBytes)
                    dst[space_idx++] = c;
                phase = 0;
                break;
            }
            prev_d = d;
        }
    }
    return space_idx;
}

