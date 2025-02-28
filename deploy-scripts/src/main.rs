use fuels::{
    crypto::SecretKey,
    prelude::*,
    types::{output::Output, Bits256, Bytes32},
};
use sha2::{Digest, Sha256};

abigen!(
    Contract(
        name = "NFT",
        abi = "../NFT-contract/out/debug/NFT-contract-abi.json"
    ),
    Predicate(
        name = "MyPredicate",
        abi = "../nft_fixed_price_swap_predicate/out/debug/nft_fixed_price_swap_predicate-abi.json"
    )
);

const PREDICATE_BINARY: &str =
    "../nft_fixed_price_swap_predicate/out/debug/nft_fixed_price_swap_predicate.bin";

pub const ASSET_ID: AssetId = AssetId::new([
    0xf8, 0xf8, 0xb6, 0x28, 0x3d, 0x7f, 0xa5, 0xb6, 0x72, 0xb5, 0x30, 0xcb, 0xb8, 0x4f, 0xcc, 0xcb,
    0x4f, 0xf8, 0xdc, 0x40, 0xf8, 0x17, 0x6e, 0xf4, 0x54, 0x4d, 0xdb, 0x1f, 0x19, 0x52, 0xad, 0x07,
]);

#[tokio::main]
pub async fn main() {
    // Create a provider pointing to the testnet.
    let provider = Provider::connect("testnet.fuel.network").await.unwrap();

    let secret_key_seller: SecretKey =
        "0x862512a2363db2b3a375c0d4bbbd27172180d89f23f2e259bac850ab02619301"
            .parse()
            .unwrap();

    let secret_key_buyer: SecretKey =
        "0x37fa81c84ccd547c30c176b118d5cb892bdb113e8e80141f266519422ef9eefd"
            .parse()
            .unwrap();

    let secret_key_treasuery: SecretKey =
        "0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb"
            .parse()
            .unwrap();

    let mut wallet_seller =
        WalletUnlocked::new_from_private_key(secret_key_seller, Some(provider.clone()));
    let mut wallet_buyer =
        WalletUnlocked::new_from_private_key(secret_key_buyer, Some(provider.clone()));
    let mut wallet_treasuery =
        WalletUnlocked::new_from_private_key(secret_key_treasuery, Some(provider.clone()));

    let mut nft_contract_owner = WalletUnlocked::new_random(Some(provider.clone()));
    dbg!(wallet_buyer.get_asset_balance(&ASSET_ID).await.unwrap());

    let contract_id = Contract::load_from(
        "../NFT-contract/out/debug/NFT-contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(
        &wallet_buyer,
        TxPolicies::new(Some(1), None, None, Some(1_000_000), None),
    )
    .await
    .unwrap();

    let nft_instance = NFT::new(contract_id.clone(), wallet_buyer.clone());
    let sub_id_1 = Bytes32::from([1u8; 32]);

    nft_instance
        .methods()
        .constructor(wallet_buyer.address().into())
        .call()
        .await
        .unwrap();

    nft_instance
        .methods()
        .mint(wallet_seller.address().into(), Bits256(*sub_id_1), 1)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    let nft_asset_id = get_asset_id(sub_id_1, contract_id.into());
    let predicate = Predicate::load_from(PREDICATE_BINARY)
        .unwrap()
        .with_configurables(
            MyPredicateConfigurables::default()
                .with_ASK_AMOUNT(40)
                .unwrap()
                .with_ASK_ASSET(ASSET_ID)
                .unwrap()
                .with_FEE_AMOUNT(2)
                .unwrap()
                .with_TREASURY_ADDRESS(wallet_treasuery.address().into())
                .unwrap()
                .with_FEE_ASSET(ASSET_ID)
                .unwrap()
                .with_NFT_ASSET_ID(nft_asset_id)
                .unwrap()
                .with_RECEIVER(wallet_seller.address().into())
                .unwrap(),
        )
        .with_provider(provider.clone());

    let offered_amount = 1;
    wallet_seller
        .transfer(
            predicate.address(),
            offered_amount,
            nft_asset_id,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    assert_eq!(
        predicate.get_asset_balance(&nft_asset_id).await.unwrap(),
        offered_amount
    );
    dbg!(predicate.get_asset_balance(&nft_asset_id).await.unwrap());

    let input_predicate = predicate
        .get_asset_inputs_for_amount(nft_asset_id, 1)
        .await
        .unwrap()[0]
        .clone();

    // Get input from taker
    let input_from_taker = wallet_buyer
        .get_asset_inputs_for_amount(ASSET_ID, 1)
        .await
        .unwrap()[0]
        .clone();

    // Output for the asked coin transferred from the taker to the receiver
    let output_to_receiver = Output::Coin {
        to: Address::from(wallet_seller.address().clone()),
        amount: 40,
        asset_id: ASSET_ID,
    };

    let output_to_treasury = Output::Coin {
        to: Address::from(wallet_treasuery.address().clone()),
        amount: 2,
        asset_id: ASSET_ID,
    };

    // Output for the offered coin transferred from the predicate to the order taker
    let output_to_taker = Output::Coin {
        to: Address::from(wallet_buyer.address()),
        amount: offered_amount,
        asset_id: nft_asset_id,
    };

    // Change output for unspent asked asset
    let output_asked_change = Output::Change {
        to: Address::from(wallet_buyer.address()),
        amount: 0,
        asset_id: nft_asset_id,
    };

    let mut tb = ScriptTransactionBuilder::prepare_transfer(
        vec![input_predicate, input_from_taker],
        vec![
            output_to_receiver,
            output_to_treasury,
            output_to_taker,
            output_asked_change,
        ],
        TxPolicies::default(),
    );
    tb.add_signer(wallet_buyer.clone()).unwrap();
    let tx = tb.build(provider.clone()).await.unwrap();

    let _tx_status = provider
        .send_transaction_and_await_commit(tx)
        .await
        .unwrap();

    println!("{:?}", _tx_status);
}

pub(crate) fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
}
