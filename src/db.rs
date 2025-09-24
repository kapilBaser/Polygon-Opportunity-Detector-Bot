use anyhow::Result;
use rusqlite::Connection;

pub fn init_db() -> Result<()> {
    let con = Connection::open("table.db")?;
    con.execute(
        "CREATE TABLE IF NOT EXISTS arbitrage_opportunities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            buy_dex TEXT,
            sell_dex TEXT,
            profit_usdc REAL,
            timestamp TEXT
        )",
        (), // No parameters
    )?;
    println!("Database and table created!");
    Ok(())
}