//! A module containing common utilities

use ethers::prelude::*;
use eyre::Result;

pub fn get_searcher_wallet() -> Result<LocalWallet> {
    let private_key = std::env::var("PK")
        .map_err(|_| eyre::eyre!("Required environment variable \"PK\" not set"))?;
    private_key
        .parse::<LocalWallet>()
        .map_err(|e| eyre::eyre!("Failed to parse private key: {:?}", e))
}
