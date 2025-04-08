use ethers::providers::{Provider, Http};
use std::env;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok(); // Load environment variables from .env

    let rpc_url = env::var("ETH_RPC_URL")?;
    let provider = Provider::<Http>::try_from(rpc_url)?;

    let block_number = 17000000u64; // You can change this to any recent block
    let block = provider.get_block(block_number).await?;

    match block {
        Some(b) => {
            println!("Block {}:\n{:#?}", block_number, b);
        }
        None => {
            println!("Block {} not found", block_number);
        }
    }

    Ok(())
}
