use alloy::signers::icp::IcpSigner;

use crate::KEY_NAME;


pub async fn create_icp_sepolia_signer() -> IcpSigner {
    let ecdsa_key_name = KEY_NAME.with(|key_name| key_name.borrow().clone());
    IcpSigner::new(vec![], &ecdsa_key_name, None).await.unwrap()
}

pub fn get_rpc_service_sepolia() -> EthSepoliaService {
    EthSepoliaService::new(RpcService::new(
        IcpConfig::new(Principal::from_text("7hfb6-caaaa-aaaar-qadga-cai").unwrap()),
    ))
}
