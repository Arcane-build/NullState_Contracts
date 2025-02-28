use crate::utils::setupnft::{Metadata, State, NFT};
use fuels::{
    prelude::{AssetId, CallParameters, TxPolicies, WalletUnlocked}, programs::responses::CallResponse, types::{transaction_builders::VariableOutputPolicy, Bits256, Identity}
};

pub(crate) async fn total_assets(contract: &NFT<WalletUnlocked>) -> u64 {
    contract
        .methods()
        .total_assets()
        .call()
        .await
        .unwrap()
        .value
}

pub(crate) async fn total_supply(contract: &NFT<WalletUnlocked>, asset: AssetId) -> Option<u64> {
    contract
        .methods()
        .total_supply(asset)
        .call()
        .await
        .unwrap()
        .value
}

pub(crate) async fn name(contract: &NFT<WalletUnlocked>, asset: AssetId) -> Option<String> {
    contract.methods().name(asset).call().await.unwrap().value
}

pub(crate) async fn symbol(contract: &NFT<WalletUnlocked>, asset: AssetId) -> Option<String> {
    contract.methods().symbol(asset).call().await.unwrap().value
}

pub(crate) async fn decimals(contract: &NFT<WalletUnlocked>, asset: AssetId) -> Option<u8> {
    contract
        .methods()
        .decimals(asset)
        .call()
        .await
        .unwrap()
        .value
}

pub(crate) async fn mint(
    contract: &NFT<WalletUnlocked>,
    recipient: Identity,
    sub_id: Bits256,
    amount: u64,
) -> CallResponse<()> {
    contract
        .methods()
        .mint(recipient, sub_id, amount)
        .with_variable_output_policy(VariableOutputPolicy::EstimateMinimum)
        .call()
        .await
        .unwrap()
}

pub(crate) async fn burn(
    contract: &NFT<WalletUnlocked>,
    asset_id: AssetId,
    sub_id: Bits256,
    amount: u64,
) -> CallResponse<()> {
    let call_params = CallParameters::new(amount, asset_id, 1_000_000);

    contract
        .methods()
        .burn(sub_id, amount)
        .with_tx_policies(TxPolicies::default().with_script_gas_limit(2_000_000))
        .call_params(call_params)
        .unwrap()
        .call()
        .await
        .unwrap()
}

pub(crate) async fn owner(contract: &NFT<WalletUnlocked>) -> State {
    contract.methods().owner().call().await.unwrap().value
}

pub(crate) async fn set_name(
    contract: &NFT<WalletUnlocked>,
    asset: AssetId,
    name: String,
) -> CallResponse<()> {
    contract
        .methods()
        .set_name(asset, name)
        .call()
        .await
        .unwrap()
}

pub(crate) async fn set_symbol(
    contract: &NFT<WalletUnlocked>,
    asset: AssetId,
    name: String,
) -> CallResponse<()> {
    contract
        .methods()
        .set_symbol(asset, name)
        .call()
        .await
        .unwrap()
}

pub(crate) async fn set_decimals(
    contract: &NFT<WalletUnlocked>,
    asset: AssetId,
    decimals: u8,
) -> CallResponse<()> {
    contract
        .methods()
        .set_decimals(asset, decimals)
        .call()
        .await
        .unwrap()
}

pub(crate) async fn constructor(
    contract: &NFT<WalletUnlocked>,
    owner: Identity,
) -> CallResponse<()> {
    contract.methods().constructor(owner).call().await.unwrap()
}

pub(crate) async fn metadata(
    contract: &NFT<WalletUnlocked>,
    asset: AssetId,
    key: String,
) -> Option<Metadata> {
    contract
        .methods()
        .metadata(asset, key)
        .call()
        .await
        .unwrap()
        .value
}

pub(crate) async fn set_metadata(
    contract: &NFT<WalletUnlocked>,
    asset: AssetId,
    key: String,
    metadata: Metadata,
) -> CallResponse<()> {
    contract
        .methods()
        .set_metadata(asset, key, metadata)
        .call()
        .await
        .unwrap()
}

pub(crate) async fn pause(contract: &NFT<WalletUnlocked>) -> CallResponse<()> {
    contract.methods().pause().call().await.unwrap()
}

pub(crate) async fn unpause(contract: &NFT<WalletUnlocked>) -> CallResponse<()> {
    contract.methods().unpause().call().await.unwrap()
}

pub(crate) async fn is_paused(contract: &NFT<WalletUnlocked>) -> bool {
    contract.methods().is_paused().call().await.unwrap().value
}