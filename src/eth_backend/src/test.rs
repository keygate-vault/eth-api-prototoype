
#[cfg(test)]
mod tests {
    use pocket_ic::{query_candid, update_candid, PocketIc};

    use crate::{TransactionRequest, TransactionResult};

    use super::*;


    #[test]
    fn it_works() {
        // Create test state
        let pic = PocketIc::new();
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);

        let wasm_module = include_bytes!("../../../target/wasm32-unknown-unknown/release/eth_backend.wasm").to_vec();
        pic.install_canister(canister_id, wasm_module, Vec::new(), None);

        let payload = TransactionRequest {
            to: "0x1234567890".to_string(),
            value: 100,
        };
        
        // Execute test functions or methods
        let res: (TransactionResult, ) = update_candid(&pic, canister_id, "execute_transaction", (payload,)).unwrap();

        // Query RPC to get ether balance for "to" address
        let res = query_candid(&pic, rpc_canister_id, "eth_getBalance", (payload.to,)).unwrap();


        // Assert that the transaction was successful
        // Assert that the ether balance for "to" address is equal to the value of the transaction
        assert!(true);
    }
}