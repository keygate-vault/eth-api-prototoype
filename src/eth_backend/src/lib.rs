use candid::{CandidType, Principal};
use ethers_core;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}


#[derive(CandidType, Serialize, Deserialize,Clone)]
struct TransactionRequest {
    to: String,
    value: u64,
}

#[derive(CandidType, Serialize, Deserialize,Clone)]
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
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "dfx_test_key".to_string(),
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
    TransactionResult {
        hash: "0x1234567890".to_string(),
        status: "success".to_string(),
    }
}