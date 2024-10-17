
#[cfg(test)]
mod tests {
    use pocket_ic::PocketIc;

    use super::*;


    #[test]
    fn it_works() {
        // Create test state
        let pic = PocketIc::new();
        let canister_id = pic.create_canister();
        pic.add_cycles(canister_id, 2_000_000_000_000);

        let wasm_module = include_bytes!("../../../target/wasm32-unknown-unknown/release/eth_backend.wasm").to_vec();
        pic.install_canister(canister_id, wasm_module, Vec::new(), None);

        // Execute test functions or methods
        

        // Execute transactions on the canister

        // Assert expected results
        assert!(true);
    }
}