#include "ckb_dlfcn.h"
#include "ckb_syscalls.h"
#include "blockchain.h"

#define SCRIPT_SIZE 32768
#define BLAKE2B_BLOCK_SIZE 32
#define MAX_TYPE_HASH 256

#define ERROR_ARGUMENTS_LEN -1
#define ERROR_ENCODING -2
#define ERROR_SYSCALL -3
#define ERROR_SCRIPT_TOO_LONG -21
#define ERROR_TOO_MUCH_TYPE_HASH_INPUTS -43
#define ERROR_DYNAMIC_LOADING -103

int read_arg(unsigned char *code_hash) {
  /* Load args */
  unsigned char script[SCRIPT_SIZE];
  uint64_t len = SCRIPT_SIZE;
  int ret = ckb_load_script(script, &len, 0);
  if (ret != CKB_SUCCESS) {
    return ERROR_SYSCALL;
  }
  if (len > SCRIPT_SIZE) {
    return ERROR_SCRIPT_TOO_LONG;
  }
  mol_seg_t script_seg;
  script_seg.ptr = (uint8_t *)script;
  script_seg.size = len;

  if (MolReader_Script_verify(&script_seg, false) != MOL_OK) {
    return ERROR_ENCODING;
  }

  mol_seg_t args_seg = MolReader_Script_get_args(&script_seg);
  mol_seg_t args_bytes_seg = MolReader_Bytes_raw_bytes(&args_seg);
  if (args_bytes_seg.size != BLAKE2B_BLOCK_SIZE) {
    return ERROR_ARGUMENTS_LEN;
  }
  memcpy(code_hash, args_bytes_seg.ptr, BLAKE2B_BLOCK_SIZE);
  return CKB_SUCCESS;
}

int main() {
    /* Load args */
    int ret;
    unsigned char code_hash[BLAKE2B_BLOCK_SIZE];
    ret = read_arg(code_hash);
    if (ret != CKB_SUCCESS) {
        return ret;
    }

    /* load_code */
    uint8_t code_buffer[68 * 1024] __attribute__((aligned(RISCV_PGSIZE)));
    uint8_t *aligned_code_start = code_buffer;
    size_t aligned_size = ROUNDDOWN(68 * 1024, RISCV_PGSIZE);
    void *handle = NULL;
    uint64_t consumed_size = 0;

    ret = ckb_dlopen(code_hash, aligned_code_start,
                         aligned_size, &handle, &consumed_size);
    if (ret != CKB_SUCCESS) {
      return ret;
    }

    int (*verify_func)();
    *(void **)(&verify_func) =
        ckb_dlsym(handle, "verify");
    if (verify_func == NULL) {
        return ERROR_DYNAMIC_LOADING;
    }

    ret = verify_func();
    if (ret != CKB_SUCCESS) {
      return ret;
    }

    return CKB_SUCCESS;
}
