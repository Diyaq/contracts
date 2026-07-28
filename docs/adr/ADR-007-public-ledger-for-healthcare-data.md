# ADR-007: Storing healthcare-related data on a public, permanent ledger

Date: 2026-07-28

Status: Proposed

## Context

This repository implements healthcare-focused smart contracts deployed to a public blockchain (Stellar/Soroban). Several existing ADRs and documents address parts of the privacy and retention model:

- ADR-002: Storage TTL retention class design (Critical/Operational/Ephemeral)
- ADR-003: Hash-based privacy for sensitive fields
- TTL_POLICY.md / DATA_RETENTION.md: retention classes and operational procedures

Mainnet readiness and compliance checklists reference legal and GDPR concerns about storing Protected Health Information (PHI) or references to it on a public, permanent ledger. However, there was no single ADR capturing the decision to use a public ledger for these contracts and the high-level mitigations that accompany that decision.

## Decision

We will continue to deploy these contracts to a public, permissionless ledger (Stellar mainnet). To reduce privacy and legal risk while preserving the benefits of a public ledger (auditability, censorship-resistance, wide availability), the following rules apply:

1. What is stored on-chain
   - Only compact, non-reversible references and protocol metadata will be persisted on-chain. Typical examples:
     - Cryptographic hashes of up-to-date records (see ADR-003 for hash-based privacy guidance).
     - Pointer tokens or opaque identifiers that reference off-chain data stores under access controls.
     - Contract-level metadata required for governance, TTL bookkeeping, and verifiability (timestamps, retention-class labels, cryptographic commitments).
   - Raw PHI, personally-identifiable data, or unencrypted patient records MUST NOT be stored on-chain.

2. Off-chain storage and access
   - Any raw healthcare data referenced by on-chain identifiers must be stored off-chain in systems that support appropriate access controls, encryption at rest, and the organization's data residency requirements.
   - The on-chain identifiers are designed to be non-reversible and to require an off-chain access path + authorization to retrieve the underlying record.

3. Retention, immutability, and TTLs
   - We acknowledge the immutability of public ledgers and the tension with rights-to-erasure (e.g., GDPR). To mitigate this, we rely on the TTL/retention-class system described in ADR-002 and TTL_POLICY.md:
     - On-chain artifacts persist but are minimized in information content (hashes, labels).
     - TTLs govern the lifecycle of pointers and metadata. When a retention period expires, the contract marks the pointer as expired and removes (or re-purposes) any mutable metadata the contract is authorized to change. Note that deleted on-chain transactions are not physically removed from the ledger; instead, the authoritative off-chain data store is deleted or redacted per policy.
     - For any requirement that a record be made inaccessible, rely primarily on off-chain deletion and revocation of access; on-chain hashes/pointers remain as non-actionable audit artifacts.

4. Consent and legal review
   - Before any use-case that places even hashed or pointer-like references to PHI on-chain, legal counsel must confirm that the practice complies with applicable law (HIPAA, GDPR, local data residency rules).
   - Consent models, notice to data subjects, and Data Processing Agreements must explicitly describe what is stored on-chain and the permanence of that storage.

5. Additional mitigations
   - Use domain-separated hashing and salts where appropriate to prevent correlation attacks (see ADR-003).
   - Minimize the surface area of on-chain data: avoid free-form text, identifiers with embedded semantics, or any field that can be trivially correlated to an individual.
   - Regularly review the retention classes, TTL defaults, and operational procedures captured in DATA_RETENTION.md and TTL_POLICY.md.

## Consequences

- Positive: Maintains the benefits of a public ledger for verifiability, decentralization, and broad availability while bounding privacy exposure.
- Negative: Immutability remains: even minimized hashes and pointers are permanent; organizations must treat on-chain artifacts as non-revocable audit records.
- Operational: Requires robust off-chain data governance, legal sign-off processes, and clear consent/DPAs for systems that place references on-chain.
- Auditability: Auditors and reviewers can locate a single decision record (this ADR) explaining the tradeoffs and referencing technical mitigations (ADR-002, ADR-003) and operational controls (DATA_RETENTION.md, TTL_POLICY.md).

## References

- ADR-002: Storage TTL retention class design (docs/adr/ADR-002.md)
- ADR-003: Hash-based privacy for sensitive fields (docs/adr/ADR-003.md)
- DATA_RETENTION.md (DATA_RETENTION.md)
- TTL_POLICY.md (TTL_POLICY.md)
- MAINNET_READINESS.md (MAINNET_READINESS.md)
