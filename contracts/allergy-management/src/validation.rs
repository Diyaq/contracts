use soroban_sdk::{symbol_short, Env, String, Symbol};

use crate::{storage, Error};

/// Validate allergen type
pub fn validate_allergen_type(allergen_type: &Symbol) -> Result<(), Error> {
    let valid_types = [
        symbol_short!("med"),  // medication
        symbol_short!("food"), // food
        symbol_short!("env"),  // environmental
    ];

    if valid_types.contains(allergen_type) {
        Ok(())
    } else {
        Err(Error::InvalidAllergenType)
    }
}

/// Validate severity level
pub fn validate_severity(severity: &Symbol) -> Result<(), Error> {
    let valid_severities = [
        symbol_short!("mild"),
        symbol_short!("moderate"),
        symbol_short!("severe"),
        symbol_short!("critical"),
    ];

    if valid_severities.contains(severity) {
        Ok(())
    } else {
        Err(Error::InvalidSeverity)
    }
}

/// Check if drug name matches allergen (case-insensitive comparison)
///
/// Soroban `String` equality is a raw byte comparison with no case-folding:
/// without normalization, an allergen recorded as "Penicillin" would fail to
/// match a drug name of "penicillin" or "PENICILLIN", silently suppressing a
/// real drug-allergy interaction warning. Both inputs are therefore
/// normalized to a canonical (ASCII lowercase) form before comparing, in the
/// same spirit as allergy-tracking's byte-level `trim_allergen` helper.
pub fn check_drug_match(allergen: &String, drug_name: &String) -> bool {
    // Fast path: byte-identical strings always match.
    if allergen == drug_name {
        return true;
    }

    // Normalize both sides to a canonical case, then compare.
    to_lowercase(allergen) == to_lowercase(drug_name)
}

/// Normalize a Soroban `String` to ASCII lowercase (A-Z => a-z).
///
/// Byte-level normalization; non-ASCII bytes are left untouched.
fn to_lowercase(value: &String) -> String {
    let mut bytes = value.to_bytes();
    for i in 0..bytes.len() {
        if let Some(byte) = bytes.get(i) {
            bytes.set(i, byte.to_ascii_lowercase());
        }
    }
    String::from(&bytes)
}

/// Check for cross-sensitivity between allergens
pub fn check_cross_sensitivity(env: &Env, allergen: &String, drug_name: &String) -> bool {
    storage::has_cross_sensitivity(env, allergen, drug_name)
}
