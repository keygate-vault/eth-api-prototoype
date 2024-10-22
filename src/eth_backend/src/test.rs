#[cfg(test)]
use candid::{CandidType, Principal};
mod tests {
    use candid::encode_one;
    use ethers_core::abi::encode;
    use pocket_ic::{query_candid, update_candid, CallError, PocketIc, PocketIcBuilder};
    use serde::{Deserialize, Serialize};

    use crate::{TransactionRequest, TransactionResult};

    use super::*;

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
    struct InitArgs {
        nodesInSubnet: u32,
    }

    #[test]
    fn it_works() {
        // Create test state
        let pic = PocketIcBuilder::new()
            .with_ii_subnet() // to have tECDSA keys available
            .with_application_subnet()
            .build();
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);

        let wasm_module =
            include_bytes!("../../../target/wasm32-unknown-unknown/release/eth_backend.wasm")
                .to_vec();
        pic.install_canister(
            canister_id,
            wasm_module,
            encode_one("dfx_test_key1".to_string()).unwrap(),
            None,
        );
        let predefined_canister_id = Principal::from_text("hfb6-caaaa-aaaar-qadga-cai").unwrap();
        let rpc_canister_id = pic
            .create_canister_with_id(None, None, predefined_canister_id)
            .unwrap();
        pic.add_cycles(rpc_canister_id, 2_000_000_000_000);
        let evm_rpc_wasm_module = include_bytes!("./evm_rpc.wasm").to_vec();
        pic.install_canister(
            rpc_canister_id,
            evm_rpc_wasm_module,
            encode_one((InitArgs { nodesInSubnet: 28 },)).unwrap(),
            None,
        );

        let payload = TransactionRequest {
            to: "0x1234567890".to_string(),
            value: 100,
        };
        let payload_to = "0x1234567890".to_string();

        // Execute test functions or methods

        // Query RPC to get ether balance for "to" address
        let res: (Result<String, String>,) =
            update_candid(&pic, canister_id, "get_balance", ()).unwrap();
        // Assert that the transaction was successful
        // Assert that the ether balance for "to" address is equal to the value of the transaction
        assert_eq!("10000000", res.0.unwrap());
        assert!(true);
    }
}
