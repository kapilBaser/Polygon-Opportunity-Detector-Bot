use anyhow::Result;
use chrono::{Utc};
use ethers::abi::Abi;
use ethers::contract::Contract;
use ethers::core::types::{Address, U256};
use ethers::providers::{Http, Provider};
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use serde::Deserialize;
use serde_json;
use rusqlite::Connection;
use tokio::time::interval;
mod db;

#[derive(Debug, Deserialize)]
struct DexAddresses {
    quickswap_router: String,
    sushiswap_router: String,
}

#[derive(Debug, Deserialize)]
struct TokenAddresses {
    weth: String,
    usdc: String,
}

#[derive(Debug, Deserialize)]
struct Simulation {
    min_profit_threshold: f64, // In USDC
    fixed_trade_size: u64, // In wei (e.g., 1 WETH = 1e18)
    simulated_gas_cost: f64, // In USDC
    check_interval_secs: u64,
}

#[derive(Debug, Deserialize)]
struct Config {
    rpc_url: String,
    dex_addresses: DexAddresses,
    token_addresses: TokenAddresses,
    simulation: Simulation,
}

fn load_abi(path: &str) -> Result<Abi> {
    let content = fs::read_to_string(path)?;
    let abi: Abi = serde_json::from_str(&content)?;
    Ok(abi)
}

fn load_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {

    let config = load_config("config.toml")?;
    println!("config loaded successfully {:?}", config);


    let abi = load_abi("abi/uniswap_v2_router02_abi.json")?;
    println!("abi loaded successfully");

    db::init_db()?;


    let provider = Provider::<Http>::try_from(config.rpc_url.clone())?;

    let dex1_address: Address = config.dex_addresses.quickswap_router.parse()?;
    let dex2_address: Address = config.dex_addresses.sushiswap_router.parse()?;
    let weth_address: Address = config.token_addresses.weth.parse()?;
    let usdc_address: Address = config.token_addresses.usdc.parse()?;
    let trade_amount: U256 = U256::from(config.simulation.fixed_trade_size);

    let dex1_contract = Contract::new(dex1_address, abi.clone(), Arc::new(provider.clone()));
    let dex2_contract = Contract::new(dex2_address, abi.clone(), Arc::new(provider.clone()));

    println!("both dex contracts created");

    let con = Connection::open("table.db")?;
    println!("Database connected!");


    let mut interval = interval(Duration::from_secs(config.simulation.check_interval_secs));
    loop {
        interval.tick().await;

        println!("checking prices now...");


        let dex1_out = dex1_contract
            .method::<_, Vec<U256>>("getAmountsOut", (trade_amount, vec![weth_address, usdc_address]))?
            .call()
            .await
            .unwrap_or_else(|err| {
                println!("error in quickswap {:?}", err);
                vec![U256::zero(), U256::zero()]
            });


        let dex2_out = dex2_contract
            .method::<_, Vec<U256>>("getAmountsOut", (trade_amount, vec![weth_address, usdc_address]))?
            .call()
            .await
            .unwrap_or_else(|err| {
                println!("error in sushiswap {:?}", err);
                vec![U256::zero(), U256::zero()]
            });

        println!("raw output quickswap: {:?}, sushiswap: {:?}", dex1_out, dex2_out);


        let dex1_price_raw = dex1_out.get(1).cloned().unwrap_or(U256::zero());
        let dex2_price_raw = dex2_out.get(1).cloned().unwrap_or(U256::zero());

        if dex1_price_raw < U256::from(1_000_000) || dex2_price_raw < U256::from(1_000_000) {
            println!("invalid price");
            continue;
        }

        let dex1_price = dex1_price_raw.as_u128() as f64 / 1e6;
        let dex2_price = dex2_price_raw.as_u128() as f64 / 1e6;

        println!("QuickSwap price in USDC: {}, SushiSwap price in USDC: {}", dex1_price, dex2_price);

        let mut buy_dex = String::from("");
        let mut sell_dex = String::from("");

        if dex1_price_raw > dex2_price_raw {
            println!("possible buy on SushiSwap and sell on QuickSwap");
            buy_dex = "SushiSwap".to_string();
            sell_dex = "QuickSwap".to_string();
        } else if dex2_price_raw > dex1_price_raw {
            println!("possible buy on QuickSwap and sell on SushiSwap");
            buy_dex = "QuickSwap".to_string();
            sell_dex = "SushiSwap".to_string();
        } else {
            println!("both are same, no arbitrage");
        }

        let gas_cost = U256::from((config.simulation.simulated_gas_cost * 1e6) as u128);
        let diff = if dex1_price_raw > dex2_price_raw {
            dex1_price_raw - dex2_price_raw
        } else {
            dex2_price_raw - dex1_price_raw
        };
        let profit = if diff > gas_cost { diff - gas_cost } else { U256::zero() };
        let profit_in_usdc = profit.as_u128() as f64 / 1e6;

        println!("simulated profit after gas: {}", profit_in_usdc);

        if profit_in_usdc > config.simulation.min_profit_threshold {
            println!("!!!! Arbitrage found !!!! Profit = {}", profit_in_usdc);
            let timestamp = Utc::now().to_rfc3339();
            con.execute(
                "INSERT INTO arbitrage_opportunities (buy_dex, sell_dex, profit_usdc, timestamp) VALUES (?1, ?2, ?3, ?4)",
                (&buy_dex, &sell_dex, &profit_in_usdc, &timestamp),
            )?;
            println!("Saved to database!");
        } else {
            println!("profit is small, not worth it");
        }
    }
}
