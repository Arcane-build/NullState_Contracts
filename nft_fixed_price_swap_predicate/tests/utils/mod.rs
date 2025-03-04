mod interface;
mod setupnft;
use fuels::{
    accounts::{predicate::Predicate, Account, ViewOnlyAccount},
    prelude::{
        abigen, launch_custom_provider_and_get_wallets, Address, AssetConfig, AssetId,
        Bech32Address, Provider, TxPolicies,
    },
    test_helpers::WalletsConfig,
    types::{
        output::Output,
        transaction_builders::{
            BuildableTransaction, ScriptTransactionBuilder, TransactionBuilder,
        },
        Bits256, Bytes32, Identity,
    },
};
use interface::{constructor, mint};
use setupnft::{get_asset_id, setup};

use crate::{ASK_AMOUNT, ASK_ASSET, FEE_AMOUNT};

abigen!(Predicate(
    name = "MyPredicate",
    abi = "./out/debug/nft_fixed_price_swap_predicate-abi.json"
));

// The fee-paying base asset
const BASE_ASSET: AssetId = AssetId::new([0u8; 32]);
// Offered asset is the asset that will be locked behind the predicate
const PREDICATE_BINARY: &str =
    "../nft_fixed_price_swap_predicate/out/debug/nft_fixed_price_swap_predicate.bin";

// Get the balance of a given asset of an address
async fn get_balance(provider: &Provider, address: &Bech32Address, asset: AssetId) -> u64 {
    provider.get_asset_balance(address, asset).await.unwrap()
}

// Create wallet config for two wallets with base, offered, and ask assets
fn configure_wallets(asked_asset: AssetId) -> WalletsConfig {
    let assets = [BASE_ASSET, asked_asset];

    WalletsConfig::new_multiple_assets(
        3,
        assets
            .map(|asset| AssetConfig {
                id: asset,
                num_coins: 1,
                coin_amount: 1_000_000_000,
            })
            .to_vec(),
    )
}

/// Tests that the predicate can be spent. Parameterized by test cases
pub async fn test_predicate_spend_with_parameters(
    ask_amount: u64,
    asked_asset: AssetId,
    receiver: &str,
    fee_amount: u64,
) {
    let receiver_address = receiver.parse().unwrap();

    let wallets =
        &launch_custom_provider_and_get_wallets(configure_wallets(asked_asset), None, None)
            .await
            .unwrap();

    let receiver_wallet = &wallets[0];
    let taker_wallet = &wallets[1];

    let (id, instance_1) = setup(receiver_wallet).await;
    let sub_id_1 = Bytes32::from([1u8; 32]);
    let offered_asset = get_asset_id(sub_id_1, id);
    let reciver_addres = Identity::Address(Address::from(receiver_wallet.address()));
    constructor(&instance_1, reciver_addres).await;
    mint(&instance_1, reciver_addres, Bits256(*sub_id_1), 1).await;

    let provider = receiver_wallet.provider().unwrap();

    let initial_taker_offered_asset_balance =
        get_balance(provider, taker_wallet.address(), offered_asset).await;
    let initial_taker_asked_asset_balance =
        get_balance(provider, taker_wallet.address(), asked_asset).await;
    let initial_receiver_balance = get_balance(provider, &receiver_address, asked_asset).await;

    let treasury_address = Address::from(wallets[2].address());
    let initial_treasury_balance =
        get_balance(provider, &treasury_address.into(), asked_asset).await;
    let predicate = Predicate::load_from(PREDICATE_BINARY)
        .unwrap()
        .with_configurables(
            MyPredicateConfigurables::default()
                .with_ASK_AMOUNT(ASK_AMOUNT)
                .unwrap()
                .with_ASK_ASSET(ASK_ASSET)
                .unwrap()
                .with_FEE_AMOUNT(FEE_AMOUNT)
                .unwrap()
                .with_TREASURY_ADDRESS(treasury_address)
                .unwrap()
                .with_FEE_ASSET(ASK_ASSET)
                .unwrap()
                .with_NFT_ASSET_ID(offered_asset)
                .unwrap()
                .with_RECEIVER(receiver_wallet.address().into())
                .unwrap(),
        )
        .with_provider(provider.clone());

    // Transfer some coins to the predicate root
    let offered_amount = 1;
    receiver_wallet
        .transfer(
            predicate.address(),
            offered_amount,
            offered_asset,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // The predicate root has received the coin
    assert_eq!(
        get_balance(provider, predicate.address(), offered_asset).await,
        offered_amount
    );

    // Configure inputs and outputs to send coins from the predicate root to another address
    // The predicate allows to spend its assets if `ask_amount` is sent to the receiver.

    // Get predicate input
    let input_predicate = predicate
        .get_asset_inputs_for_amount(offered_asset, 1, None)
        .await
        .unwrap()[0]
        .clone();

    // Get input from taker
    let input_from_taker = taker_wallet
        .get_asset_inputs_for_amount(asked_asset, 44, None)
        .await
        .unwrap()[0]
        .clone();

        let fee_input = taker_wallet
        .get_asset_inputs_for_amount(BASE_ASSET, 1, None)
        .await
        .unwrap()[0]
        .clone();

    // Output for the asked coin transferred from the taker to the receiver
    let output_to_receiver = Output::Coin {
        to: Address::from(receiver_address.clone()),
        amount: ask_amount,
        asset_id: asked_asset,
    };


    let output_to_treasury = Output::Coin {
        to: Address::from(treasury_address.clone()),
        amount: fee_amount,
        asset_id: asked_asset,
    };

    // Output for the offered coin transferred from the predicate to the order taker
    let output_to_taker = Output::Coin {
        to: Address::from(taker_wallet.address()),
        amount: offered_amount,
        asset_id: offered_asset,
    };

    // Change output for unspent asked asset
    let output_asked_change = Output::Change {
        to: Address::from(taker_wallet.address()),
        amount: 0,
        asset_id: asked_asset,
    };

    let mut tb = ScriptTransactionBuilder::prepare_transfer(
        vec![input_predicate, input_from_taker, fee_input],
        vec![
            output_to_receiver,
            output_to_treasury,
            output_to_taker,
            output_asked_change,
        ],
        TxPolicies::default(),
    ).enable_burn(true);
    tb.add_signer(taker_wallet.clone()).unwrap();
    let tx = tb.build(provider).await.unwrap();

    let _tx_status = provider
        .send_transaction_and_await_commit(tx)
        .await
        .unwrap();

    let predicate_balance = get_balance(provider, predicate.address(), offered_asset).await;
    let taker_asked_asset_balance =
        get_balance(provider, taker_wallet.address(), asked_asset).await;
    let taker_offered_asset_balance =
        get_balance(provider, taker_wallet.address(), offered_asset).await;
    let receiver_balance = get_balance(provider, &receiver_address, asked_asset).await;
    let treasury_balance = get_balance(provider, &treasury_address.into(), asked_asset).await;

    // The predicate root's coin has been spent
    assert_eq!(predicate_balance, 0);

    // Receiver has been paid `ask_amount`
    assert_eq!(receiver_balance, initial_receiver_balance + ask_amount);

    assert_eq!(treasury_balance, initial_treasury_balance + fee_amount);

    // Taker has sent `ask_amount` of the asked asset and received `offered_amount` of the offered asset in return
    assert_eq!(
        taker_asked_asset_balance,
        initial_taker_asked_asset_balance - (ask_amount + fee_amount)
    );
    assert_eq!(
        taker_offered_asset_balance,
        initial_taker_offered_asset_balance + offered_amount
    );
}

// Tests that the predicate can be recovered by the owner
// `correct_owner` is a boolean flag to set in order to test passing and failing conditions
pub async fn recover_predicate_as_owner(correct_owner: bool) {
    let wallets =
        &launch_custom_provider_and_get_wallets(configure_wallets(BASE_ASSET), None, None)
            .await
            .unwrap();

    let wallet = match correct_owner {
        true => &wallets[0],
        false => &wallets[1],
    };

    let provider = wallet.provider().unwrap();
    let (id, instance_1) = setup(wallet).await;
    let sub_id_1 = Bytes32::from([1u8; 32]);
    let offered_asset = get_asset_id(sub_id_1, id);
    let reciver_addres = Identity::Address(Address::from(wallet.address()));
    constructor(&instance_1, reciver_addres).await;
    mint(&instance_1, reciver_addres, Bits256(*sub_id_1), 1).await;

    let initial_wallet_balance = get_balance(provider, wallet.address(), offered_asset).await;

    let predicate = Predicate::load_from(PREDICATE_BINARY)
        .unwrap()
        .with_configurables(
            MyPredicateConfigurables::default()
                .with_RECEIVER(wallets[0].address().into())
                .unwrap(),
        )
        .with_provider(provider.clone());

    // Transfer some coins to the predicate root
    let offered_amount = 1;
    wallet
        .transfer(
            &predicate.address().clone(),
            offered_amount,
            offered_asset,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Get predicate input
    let input_predicate = predicate
        .get_asset_inputs_for_amount(offered_asset, 1, None)
        .await
        .unwrap()[0]
        .clone();

    // Get input from wallet
    let input_from_taker = wallet
        .get_asset_inputs_for_amount(BASE_ASSET, 1, None)
        .await
        .unwrap()[0]
        .clone();

    // Use a change output to send the unlocked coins back to the wallet
    let output_offered_change = Output::Change {
        to: Address::from(wallet.address()),
        amount: 0,
        asset_id: offered_asset,
    };

    let mut tb = ScriptTransactionBuilder::prepare_transfer(
        vec![input_predicate, input_from_taker],
        vec![output_offered_change],
        TxPolicies::default(),
    ).enable_burn(true);
    tb.add_signer(wallet.clone()).unwrap();

    let tx = tb.build(provider).await.unwrap();

    let _tx_status = provider
        .send_transaction_and_await_commit(tx)
        .await
        .unwrap();

    // The predicate root's coin has been spent
    let predicate_balance = get_balance(provider, predicate.address(), offered_asset).await;
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = get_balance(provider, wallet.address(), offered_asset).await;
    assert_eq!(wallet_balance, initial_wallet_balance);
}

pub async fn test_predicate_spend_with_wrong_output() {
    let wallets =
        &launch_custom_provider_and_get_wallets(configure_wallets(BASE_ASSET), None, None)
            .await
            .unwrap();

    let wallet = &wallets[0];

    let provider = wallet.provider().unwrap();
    let (id, instance_1) = setup(wallet).await;
    let sub_id_1 = Bytes32::from([1u8; 32]);
    let offered_asset = get_asset_id(sub_id_1, id);
    let reciver_addres = Identity::Address(Address::from(wallet.address()));
    constructor(&instance_1, reciver_addres).await;
    mint(&instance_1, reciver_addres, Bits256(*sub_id_1), 1).await;

    let initial_wallet_balance = get_balance(provider, wallet.address(), offered_asset).await;

    let predicate = Predicate::load_from(PREDICATE_BINARY)
        .unwrap()
        .with_configurables(
            MyPredicateConfigurables::default()
                .with_RECEIVER(wallets[0].address().into())
                .unwrap(),
        )
        .with_provider(provider.clone());

    // Transfer some coins to the predicate root
    let offered_amount = 1;
    wallet
        .transfer(
            &predicate.address().clone(),
            offered_amount,
            offered_asset,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Get predicate input
    let input_predicate = predicate
        .get_asset_inputs_for_amount(offered_asset, 1, None)
        .await
        .unwrap()[0]
        .clone();

    // Get input from wallet
    let input_from_taker = wallets[1]
        .get_asset_inputs_for_amount(BASE_ASSET, 1, None)
        .await
        .unwrap()[0]
        .clone();

    // Use a change output to send the unlocked coins back to the wallet
    let output_offered_change = Output::Coin {
        to: Address::from(wallets[1].address()),
        amount: 1,
        asset_id: offered_asset,
    };

    let mut tb = ScriptTransactionBuilder::prepare_transfer(
        vec![input_predicate, input_from_taker],
        vec![output_offered_change],
        TxPolicies::default(),
    );
    tb.add_signer(wallet.clone()).unwrap();

    let tx = tb.build(provider).await.unwrap();

    let _tx_status = provider
        .send_transaction_and_await_commit(tx)
        .await
        .unwrap();

    // The predicate root's coin has been spent
    let predicate_balance = get_balance(provider, predicate.address(), offered_asset).await;
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = get_balance(provider, wallet.address(), offered_asset).await;
    assert_eq!(wallet_balance, initial_wallet_balance);
}
