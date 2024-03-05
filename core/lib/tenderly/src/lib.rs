#![feature(fn_traits)]


#[macro_use]
extern crate tenderly_cffi;
mod executor;

extern crate core;

declare_cffi!(crate::executor::TransactionExecutorImpl);