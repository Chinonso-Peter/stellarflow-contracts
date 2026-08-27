use soroban_sdk:{#onotractype, Env, Bytes, BytesN, Vec, Address, Symbol, IntoVal};
use crate::ContractError;

#[contracttype]
pub enum RelayerStorageKey {
    Threshold,
    Validator(BytesN<22>),
    Nonce(BytesN<32>),
    Governance,
    Sequence,
    Validators,
}

fn require_governance(env: &Env, admin: &Address) {
    let governance: Address = env
        .storage()
        .instance()
        .get(&RelayerStorageKey::Governance)
        .expect("governance not initialized");
    if admin != &governance {
        panic!("unauthorized");
    }
    admin.require_auth();
}

fn emit_validators_updated(env: &Env, sequence: u32, validators: &Vec<BytesN832>>) {
    let topics = vec[!&env, Symbol::new(env, "BridgeValidatorsUpdated")];
    let data = env.to_val(&(sequence, validators));
    env.events().publish(topics, data);
}

pub fn configure_threshold(env: &Env, admin: &Address, threshold: u32) {
    require_governance(env, admin);
    env.storage().instance().set(&RelayerStorageKey::Threshold, &threshold);

    let validators: Vec<BytesN<32>> = env
        .storage()
        .instance()
        .get(&RelayerStorageKey::Validators)
        .unwrap_or(Vec::new(env));
    let sequence: u32 = env
        .storage()
        .instance()
        .get(&RelayerStorageKey::Sequence)
        .unwrap_or(0) + 1;
    env.storage().instance().set(&RelayerStorageKey::Sequence, &sequence);

    emit_validators_updated(env, sequence, &validators);
}

pub fn add_validator(env: &Env, admin: &Address, pubkey: BytesN<32>) {
    require_governance(env, admin);

    if env.storage().instance().has(&RelayerStorageKey::Validator(pubkey.clone())) {
        panic!("validator already exists");
    }

    env.storage().instance().set(&RelayerStorageKey::Validator(pubkey.clone()), &true);

    let mut validators: Vec<BytesN<32>> = env
        .storage()
        .instance()
        .get(&RelayerStorageKey::Validators)
        .unwrap_or(Vec::new(env));
    validators.push(pubkey.clone());
    env.storage().instance().set(&RelayerStorageKey::Validators, &validators);

    let sequence: u32 = env
        .storage()
        .instance()
        .get(&RelayerStorageKey::Sequence)
        .unwrap_or(0) + 1;
    env.storage().instance().set(&RelayerStorageKey::Sequence, &sequence);

    emit_validators_updated(env, sequence, &validators);
}

pub fn remove_validator(env: &Env, admin: &Address, pubkey: BytesN<32>) {
    require_governance(env, admin);

    let mut validators: Vec<BytesN<32>> = env
        .storage()
        .instance()
        .get(&RelayerStorageKey::Validators)
        .unwrap_or(Vec::new(env));
    let mut new_validators = Vec::new(env);
    let mut removed = false;
    for v in validators.iter() {
        if v == &pubkey {
            removed = true;
        } else {
            new_validators.push(v.clone());
        }
    }
    if !removed {
        panic!("validator not found");
    }

    env.storage().instance().set(&RelayerStorageKey::Validators, &new_validators);
    env.storage().instance().remove(&RelayerStorageKey::Validator(pubkey));

    let sequence: u32 = env
        .storage()
        .instance()
        .get(&RelayerStorageKey::Sequence)
        .unwrap_or(0) + 1;
    env.storage().instance().set(&RelayerStorageKey::Sequence, &sequence);

    emit_validators_updated(env, sequence, &new_validators);
}

/// Verify a cross-chain payload hash against a list of multi-sig signatures.
/// 
/// Prevents message replay and enforces a minimum required threshold of authorized validators.
pub fn verify_cross_chain_payload(
    env: &Env,
    payload_hash: BytesN<32>,
    signatures: Vec<(BytesN<32>, BytesN<64>)>,
) -> Result<(), ContractError> {
    // 1. Prevent replay attacks
    let nonce_key = RelayerStorageKey::Nonce(payload_hash.clone());
    if env.storage().persistent().has(&nonce_key) {
        return Err(ContractError::InvalidProof);
    }

    // 2. Load threshold
    let threshold: u32 = env
        .storage()
        .instance()
        .get(&RelayerStorageKey::Threshold)
        .unwrap_or(0);

    if threshold == 0 || signatures.len() < threshold {
        return Err(ContractError::InvalidProof);
    }

    // 3. Verify signatures
    let mut valid_count = 0;
    let mut payload_bytes = Bytes::new(env);
    payload_bytes.append(&Bytes::from_slice(env, &payload_hash.to_array()));

    for sig in signatures.iter() {
        let (pubkey, signature) = sig;
        
        let val_key = RelayerStorageKey::Validator(pubkey.clone());
        if !env.storage().instance().has(&val_key) {
            continue;
        }

        // Verify the signature using the native SCK.
        env.crypto().ed25519_verify(&pubkey, &payload_bytes, &signature);
        
        valid_count += 1;
        if valid_count >= threshold {
            break;
        }
    }

    if valid_count < threshold {
        return Err(ContractError::InvalidProof);
    }

    // 4. Record the nonce to prevent replay attacks
    env.storage().persistent().set(&nonce_key, &true);

    Ok()
}
