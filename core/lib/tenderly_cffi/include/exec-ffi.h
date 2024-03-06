#ifndef EXEC_FFI_H
#define EXEC_FFI_H

#include <stdint.h>
#include <stdlib.h>

typedef struct TransactionExecutor TransactionExecutor;

TransactionExecutor *exec_new_tx_executor();
void exec_free_tx_executor(TransactionExecutor *tx);

#define TX_PROPERTY_BLOCK_NUMBER 0x0
#define TX_PROPERTY_BLOCK_COINBASE 0x1
#define TX_PROPERTY_BLOCK_GAS_LIMIT 0x2
#define TX_PROPERTY_BLOCK_TIMESTAMP 0x3
#define TX_PROPERTY_BLOCK_DIFFICULTY 0x4
#define TX_PROPERTY_BLOCK_BASE_FEE 0x5
#define TX_PROPERTY_BLOCK_PREVRANDAO 0x6
#define TX_PROPERTY_BLOCK_EXCESS_BLOB_GAS 0x7
#define TX_PROPERTY_BLOCK_PARENT_HASH 0x8

#define TX_PROPERTY_TX_HASH 0x100
#define TX_PROPERTY_TX_FROM 0x101
#define TX_PROPERTY_TX_TO 0x102
#define TX_PROPERTY_TX_NONCE 0x103
#define TX_PROPERTY_TX_VALUE 0x104
#define TX_PROPERTY_TX_GAS_LIMIT 0x105
#define TX_PROPERTY_TX_GAS_PRICE 0x106
#define TX_PROPERTY_TX_FEE_CAP 0x107
#define TX_PROPERTY_TX_TIP 0x108
#define TX_PROPERTY_TX_MAX_FEE_PER_BLOB_GAS 0x109
#define TX_PROPERTY_TX_DATA 0x10A
#define TX_PROPERTY_TX_ACCESS_LIST 0x10B
#define TX_PROPERTY_TX_BLOB_HASHES 0x10C

#define TX_PROPERTY_OPT_CHECK_NONCE 0x200
#define TX_PROPERTY_OPT_NO_BASE_FEE 0x201

#define TX_PROPERTY_ENV_GET_NONCE 0x300
#define TX_PROPERTY_ENV_GET_BALANCE 0x301
#define TX_PROPERTY_ENV_GET_CODE_HASH 0x302
#define TX_PROPERTY_ENV_GET_CODE_LENGTH 0x303
#define TX_PROPERTY_ENV_GET_CODE 0x304
#define TX_PROPERTY_ENV_GET_STORAGE 0x305

typedef uint64_t (*GetNonceCallback)(uint8_t *addr, void *data);
typedef void (*GetBalanceCallback)(uint8_t *addr, uint8_t *result, void *data);
typedef void (*GetCodeHashCallback)(uint8_t *addr, uint8_t *result, void *data);
typedef uint64_t (*GetCodeLengthCallback)(uint8_t *addr, void *data);
typedef void (*GetCodeCallback)(uint8_t *addr, uint8_t *result, void *data);
typedef void (*GetStorageCallback)(uint8_t *addr, uint8_t *key, uint8_t *result, void *data);

void exec_tx_set_property_uint64(TransactionExecutor *tx, uint64_t property, uint64_t value);
void exec_tx_set_property_data(TransactionExecutor *tx, uint64_t property, void *data, uint64_t size);
void exec_tx_set_property_func(TransactionExecutor *tx, uint64_t property, void *callback, void *data);

void exec_tx_execute(TransactionExecutor *tx);

#define TX_OUTPUT_USED_GAS 0
#define TX_OUTPUT_RETURN_DATA 1

uint64_t exec_tx_get_output_uint64(TransactionExecutor *tx, uint64_t output);
uint64_t exec_tx_get_output_data(TransactionExecutor *tx, uint64_t output, void *data);

#endif // EXEC_FFI_H