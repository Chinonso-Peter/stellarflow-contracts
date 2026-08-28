use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Subscription(Address),
}

pub const RENT_THRESHOLD: u32 = 259_200;
pub const RENT_EXTEND_TO: u32 = 518_400;

pub fn extend_subscription_rent(env: &Env, consumer_id: Address) {
    let key = DataKey::Subscription(consumer_id);
    env.storage().persistent().extend_ttl(&key, RENT_THRESHOLD, RENT_EXTEND_TO);
}

pub fn check_subscription(env: &Env, consumer_id: Address) -> bool {
    let key = DataKey::Subscription(consumer_id.clone());
    if env.storage().persistent().has(&key) {
        extend_subscription_rent(env, consumer_id);
        true
    } else {
        false
    }
}

/// Pre-flight rent check hook (Issue #432).
///
/// Verifies that persistent storage entries have sufficient TTL remaining
/// before the contract commits to a state-changing operation.  This is a
/// no-op safety check that can be expanded with concrete rent validation
/// logic as storage rent policies evolve.
pub fn preflight_rent_check(_env: &Env) {
    // Intentionally a no-op for now; callers may expand this with
    // concrete rent-budget checks as the storage rent lifecycle
    // (Issue #432) matures.
}
