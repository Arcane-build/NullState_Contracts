mod utils;

use fuels::prelude::AssetId;

// These constants should match those hard-coded in the predicate
const ASK_AMOUNT: u64 = 42;
const ASK_ASSET: AssetId = AssetId::new([1u8; 32]);
const RECEIVER: &str = "fuel1p8qt95dysmzrn2rmewntg6n6rg3l8ztueqafg5s6jmd9cgautrdslwdqdw";


//outlined tests
mod success {

    use super::*;

    #[tokio::test]
    async fn valid_predicate_spend_with_swap() {
    }
    #[tokio::test]
    async fn owner_recover_funds() {
    }
}

mod revert {

    use super::*;
    #[tokio::test]
    #[should_panic]
    async fn incorrect_ask_amount() {
    }

    #[tokio::test]
    #[should_panic]
    async fn incorrect_ask_asset() {
    }

    #[tokio::test]
    #[should_panic]
    async fn incorrect_receiver_address() {
        
    }

    #[tokio::test]
    #[should_panic]
    async fn incorrect_owner_recover_funds() {
    }
}