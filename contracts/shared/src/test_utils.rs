#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, Vec,
};

/// Generate a dummy hash filled with a specific byte value
pub fn dummy_hash(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

/// Generate a vector of dummy addresses
pub fn generate_addresses(env: &Env, n: usize) -> Vec<Address> {
    let mut addresses = Vec::new(env);
    for _ in 0..n {
        addresses.push_back(Address::generate(env));
    }
    addresses
}

/// Advance the ledger by the specified number of seconds
pub fn advance_ledger(env: &Env, seconds: u64) {
    env.ledger().with_mut(|l| {
        l.sequence_number = l.sequence_number.saturating_add(1);
        l.timestamp = l.timestamp.saturating_add(seconds);
    });
}
