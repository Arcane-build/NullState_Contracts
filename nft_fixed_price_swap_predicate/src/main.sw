predicate;

use std::{
    inputs::{
        input_coin_owner,
        input_count,
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
    FEE_AMOUNT: u64 = 2,
    FEE_ASSET: AssetId = AssetId::from(0x0101010101010101010101010101010101010101010101010101010101010101),
    TREASURY_ADDRESS: Address = Address::from(0x09c0b2d1a486c439a87bcba6b46a7a1a23f3897cc83a94521a96da5c23bc58db),
    ASK_AMOUNT: u64 = 42,
    ASK_ASSET: AssetId = AssetId::from(0x0101010101010101010101010101010101010101010101010101010101010101),
    RECEIVER: Address = Address::from(0x09c0b2d1a486c439a87bcba6b46a7a1a23f3897cc83a94521a96da5c23bc58db),
}

fn main() -> bool {
    // Allow cancellation by receiver if they provide input coins
    if input_count() == 2 {
        return match (input_coin_owner(0), input_coin_owner(1)) {
            (Some(owner1), Some(owner2)) => owner1 == RECEIVER || owner2 == RECEIVER,
            _ => false,
        }
    }

    // Validate output configuration
    if output_count() != 2 {
        return false
    }

    // Ensure both outputs are Coin type
    match (output_type(0), output_type(1)) {
        (Output::Coin, Output::Coin) => (),
        _ => return false,
    };

    let output1 = get_output_details(0);
    let output2 = get_output_details(1);

    match (output1, output2) {
        (Some((to1, asset1, amt1)), Some((to2, asset2, amt2))) => {
            // Check both possible output orderings
            let valid_case = to1 == RECEIVER && asset1 == ASK_ASSET && amt1 == ASK_AMOUNT && to2 == TREASURY_ADDRESS && asset2 == FEE_ASSET && amt2 == FEE_AMOUNT;

            valid_case
        },
        _ => false,
    }
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

    let amount = output_amount(output_index);

    Some((to, asset_id, amount))
}
