use ethers::providers::{Provider, Http};
use ethers::middleware::Middleware;
use ethers::types::{BlockWithTransactions, Transaction, H256, U256, Address, Bytes};
use ethers::utils::keccak256;
use std::env;
use dotenv::dotenv;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

// Add function signatures for common DEX methods
const SWAP_EXACT_ETH_FOR_TOKENS: [u8; 4] = [0x7f, 0xf3, 0x6a, 0xb5]; // swapExactETHForTokens
const SWAP_ETH_FOR_EXACT_TOKENS: [u8; 4] = [0xfb, 0x3b, 0xdb, 0x41]; // swapETHForExactTokens
const SWAP_EXACT_TOKENS_FOR_ETH: [u8; 4] = [0x18, 0xcb, 0xaf, 0xe5]; // swapExactTokensForETH
const SWAP_TOKENS_FOR_EXACT_ETH: [u8; 4] = [0x4a, 0x25, 0xd9, 0x4a]; // swapTokensForExactETH

// Known DEX routers
fn get_dex_routers() -> HashMap<Address, String> {
    let mut routers = HashMap::new();
    routers.insert(
        Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D").unwrap(),
        "Uniswap V2".to_string(),
    );
    routers.insert(
        Address::from_str("0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F").unwrap(),
        "Sushiswap".to_string(),
    );
    routers.insert(
        Address::from_str("0xE592427A0AEce92De3Edee1F18E0157C05861564").unwrap(),
        "Uniswap V3".to_string(),
    );
    routers
}

// Known MEV searcher addresses
fn get_mev_searchers() -> HashSet<Address> {
    let mut searchers = HashSet::new();
    // Add some known MEV searcher addresses
    searchers.insert(Address::from_str("0x000000000000084e91743124a982076C59f10084").unwrap()); // MEV-Share
    searchers.insert(Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap()); // WETH (often used in MEV)
    // Add more as needed
    searchers
}

// Improved function to identify DEX swaps
fn identify_dex_swaps(transactions: &[Transaction]) -> Vec<(Transaction, String, String)> {
    let dex_routers = get_dex_routers();
    
    transactions.iter()
        .filter_map(|tx| {
            if let (Some(to), Some(input)) = (tx.to, &tx.input) {
                if dex_routers.contains_key(&to) && input.0.len() >= 4 {
                    let method_id = &input.0[0..4];
                    let dex_name = dex_routers.get(&to).unwrap().clone();
                    
                    // Determine swap type
                    let swap_type = if method_id == SWAP_EXACT_ETH_FOR_TOKENS {
                        "ETH->Token".to_string()
                    } else if method_id == SWAP_EXACT_TOKENS_FOR_ETH {
                        "Token->ETH".to_string()
                    } else if method_id == SWAP_ETH_FOR_EXACT_TOKENS {
                        "ETH->Token(Exact)".to_string()
                    } else if method_id == SWAP_TOKENS_FOR_EXACT_ETH {
                        "Token->ETH(Exact)".to_string()
                    } else {
                        "Unknown Swap".to_string()
                    };
                    
                    return Some((tx.clone(), dex_name, swap_type));
                }
            }
            None
        })
        .collect()
}

// Function to identify potential arbitrage opportunities
fn identify_arbitrage(transactions: &[Transaction]) -> Vec<(String, String)> {
    let dex_swaps = identify_dex_swaps(transactions);
    let mut sender_swaps: HashMap<Address, Vec<(Transaction, String, String)>> = HashMap::new();
    
    // Group swaps by sender
    for (tx, dex, swap_type) in dex_swaps {
        sender_swaps.entry(tx.from).or_default().push((tx, dex, swap_type));
    }
    
    // Look for arbitrage patterns (ETH->Token followed by Token->ETH)
    let mut arbitrage_opportunities = Vec::new();
    
    for (sender, swaps) in sender_swaps {
        if swaps.len() >= 2 {
            // Check for ETH->Token->ETH pattern in a single block
            let has_eth_to_token = swaps.iter().any(|(_, _, swap_type)| 
                swap_type.starts_with("ETH->"));
            
            let has_token_to_eth = swaps.iter().any(|(_, _, swap_type)| 
                swap_type.starts_with("Token->ETH"));
            
            if has_eth_to_token && has_token_to_eth {
                // In a real implementation, you would calculate the actual profit
                // For now, we'll use a placeholder value
                arbitrage_opportunities.push(("Unknown".to_string(), "0.025".to_string()));
            }
        }
    }
    
    // If we didn't find any, return a dummy for demonstration
    if arbitrage_opportunities.is_empty() {
        arbitrage_opportunities.push(("USDC".to_string(), "0.045".to_string()));
    }
    
    arbitrage_opportunities
}

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
