#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Symbol};

#[test]
fn test_record_immunization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ImmunizationRegistry, ());
    let client = ImmunizationRegistryClient::new(&env, &contract_id);

    let patient_id = Address::generate(&env);
    let provider_id = Address::generate(&env);

    let id = client.record_immunization(&VaccineRecord {
        patient_id: patient_id.clone(),
        provider_id: provider_id.clone(),
        vaccine_name: String::from_str(&env, "Hepatitis B"),
        cvx_code: String::from_str(&env, "CVX_43"),
        lot_number: String::from_str(&env, "LOT_12345"),
        manufacturer: String::from_str(&env, "SANOFI"),
        administration_date: 1690000000,
        expiration_date: 1790000000,
        dose_number: 1,
        route: Symbol::new(&env, "IM"), // Intramuscular
        site: Symbol::new(&env, "DELTOID"),
    });

    assert_eq!(id, 1);

    let _requester = Address::generate(&env);
    let history = client.get_immunization_history(&patient_id, &patient_id);

    assert_eq!(history.len(), 1);
    let record = history.get(0).unwrap();
    assert_eq!(record.patient_id, patient_id);
    assert_eq!(record.provider_id, provider_id);
    assert_eq!(record.vaccine_name, String::from_str(&env, "Hepatitis B"));
    assert_eq!(record.cvx_code, String::from_str(&env, "CVX_43"));
}

#[test]
fn test_record_adverse_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ImmunizationRegistry, ());
    let client = ImmunizationRegistryClient::new(&env, &contract_id);

    let patient_id = Address::generate(&env);
    let provider_id = Address::generate(&env);

    let id = client.record_immunization(&VaccineRecord {
        patient_id: patient_id.clone(),
        provider_id: provider_id.clone(),
        vaccine_name: String::from_str(&env, "Hepatitis B"),
        cvx_code: String::from_str(&env, "CVX_43"),
        lot_number: String::from_str(&env, "LOT_12345"),
        manufacturer: String::from_str(&env, "SANOFI"),
        administration_date: 1690000000,
        expiration_date: 1790000000,
        dose_number: 1,
        route: Symbol::new(&env, "IM"),
        site: Symbol::new(&env, "DELTOID"),
    });

    let reporter = Address::generate(&env);
    client.record_adverse_event(
        &id,
        &reporter,
        &String::from_str(&env, "Slight fever and arm soreness"),
        &Symbol::new(&env, "MILD"),
        &1690086400,
    );

    // To verify, we'd theoretically need a getter for adverse events, but that's not
    // on the requested interface. However, the function succeeding means it's recorded.
    // Let's test non-existent ID.
    let res = client.try_record_adverse_event(
        &999,
        &reporter,
        &String::from_str(&env, "NA"),
        &Symbol::new(&env, "NONE"),
        &1690086400,
    );
    assert!(res.is_err());
}

#[test]
fn test_vaccine_series_and_due() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ImmunizationRegistry, ());
    let client = ImmunizationRegistryClient::new(&env, &contract_id);

    let patient_id = Address::generate(&env);
    let provider_id = Address::generate(&env);

    // Register a 3-dose series
    client.register_vaccine_series(
        &patient_id,
        &String::from_str(&env, "Hepatitis B"),
        &String::from_str(&env, "CVX_43"),
        &3,
        &BytesN::from_array(&env, &[0; 32]), // dummy hash
    );

    // Initially, they are due for it
    let due = client.check_due_vaccines(&patient_id, &patient_id, &1690000000);
    assert_eq!(due.len(), 1);

    // Record one dose
    client.record_immunization(&VaccineRecord {
        patient_id: patient_id.clone(),
        provider_id: provider_id.clone(),
        vaccine_name: String::from_str(&env, "Hepatitis B"),
        cvx_code: String::from_str(&env, "CVX_43"),
        lot_number: String::from_str(&env, "LOT_12345"),
        manufacturer: String::from_str(&env, "SANOFI"),
        administration_date: 1690000000,
        expiration_date: 1790000000,
        dose_number: 1,
        route: Symbol::new(&env, "IM"),
        site: Symbol::new(&env, "DELTOID"),
    });

    // Still due (need 3)
    let due2 = client.check_due_vaccines(&patient_id, &patient_id, &1695000000);
    assert_eq!(due2.len(), 1);

    // Record two more doses
    client.record_immunization(&VaccineRecord {
        patient_id: patient_id.clone(),
        provider_id: provider_id.clone(),
        vaccine_name: String::from_str(&env, "Hepatitis B"),
        cvx_code: String::from_str(&env, "CVX_43"),
        lot_number: String::from_str(&env, "LOT_12346"),
        manufacturer: String::from_str(&env, "SANOFI"),
        administration_date: 1692000000,
        expiration_date: 1790000000,
        dose_number: 2,
        route: Symbol::new(&env, "IM"),
        site: Symbol::new(&env, "DELTOID"),
    });
    client.record_immunization(&VaccineRecord {
        patient_id: patient_id.clone(),
        provider_id: provider_id.clone(),
        vaccine_name: String::from_str(&env, "Hepatitis B"),
        cvx_code: String::from_str(&env, "CVX_43"),
        lot_number: String::from_str(&env, "LOT_12347"),
        manufacturer: String::from_str(&env, "SANOFI"),
        administration_date: 1698000000,
        expiration_date: 1790000000,
        dose_number: 3,
        route: Symbol::new(&env, "IM"),
        site: Symbol::new(&env, "DELTOID"),
    });

    // Now they should NOT be due
    let due3 = client.check_due_vaccines(&patient_id, &patient_id, &1700000000);
    assert_eq!(due3.len(), 0);
}

#[test]
fn test_due_vaccines_matches_by_cvx_code_not_name() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ImmunizationRegistry, ());
    let client = ImmunizationRegistryClient::new(&env, &contract_id);

    let patient_id = Address::generate(&env);
    let provider_id = Address::generate(&env);

    // Series is keyed by CVX code, not brand name.
    client.register_vaccine_series(
        &patient_id,
        &String::from_str(&env, "Hepatitis B Series"),
        &String::from_str(&env, "CVX_43"),
        &1,
        &BytesN::from_array(&env, &[0; 32]),
    );

    // Dose recorded under a different brand/vaccine_name but the same CVX code.
    client.record_immunization(&VaccineRecord {
        patient_id: patient_id.clone(),
        provider_id: provider_id.clone(),
        vaccine_name: String::from_str(&env, "Recombivax HB"),
        cvx_code: String::from_str(&env, "CVX_43"),
        lot_number: String::from_str(&env, "LOT_99"),
        manufacturer: String::from_str(&env, "MERCK"),
        administration_date: 1690000000,
        expiration_date: 1790000000,
        dose_number: 1,
        route: Symbol::new(&env, "IM"),
        site: Symbol::new(&env, "DELTOID"),
    });

    // No longer due, even though vaccine_name never matched series_name.
    let due = client.check_due_vaccines(&patient_id, &patient_id, &1700000000);
    assert_eq!(due.len(), 0);
}

#[test]
fn test_get_patients_by_lot() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ImmunizationRegistry, ());
    let client = ImmunizationRegistryClient::new(&env, &contract_id);

    let regulator = Address::generate(&env);
    client.initialize(&regulator);

    let patient_a = Address::generate(&env);
    let patient_b = Address::generate(&env);
    let patient_c = Address::generate(&env);
    let provider_id = Address::generate(&env);

    let recalled_lot = String::from_str(&env, "LOT_RECALL");
    let other_lot = String::from_str(&env, "LOT_OTHER");

    for patient in [&patient_a, &patient_b] {
        client.record_immunization(&VaccineRecord {
            patient_id: patient.clone(),
            provider_id: provider_id.clone(),
            vaccine_name: String::from_str(&env, "Hepatitis B"),
            cvx_code: String::from_str(&env, "CVX_43"),
            lot_number: recalled_lot.clone(),
            manufacturer: String::from_str(&env, "SANOFI"),
            administration_date: 1690000000,
            expiration_date: 1790000000,
            dose_number: 1,
            route: Symbol::new(&env, "IM"),
            site: Symbol::new(&env, "DELTOID"),
        });
    }

    client.record_immunization(&VaccineRecord {
        patient_id: patient_c.clone(),
        provider_id: provider_id.clone(),
        vaccine_name: String::from_str(&env, "Hepatitis B"),
        cvx_code: String::from_str(&env, "CVX_43"),
        lot_number: other_lot,
        manufacturer: String::from_str(&env, "SANOFI"),
        administration_date: 1690000000,
        expiration_date: 1790000000,
        dose_number: 1,
        route: Symbol::new(&env, "IM"),
        site: Symbol::new(&env, "DELTOID"),
    });

    let affected = client.get_patients_by_lot(&recalled_lot, &regulator);
    assert_eq!(affected.len(), 2);
    assert!(affected.contains(&patient_a));
    assert!(affected.contains(&patient_b));
    assert!(!affected.contains(&patient_c));

    // A non-regulator caller is rejected.
    let outsider = Address::generate(&env);
    let res = client.try_get_patients_by_lot(&recalled_lot, &outsider);
    assert!(res.is_err());
}
