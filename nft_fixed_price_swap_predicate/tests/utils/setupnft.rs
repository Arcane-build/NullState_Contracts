use fuels::{
    prelude::{
        abigen, Contract, ContractId,
        LoadConfiguration, TxPolicies, WalletUnlocked,
    },
    types::{AssetId, Bytes32},
};
use sha2::{Digest, Sha256};

abigen!(Contract(
    name = "NFT",
    abi = "../NFT-contract/out/debug/NFT-contract-abi.json"
),);

const NFT_CONTRACT_BINARY_PATH: &str = "../NFT-contract/out/debug/NFT-contract.bin";

pub(crate) async fn setup(wallet: &WalletUnlocked) -> (
    ContractId,
    NFT<WalletUnlocked>,
) {

    let id = Contract::load_from(NFT_CONTRACT_BINARY_PATH, LoadConfiguration::default())
        .unwrap()
        .deploy(wallet, TxPolicies::default())
        .await
        .unwrap();

    let instance_1 = NFT::new(id.clone(), wallet.clone());

    (id.into(), instance_1)
}

pub(crate) fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
}
