pub mod relayer;
pub mod utils;
use ethers::types::Eip1559TransactionRequest;
use ethers::{
    abi::Address,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider, ProviderExt, StreamExt, TransactionStream, Ws},
    signers::{LocalWallet, Signer},
    types::{transaction::eip2718::TypedTransaction, BlockId, BlockNumber, TransactionRequest},
};
use ethers_flashbots::*;
use eyre::Result;
use reqwest::Url;
use std::ops::{Add, Mul};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = std::env::var("RPC_URL").expect("RPC missing");
    let http_provider = Provider::<Http>::connect(&rpc_url).await;
    let pk = std::env::var("FB_AUTH").expect("No PK found");

    let chain_id = &http_provider.get_chainid().await.unwrap().as_u64();
    let bundle_signer = LocalWallet::from_str(&pk)
        .unwrap()
        .with_chain_id(chain_id.clone());
    let searcher_wallet = utils::get_searcher_wallet()
        .unwrap()
        .with_chain_id(chain_id.clone());
    let searcher_wallet_address = searcher_wallet.address();
    let client = SignerMiddleware::new(
        FlashbotsMiddleware::new(
            http_provider.clone(),
            Url::parse("https://relay-goerli.flashbots.net").unwrap(),
            bundle_signer.clone(),
        ),
        searcher_wallet,
    );

    let legacy_gas_price = u64::from(1e9 as u64).mul(200);
    let priority_fee = u64::from(1e9 as u64).mul(100);

    let nonce = client
        .get_transaction_count(searcher_wallet_address, None)
        .await?;
    let tx1: TypedTransaction = TransactionRequest::pay(
        Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
        1000,
    )
    .gas_price(legacy_gas_price)
    .gas(21000)
    .chain_id(chain_id.clone())
    .nonce(&nonce)
    .into();

    let tx2: TypedTransaction = Eip1559TransactionRequest::new()
        .to(Address::from_str("0x0000000000000000000000000000000000000000").unwrap())
        .value(1000)
        .gas(21000)
        .max_fee_per_gas(priority_fee.add(legacy_gas_price))
        .max_priority_fee_per_gas(priority_fee)
        .chain_id(chain_id.clone())
        .nonce(&nonce + 1)
        .into();
    let signature1 = client.signer().sign_transaction(&tx1).await?;
    let signed_tx1 = tx1.rlp_signed(&signature1);
    let signature2 = client.signer().sign_transaction(&tx2).await?;
    let signed_tx2 = tx2.rlp_signed(&signature2);
    let signed_transactions = vec![signed_tx1, signed_tx2];

    let block = match http_provider
        .clone()
        .get_block(BlockId::Number(BlockNumber::Latest))
        .await
    {
        Ok(Some(b)) => b,
        Ok(None) => {
            println!("No block found");
            return Ok(());
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Ok(());
        }
    };
    let block_number = block.number.unwrap();
    println!("block number: {:?}", &block_number);

    for x in 0..20 {
        let bundle =
            relayer::construct_bundle(signed_transactions.clone(), block_number + x).unwrap();
        let simulated_bundle = match client.inner().simulate_bundle(&bundle).await {
            Ok(sb) => sb,
            Err(e) => {
                println!("Failed to simulate bundle: {:?}", e);
                return Ok(());
            }
        };

        let pending_bundle = client.inner().send_bundle(&bundle).await?;
        match pending_bundle.await {
            Ok(bundle_hash) => {
                println!(
                    "Bundle with hash {:?} was included in target block",
                    bundle_hash
                );
                std::process::exit(0);
            }
            Err(PendingBundleError::BundleNotIncluded) => {
                println!("Bundle was not included in target block.")
            }
            Err(e) => println!("An error occured: {}", e),
        }
    }
    Ok(())
}
