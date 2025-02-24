mod interface;
mod setupnft;
use crate::utils::{
    interface::{constructor, mint},
    setupnft::{defaults, get_wallet_balance, setup},
};
use fuels::{
    accounts::{predicate::Predicate, Account},
    prelude::{abigen, Address, AssetId, Bech32Address, Provider, TxPolicies},
    types::{
        output::Output,
        transaction_builders::{
            BuildableTransaction, ScriptTransactionBuilder, TransactionBuilder,
        },
    },
};

abigen!(Predicate(
    name = "MyPredicate",
    abi = "../nft_fixed_price_swap_predicate/out/debug/nft_fixed_price_swap_predicate-abi.json"
));

const ASK_ASSET: AssetId = AssetId::new([1u8; 32]);
const BASE_ASSET: AssetId = AssetId::new([0u8; 32]);
const PREDICATE_BINARY: &str =
    "../nft_fixed_price_swap_predicate/out/debug/nft_fixed_price_swap_predicate.bin";

// Get the balance of a given asset of an address
async fn get_balance(provider: &Provider, address: &Bech32Address, asset: AssetId) -> u64 {
    provider.get_asset_balance(address, asset).await.unwrap()
}

/// Tests that the predicate can be spent. Parameterized by test cases
pub async fn test_predicate_spend_with_parameters(
    ask_amount: u64,
    asked_asset: AssetId,
    receiver: &str,
    fee_amount: u64,
    fee_asset: AssetId,
    treasury: &str,
) {
    let receiver_address = receiver.parse().unwrap();

    let (wallet1, wallet2, id, instance_1, _instance_2) = setup(asked_asset).await;
    let (asset_id_1, sub_id_1, owner_identity, other_identity) =
        defaults(id, wallet1, wallet2.clone());

    let receiver_wallet = &wallet1;
    let taker_wallet = &wallet2;

    constructor(&instance_1, owner_identity).await;
    mint(&instance_1, owner_identity, sub_id_1, 1).await;

    assert_eq!(get_wallet_balance(receiver_wallet, &asset_id_1).await, 1);

    let provider = receiver_wallet.provider().unwrap();

    let initial_taker_offered_asset_balance =
        get_balance(provider, taker_wallet.address(), asset_id_1).await;
    let initial_taker_asked_asset_balance =
        get_balance(provider, taker_wallet.address(), asked_asset).await;
    let initial_receiver_balance = get_balance(provider, &receiver_address, asked_asset).await;

    let predicate = Predicate::load_from(PREDICATE_BINARY)
        .unwrap()
        .with_provider(provider.clone());

    receiver_wallet
        .transfer(predicate.address(), 1, asset_id_1, TxPolicies::default())
        .await
        .unwrap();

    // The predicate root has received the coin
    assert_eq!(
        get_balance(provider, predicate.address(), asset_id_1).await,
        1
    );

    // Configure inputs and outputs to send coins from the predicate root to another address
    // The predicate allows to spend its assets if `ask_amount` is sent to the receiver.

    // Get predicate input
    let input_predicate = predicate
        .get_asset_inputs_for_amount(asset_id_1, 1)
        .await
        .unwrap()[0]
        .clone();

    // Get input from taker
    let input_from_taker = taker_wallet
        .get_asset_inputs_for_amount(asked_asset, 1)
        .await
        .unwrap()[0]
        .clone();

    // Output for the asked coin transferred from the taker to the receiver
    let output_to_receiver = Output::Coin {
        to: Address::from(receiver_address.clone()),
        amount: ask_amount,
        asset_id: asked_asset,
    };

    //output to treasury for fee coin
    let output_to_receiver = Output::Coin {
        to: Address::from(receiver_address.clone()),
        amount: fee_amount,
        asset_id: fee_asset,
    };

    // Output for the offered coin transferred from the predicate to the order taker
    let output_to_taker = Output::Coin {
        to: Address::from(taker_wallet.address()),
        amount: 1,
        asset_id: asset_id_1,
    };

    // Change output for unspent asked asset
    let output_asked_change = Output::Change {
        to: Address::from(taker_wallet.address()),
        amount: 0,
        asset_id: asked_asset,
    };

    let mut tb = ScriptTransactionBuilder::prepare_transfer(
        vec![input_predicate, input_from_taker],
        vec![output_to_receiver, output_to_taker, output_asked_change],
        TxPolicies::default(),
    );
    tb.add_signer(taker_wallet.clone()).unwrap();
    let tx = tb.build(provider).await.unwrap();

    let _tx_status = provider
        .send_transaction_and_await_commit(tx)
        .await
        .unwrap();

    let predicate_balance = get_balance(provider, predicate.address(), asset_id_1).await;
    let taker_asked_asset_balance =
        get_balance(provider, taker_wallet.address(), asked_asset).await;
    let taker_offered_asset_balance =
        get_balance(provider, taker_wallet.address(), asset_id_1).await;
    let receiver_balance = get_balance(provider, &receiver_address, asked_asset).await;

    // The predicate root's coin has been spent
    assert_eq!(predicate_balance, 0);

    // Receiver has been paid `ask_amount`
    assert_eq!(receiver_balance, initial_receiver_balance + ask_amount);

    // Taker has sent `ask_amount` of the asked asset and received `offered_amount` of the offered asset in return
    assert_eq!(
        taker_asked_asset_balance,
        initial_taker_asked_asset_balance - ask_amount
    );
    assert_eq!(taker_offered_asset_balance, 1);
}

// Tests that the predicate can be recovered by the owner
// `correct_owner` is a boolean flag to set in order to test passing and failing conditions
pub async fn recover_predicate_as_owner(correct_owner: bool) {
    let (wallet1, wallet2, id, instance_1, _instance_2) = setup(ASK_ASSET).await;
    let (asset_id_1, sub_id_1, owner_identity, other_identity) =
        defaults(id, wallet1, wallet2.clone());

    let wallet = match correct_owner {
        true => &wallet1,
        false => &wallet2,
    };

    let provider = wallet.provider().unwrap();

    let initial_wallet_balance = get_balance(provider, wallet.address(), asset_id_1).await;

    let predicate = Predicate::load_from(PREDICATE_BINARY)
        .unwrap()
        .with_configurables(
            SwapPredicateConfigurables::default()
                .with_RECEIVER(wallet1.address().into())
                .unwrap(),
        )
        .with_provider(provider.clone());

    // Transfer some coins to the predicate root
    let offered_amount = 1000;
    wallet
        .transfer(
            &predicate.address().clone(),
            offered_amount,
            asset_id_1,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Get predicate input
    let input_predicate = predicate
        .get_asset_inputs_for_amount(asset_id_1, 1)
        .await
        .unwrap()[0]
        .clone();

    // Get input from wallet
    let input_from_taker = wallet
        .get_asset_inputs_for_amount(BASE_ASSET, 1)
        .await
        .unwrap()[0]
        .clone();

    let output_to_receiver = Output::Coin {
        to: Address::from(wallet.address()),
        amount: 1,
        asset_id: asset_id_1,
    };

    // Use a change output to send the unlocked coins back to the wallet
    let output_offered_change = Output::Change {
        to: Address::from(wallet.address()),
        amount: 0,
        asset_id: asset_id_1,
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
    let predicate_balance = get_balance(provider, predicate.address(), asset_id_1).await;
    assert_eq!(predicate_balance, 0);

    // Wallet balance is the same as before it sent the coins to the predicate
    let wallet_balance = get_balance(provider, wallet.address(), asset_id_1).await;
    assert_eq!(wallet_balance, initial_wallet_balance);
}
