use fuels::{
    accounts::ViewOnlyAccount,
    prelude::{
        abigen, launch_custom_provider_and_get_wallets, AssetConfig, Contract, ContractId,
        LoadConfiguration, TxPolicies, WalletUnlocked, WalletsConfig,
    },
    types::{Address, AssetId, Bits256, Bytes32, Identity},
};
use sha2::{Digest, Sha256};

abigen!(Contract(
    name = "NFT",
    abi = "../NFT-contract/out/debug/NFT-contract-abi.json"
),);

// The fee-paying base asset
const BASE_ASSET: AssetId = AssetId::new([0u8; 32]);
const NFT_CONTRACT_BINARY_PATH: &str = "../NFT-contract/out/debug/NFT-contract.bin";

// Create wallet config for two wallets with base, offered, and ask assets
fn configure_wallets(asked_asset: AssetId) -> WalletsConfig {
    let assets = [BASE_ASSET, asked_asset];

    WalletsConfig::new_multiple_assets(
        2,
        assets
            .map(|asset| AssetConfig {
                id: asset,
                num_coins: 1,
                coin_amount: 1_000_000_000,
            })
            .to_vec(),
    )
}

pub(crate) fn defaults(
    contract_id: ContractId,
    wallet_1: WalletUnlocked,
    wallet_2: WalletUnlocked,
) -> (
    AssetId,
    Bits256,
    Identity,
    Identity,
) {
    let sub_id_1 = Bytes32::from([1u8; 32]);
    let asset1 = get_asset_id(sub_id_1, contract_id);

    let identity_1 = Identity::Address(Address::from(wallet_1.address()));
    let identity_2 = Identity::Address(Address::from(wallet_2.address()));

    (
        asset1,
        Bits256(*sub_id_1),
        identity_1,
        identity_2,
    )
}

pub(crate) async fn setup() -> (
    WalletUnlocked,
    WalletUnlocked,
    ContractId,
    NFT<WalletUnlocked>,
) {
    let wallets =
        &launch_custom_provider_and_get_wallets(configure_wallets(asked_asset), None, None)
            .await
            .unwrap();

    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let id = Contract::load_from(NFT_CONTRACT_BINARY_PATH, LoadConfiguration::default())
        .unwrap()
        .deploy(&wallet1, TxPolicies::default())
        .await
        .unwrap();

    let instance_1 = NFT::new(id.clone(), wallet1.clone());

    (wallet1, wallet2, id.into(), instance_1)
}

pub(crate) fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
}

pub(crate) async fn get_wallet_balance(wallet: &WalletUnlocked, asset: &AssetId) -> u64 {
    wallet.get_asset_balance(asset).await.unwrap()
}