#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Ledger, Env};

fn setup() -> (Env, PriceOracleClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_get_price_existing_asset() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    env.ledger().set_timestamp(1_234_567_890);
    env.ledger().set_sequence_number(1);

    let asset = symbol_short!("XLM");

    client.set_price(&asset, &1_000_000_i128);

    let result = client.try_get_price(&asset);
    assert!(result.is_ok());

    let retrieved_price = result.unwrap().unwrap();
    assert_eq!(retrieved_price.asset, asset);
    assert_eq!(retrieved_price.price, 1_000_000_i128);
    assert_eq!(retrieved_price.timestamp, 1_234_567_890);
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
    let xlm_asset = symbol_short!("XLM");
    let btc_asset = symbol_short!("BTC");

    client.set_price(&xlm_asset, &1_000_000_i128);
    client.set_price(&btc_asset, &50_000_000_000_i128);

    let xlm_result = client.try_get_price(&xlm_asset).unwrap().unwrap();
    assert_eq!(xlm_result.price, 1_000_000_i128);

    let btc_result = client.try_get_price(&btc_asset).unwrap().unwrap();
    assert_eq!(btc_result.price, 50_000_000_000_i128);
}

#[test]
fn test_get_price_after_update() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    let asset = symbol_short!("XLM");
    env.ledger().set_timestamp(1_234_567_890);
    env.ledger().set_sequence_number(1);
    client.set_price(&asset, &1_000_000_i128);

    let result = client.try_get_price(&asset).unwrap().unwrap();
    assert_eq!(result.price, 1_000_000_i128);
    assert_eq!(result.timestamp, 1_234_567_890);

    env.ledger().set_timestamp(1_234_567_900);
    env.ledger().set_sequence_number(2);
    client.set_price(&asset, &1_200_000_i128);

    let result = client.try_get_price(&asset).unwrap().unwrap();
    assert_eq!(result.price, 1_200_000_i128);
    assert_eq!(result.timestamp, 1_234_567_900);
}

#[test]
fn test_get_price_safe_nonexistent_returns_none() {
    let (_, client) = setup();
    // Must return None, not panic or error
    assert_eq!(client.get_price_safe(&symbol_short!("NGN")), None);
}

#[test]
fn test_get_all_assets_returns_tracked_symbols() {
    let (_, client) = setup();

    let ngn = symbol_short!("NGN");
    let kes = symbol_short!("KES");

    client.set_price(&ngn, &1_500_i128);
    client.set_price(&kes, &800_i128);

    let assets = client.get_all_assets();
    assert_eq!(assets.len(), 2);
    assert!(assets.contains(&ngn));
    assert!(assets.contains(&kes));
}

#[test]
fn test_set_price_uses_current_ledger_timestamp() {
    let env = Env::default();
    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    let asset = symbol_short!("NGN");

    env.ledger().set_timestamp(1_700_000_123);
    env.ledger().set_sequence_number(77);

    client.set_price(&asset, &950_i128);

    let stored = client.get_price(&asset);
    assert_eq!(stored.price, 950_i128);
    assert_eq!(stored.timestamp, 1_700_000_123);
}
