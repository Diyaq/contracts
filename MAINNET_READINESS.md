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

### Key management

- [ ] Deployer identity is a hardware wallet or multi-sig Stellar account
  — **never** a plain CI secret key for mainnet
- [ ] Governance signers (multisig-governance) have been confirmed and keys
      are secured
- [ ] Admin keys are stored in offline / HSM storage

### Governance setup

- [ ] `multisig-governance` contract deployed first (dependency for upgrades)
- [ ] `upgrade-governance` contract deployed second
- [ ] Governance thresholds and signer set verified on-chain

### Deployment

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

1. Complete all checklist items above and tick each box.
2. Update the status banner at the top from `NOT YET DEPLOYED` to
   `DEPLOYED – <date>`.
3. Commit the updated `deployments/mainnet.json` (with real contract IDs)
   and this file together in the same PR.
