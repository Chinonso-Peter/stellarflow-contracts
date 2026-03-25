#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

fn setup() -> (Env, PriceOracleClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_get_price_existing_asset() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    // Create a test asset and price data
    let asset = symbol_short!("XLM");
    let source = Address::generate(&env);
    let price_data = PriceData {
        asset: asset.clone(),
        price: 1_000_000, // $1.00 (scaled by 1e6)
        timestamp: 1234567890,
        source: source.clone(),
    };

    // Set the price first
    client.set_price(&asset, &price_data);

    // Get the price and verify it matches (using try_get_price to get Result)
    let result = client.try_get_price(&asset);
    assert!(result.is_ok());

    let retrieved_price = result.unwrap().unwrap();
    assert_eq!(retrieved_price.asset, asset);
    assert_eq!(retrieved_price.price, 1_000_000);
    assert_eq!(retrieved_price.timestamp, 1234567890);
    assert_eq!(retrieved_price.source, source);
}

#[test]
fn test_get_price_nonexistent_asset() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    // Try to get price for an asset that doesn't exist
    let asset = symbol_short!("BTC");

    // Get the price and verify it returns an error
    let result = client.try_get_price(&asset);
    assert!(result.is_err());

    // Verify the error is AssetNotFound
    let err = result.unwrap_err().unwrap();
    assert_eq!(err, Error::AssetNotFound);
}

#[test]
fn test_get_price_multiple_assets() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    let source = Address::generate(&env);

    // Create price data for multiple assets
    let xlm_asset = symbol_short!("XLM");
    let btc_asset = symbol_short!("BTC");

    let xlm_price = PriceData {
        asset: xlm_asset.clone(),
        price: 1_000_000,
        timestamp: 1234567890,
        source: source.clone(),
    };

    let btc_price = PriceData {
        asset: btc_asset.clone(),
        price: 50_000_000_000, // $50,000 (scaled by 1e6)
        timestamp: 1234567890,
        source: source.clone(),
    };

    // Set prices for both assets
    client.set_price(&xlm_asset, &xlm_price);
    client.set_price(&btc_asset, &btc_price);

    // Verify both prices can be retrieved
    let xlm_result = client.try_get_price(&xlm_asset).unwrap().unwrap();
    assert_eq!(xlm_result.price, 1_000_000);

    let btc_result = client.try_get_price(&btc_asset).unwrap().unwrap();
    assert_eq!(btc_result.price, 50_000_000_000);
}

#[test]
fn test_get_price_after_update() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);

    let source = Address::generate(&env);
    let asset = symbol_short!("XLM");

    // Set initial price
    let initial_price = PriceData {
        asset: asset.clone(),
        price: 1_000_000,
        timestamp: 1234567890,
        source: source.clone(),
    };
    client.set_price(&asset, &initial_price);

    // Verify initial price
    let result = client.try_get_price(&asset).unwrap().unwrap();
    assert_eq!(result.price, 1_000_000);

    // Update price
    let updated_price = PriceData {
        asset: asset.clone(),
        price: 1_200_000, // Price increased to $1.20
        timestamp: 1234567900,
        source: source.clone(),
    };
    client.set_price(&asset, &updated_price);

    // Verify updated price
    let result = client.try_get_price(&asset).unwrap().unwrap();
    assert_eq!(result.price, 1_200_000);
    assert_eq!(result.timestamp, 1234567900);
}

#[test]
fn test_get_price_safe_nonexistent_returns_none() {
    let (_, client) = setup();
    // Must return None, not panic or error
    assert_eq!(client.get_price_safe(&symbol_short!("NGN")), None);
}

#[test]
fn test_get_all_assets_returns_tracked_symbols() {
    let (env, client) = setup();
    let source = Address::generate(&env);

    let ngn = symbol_short!("NGN");
    let kes = symbol_short!("KES");

    client.set_price(
        &ngn,
        &PriceData {
            asset: ngn.clone(),
            price: 1500,
            timestamp: 1000,
            source: source.clone(),
        },
    );
    client.set_price(
        &kes,
        &PriceData {
            asset: kes.clone(),
            price: 800,
            timestamp: 1000,
            source: source.clone(),
        },
    );

    let assets = client.get_all_assets();
    assert_eq!(assets.len(), 2);
    assert!(assets.contains(&ngn));
    assert!(assets.contains(&kes));
}
