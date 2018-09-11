#include <stdio.h>
#include <assert.h>
#include <string>
#include <cstring>
#include <random>
#include <openssl/bio.h>
#include <openssl/evp.h>

// #define PARSE_ALL_FILE

#define OUT_BLOCK_SIZE (16*(1<<16)) //1MB output file blocks

typedef unsigned char uint8;

static uint8 aes_key[] = {
    0x4c, 0x86, 0xaa, 0xf6, 
    0xaf, 0xc9, 0x5e, 0x87, 
    0xa6, 0x85, 0x18, 0xdf, 
    0x8a, 0xe7, 0x58, 0x29
};


void aes_gcm_encrypt(uint8* key, const uint8* pt_buff, unsigned int pt_size, uint8* ct_buff, uint8* iv, uint8* mac)
{
  int outlen;
  EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
  /* Set cipher type and mode */
  EVP_EncryptInit_ex(ctx, EVP_aes_128_gcm(), NULL, NULL, NULL);

  /* Set IV length if default 96 bits is not appropriate */
  EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_SET_IVLEN, 12, NULL);

  // printf("AES GCM Encrypt:\n");
  // printf("Plaintext %d:\n", 64);
  // BIO_dump_fp(stdout, pt_buff, 64);

  /* Initialise key and IV */
  EVP_EncryptInit_ex(ctx, NULL, NULL, key, iv);

  // /* Zero or more calls tol specify any AAD */
  // EVP_EncryptUpdate(ctx, NULL, &outlen, gcm_iv, sizeof(gcm_iv));

  /* Encrypt plaintext */
  EVP_EncryptUpdate(ctx, ct_buff, &outlen, pt_buff, pt_size);

  // /* Output encrypted block */
  // printf("CiphertextA %d:\n", outlen);
  // BIO_dump_fp(stdout, ct_buff, 64);

  /* Finalise: note get no output for GCM */
  EVP_EncryptFinal_ex(ctx, ct_buff, &outlen);

  // /* Output encrypted block */
  // printf("CiphertextB %d:\n", outlen);
  // BIO_dump_fp(stdout, ct_buff, 64);

  /* Get tag */
  EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_GCM_GET_TAG, 16, mac);

  // /* Output tag */
  // printf("Tag %d:\n", 16);
  // BIO_dump_fp(stdout, mac, 16);

  // /* Output tag */
  // printf("Buff %d:\n", 16+64);
  // BIO_dump_fp(stdout, iv, 16+64);

  EVP_CIPHER_CTX_free(ctx);
}

void write_out_buff(FILE* file, uint8* buff, unsigned int size, uint8* buff_enc) {
  // printf("write block of size %d\n", size);

  //generate random iv
  for (int i = 0; i < 12; ++i)
    *(buff_enc+i) = rand() % 256;
  //fill in block size
  for (int i = 0; i < 4; ++i)
    *(buff_enc+15-i) = (size >> (i*8)) & 0xFF;

  aes_gcm_encrypt(aes_key, buff, size, buff_enc+32, buff_enc, buff_enc+16);
  fwrite(buff_enc, sizeof(uint8), size+32, file);
  // fwrite(buff, sizeof(uint8), size, file);
}

struct L {
  unsigned char chrom;
  unsigned int pos;
  unsigned long id;
  unsigned char ref;
  unsigned char alt;
  unsigned char het_hom;
  void init() {
    this->chrom = 0;
    this->pos = 0;
    this->id = 0;
    this->ref = '-';
    this->alt = '-';
    this->het_hom = 0;
  }
};

inline void pack_elems_buff(uint8* buff, struct L& line) {
  *(buff+0) = line.chrom;
  for (int i = 0; i < 4; ++i)
    *(buff+4-i) = (line.pos >> i*8) & 0xFF;
  for (int i = 0; i < 8; ++i)
    *(buff+12-i) = (line.id >> i*8) & 0xFF;
  *(buff+13) = line.ref;
  *(buff+14) = line.alt;
  *(buff+15) = line.het_hom;
}

#ifdef PARSE_ALL_FILE
void inp_file_parse(FILE* inp_file, FILE* out_file) {
  fseek(inp_file, 0L, SEEK_END);
  auto inp_sz = ftell(inp_file);
  rewind(inp_file);

  uint8* inp_buff_start = (uint8*)malloc(inp_sz);
  fread(inp_buff_start, sizeof(unsigned char), inp_sz, inp_file);
  uint8* inp_buff_end = inp_buff_start+inp_sz;
  uint8* inp_ptr = inp_buff_start;

  uint8* out_buff = (uint8*)malloc(OUT_BLOCK_SIZE);
  uint8* out_buff_aes = (uint8*)malloc(OUT_BLOCK_SIZE + 16*2); //12: IV, 4: block size, 16: MAC
  uint8* out_buff_end = out_buff + OUT_BLOCK_SIZE;
  uint8* out_ptr = out_buff;

  struct L line;
  line.init();

  bool ignore_line = false;
  int state = 0; //0-chrom, 1-pos, 2-id, 3-ref, 4-alt, 5-qual, 6-filter, 7-type

  while (inp_ptr != inp_buff_end)
  {
    if (*inp_ptr == '#') {
      ignore_line = true;
    } else if (*inp_ptr == '\t' ) {
      state++;
      if (state == 2) {
        inp_ptr++;
        if (*inp_ptr == '.') {
          state = 3;
          line.id = 0;
        }
        inp_ptr++;
      }
    } else if (state == 0) {
      line.chrom *= 10;
      line.chrom += *inp_ptr - '0';
    } else if (state == 1) {
      line.pos *= 10;
      line.pos += *inp_ptr - '0';
    } else if (state == 2) {
      line.id *= 10;
      line.id += *inp_ptr - '0';
    } else if (state == 3) {
      line.ref = *inp_ptr;
    } else if (state == 4) {
      line.alt = *inp_ptr; 
    } else if (state <= 6) {
    } else if (state == 7) {
      inp_ptr++;
      line.het_hom = (*inp_ptr == 'o') + 1;
      ignore_line = true;
      state = 0;
      
      // pack elements to byte buffer
      pack_elems_buff(out_ptr, line);
      out_ptr+=16;

      if (out_ptr == out_buff_end) {
        write_out_buff(out_file, out_buff, OUT_BLOCK_SIZE, out_buff_aes);
        out_ptr = out_buff;
      }
      line.init();
    }
    while (ignore_line and (*inp_ptr != '\n')) inp_ptr++;
    ignore_line = false;
    inp_ptr++;
  }
  write_out_buff(out_file, out_buff, out_ptr - out_buff, out_buff_aes);

  free(inp_buff_start);
  free(out_buff);
  free(out_buff_aes);
}

#else
template<typename T>
T parse(char* sptr) {
  T tmp = 0;
  while (*sptr != '\0') {
    tmp *= 10;
    tmp += *sptr - '0';
    sptr++;
  }
  return tmp;
}

int inp_file_parse_line(uint8* inp_ptr, struct L& elem) {
  elem.init();

  while (*inp_ptr != '\t') {
    elem.chrom *= 10;
    elem.chrom += *(inp_ptr) - '0';
    inp_ptr++;
  }
  inp_ptr++;
  
  while (*inp_ptr != '\t') {
    elem.pos *= 10;
    elem.pos += *(inp_ptr) - '0';
    inp_ptr++;
  }
  inp_ptr++;

  if (*inp_ptr != '.') {
    inp_ptr+=2;
    while (*inp_ptr != '\t') {
      elem.id *= 10;
      elem.id += *(inp_ptr) - '0';
      inp_ptr++;
    }
  } else {
    inp_ptr++;
  }
  inp_ptr++;

  elem.ref = *inp_ptr;
  inp_ptr+=2;
  
  elem.alt = *inp_ptr;
  inp_ptr+=2;

  while (*(inp_ptr++) != '\t');
  
  while (*(inp_ptr++) != '\t');

  inp_ptr++;
  elem.het_hom = (*inp_ptr == 'o') + 1;
}

void inp_file_parse(FILE* inp_file, FILE* out_file) {
  char line[256];
  char* ptr_tab[8];
  uint8* out_buff = (uint8*)malloc(OUT_BLOCK_SIZE);
  uint8* out_buff_aes = (uint8*)malloc(OUT_BLOCK_SIZE + 16*2); //12: IV, 4: block size, 16: MAC
  uint8* out_buff_end = out_buff + OUT_BLOCK_SIZE;
  uint8* out_ptr = out_buff;

  struct L elem;

  while (fgets(line, sizeof(line), inp_file)) {
    if (line[0] == '#') continue;

    inp_file_parse_line(line, elem);

    // pack elements to byte buffer
    pack_elems_buff(out_ptr, elem);
    out_ptr+=16;

    if (out_ptr == out_buff_end) {
      write_out_buff(out_file, out_buff, OUT_BLOCK_SIZE, out_buff_aes);
      out_ptr = out_buff;
    }
  }
  write_out_buff(out_file, out_buff, out_ptr - out_buff, out_buff_aes);
}
#endif

int main(int argc, char** argv) {
  if (argc < 2) {
    printf("Please specify an input file\n");
    exit(-1);
  }
  if (argc < 3) {
    printf("Please specify an output file\n");
    exit(-1);
  }
  if (argc > 3) {
    char* str_key = argv[3];
    assert(strlen(str_key) == 16*2);
    
    uint8 tmp[3] = {0x00};
    for (int i = 0; i < 16; ++i) {
      tmp[0] = str_key[2*i];
      tmp[1] = str_key[2*i+1];
      aes_key[i] = (uint8)strtol(tmp, NULL, 16);
    }
  }
  // printf("AES key to use: ");
  // for (int i = 0; i < 16; ++i) printf("0x%x ", aes_key[i]);
  // printf("\n");

  char* inp_fname = argv[1];
  char* out_fname = argv[2];

  FILE* inp_file = fopen(inp_fname, "r"); 
  assert(inp_file);
  FILE* out_file = fopen(out_fname, "wb"); 
  assert(out_file);

  // srand(42);

  inp_file_parse(inp_file, out_file);

  fclose(inp_file);
  fclose(out_file);
}