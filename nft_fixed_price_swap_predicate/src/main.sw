predicate;

use std::{
    inputs::{
        input_coin_owner,
        input_count,
        input_asset_id,
    },
    outputs::{
        Output,
        output_amount,
        output_asset_id,
        output_asset_to,
        output_count,
        output_type,
    },
};

/// configurable should be set before we deploy predicate
configurable {
    FEE_AMOUNT: u64 = 0,
    FEE_ASSET: AssetId = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
    TREASURY_ADDRESS: Address = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000),
    ASK_AMOUNT: u64 = 0,
    ASK_ASSET: AssetId = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
    RECEIVER: Address = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000),
    NFT_ASSET_ID: AssetId = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
}

/// extracts output details
fn get_output_details(output_index: u64) -> Option<(Address, AssetId, u64)> {
    let to = match output_asset_to(output_index) {
        Some(address) => address,
        None => return None,
    };

    let asset_id = match output_asset_id(output_index) {
        Some(asset_id) => asset_id,
        None => return None,
    };

    let amount = match output_amount(output_index) {
        Some(amount) => amount,
        None => return None,
    };

    Some((to, asset_id, amount))
}

fn main() -> bool {
    // Allow cancellation by receiver if they provide input coins
    if input_count() == 2 {
        match (input_coin_owner(0), input_coin_owner(1)) {
            (Some(owner1), Some(owner2)) => {
                if owner1 == RECEIVER || owner2 == RECEIVER {
                    return true;
                }
            }
            _ => return false,
        }
    }

    // validate input
    if let (Some(nft_asset), Some(ask_asset)) = (input_asset_id(0), input_asset_id(1)) {
    if ask_asset != ASK_ASSET || nft_asset != NFT_ASSET_ID {
        return false;
    }}

    // Validate output configuration
    if output_count() < 2 {
        return false
    }

    // Ensure both outputs are Coin type
    match (output_type(0), output_type(1)) {
        (Some(Output::Coin), Some(Output::Coin)) => (),
        _ => return false,
    };

    let output1 = get_output_details(0);
    let output2 = get_output_details(1);

    match (output1, output2) {
        (Some((to_reciver, ask_asset, ask_amount)), Some((to_treasury, fee_asset, fee_amount))) => {
            // Check both possible output orderings
            let valid_case = to_reciver == RECEIVER && ask_asset == ASK_ASSET && ask_amount == ASK_AMOUNT && to_treasury == TREASURY_ADDRESS && fee_asset == FEE_ASSET && fee_amount == FEE_AMOUNT;

            valid_case
        },
        _ => false,
    }

}

