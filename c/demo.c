/* for practice purpose
*  This demo script restricts that there must exists one and only one output with this script in the transaction
*/

#include "ckb_syscalls.h"

#define ERROR_OUTPUT_NUMBER -99

int has_type_id_output(size_t index) {
    uint64_t len = 0;
    int ret = ckb_load_cell(NULL, &len, 0, index, CKB_SOURCE_GROUP_OUTPUT);
    return ret == CKB_SUCCESS ? 1 : 0;
}

__attribute__((visibility("default"))) int verify() {
    int has_one_type_id_output = has_type_id_output(0);
    int has_second_type_id_output = has_type_id_output(1);

    if (has_one_type_id_output && !has_second_type_id_output) {
        return CKB_SUCCESS;
    }

    return ERROR_OUTPUT_NUMBER;
}

int main() {
    int has_one_type_id_output = has_type_id_output(0);
    int has_second_type_id_output = has_type_id_output(1);

    if (has_one_type_id_output && !has_second_type_id_output) {
        return CKB_SUCCESS;
    }

    return ERROR_OUTPUT_NUMBER;
}
