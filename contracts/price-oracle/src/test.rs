#![cfg(test)]
extern crate alloc;

use super::*;
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, testutils::Events,
    testutils::Ledger, Address, Env,
};

#[soroban_sdk::contractevent]
pub struct TokenTransferEvent {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

#[contract]
pub struct DummyToken;

#[contractimpl]
impl DummyToken {
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        env.events()
            .publish_event(&TokenTransferEvent { from, to, amount });
    }
}

fn setup() -> (Env, Address, PriceOracleClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PriceOracle, ());
    let client = PriceOracleClient::new(&env, &contract_id);
    (env, contract_id, client)
}

fn set_admin(env: &Env, contract_id: &Address, admin: &Address) {
    env.as_contract(contract_id, || {
        crate::auth::_set_admin(env, &soroban_sdk::vec![env, admin.clone()]);
    });
}

fn add_provider(env: &Env, contract_id: &Address, provider: &Address) {
    env.as_contract(contract_id, || {
        crate::auth::_add_provider(env, provider);
    });
}

#[test]
fn test_initialize_sets_admin_and_assets() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let pairs = soroban_sdk::vec![&env, symbol_short!("NGN"), symbol_short!("KES")];

    client.initialize(&admin, &pairs);

    env.as_contract(&contract_id, || {
        let admins = crate::auth::_get_admin(&env);
        assert_eq!(admins.len(), 1);
        assert_eq!(admins.get(0).unwrap(), admin);
    });
    assert_eq!(client.get_all_assets(), pairs);
}

#[test]
#[should_panic]
fn test_initialize_rejects_second_call() {
    let (env, _, client) = setup();
    let admin = Address::generate(&env);
    let pairs = soroban_sdk::vec![&env, symbol_short!("NGN")];

    client.initialize(&admin, &pairs);
    client.initialize(&admin, &pairs);
}

#[test]
fn test_set_price_round_trip_tracks_asset() {
    let (env, _, client) = setup();
    let asset = symbol_short!("NGN");

    env.ledger().set_timestamp(1_234_567);
    client.set_price(&asset, &1_500_i128, &2u32, &3_600u64);

    let price = client.get_price(&asset);
    assert_eq!(price.price, 1_500_i128);
    assert_eq!(price.timestamp, 1_234_567);
    assert_eq!(price.decimals, 2u32);
    assert_eq!(client.get_all_assets(), soroban_sdk::vec![&env, asset]);
}

#[test]
fn test_get_price_with_status_marks_stale_entries() {
    let (env, _, client) = setup();
    let asset = symbol_short!("NGN");

    env.ledger().set_timestamp(1_000);
    client.set_price(&asset, &1_500_i128, &2u32, &100u64);

    env.ledger().set_timestamp(1_101);
    let result = client.get_price_with_status(&asset);

    assert_eq!(result.data.price, 1_500_i128);
    assert!(result.is_stale);
}

#[test]
fn test_update_price_rejects_untracked_asset() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    set_admin(&env, &contract_id, &admin);
    add_provider(&env, &contract_id, &provider);

    let result = client.try_update_price(
        &provider,
        &symbol_short!("BTC"),
        &50_000_i128,
        &6u32,
        &100u32,
        &3_600u64,
    );
    match result {
        Err(Ok(err)) => assert_eq!(err, Error::InvalidAssetSymbol),
        other => panic!("expected InvalidAssetSymbol, got {:?}", other),
    }
}

#[test]
fn test_update_price_rejects_non_provider() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    set_admin(&env, &contract_id, &admin);
    client.add_asset(&admin, &symbol_short!("NGN"));

    let result = client.try_update_price(
        &provider,
        &symbol_short!("NGN"),
        &1_000_i128,
        &6u32,
        &100u32,
        &3_600u64,
    );
    match result {
        Err(Ok(err)) => assert_eq!(err, Error::NotAuthorized),
        other => panic!("expected NotAuthorized, got {:?}", other),
    }
}

#[test]
fn test_update_price_rejects_flash_crash() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let asset = symbol_short!("NGN");

    set_admin(&env, &contract_id, &admin);
    add_provider(&env, &contract_id, &provider);

    client.set_price(&asset, &1_000_i128, &2u32, &3_600u64);

    let result = client.try_update_price(&provider, &asset, &1_200_i128, &2u32, &100u32, &3_600u64);
    match result {
        Err(Ok(err)) => assert_eq!(err, Error::FlashCrashDetected),
        other => panic!("expected FlashCrashDetected, got {:?}", other),
    }
}

#[test]
fn test_set_and_get_price_bounds() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let asset = symbol_short!("NGN");

    set_admin(&env, &contract_id, &admin);
    client.set_price_bounds(&admin, &asset, &500_i128, &2_000_i128);

    let bounds = client.get_price_bounds(&asset).unwrap();
    assert_eq!(bounds.min_price, 500_i128);
    assert_eq!(bounds.max_price, 2_000_i128);
}

#[test]
fn test_update_price_rejects_price_outside_bounds() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let asset = symbol_short!("NGN");

    set_admin(&env, &contract_id, &admin);
    add_provider(&env, &contract_id, &provider);

    client.add_asset(&admin, &asset);
    client.set_price_bounds(&admin, &asset, &500_i128, &2_000_i128);

    let result = client.try_update_price(&provider, &asset, &250_i128, &2u32, &100u32, &3_600u64);
    match result {
        Err(Ok(err)) => assert_eq!(err, Error::PriceOutOfBounds),
        other => panic!("expected PriceOutOfBounds, got {:?}", other),
    }
}

#[test]
fn test_set_and_get_price_floor() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let asset = symbol_short!("NGN");

    set_admin(&env, &contract_id, &admin);
    client.set_price_floor(&admin, &asset, &700_i128);

    assert_eq!(client.get_price_floor(&asset), Some(700_i128));
}

#[test]
fn test_update_price_rejects_price_below_floor() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let asset = symbol_short!("NGN");

    set_admin(&env, &contract_id, &admin);
    add_provider(&env, &contract_id, &provider);

    client.add_asset(&admin, &asset);
    client.set_price_floor(&admin, &asset, &700_i128);

    let result = client.try_update_price(&provider, &asset, &650_i128, &2u32, &100u32, &3_600u64);
    match result {
        Err(Ok(err)) => assert_eq!(err, Error::PriceOutOfBounds),
        other => panic!("expected PriceOutOfBounds, got {:?}", other),
    }
}

#[test]
#[should_panic]
fn test_set_price_rejects_price_below_floor() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let asset = symbol_short!("NGN");

    set_admin(&env, &contract_id, &admin);
    client.set_price_floor(&admin, &asset, &700_i128);
    client.set_price(&asset, &650_i128, &2u32, &3_600u64);
}

#[test]
fn test_remove_asset_clears_related_configuration() {
    let (env, contract_id, client) = setup();
    let admin = Address::generate(&env);
    let asset = symbol_short!("NGN");

    set_admin(&env, &contract_id, &admin);
    client.set_price(&asset, &1_000_i128, &2u32, &3_600u64);
    client.set_price_floor(&admin, &asset, &700_i128);
    client.set_price_bounds(&admin, &asset, &500_i128, &2_000_i128);

    client.remove_asset(&admin, &asset);

    assert_eq!(client.get_price_safe(&asset), None);
    assert_eq!(client.get_price_floor(&asset), None);
    assert_eq!(client.get_price_bounds(&asset), None);
    assert!(client.get_all_assets().is_empty());
}

#[test]
fn test_rescue_tokens_admin_can_recover_assets() {
    let (env, contract_id, client) = setup();
    let token_id = env.register(DummyToken, ());
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    set_admin(&env, &contract_id, &admin);
    client.rescue_tokens(&admin, &token_id, &recipient, &1_000_i128);

    let debug = alloc::format!("{:?}", env.events().all());
    assert!(debug.contains("token_transfer_event"));
    assert!(debug.contains("rescue_tokens_event"));
    assert!(debug.contains("1000"));
}

#[test]
#[should_panic(expected = "Unauthorised")]
fn test_rescue_tokens_rejects_non_admin() {
    let (env, contract_id, client) = setup();
    let token_id = env.register(DummyToken, ());
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    set_admin(&env, &contract_id, &admin);
    client.rescue_tokens(&non_admin, &token_id, &recipient, &1_000_i128);
}

#[test]
fn test_toggle_pause_requires_two_admins() {
    let (env, contract_id, client) = setup();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    client.init_admin(&admin1);
    env.as_contract(&contract_id, || {
        crate::auth::_add_authorized(&env, &admin2);
    });

    assert!(client.toggle_pause(&admin1, &admin2));
    assert!(!client.toggle_pause(&admin1, &admin2));
}

#[test]
fn test_register_and_remove_admin_updates_count() {
    let (env, contract_id, client) = setup();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);

    client.init_admin(&admin1);
    env.as_contract(&contract_id, || {
        crate::auth::_add_authorized(&env, &admin2);
    });

    assert_eq!(client.get_admin_count(), 2);
    client.register_admin(&admin1, &admin2, &admin3);
    assert_eq!(client.get_admin_count(), 3);
    assert!(client.is_admin(&admin3));

    client.remove_admin(&admin1, &admin2, &admin3);
    assert_eq!(client.get_admin_count(), 2);
    assert!(!client.is_admin(&admin3));
}

#[test]
#[should_panic(expected = "Unauthorised")]
fn test_renounce_ownership_blocks_admin_calls() {
    let (env, _, client) = setup();
    let admin = Address::generate(&env);

    client.init_admin(&admin);
    client.renounce_ownership(&admin);
    client.add_asset(&admin, &symbol_short!("NGN"));
}
