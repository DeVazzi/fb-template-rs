use ethers::prelude::*;
use ethers_flashbots::{BundleRequest, BundleTransaction};

/// Construct a Bundle Request for FlashBots
pub fn construct_bundle<T: Into<BundleTransaction>>(
    signed_transactions: Vec<T>,
    block_number: U64,
) -> eyre::Result<BundleRequest> {
    // Create the ethers-flashbots bundle request
    let mut bundle_request = BundleRequest::new();

    // Sign the transactions and add to the bundle
    for tx in signed_transactions {
        let bundled: BundleTransaction = tx.into();
        bundle_request = bundle_request.push_transaction(bundled);
    }

    // Set other bundle parameters
    bundle_request = bundle_request
        .set_block(block_number + 1)
        .set_simulation_block(block_number)
        .set_simulation_timestamp(0);
    //.set_simulation_timestamp(now.duration_since(UNIX_EPOCH)?.as_secs());
    // Return the constructed bundle request

    Ok(bundle_request)
}
