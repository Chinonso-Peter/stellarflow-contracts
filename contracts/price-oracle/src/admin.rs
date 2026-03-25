use soroban_sdk::{contracttype, Address, Env};

// ─────────────────────────────────────────────────────────────────────────────
// Storage Key
// ─────────────────────────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    Admin,
}

// ─────────────────────────────────────────────────────────────────────────────
// Storage Helpers
// ─────────────────────────────────────────────────────────────────────────────

pub fn _set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn _get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Admin not set: contract not initialised")
}

pub fn _has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn _is_admin(env: &Env, caller: &Address) -> bool {
    env.storage()
        .instance()
        .get::<DataKey, Address>(&DataKey::Admin)
        .map(|admin| admin == *caller)
        .unwrap_or(false) // no admin set → not an admin
}

pub fn _require_admin(env: &Env, caller: &Address) {
    if !_is_admin(env, caller) {
        panic!("Unauthorised: caller is not the admin");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup() -> (Env, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        _set_admin(&env, &admin);
        (env, admin)
    }

    #[test]
    fn test_is_admin_true_for_admin() {
        let (env, admin) = setup();
        assert!(_is_admin(&env, &admin));
    }

    #[test]
    fn test_is_admin_false_for_non_admin() {
        let (env, _) = setup();
        let other = Address::generate(&env);
        assert!(!_is_admin(&env, &other));
    }

    #[test]
    fn test_is_admin_false_when_no_admin_set() {
        let env = Env::default();
        let random = Address::generate(&env);
        // No set_admin call — should return false, not panic
        assert!(!_is_admin(&env, &random));
    }

    #[test]
    fn test_require_admin_passes_for_admin() {
        let (env, admin) = setup();
        // Must not panic
        _require_admin(&env, &admin);
    }

    #[test]
    #[should_panic(expected = "Unauthorised: caller is not the admin")]
    fn test_require_admin_panics_for_non_admin() {
        let (env, _) = setup();
        let other = Address::generate(&env);
        _require_admin(&env, &other);
    }

    #[test]
    fn test_get_admin_returns_correct_address() {
        let (env, admin) = setup();
        assert_eq!(_get_admin(&env), admin);
    }

    #[test]
    fn test_has_admin_true_after_set() {
        let (env, _) = setup();
        assert!(_has_admin(&env));
    }

    #[test]
    fn test_has_admin_false_before_set() {
        let env = Env::default();
        assert!(!_has_admin(&env));
    }
}
