use zksync_multivm::VmInstance;

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

pub fn initVM(...) {
    VmInstance::new(l1_batch_env, system_env, storage_view.clone());
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
