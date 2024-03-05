use std::fmt::{Debug, Formatter};
use multivm::interface::{L1BatchEnv, L2BlockEnv, SystemEnv, TxExecutionMode, VmExecutionMode, VmExecutionResultAndLogs, VmInterface};
use multivm::interface::ExecutionResult::Success;
use multivm::vm_latest::constants::BLOCK_GAS_LIMIT;
use multivm::vm_latest::HistoryEnabled;
use multivm::VmInstance;
use multivm::zk_evm_latest::ethereum_types::H256;
use multivm::zkevm_test_harness_latest::ethereum_types::Address;
use zksync_state::{ReadStorage, StorageView};
use zksync_types::{Execute, InputData, L2ChainId, L2TxCommonData, Nonce, StorageKey, StorageValue, Transaction};
use tenderly_cffi::{GetBalanceFunc, GetCodeFunc, GetCodeHashFunc, GetCodeLengthFunc, GetNonceFunc, GetStorageFunc, TransactionExecutor};
use zksync_contracts::BaseSystemContracts;
use zksync_types::ExecuteTransactionCommon::L2;
use zksync_types::l2::TransactionType;

#[derive(Debug)]
pub struct TransactionExecutorImpl {
    l2_transaction: L2TxCommonData,
    l2_execution: Execute,
    l2_block: L2BlockEnv,

    execution_result: VmExecutionResultAndLogs,

    storage : TenderlyStorage
}

struct TenderlyStorage {
    get_storage: GetStorageFunc
}

impl TransactionExecutorImpl {
    fn new() -> TransactionExecutorImpl {
        TransactionExecutorImpl {
            l2_transaction: L2TxCommonData {
                    nonce: Nonce(0),
                    fee: Default::default(),
                    initiator_address: Default::default(),
                    signature: vec![],
                    transaction_type: TransactionType::LegacyTransaction,
                    input: Some(InputData { hash: Default::default(), data: vec![] }),
                    paymaster_params: Default::default(),
            },
            l2_execution: Default::default(),
            l2_block: L2BlockEnv {
                number: 0,
                timestamp: 0,
                prev_block_hash: Default::default(),
                max_virtual_blocks_to_create: 1,
            },
            execution_result: VmExecutionResultAndLogs {
                result: Success { output: vec![] },
                logs: Default::default(),
                statistics: Default::default(),
                refunds: Default::default(),
            },
            storage : Default::default()
        }
    }

    fn l1_batch_env(&self) -> L1BatchEnv {
        L1BatchEnv {
            previous_batch_hash: None,
            number: Default::default(),
            timestamp: 0,
            fee_input: Default::default(),
            fee_account: Default::default(),
            enforced_base_fee: None,
            first_l2_block: self.l2_block.clone(),
        }
    }

    fn system_env(&self) -> SystemEnv {
        SystemEnv {
            zk_porter_available: false,
            version: Default::default(),
            base_system_smart_contracts: BaseSystemContracts::playground(),
            gas_limit: BLOCK_GAS_LIMIT,
            execution_mode: TxExecutionMode::VerifyExecute,
            default_validation_computational_gas_limit: BLOCK_GAS_LIMIT,
            chain_id: L2ChainId::from(324),
        }
    }

    fn transaction(&self) -> Transaction {
        Transaction {
            common_data: L2(self.l2_transaction.clone()),
            execute: self.l2_execution.clone(),
            received_timestamp_ms: 0,
            raw_bytes: None,
        }
    }
}

impl Default for TransactionExecutorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionExecutor for TransactionExecutorImpl {
    fn set_block_number(&mut self, _value: u64) { self.l2_block.number = _value as u32; }
    fn set_block_coinbase(&mut self, _value: &[u8]) {}
    fn set_block_gas_limit(&mut self, _value: u64) {}
    fn set_block_timestamp(&mut self, _value: u64) { self.l2_block.timestamp = _value; }
    fn set_block_difficulty(&mut self, _value: &[u8]) {}
    fn set_block_base_fee(&mut self, _value: &[u8]) {}
    fn set_block_prevrandao(&mut self, _value: &[u8]) {}
    fn set_block_excess_blob_gas(&mut self, _value: u64) {}

    fn set_tx_hash(&mut self, _value: &[u8]) {
        if let Some(mut input) = self.l2_transaction.input.take() {
            input.hash = H256::from_slice(_value);
            self.l2_transaction.input = Some(input);
        }
    }
    fn set_tx_from(&mut self, _value: &[u8]) { self.l2_transaction.initiator_address = Address::from_slice(_value); }
    fn set_tx_to(&mut self, _value: &[u8]) { self.l2_execution.contract_address = Address::from_slice(_value); }
    fn set_tx_nonce(&mut self, _value: u64) { self.l2_transaction.nonce = Nonce(_value as u32); }
    fn set_tx_value(&mut self, _value: &[u8]) { todo!() }
    fn set_tx_gas_limit(&mut self, _value: &[u8]) {}
    fn set_tx_gas_price(&mut self, _value: &[u8]) {}
    fn set_tx_fee_cap(&mut self, _value: &[u8]) {}
    fn set_tx_tip(&mut self, _value: &[u8]) {}
    fn set_tx_max_fee_per_blob_gas(&mut self, _value: &[u8]) {}
    fn set_tx_data(&mut self, _value: &[u8]) { self.l2_execution.calldata = _value.to_vec(); }
    fn set_tx_access_list(&mut self, _value: &[u8]) {}
    fn set_tx_blob_hashes(&mut self, _value: &[u8]) {}

    fn set_opt_check_nonce(&mut self, _value: bool) {}
    fn set_opt_no_base_fee(&mut self, _value: bool) {}

    fn set_env_get_nonce(&mut self, _value: GetNonceFunc) {}
    fn set_env_get_balance(&mut self, _value: GetBalanceFunc) {}
    fn set_env_get_code_hash(&mut self, _value: GetCodeHashFunc) {}
    fn set_env_get_code_length(&mut self, _value: GetCodeLengthFunc) {}
    fn set_env_get_code(&mut self, _value: GetCodeFunc) {}
    fn set_env_get_storage(&mut self, _value: GetStorageFunc) { self.storage.get_storage = _value; }

    fn execute(&mut self) {
        let l1_batch_env = self.l1_batch_env();
        let system_env = self.system_env();
        let transaction = self.transaction();
        let storage_ptr = StorageView::new(&mut self.storage).to_rc_ptr();

        let mut vm:VmInstance<StorageView<&mut TenderlyStorage>, HistoryEnabled> = VmInstance::new(l1_batch_env, system_env, storage_ptr);
        vm.push_transaction(transaction);
        self.execution_result = vm.execute(VmExecutionMode::OneTx);
    }

    fn get_used_gas(&self) -> u64 {
        self.execution_result.statistics.gas_used as u64
    }
    fn get_return_data(&self) -> Vec<u8> {
        match &self.execution_result.result {
            Success { output } => { output.clone() }
            _ => vec![]
        }
    }

    fn close(&mut self) {}
}

impl Default for TenderlyStorage {
    fn default() -> Self {
        TenderlyStorage {
            get_storage: Box::new(|_, _, _| {}),
        }
    }
}

impl Debug for TenderlyStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl ReadStorage for &mut TenderlyStorage {
    fn read_value(&mut self, key: &StorageKey) -> StorageValue {
        let mut val = [0; 32];
        let address = &key.address().0;
        let slot = &key.key().0;
        self.get_storage.call_mut((address, slot, &mut val));
        StorageValue::from_slice(&val)
    }

    fn is_write_initial(&mut self, key: &StorageKey) -> bool {
        todo!()
    }

    fn load_factory_dep(&mut self, hash: H256) -> Option<Vec<u8>> {
        todo!()
    }

    fn get_enumeration_index(&mut self, key: &StorageKey) -> Option<u64> {
        todo!()
    }
}