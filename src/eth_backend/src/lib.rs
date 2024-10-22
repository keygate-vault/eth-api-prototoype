use candid::{CandidType, Principal};
use ethers_core;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::U256,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::Signer,
    transports::icp::{EthSepoliaService, IcpConfig, RpcService},
};
use types::create_icp_sepolia_signer;

#[cfg(test)]
mod test;

mod types;

thread_local! {
    static KEY_NAME : std::cell::RefCell<String> = std::cell::RefCell::new("dfx_test_key".to_string());
    static NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

static RPC_SERVICE: RpcService = RpcService::EthSepolia(EthSepoliaService::PublicNode);

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
async fn pubkey_bytes_to_address() -> Result<String, String> {
    let signer = create_icp_sepolia_signer().await;
    let address = signer.address();
    Ok(address.to_string())
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
async fn execute_transaction() -> TransactionResult {
    // Setup signer
    let signer = create_icp_sepolia_signer().await;
    let address = signer.address();

    // Setup provider
    let wallet = EthereumWallet::from(signer);
    let config = IcpConfig::new(RPC_SERVICE.clone());
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);

    // Attempt to get nonce from thread-local storage
    let maybe_nonce = NONCE.with_borrow(|maybe_nonce| {
        // If a nonce exists, the next nonce to use is latest nonce + 1
        maybe_nonce.map(|nonce| nonce + 1)
    });

    // If no nonce exists, get it from the provider
    let nonce = if let Some(nonce) = maybe_nonce {
        nonce
    } else {
        provider.get_transaction_count(address).await.unwrap_or(0)
    };

    let tx = TransactionRequest::default()
        .with_to(address)
        .with_value(U256::from(100))
        .with_nonce(nonce)
        .with_gas_limit(21_000)
        .with_chain_id(11155111);

    let transport_result = provider.send_transaction(tx.clone()).await;
    match transport_result {
        Ok(builder) => {
            let node_hash = *builder.tx_hash();
            let tx_response = provider.get_transaction_by_hash(node_hash).await.unwrap();

            match tx_response {
                Some(tx) => {
                    // The transaction has been mined and included in a block, the nonce
                    // has been consumed. Save it to thread-local storage. Next transaction
                    // for this address will use a nonce that is = this nonce + 1
                    NONCE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    TransactionResult {
                        hash: format!("{:?}", tx.hash),
                        status: "Success".to_string(),
                    }
                }
                None => TransactionResult {
                    hash: String::new(),
                    status: "Failed: Could not get transaction.".to_string(),
                },
            }
        }
        Err(e) => TransactionResult {
            hash: String::new(),
            status: "Failed: cock.".to_string(),
        },
    }
}

#[ic_cdk::update]
async fn get_balance() -> Result<String, String> {
    //let address = pubkey_bytes_to_address().await;
    let address = create_icp_sepolia_signer().await.address();
    let config = IcpConfig::new(RPC_SERVICE.clone());
    let provider = ProviderBuilder::new().on_icp(config);
    let result = provider.get_balance(address).await;

    match result {
        Ok(balance) => Ok(balance.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
