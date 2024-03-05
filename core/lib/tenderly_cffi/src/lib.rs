use std::{
    ffi::c_void,
    ops::{Deref, DerefMut},
};

mod capi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

type GetNonceFn = extern "C" fn(address: *const u8, data: *const c_void) -> u64;
type GetBalanceFn = extern "C" fn(address: *const u8, result: *mut u8, data: *const c_void);
type GetCodeHashFn = extern "C" fn(address: *const u8, result: *mut u8, data: *const c_void);
type GetCodeLengthFn = extern "C" fn(address: *const u8, data: *const c_void) -> u64;
type GetCodeFn = extern "C" fn(address: *const u8, result: *mut u8, data: *const c_void);
type GetStorageFn =
    extern "C" fn(address: *const u8, key: *const u8, result: *mut u8, data: *const c_void);

pub type Address = [u8; 20];
pub type Hash = [u8; 32];

pub type GetNonceFunc = Box<dyn FnMut(&Address) -> u64>;
pub type GetBalanceFunc = Box<dyn FnMut(&Address, &mut Hash)>;
pub type GetCodeHashFunc = Box<dyn FnMut(&Address, &mut Hash)>;
pub type GetCodeLengthFunc = Box<dyn FnMut(&Address) -> u64>;
pub type GetCodeFunc = Box<dyn FnMut(&Address, &mut [u8])>;
pub type GetStorageFunc = Box<dyn FnMut(&Address, &Hash, &mut Hash)>;

pub trait TransactionExecutor: Default {
    fn set_block_number(&mut self, _value: u64) {}
    fn set_block_coinbase(&mut self, _value: &[u8]) {}
    fn set_block_gas_limit(&mut self, _value: u64) {}
    fn set_block_timestamp(&mut self, _value: u64) {}
    fn set_block_difficulty(&mut self, _value: &[u8]) {}
    fn set_block_base_fee(&mut self, _value: &[u8]) {}
    fn set_block_prevrandao(&mut self, _value: &[u8]) {}
    fn set_block_excess_blob_gas(&mut self, _value: u64) {}

    fn set_tx_hash(&mut self, _value: &[u8]) {}
    fn set_tx_from(&mut self, _value: &[u8]) {}
    fn set_tx_to(&mut self, _value: &[u8]) {}
    fn set_tx_nonce(&mut self, _value: u64) {}
    fn set_tx_value(&mut self, _value: &[u8]) {}
    fn set_tx_gas_limit(&mut self, _value: &[u8]) {}
    fn set_tx_gas_price(&mut self, _value: &[u8]) {}
    fn set_tx_fee_cap(&mut self, _value: &[u8]) {}
    fn set_tx_tip(&mut self, _value: &[u8]) {}
    fn set_tx_max_fee_per_blob_gas(&mut self, _value: &[u8]) {}
    fn set_tx_data(&mut self, _value: &[u8]) {}
    fn set_tx_access_list(&mut self, _value: &[u8]) {}
    fn set_tx_blob_hashes(&mut self, _value: &[u8]) {}

    fn set_opt_check_nonce(&mut self, _value: bool) {}
    fn set_opt_no_base_fee(&mut self, _value: bool) {}

    fn set_env_get_nonce(&mut self, _value: GetNonceFunc) {}
    fn set_env_get_balance(&mut self, _value: GetBalanceFunc) {}
    fn set_env_get_code_hash(&mut self, _value: GetCodeHashFunc) {}
    fn set_env_get_code_length(&mut self, _value: GetCodeLengthFunc) {}
    fn set_env_get_code(&mut self, _value: GetCodeFunc) {}
    fn set_env_get_storage(&mut self, _value: GetStorageFunc) {}

    fn execute(&mut self) {}

    fn get_used_gas(&self) -> u64 {
        0
    }
    fn get_return_data(&self) -> Vec<u8> {
        vec![]
    }

    fn close(&mut self) {}
}

pub struct TxExecImpl<T: TransactionExecutor>(T);

impl<T: TransactionExecutor> Deref for TxExecImpl<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: TransactionExecutor> DerefMut for TxExecImpl<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: TransactionExecutor> TxExecImpl<T> {
    fn new() -> Self {
        Self(T::default())
    }
}

pub fn exec_new_tx_executor<T: TransactionExecutor>() -> *mut TxExecImpl<T> {
    let tx = Box::new(TxExecImpl::new());
    Box::into_raw(tx)
}

pub fn exec_free_tx_executor<T: TransactionExecutor>(tx: *mut TxExecImpl<T>) {
    unsafe { drop(Box::from_raw(tx)) };
}

pub fn exec_tx_set_property_uint64<T: TransactionExecutor>(
    tx: *mut TxExecImpl<T>,
    property: u64,
    value: u64,
) {
    let tx = unsafe { &mut *tx };

    match property {
        capi::TX_PROPERTY_BLOCK_NUMBER => tx.set_block_number(value),
        capi::TX_PROPERTY_BLOCK_GAS_LIMIT => tx.set_block_gas_limit(value),
        capi::TX_PROPERTY_BLOCK_TIMESTAMP => tx.set_block_timestamp(value),
        capi::TX_PROPERTY_BLOCK_EXCESS_BLOB_GAS => tx.set_block_excess_blob_gas(value),
        capi::TX_PROPERTY_TX_NONCE => tx.set_tx_nonce(value),
        capi::TX_PROPERTY_TX_GAS_LIMIT => tx.set_tx_gas_limit(&value.to_be_bytes()),
        capi::TX_PROPERTY_OPT_CHECK_NONCE => tx.set_opt_check_nonce(value != 0),
        capi::TX_PROPERTY_OPT_NO_BASE_FEE => tx.set_opt_no_base_fee(value != 0),
        _ => {
            println!("Unknown property [uint64]: 0x{:x}", property);
        }
    }
}

pub fn exec_tx_set_property_data<T: TransactionExecutor>(
    tx: *mut TxExecImpl<T>,
    property: u64,
    value: *const u8,
    size: u64,
) {
    let tx = unsafe { &mut *tx };

    let slice = unsafe { std::slice::from_raw_parts(value, size as usize) };

    match property {
        capi::TX_PROPERTY_BLOCK_COINBASE => tx.set_block_coinbase(slice),
        capi::TX_PROPERTY_BLOCK_DIFFICULTY => tx.set_block_difficulty(slice),
        capi::TX_PROPERTY_BLOCK_BASE_FEE => tx.set_block_base_fee(slice),
        capi::TX_PROPERTY_BLOCK_PREVRANDAO => tx.set_block_prevrandao(slice),
        capi::TX_PROPERTY_TX_HASH => tx.set_tx_hash(slice),
        capi::TX_PROPERTY_TX_FROM => tx.set_tx_from(slice),
        capi::TX_PROPERTY_TX_TO => tx.set_tx_to(slice),
        capi::TX_PROPERTY_TX_VALUE => tx.set_tx_value(slice),
        capi::TX_PROPERTY_TX_GAS_PRICE => tx.set_tx_gas_price(slice),
        capi::TX_PROPERTY_TX_FEE_CAP => tx.set_tx_fee_cap(slice),
        capi::TX_PROPERTY_TX_TIP => tx.set_tx_tip(slice),
        capi::TX_PROPERTY_TX_MAX_FEE_PER_BLOB_GAS => tx.set_tx_max_fee_per_blob_gas(slice),
        capi::TX_PROPERTY_TX_DATA => tx.set_tx_data(slice),
        capi::TX_PROPERTY_TX_ACCESS_LIST => tx.set_tx_access_list(slice),
        capi::TX_PROPERTY_TX_BLOB_HASHES => tx.set_tx_blob_hashes(slice),
        _ => {
            println!("Unknown property [data]: 0x{:x}", property);
        }
    }
}

pub fn exec_tx_set_property_func<T: TransactionExecutor>(
    tx: *mut TxExecImpl<T>,
    property: u64,
    callback: *const c_void,
    data: *const c_void,
) {
    let tx = unsafe { &mut *tx };

    match property {
        capi::TX_PROPERTY_ENV_GET_NONCE => {
            let callback: GetNonceFn = unsafe { std::mem::transmute(callback) };
            tx.set_env_get_nonce(Box::new(move |address| callback(address.as_ptr(), data)))
        }
        capi::TX_PROPERTY_ENV_GET_BALANCE => {
            let callback: GetBalanceFn = unsafe { std::mem::transmute(callback) };
            tx.set_env_get_balance(Box::new(move |address, result| {
                callback(address.as_ptr(), result.as_mut_ptr(), data)
            }))
        }
        capi::TX_PROPERTY_ENV_GET_CODE_HASH => {
            let callback: GetCodeHashFn = unsafe { std::mem::transmute(callback) };
            tx.set_env_get_code_hash(Box::new(move |address, result| {
                callback(address.as_ptr(), result.as_mut_ptr(), data)
            }))
        }
        capi::TX_PROPERTY_ENV_GET_CODE_LENGTH => {
            let callback: GetCodeLengthFn = unsafe { std::mem::transmute(callback) };
            tx.set_env_get_code_length(Box::new(move |address| callback(address.as_ptr(), data)))
        }
        capi::TX_PROPERTY_ENV_GET_CODE => {
            let callback: GetCodeFn = unsafe { std::mem::transmute(callback) };
            tx.set_env_get_code(Box::new(move |address, result| {
                callback(address.as_ptr(), result.as_mut_ptr(), data)
            }))
        }
        capi::TX_PROPERTY_ENV_GET_STORAGE => {
            let callback: GetStorageFn = unsafe { std::mem::transmute(callback) };
            tx.set_env_get_storage(Box::new(move |address, key, result| {
                callback(address.as_ptr(), key.as_ptr(), result.as_mut_ptr(), data)
            }))
        }
        _ => {
            println!("Unknown property [func]: 0x{:x}", property);
        }
    }
}

pub fn exec_tx_execute<T: TransactionExecutor>(tx: *mut TxExecImpl<T>) {
    let tx = unsafe { &mut *tx };
    tx.execute();
}

pub fn exec_tx_get_output_uint64<T: TransactionExecutor>(
    tx: *mut TxExecImpl<T>,
    output: u64,
) -> u64 {
    let tx = unsafe { &mut *tx };
    match output {
        capi::TX_OUTPUT_USED_GAS => tx.get_used_gas(),
        _ => {
            println!("Unknown output [uint64]: 0x{:x}", output);
            0
        }
    }
}

pub fn exec_tx_get_output_data<T: TransactionExecutor>(
    tx: *mut TxExecImpl<T>,
    property: u64,
    data: *mut u8,
) -> u64 {
    let tx = unsafe { &mut *tx };

    let result = match property {
        capi::TX_OUTPUT_RETURN_DATA => tx.get_return_data(),
        _ => {
            println!("Unknown output [data]: 0x{:x}", property);
            return 0;
        }
    };

    if !data.is_null() {
        unsafe {
            std::ptr::copy_nonoverlapping(result.as_ptr(), data, result.len());
        }
    }

    result.len() as u64
}

#[macro_export]
macro_rules! declare_cffi {
    ($imp : ty) => {
        mod cffi {
            #[no_mangle]
            pub extern "C" fn exec_new_tx_executor() -> *mut $crate::TxExecImpl<$imp> {
                $crate::exec_new_tx_executor()
            }

            #[no_mangle]
            pub extern "C" fn exec_free_tx_executor(tx: *mut $crate::TxExecImpl<$imp>) {
                $crate::exec_free_tx_executor(tx)
            }

            #[no_mangle]
            pub extern "C" fn exec_tx_set_property_uint64(
                tx: *mut $crate::TxExecImpl<$imp>,
                property: u64,
                value: u64,
            ) {
                $crate::exec_tx_set_property_uint64(tx, property, value)
            }

            #[no_mangle]
            pub extern "C" fn exec_tx_set_property_data(
                tx: *mut $crate::TxExecImpl<$imp>,
                property: u64,
                value: *const u8,
                size: u64,
            ) {
                $crate::exec_tx_set_property_data(tx, property, value, size)
            }

            #[no_mangle]
            pub extern "C" fn exec_tx_set_property_func(
                tx: *mut $crate::TxExecImpl<$imp>,
                property: u64,
                callback: *const std::ffi::c_void,
                data: *const std::ffi::c_void,
            ) {
                $crate::exec_tx_set_property_func(tx, property, callback, data)
            }

            #[no_mangle]
            pub extern "C" fn exec_tx_execute(tx: *mut $crate::TxExecImpl<$imp>) {
                $crate::exec_tx_execute(tx)
            }

            #[no_mangle]
            pub extern "C" fn exec_tx_get_output_uint64(
                tx: *mut $crate::TxExecImpl<$imp>,
                property: u64,
            ) -> u64 {
                $crate::exec_tx_get_output_uint64(tx, property)
            }

            #[no_mangle]
            pub extern "C" fn exec_tx_get_output_data(
                tx: *mut $crate::TxExecImpl<$imp>,
                property: u64,
                data: *mut u8,
            ) -> u64 {
                $crate::exec_tx_get_output_data(tx, property, data)
            }
        }
    };
}
