# Mainnet Readiness

> **Current status: NOT YET DEPLOYED**
>
> No contracts from this repository have been deployed to Stellar Mainnet.
> `deployments/mainnet.json` is a placeholder stub. Until the pre-flight
> checklist below is satisfied and a real deployment run is executed, TTL
> extension and any other mainnet-specific automation is **inactive** and
> not urgent.

## Pre-flight checklist

Complete every item before running the first mainnet deployment.

### Code quality

- [ ] `cargo test --workspace` passes with zero failures
- [ ] `cargo clippy --workspace -- -D warnings` produces no warnings
- [ ] `cargo fmt --all --check` passes
- [ ] All contracts audited or peer-reviewed for logic errors
- [ ] Security audit (`cargo audit`) shows no unaddressed vulnerabilities

- [ ] HIPAA Security Rule gap analysis completed
- [ ] Data Processing Agreement with Stellar Foundation reviewed and signed
- [ ] GDPR data residency requirements assessed (acknowledge on-chain data is public and permanent)
- [ ] Legal review of storing PHI references on a public blockchain completed ([ADR-007: Public ledger for healthcare data](docs/adr/ADR-007-public-ledger-for-healthcare-data.md))
- [ ] Documentation on data minimization and encryption strategies in place

- [ ] Deployer identity is a hardware wallet or multi-sig Stellar account
  — **never** a plain CI secret key for mainnet
- [ ] Governance signers (multisig-governance) have been confirmed and keys
      are secured
- [ ] Admin keys are stored in offline / HSM storage

### Governance setup

- [ ] `multisig-governance` contract deployed first (dependency for upgrades)
- [ ] `upgrade-governance` contract deployed second
- [ ] Governance thresholds and signer set verified on-chain

- [ ] All 80 open issues resolved or explicitly deferred (with rationale documented)
- [ ] `cargo test --workspace` passes with zero compilation errors
- [ ] `cargo clippy --workspace` runs with no warnings
- [ ] WASM sizes verified within Stellar's contract size limit (current limit: 128 KB) and no contract is within the defined safety margin (10% / ≥ 115.2 KB)
- [ ] `upgrade-governance` contract controls all production admin keys
- [ ] All contract interfaces reviewed and stabilized (API changes should be minimal post-launch)
- [ ] Deployment manifest published and verified (see SECURITY.md)
- [ ] Dry-run deployment executed against a Mainnet preview/staging environment

- [ ] Dry-run completed: `./scripts/deploy_all.sh --network mainnet --dry-run`
- [ ] All contract WASMs build cleanly for `wasm32v1-none`
- [ ] WASM hashes recorded before deployment submission
- [ ] Deployment run: `./scripts/deploy_all.sh --network mainnet`
- [ ] `deployments/mainnet.json` populated with all deployed contract IDs
      (status field set to `"complete"`)

### Post-deployment verification

- [ ] Every contract ID in `deployments/mainnet.json` verified against
      Horizon / Stellar Expert WASM hash
- [ ] Smoke test: read-only invocation on each deployed contract succeeds
- [ ] Governance contracts accept a test proposal and reject unauthorised
      callers

### TTL management

Once contracts are live, set up the `extend-ttls.yml` cron workflow (see
`.github/workflows/extend-ttls.yml`) to extend contract storage TTLs on a
weekly schedule.  The workflow will **fail loudly** (`exit 1`) if
`deployments/mainnet.json` is missing or has `_status != "complete"`, so
it is safe to enable it as soon as the manifest is populated.

## How to update this document after deployment

1. Configure `STELLAR_IDENTITY` to point to the production admin multi-sig account or HSM
2. Execute deployment with monitoring enabled:
   ```bash
   ./scripts/deploy_all.sh --network mainnet
   ```
3. Verify contract IDs in `deployments/mainnet.json` match the on-chain state
4. Record all deployed contract IDs in a secure, versioned log

### Post-Deployment Validation

- [ ] All contracts successfully initialized on Mainnet
- [ ] Governance contracts (`multisig-governance`, `upgrade-governance`) operational
- [ ] Each deployed contract responds to a no-op or read-only query
- [ ] Deployment manifest hashes verified against on-chain bytecode using Stellar Expert or Horizon API
- [ ] No unexpected errors or warnings in logs

## Sign-Off

Production readiness requires explicit sign-off from:

1. **Lead Developer** — confirms code quality, testing, and deployment plan
   - Name: ________________
   - Date: ________________
   - Signature: ________________

2. **Security Lead** — confirms audit findings resolved and security architecture sound
   - Name: ________________
   - Date: ________________
   - Signature: ________________

3. **Legal Counsel** — confirms compliance and data privacy requirements met
   - Name: ________________
   - Date: ________________
   - Signature: ________________

## Post-Launch Monitoring

After Mainnet launch:

- [ ] Monitor alerting dashboards for 72 hours continuously
- [ ] Weekly review of anomaly detection alerts for first month
- [ ] Monthly operational review with on-call team
- [ ] Quarterly security audit of governance decisions and contract state
- [ ] Incident postmortems completed within 24 hours of any production issue

## Rollback Plan

In case of critical issues post-launch:

1. **Minor issues**: Use `upgrade-governance` to deploy a patched version
2. **Critical issues**: Execute emergency governance proposal to pause high-risk functions
3. **Severe compromise**: Invoke emergency pause via multi-sig (if implemented)

Document any rollback decisions in the incident log and notify stakeholders.

## References

- [DEPLOYMENT.md](./DEPLOYMENT.md) — Deployment guide and procedures
- [SECURITY.md](./SECURITY.md) — Security architecture and policies
- [TTL_POLICY.md](./TTL_POLICY.md) — TTL management strategy
- Stellar Documentation: https://developers.stellar.org/ (current limit: 128 KB on Mainnet)
- [WASM_SIZE_BASELINE.md](./WASM_SIZE_BASELINE.md) — Measured contract sizes and optimization targets
