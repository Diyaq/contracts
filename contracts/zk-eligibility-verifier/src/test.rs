#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;

fn setup() -> (Env, ZkEligibilityVerifierClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(ZkEligibilityVerifier, ());
    let client = ZkEligibilityVerifierClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_initialize() {
    let (env, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let zk_contract = Address::generate(&env);

    client.initialize(&admin, &zk_contract);
}

#[test]
#[should_panic]
fn test_unauthorized_caller_rejected() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let zk_contract = Address::generate(&env);

    // No auths mocked: admin.require_auth() must reject this call.
    client.initialize(&admin, &zk_contract);
}

#[test]
fn test_double_initialize_rejected() {
    let (env, client) = setup();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let zk_contract = Address::generate(&env);

    client.initialize(&admin, &zk_contract);
    let result = client.try_initialize(&admin, &zk_contract);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}
