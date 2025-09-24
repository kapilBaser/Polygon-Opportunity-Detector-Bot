# Polygon Arbitrage Opportunity Detector Bot

## Project Structure

```
polygon-arbitrage-bot/
├── Cargo.toml                       # Rust dependencies and project configuration
├── README.md                        # Project documentation
├── config.toml                      # Bot configuration (RPC, addresses, thresholds)
├── test.db                           # SQLite database storing arbitrage opportunities
├── src/
│   ├── main.rs                      # Main bot logic: price fetching, arbitrage detection
│   └── db.rs                        # Database connection and table creation
└── abi/
    └── uniswap_v2_router02_abi.json  # ABI for DEX routers (already included)
```

---

## Introduction

This Rust bot detects potential arbitrage opportunities on the Polygon network.
Arbitrage means identifying situations where a token (e.g., USDC, WETH, WBTC) can be bought on one DEX and sold for a higher price on another.

The bot focuses on the WETH/USDC pair and interacts with QuickSwap and SushiSwap. It periodically fetches prices, detects differences, calculates simulated profits (accounting for gas costs), and stores opportunities in a SQLite database.

---

## Goal

* Periodically fetch token pair prices from two DEXes.
* Identify profitable arbitrage opportunities exceeding a configurable threshold.
* Log and store the opportunities in a database for further analysis.

---

## Key Features

1. **Multi-DEX Price Fetching** – Queries Polygon RPC and DEX routers for current token prices.
2. **Arbitrage Detection** – Compares prices and detects profitable opportunities.
3. **Profit Simulation** – Calculates estimated profit for a fixed trade size considering gas costs.
4. **Configuration Management** – Easy setup via `config.toml` (RPC URL, DEX addresses, thresholds).
5. **Database Logging** – Saves arbitrage opportunities to `test.db` in SQLite.

---

## Technology Stack

* **Blockchain Network**: Polygon
* **DEX Interaction**: QuickSwap, SushiSwap (Uniswap V2/V3 Router ABIs)
* **Programming Language**: Rust
* **Database**: SQLite (`rusqlite`)
* **RPC Access**: Alchemy, Ankr, or any Polygon RPC endpoint

---

## Database

**File:** `test.db` (auto-created by the bot)
**Table:** `arbitrage_opportunities`

| Column       | Type    | Description                         |
| ------------ | ------- | ----------------------------------- |
| id           | INTEGER | Auto-incrementing ID                |
| buy\_dex     | TEXT    | DEX to buy from (e.g., "SushiSwap") |
| sell\_dex    | TEXT    | DEX to sell on (e.g., "QuickSwap")  |
| profit\_usdc | REAL    | Estimated profit in USDC            |
| timestamp    | TEXT    | UTC timestamp of the opportunity    |

---

## Setup Instructions

### 1. Install Rust

Install Rust from [rust-lang.org](https://www.rust-lang.org/tools/install).

### 2. Configure the Bot

* Edit `config.toml`:

  * `rpc_url`: Add your Polygon RPC URL (e.g., Alchemy: `https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY`)
  * `dex_addresses`: QuickSwap and SushiSwap router addresses.
  * `threshold`: Minimum profit to consider.
* ABI is already included in `abi/uniswap_v2_router02_abi.json`.

### 3. Run the Bot

```bash
cargo run
```

* Creates `test.db` automatically.
* Starts fetching prices every 30 seconds.
* Example console output:

```
checking prices now...
raw output quickswap: [1000000000000000000, 4147445571], sushiswap: [1000000000000000000, 4097557421]
QuickSwap price in USDC: 4147.445571, SushiSwap price in USDC: 4097.557421
possible buy on SushiSwap and sell on QuickSwap
simulated profit after gas: 44.88815
!!!! Arbitrage found !!!! Profit = 44.88815
Saved to database!
```

---

## How It Works

1. **Load Config & ABI** – Reads RPC, DEX addresses, and token info.
2. **Connect to RPC** – Establishes Polygon network connection.
3. **Setup Contracts** – Prepares DEX routers for queries.
4. **Loop** (every 30 seconds):

   * Calls `getAmountsOut` for 1 WETH → USDC on both DEXes.
   * Compares prices.
   * Calculates simulated profit (minus gas, default 5 USDC).
   * Logs and saves profitable opportunities.
5. **Database** – Saves each opportunity to `test.db`.

---

### Arbitrage Logic

* **Price Fetching**: `getAmountsOut(1 WETH, [WETH, USDC])`
* **Detection**: Buy on lower-price DEX, sell on higher-price DEX.
* **Profit Calculation**: `profit = price_diff - gas_cost`
* **Threshold Filter**: Only log if `profit > threshold`.

---

### System Architecture

```
config.toml (RPC, addresses, thresholds)
            |
            v
Rust Bot (main.rs)
            |
            v
Polygon RPC <-> DEX Routers (QuickSwap, SushiSwap)
            |
            v
getAmountsOut -> Price Comparison -> Profit Calculation
            |
            v
Console Logs & SQLite Database (test.db)
```

---

## Usage

* **Start Monitoring**: `cargo run`
* **Stop**: Ctrl+C
* **View Opportunities**: Open `test.db` with [DB Browser for SQLite](https://sqlitebrowser.org/)

  ```sql
  SELECT * FROM arbitrage_opportunities;
  ```

---

## Troubleshooting

* **"Invalid price detected"**: Low liquidity or RPC error → try different RPC or token path.
* **Database not created**: Bot automatically creates `test.db`.
* **RPC Errors**: Ensure your Alchemy API key is valid.
* **Rust Errors**: Copy the error and debug; check dependencies in `Cargo.toml`.

---
