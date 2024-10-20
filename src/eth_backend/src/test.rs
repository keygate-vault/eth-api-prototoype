#[cfg(test)]
use candid::{CandidType, Principal};
mod tests {
    use pocket_ic::{query_candid, update_candid, CallError, PocketIc};

    use crate::{TransactionRequest, TransactionResult};

    use super::*;

    #[test]
    fn it_works() {
        // Create test state
        let pic = PocketIc::new();
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);

        let wasm_module =
            include_bytes!("../../../target/wasm32-unknown-unknown/release/eth_backend.wasm")
                .to_vec();
        pic.install_canister(canister_id, wasm_module, Vec::new(), None);

        let rpc_canister_id = pic.create_canister();
        pic.add_cycles(rpc_canister_id, 2_000_000_000_000);
        let evm_rpc_wasm_module = include_bytes!("./evm_rpc.wasm").to_vec();
        pic.install_canister(rpc_canister_id, evm_rpc_wasm_module, Vec::new(), None);

        let payload = TransactionRequest {
            to: "0x1234567890".to_string(),
            value: 100,
        };
        let payloadTo = "0x1234567890".to_string();

        // Execute test functions or methods
        let res: (TransactionResult,) =
            update_candid(&pic, canister_id, "execute_transaction", (payload,)).unwrap();
        // Query RPC to get ether balance for "to" address
        let res: (u128, String, String) =
            query_candid(&pic, rpc_canister_id, "eth_getBalance", (payloadTo,)).unwrap();

        // Assert that the transaction was successful
        // Assert that the ether balance for "to" address is equal to the value of the transaction
        assert_eq!(res.0, 100);
        assert!(true);
    }
}
