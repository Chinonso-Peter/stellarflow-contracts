#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

/// Error types for the price oracle contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Asset does not exist in the price oracle
    AssetNotFound = 1,
}

/// Price data structure containing price information for an asset
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    /// The asset symbol (e.g., "XLM", "BTC")
    pub asset: Symbol,
    /// The price value (stored as scaled integer, e.g., 1000000 = 1.00 USD)
    pub price: u64,
    /// Timestamp when the price was last updated
    pub timestamp: u64,
    /// The source/authority that provided this price
    pub source: Address,
}

/// Storage key for the price data map
const PRICE_DATA_KEY: Symbol = symbol_short!("PRICES");

#[contract]
pub struct PriceOracle;

#[contractimpl]
impl PriceOracle {
    /// Get the price data for a specific asset
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `asset` - The asset symbol to look up
    ///
    /// # Returns
    /// * `Ok(PriceData)` - The price data for the asset
    /// * `Err(Error::AssetNotFound)` - If the asset doesn't exist
    pub fn get_price(env: Env, asset: Symbol) -> Result<PriceData, Error> {
        // Get the storage instance
        let storage = env.storage().instance();

        // Try to retrieve the price data map
        let prices: soroban_sdk::Map<Symbol, PriceData> = storage
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));

        // Try to get the price for the specified asset
        match prices.get(asset) {
            Some(price_data) => Ok(price_data),
            None => Err(Error::AssetNotFound),
        }
    }

    /// Returns None instead of an error when asset is not found — safe for frontend callers.
    pub fn get_price_safe(env: Env, asset: Symbol) -> Option<PriceData> {
        let prices: soroban_sdk::Map<Symbol, PriceData> = env
            .storage()
            .instance()
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));
        prices.get(asset)
    }

    /// Returns a Vec of all currently tracked asset symbols.
    pub fn get_all_assets(env: Env) -> soroban_sdk::Vec<Symbol> {
        let prices: soroban_sdk::Map<Symbol, PriceData> = env
            .storage()
            .instance()
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));
        prices.keys()
    }

    /// Set the price data for a specific asset (admin function)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `asset` - The asset symbol
    /// * `price_data` - The price data to store
    pub fn set_price(env: Env, asset: Symbol, price_data: PriceData) {
        // Get the storage instance
        let storage = env.storage().instance();

        // Get existing prices or create new map
        let mut prices: soroban_sdk::Map<Symbol, PriceData> = storage
            .get(&PRICE_DATA_KEY)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));

        // Set the price for the asset
        prices.set(asset, price_data);

        // Store the updated map
        storage.set(&PRICE_DATA_KEY, &prices);
    }
}

mod auth;
mod median;
mod test;
