use candid::{CandidType, Principal};
use ethers_core;
use serde::{Deserialize, Serialize};

use alloy::{
    network::{EthereumWallet, TxSigner}, primitives::Address, providers::{Provider, ProviderBuilder}, transports::icp::{EthSepoliaService, IcpConfig, RpcService}
};
use types::create_icp_sepolia_signer;


#[cfg(test)]
mod test;

mod types;

thread_local! {
    static KEY_NAME : std::cell::RefCell<String> = std::cell::RefCell::new("dfx_test_key".to_string());
}


static RPC_SERVICE: RpcService = RpcService::EthSepolia(EthSepoliaService::PublicNode);

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct TransactionRequest {
    to: String,
    value: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct TransactionResult {
    hash: String,
    status: String,
}

#[derive(CandidType, Serialize, Debug, Clone)]
struct EcdsaKeyId {
    pub curve: EcdsaCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

#[derive(CandidType, Serialize, Debug)]
struct ECDSAPublicKey {
    pub canister_id: Option<Principal>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct ECDSAPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
struct PublicKeyReply {
    pub public_key: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct AccountInitArgs {
    key_name: String,
}

#[ic_cdk::init]
fn init(key_name: String) {
    KEY_NAME.with_borrow_mut(|k| *k = key_name);
}

#[ic_cdk::update]
async fn pubkey_bytes_to_address() -> String {
    use ethers_core::k256::elliptic_curve::sec1::ToEncodedPoint;
    use ethers_core::k256::Secp256k1;
    let public_key = get_public_key().await.unwrap().public_key;
    let key: ethers_core::k256::elliptic_curve::PublicKey<Secp256k1> =
        ethers_core::k256::elliptic_curve::PublicKey::from_sec1_bytes(&public_key)
            .expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    // we re-encode the key to the decompressed representation.
    let point_bytes: &[u8] = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);

    let hash = ethers_core::utils::keccak256(&point_bytes[1..]);

    ethers_core::utils::to_checksum(
        &ethers_core::types::Address::from_slice(&hash[12..32]),
        None,
    )
}

#[ic_cdk::update]
async fn get_public_key() -> Result<PublicKeyReply, String> {
    let test_key_name = KEY_NAME.with_borrow(|k| k.clone());
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: test_key_name.to_string(),
    };
    let ic_canister_id = "aaaaa-aa";
    let ic = Principal::from_text(&ic_canister_id).unwrap();

    let caller = ic_cdk::caller().as_slice().to_vec();
    let request = ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![caller],
        key_id: key_id.clone(),
    };

    let (res,): (ECDSAPublicKeyReply,) = ic_cdk::call(ic, "ecdsa_public_key", (request,))
        .await
        .map_err(|e| format!("Failed to call ecdsa_public_key {}", e.1))?;
    Ok(PublicKeyReply {
        public_key: res.public_key,
    })
}

#[ic_cdk::update]
async fn execute_transaction(request: TransactionRequest) -> TransactionResult {
    let signer = create_icp_sepolia_signer().await;
    let address = signer.address();

    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_sepolia();

    TransactionResult {
        hash: "0x1234567890".to_string(),
        status: "success".to_string(),
    }
}

#[ic_cdk::update]
async fn get_balance() -> Result<String, String> {
    let address = pubkey_bytes_to_address().await;
    let address = address.parse::<Address>().map_err(|e| e.to_string())?;
    let config = IcpConfig::new(RPC_SERVICE.clone());
    let provider = ProviderBuilder::new().on_icp(config);
    let result = provider.get_balance(address).await;

    match result {
        Ok(balance) => Ok(balance.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
