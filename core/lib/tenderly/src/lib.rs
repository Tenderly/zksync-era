use zksync_multivm::VmInstance;
use zksync_utils::{bytecode::hash_bytecode, u256_to_h256};

fn default_l1_batch() -> L1BatchEnv {
    L1BatchEnv {
        previous_batch_hash: None,
        number: L1BatchNumber(1),
        timestamp: 100,
        fee_input: BatchFeeInput::l1_pegged(
            50_000_000_000, // 50 gwei
            250_000_000,    // 0.25 gwei
        ),
        fee_account: Address::random(),
        enforced_base_fee: None,
        first_l2_block: L2BlockEnv {
            number: 1,
            timestamp: 100,
            prev_block_hash: MiniblockHasher::legacy_hash(MiniblockNumber(0)),
            max_virtual_blocks_to_create: 100,
        },
    }
}

fn default_system_env() -> SystemEnv {
    SystemEnv {
        zk_porter_available: false,
        version: ProtocolVersionId::latest(),
        base_system_smart_contracts: BaseSystemContracts::playground(),
        gas_limit: BLOCK_GAS_LIMIT,
        execution_mode: TxExecutionMode::VerifyExecute,
        default_validation_computational_gas_limit: BLOCK_GAS_LIMIT,
        chain_id: L2ChainId::from(270),
    }
}

fn default_empty_storage() -> InMemoryStorage {
    InMemoryStorage::with_system_contracts(hash_bytecode)
}

pub fn init_vm() {
    let l1_batch_env = default_l1_batch();
    let system_env = default_system_env();
    let mut raw_storage = default_empty_storage();
    let storage_ptr = StorageView::new(raw_storage).to_rc_ptr();

    let vm = VmInstance::new(l1_batch_env, system_env, storage_ptr.clone());
    vm.execute(VmExecutionMode::OneTx);
}
