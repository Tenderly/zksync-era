use std::fmt::{Debug, Formatter};
use std::panic;
use std::panic::{AssertUnwindSafe};
use multivm::interface::{L1BatchEnv, L2BlockEnv, SystemEnv, TxExecutionMode, VmExecutionMode, VmExecutionResultAndLogs, VmInterface};
use multivm::interface::ExecutionResult::Success;
use multivm::vm_latest::constants::BLOCK_GAS_LIMIT;
use multivm::vm_latest::HistoryEnabled;
use multivm::VmInstance;
use multivm::zk_evm_latest::ethereum_types::{H256, U256};
use zksync_state::{ReadStorage, StorageView};
use zksync_types::{Address, Execute, L1BatchNumber, L2ChainId, L2TxCommonData, Nonce, PackedEthSignature, ProtocolVersionId, StorageKey, StorageValue, Transaction};
use tenderly_cffi::{GetBalanceFunc, GetCodeByHashFunc, GetCodeFunc, GetCodeHashFunc, GetCodeLengthByHashFunc, GetCodeLengthFunc, GetNonceFunc, GetStorageFunc, TransactionExecutor};
use zksync_contracts::BaseSystemContracts;
use zksync_types::ExecuteTransactionCommon::L2;
use zksync_types::fee::Fee;
use zksync_types::fee_model::BatchFeeInput::L1Pegged;
use zksync_types::fee_model::L1PeggedBatchFeeModelInput;
use zksync_types::l2::TransactionType;
use zksync_types::transaction_request::PaymasterParams;

#[derive(Debug)]
pub struct TransactionExecutorImpl {
    nonce: Nonce,
    fee: Fee,
    from: Address,
    to: Address,
    r: H256,
    s: H256,
    v: u8,
    transaction_type: TransactionType,
    paymaster_params: PaymasterParams,
    calldata: Vec<u8>,
    value: U256,
    factory_deps: Option<Vec<Vec<u8>>>,

    block_number: u32,
    block_timestamp: u64,
    block_parent_hash: H256,

    batch_number: u32,
    batch_timestamp: u64,
    batch_l1_gas_price: u64,
    batch_l2_fair_gas_price: u64,
    batch_parent_hash: Option<H256>,

    execution_result: VmExecutionResultAndLogs,

    storage: DataProvider
}

struct DataProvider {
    get_storage: GetStorageFunc,
    get_code: GetCodeFunc,
    get_code_length: GetCodeLengthFunc,
    get_code_hash: GetCodeHashFunc,
    get_balance: GetBalanceFunc,
    get_nonce: GetNonceFunc,
    get_code_by_hash: GetCodeByHashFunc,
    get_code_length_by_hash: GetCodeLengthByHashFunc,
}

impl TransactionExecutorImpl {
    fn new() -> TransactionExecutorImpl {
        TransactionExecutorImpl {
            nonce: Nonce(0),
            fee: Default::default(),
            from: Default::default(),
            to: Default::default(),
            r: Default::default(),
            s: Default::default(),
            v: 0,
            transaction_type: TransactionType::LegacyTransaction,
            paymaster_params: Default::default(),
            calldata: vec![],
            value: Default::default(),
            factory_deps: None,
            block_number: 0,
            block_timestamp: 0,
            block_parent_hash: Default::default(),
            batch_number: 0,
            batch_timestamp: 0,
            batch_l1_gas_price: 0,
            batch_l2_fair_gas_price: 0,
            batch_parent_hash: Default::default(),
            execution_result: VmExecutionResultAndLogs {
                result: Success { output: vec![] },
                logs: Default::default(),
                statistics: Default::default(),
                refunds: Default::default(),
            },
            storage: Default::default(),
        }
    }

    fn l1_batch_env(&self) -> L1BatchEnv {
        L1BatchEnv {
            previous_batch_hash: self.batch_parent_hash, // OK
            number: L1BatchNumber::from(self.batch_number), // OK
            timestamp: self.batch_timestamp, // OK
            fee_input: L1Pegged(L1PeggedBatchFeeModelInput {
                fair_l2_gas_price: self.batch_l2_fair_gas_price,
                l1_gas_price: self.batch_l1_gas_price
            }), // OK
            fee_account: Default::default(), // TODO
            enforced_base_fee: None, // TODO
            first_l2_block: L2BlockEnv {
                number: self.block_number, // OK
                timestamp: self.block_timestamp, // OK
                prev_block_hash: self.block_parent_hash, // OK
                max_virtual_blocks_to_create: 0, // TODO
            },
        }
    }

    fn system_env(&self) -> SystemEnv {
        SystemEnv {
            zk_porter_available: false, // Probably OK
            version: ProtocolVersionId::Version22, // TODO
            base_system_smart_contracts: BaseSystemContracts::load_from_disk(), // TODO: probably okay
            gas_limit: BLOCK_GAS_LIMIT, // TODO: probably okay
            execution_mode: TxExecutionMode::EthCall, // TODO: probably okay
            default_validation_computational_gas_limit: BLOCK_GAS_LIMIT, // TODO: probably okay
            chain_id: L2ChainId::from(324), // OK, maybe take as argument because of testnets / rollups
        }
    }

    fn transaction(&self) -> Transaction {
        Transaction {
            common_data: L2(L2TxCommonData {
                nonce: self.nonce, // OK
                fee: self.fee.clone(), // OK
                initiator_address: self.from, // OK
                signature: PackedEthSignature::from_rsv(&self.r, &self.s, self.v).serialize_packed().to_vec(), // TODO check if this is okay
                transaction_type: self.transaction_type, // OK
                input: None, // TODO check if needed, not to be confused with calldata
                paymaster_params: self.paymaster_params.clone(), // TODO
            }),
            execute: Execute{
                contract_address: self.to, // OK
                calldata: self.calldata.clone(), // OK
                value: self.value, // Should be OK
                factory_deps: self.factory_deps.clone(), // TODO
            },
            received_timestamp_ms: 0, // TODO
            raw_bytes: None, // Should be OK
        }
    }
}

impl Default for TransactionExecutorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionExecutor for TransactionExecutorImpl {
    fn set_block_number(&mut self, _value: u64) { self.block_number = _value as u32; }
    fn set_block_coinbase(&mut self, _value: &[u8]) {}
    fn set_block_gas_limit(&mut self, _value: u64) { self.fee.gas_limit = U256::from(_value); }
    fn set_block_timestamp(&mut self, _value: u64) { self.block_timestamp = _value; }
    fn set_block_difficulty(&mut self, _value: &[u8]) {}
    fn set_block_base_fee(&mut self, _value: &[u8]) {}
    fn set_block_prevrandao(&mut self, _value: &[u8]) {}
    fn set_block_excess_blob_gas(&mut self, _value: u64) {}
    fn set_block_parent_hash(&mut self, _value: &[u8]) { self.block_parent_hash.assign_from_slice(_value); }

    fn set_tx_hash(&mut self, _value: &[u8]) {}
    fn set_tx_from(&mut self, _value: &[u8]) { self.from = Address::from_slice(_value); }
    fn set_tx_to(&mut self, _value: &[u8]) { self.to = Address::from_slice(_value); }
    fn set_tx_nonce(&mut self, _value: u64) { self.nonce = Nonce(_value as u32); }
    fn set_tx_value(&mut self, _value: &[u8]) { self.value = U256::from(_value);}
    fn set_tx_gas_limit(&mut self, _value: &[u8]) { self.fee.gas_limit = U256::from(_value); }
    fn set_tx_gas_price(&mut self, _value: &[u8]) {}
    fn set_tx_fee_cap(&mut self, _value: &[u8]) { self.fee.max_fee_per_gas = U256::from(_value);}
    fn set_tx_tip(&mut self, _value: &[u8]) {}
    fn set_tx_max_fee_per_blob_gas(&mut self, _value: &[u8]) {}
    fn set_tx_data(&mut self, _value: &[u8]) { self.calldata = _value.to_vec(); }
    fn set_tx_access_list(&mut self, _value: &[u8]) {}
    fn set_tx_blob_hashes(&mut self, _value: &[u8]) {}
    fn set_tx_r(&mut self, _value: &[u8]) { self.r.assign_from_slice(_value); }
    fn set_tx_s(&mut self, _value: &[u8]) { self.s.assign_from_slice(_value); }
    fn set_tx_v(&mut self, _value: u64) { self.v = _value as u8; }
    fn set_tx_type(&mut self, _value: u64) { self.transaction_type = TransactionType::try_from(_value as u32).unwrap(); }
    fn set_tx_max_fee_per_gas(&mut self, _value: &[u8]) { self.fee.max_fee_per_gas = U256::from(_value); }
    fn set_tx_max_priority_fee_per_gas(&mut self, _value: &[u8]) { self.fee.max_priority_fee_per_gas = U256::from(_value); }
    fn set_tx_gas_per_pubdata(&mut self, _value: &[u8]) { self.fee.gas_per_pubdata_limit = U256::from(_value); }

    fn set_opt_check_nonce(&mut self, _value: bool) {}
    fn set_opt_no_base_fee(&mut self, _value: bool) {}

    fn set_env_get_nonce(&mut self, _value: GetNonceFunc) { self.storage.get_nonce = _value; }
    fn set_env_get_balance(&mut self, _value: GetBalanceFunc) { self.storage.get_balance = _value; }
    fn set_env_get_code_hash(&mut self, _value: GetCodeHashFunc) { self.storage.get_code_hash = _value; }
    fn set_env_get_code_length(&mut self, _value: GetCodeLengthFunc) { self.storage.get_code_length = _value; }
    fn set_env_get_code(&mut self, _value: GetCodeFunc) { self.storage.get_code = _value; }
    fn set_env_get_storage(&mut self, _value: GetStorageFunc) { self.storage.get_storage = _value; }
    fn set_env_get_code_by_hash(&mut self, _value: GetCodeByHashFunc) { self.storage.get_code_by_hash = _value; }
    fn set_env_get_code_length_by_hash(&mut self, _value: GetCodeLengthByHashFunc) { self.storage.get_code_length_by_hash = _value; }

    fn set_batch_number(&mut self, _value: u64) { self.batch_number = _value as u32; }
    fn set_batch_timestamp(&mut self, _value: u64) { self.batch_timestamp = _value; }
    fn set_batch_l1_gas_price(&mut self, _value: u64) { self.batch_l1_gas_price = _value; }
    fn set_batch_l2_fair_gas_price(&mut self, _value: u64) { self.batch_l2_fair_gas_price = _value; }
    fn set_batch_parent_hash(&mut self, _value: &[u8]) { self.batch_parent_hash = Some(H256::from_slice(_value)); }

    fn execute(&mut self) {
        let l1_batch_env = self.l1_batch_env();
        let system_env = self.system_env();
        let transaction = self.transaction();

        let panic_result = panic::catch_unwind(AssertUnwindSafe(|| {
            let storage_ptr = StorageView::new(&mut self.storage).to_rc_ptr();

            let mut vm: VmInstance<StorageView<&mut DataProvider>, HistoryEnabled> = VmInstance::new(l1_batch_env, system_env, storage_ptr);
            vm.push_transaction(transaction);
            vm.execute(VmExecutionMode::OneTx)
        }));

        match panic_result {
            Ok(vm_execution_result) => { self.execution_result = vm_execution_result; }
            Err(_) => {}
        }
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

impl Default for DataProvider {
    fn default() -> Self {
        DataProvider {
            get_storage: Box::new(|_, _, _| {}),
            get_code: Box::new(|_, _| {}),
            get_code_length: Box::new(|_| 0),
            get_code_hash: Box::new(|_, _| {}),
            get_balance: Box::new(|_, _| {}),
            get_nonce: Box::new(|_| 0),
            get_code_by_hash: Box::new(|_, _| {}),
            get_code_length_by_hash: Box::new(|_| 0),
        }
    }
}

impl Debug for DataProvider {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl ReadStorage for &mut DataProvider {
    fn read_value(&mut self, key: &StorageKey) -> StorageValue {
        let mut val = [0; 32];
        let address = &key.address().0;
        let slot = &key.key().0;
        self.get_storage.call_mut((address, slot, &mut val));
        StorageValue::from_slice(&val)
    }

    fn is_write_initial(&mut self, _key: &StorageKey) -> bool {
        todo!("is_write_initial not implemented")
    }

    fn load_factory_dep(&mut self, hash: H256) -> Option<Vec<u8>> {
        let bytecode_size = u16::from_be_bytes([hash[2], hash[3]]) * 32;
        // println!("Calculated bytecode size: {}", bytecode_size);
        // let bytecode_size = self.get_code_length_by_hash.call_mut((hash.as_fixed_bytes(),));
        // println!("Fetched bytecode size: {}", bytecode_size);
        let mut bytecode = vec![0u8; bytecode_size as usize];
        self.get_code_by_hash.call_mut((hash.as_fixed_bytes(), bytecode.as_mut_slice()));
        Some(bytecode)
    }

    fn get_enumeration_index(&mut self, _key: &StorageKey) -> Option<u64> {
        todo!("get_enumeration_index not implemented")
    }
}