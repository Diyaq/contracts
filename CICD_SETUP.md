# CI/CD Setup Guide

This guide walks you through setting up the complete CI/CD pipeline for the Healthy-Stellar contracts repository.

## Prerequisites

- Repository admin access
- Stellar testnet and/or mainnet account with XLM for deployments
- Basic understanding of GitHub Actions

---

## Secrets Reference

> **Which secret does which workflow use?** Use this table as the authoritative reference. If the table and any other section of this document disagree, the table wins — and please open a PR to fix the discrepancy.

| Secret Name | Workflow(s) | Purpose |
|---|---|---|
| `STELLAR_SECRET_KEY` | `deploy-testnet.yml` | Raw secret key (`S...`) imported as the `deployer` identity for testnet deployments |
| `TESTNET_DEPLOYER_IDENTITY` | `extend-ttls.yml` | Stellar identity credential used by the `stellar` CLI to extend contract TTLs on testnet |
| `MAINNET_DEPLOYER_IDENTITY` | `extend-ttls.yml` | Stellar identity credential used by the `stellar` CLI to extend contract TTLs on mainnet |

`ci.yml` requires **no secrets** — it only runs `cargo fmt`, `cargo clippy`, `cargo test`, and a WASM build.

---

## Step 1: Configure Repository Secrets

1. Navigate to your repository on GitHub
2. Go to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add each secret described below

### `STELLAR_SECRET_KEY`

- **Name:** `STELLAR_SECRET_KEY`
- **Value:** Your Stellar testnet secret key (starts with `S...`)
- **Used by:** `deploy-testnet.yml`
- **Purpose:** Imported as the `deployer` identity that signs testnet contract deployments

**How to create a testnet deployer account:**

```bash
# Install the Stellar CLI
cargo install --locked stellar-cli --features opt

# Generate a new identity (writes keys to ~/.config/stellar/identity/)
stellar keys generate deployer --network testnet

# Show the secret key to copy into the GitHub secret
stellar keys show deployer

# Get the public address (to fund the account)
stellar keys address deployer
```

Then fund the account at the [Stellar Laboratory Friendbot](https://laboratory.stellar.org/#account-creator) by pasting the public address.

> ⚠️ **Security:** Never commit secret keys to the repository or share them publicly. Rotate them if they are ever exposed.

---

### `TESTNET_DEPLOYER_IDENTITY`

- **Name:** `TESTNET_DEPLOYER_IDENTITY`
- **Used by:** `extend-ttls.yml`
- **Purpose:** Passed directly to `extend-ttls.sh` as the `--identity` argument and exported as `STELLAR_IDENTITY` so the `stellar` CLI can sign TTL-extension transactions on testnet

**How to obtain the value:**

```bash
# After generating your testnet identity (see above), export it
stellar keys show deployer
# Copy the output — this is the value for TESTNET_DEPLOYER_IDENTITY
```

---

### `MAINNET_DEPLOYER_IDENTITY`

- **Name:** `MAINNET_DEPLOYER_IDENTITY`
- **Used by:** `extend-ttls.yml` (scheduled runs and manual mainnet triggers)
- **Purpose:** Passed to `extend-ttls.sh` and exported as `STELLAR_IDENTITY` so the `stellar` CLI can sign TTL-extension transactions on mainnet

**How to obtain the value:**

```bash
# Generate a separate mainnet identity — never reuse testnet keys on mainnet
stellar keys generate mainnet-deployer

# Show the secret key for the GitHub secret value
stellar keys show mainnet-deployer

# Get the address to fund with real XLM
stellar keys address mainnet-deployer
```

> ⚠️ **Mainnet keys carry real financial risk.** Use a dedicated account with the minimum XLM needed for TTL extensions. Rotate this secret immediately if it is ever exposed.

---

## Step 2: Enable GitHub Actions

1. Go to **Settings** → **Actions** → **General**
2. Under "Actions permissions", select:
   - ✅ **Allow all actions and reusable workflows**
3. Under "Workflow permissions", select:
   - ✅ **Read and write permissions**
   - ✅ **Allow GitHub Actions to create and approve pull requests**
4. Click **Save**

---

## Step 3: Configure Branch Protection Rules

Protect the `main` branch to enforce CI requirements:

1. Go to **Settings** → **Branches**
2. Click **Add rule** (or edit existing rule for `main`)
3. Configure the following:

### Branch name pattern
```
main
```

### Protect matching branches

#### Require a pull request before merging
- ✅ Enable
- **Required approvals:** 1
- ✅ Dismiss stale pull request approvals when new commits are pushed
- ✅ Require review from Code Owners (if you have a CODEOWNERS file)

#### Require status checks to pass before merging
- ✅ Enable
- ✅ Require branches to be up to date before merging
- **Required status checks** (these are the job names defined in `ci.yml`):
  - `CI Success`
  - `Format Check`
  - `Clippy Lint`
  - `Test Suite`
  - `Build WASM`

#### Require conversation resolution before merging
- ✅ Enable

#### Require signed commits
- ⬜ Optional (recommended for enhanced security)

#### Require linear history
- ✅ Enable (keeps git history clean)

#### Do not allow bypassing the above settings
- ✅ Enable (even for administrators)

4. Click **Create** or **Save changes**

---

## Step 4: Verify Workflows

### Test CI Workflow (`ci.yml`)

The CI workflow runs on every push and pull request to `main`. It has four jobs — `Format Check`, `Clippy Lint`, `Test Suite`, and `Build WASM` — plus a gating `CI Success` job. No secrets are required.

1. Create a test branch:
```bash
git checkout -b test-ci
```

2. Make a small change (e.g., add a comment to a source file)

3. Commit and push:
```bash
git add .
git commit -m "test: trigger CI workflow"
git push origin test-ci
```

4. Open a pull request on GitHub and confirm all five checks turn green.

---

### Test Deployment Workflow (`deploy-testnet.yml`)

This workflow triggers automatically on push to `main` and can also be triggered manually. It requires the `STELLAR_SECRET_KEY` secret.

The workflow:
1. Detects which contracts changed (or uses your manual input)
2. Builds each contract to WASM
3. Runs `stellar contract optimize` on the WASM
4. Runs `stellar contract deploy` using the `deployer` identity imported from `STELLAR_SECRET_KEY`

To trigger manually:
1. Go to **Actions** tab → **Deploy to Testnet**
2. Click **Run workflow**
3. Optionally specify a comma-separated list of contract names (leave blank to deploy all changed contracts)
4. Verify the deployment summary in the workflow run output

---

### Test TTL Extension Workflow (`extend-ttls.yml`)

This workflow runs on a schedule (every Monday) and can be triggered manually. It uses the `stellar` CLI (not `soroban`) and requires `MAINNET_DEPLOYER_IDENTITY` and/or `TESTNET_DEPLOYER_IDENTITY`.

To test with a dry run:
1. Go to **Actions** tab → **Extend Contract TTLs**
2. Click **Run workflow**
3. Set **Network** to `testnet`
4. Set **Dry run** to `true`
5. Verify the workflow completes without errors

---

### Test Security Audit

1. Go to **Actions** tab → **Security Audit**
2. Click **Run workflow** → **Run workflow**
3. Verify the audit completes successfully
4. Check for any security issues in the summary

---

## Step 5: Configure Notifications (Optional)

### Email Notifications

GitHub automatically sends email notifications for failed workflow runs, security issues, and pull request reviews. Configure in **Settings** → **Notifications**.

### Slack Integration (Optional)

1. Create a Slack Incoming Webhook for your workspace and copy the URL.

2. Add it as a repository secret:
   - **Name:** `SLACK_WEBHOOK_URL`
   - **Value:** Your webhook URL

3. Add a notification step to any workflow:

```yaml
- name: Notify Slack on failure
  if: failure()
  uses: slackapi/slack-github-action@v1
  with:
    webhook-url: ${{ secrets.SLACK_WEBHOOK_URL }}
    payload: |
      {
        "text": "CI Failed for ${{ github.repository }}",
        "blocks": [
          {
            "type": "section",
            "text": {
              "type": "mrkdwn",
              "text": "❌ *CI Failed*\n*Repository:* ${{ github.repository }}\n*Branch:* ${{ github.ref }}\n*Commit:* ${{ github.sha }}"
            }
          }
        ]
      }
```

---

## Step 6: Set Up Dependabot (Optional but Recommended)

Create `.github/dependabot.yml`:

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
    labels:
      - "dependencies"
      - "rust"
    commit-message:
      prefix: "chore"
      include: "scope"
```

Then commit and push:
```bash
git add .github/dependabot.yml
git commit -m "chore: add Dependabot configuration"
git push
```

---

## Step 7: Monitor and Maintain

### Weekly Tasks

1. **Review Security Audit Results** — check **Actions** → **Security Audit**, review any created issues, and update vulnerable dependencies.
2. **Review Dependabot PRs** — check changelogs and merge after CI passes.

### Monthly Tasks

1. **Review Deployment History** — check deployment artifacts, verify contract IDs are documented, and clean up old artifacts if needed.
2. **Update Workflows** — check for GitHub Actions version updates, update Rust and `stellar-cli` versions as needed, and review caching strategies.

### Quarterly Tasks

1. **Security Review** — run a comprehensive security audit, review all dependencies, and update security policies.
2. **Performance Review** — analyze CI run times, optimize slow jobs, and review caching effectiveness.

---

## Troubleshooting

### CI Fails on First Run

**Problem:** CI fails with "required checks not found"

**Solution:**
1. Let the CI run complete at least once so the job names are registered
2. Then add the checks to branch protection
3. Status checks must exist before GitHub allows them to be required

---

### Deployment Fails with "Secret not found" or "identity not found"

**Problem:** `deploy-testnet.yml` exits with an error about the deployer identity or missing secret

**Solution:**
1. Confirm `STELLAR_SECRET_KEY` is added under **Settings** → **Secrets and variables** → **Actions**
2. Check the secret name is exactly `STELLAR_SECRET_KEY` (case-sensitive)
3. Confirm the value starts with `S` (Stellar secret key format)
4. The workflow imports this key with `stellar keys add deployer --secret-key "$STELLAR_SECRET_KEY"` — if the key is malformed, that step will fail

---

### TTL Extension Fails with "identity not found" or Permission Error

**Problem:** `extend-ttls.yml` exits with an error about the deployer identity

**Solution:**
1. Confirm both `MAINNET_DEPLOYER_IDENTITY` and `TESTNET_DEPLOYER_IDENTITY` are configured in repository secrets
2. Verify neither value is empty — an empty identity will silently fail or produce a cryptic CLI error
3. Check that the identity's account has enough XLM to cover the transaction fees on the target network

---

### Security Audit Creates Too Many Issues

**Problem:** Multiple security issues created for the same vulnerabilities

**Solution:**
1. The workflow checks for existing issues before creating new ones
2. If duplicates occur, manually close the extras
3. Open a PR to improve the deduplication logic in the workflow

---

### WASM Build Fails

**Problem:** Contracts don't compile to WASM

**Solution:**
1. Test locally: `cargo build --release --target wasm32-unknown-unknown`
2. Check for platform-specific dependencies
3. Ensure all contracts use `#![no_std]`
4. Review `soroban-sdk` compatibility

---

### Deployment Takes Too Long

**Problem:** Deployment workflow times out

**Solution:**
1. The workflow only deploys changed contracts automatically — verify the change detection is working
2. Use the manual trigger to deploy specific contracts by name
3. Optimize WASM binaries before deployment
4. Consider parallel deployment for large contract sets

---

## Advanced Configuration

### Custom Deployment Environments

Create a separate workflow for mainnet releases:

```yaml
# .github/workflows/deploy-mainnet.yml
name: Deploy to Mainnet

on:
  release:
    types: [published]

env:
  STELLAR_NETWORK_PASSPHRASE: "Public Global Stellar Network ; September 2015"
  STELLAR_RPC_URL: "https://soroban-mainnet.stellar.org"

# ... rest of deployment workflow
# Use secrets.MAINNET_DEPLOYER_IDENTITY, not STELLAR_SECRET_KEY
```

### Matrix Testing

Test across multiple Rust versions:

```yaml
test:
  strategy:
    matrix:
      rust: [stable, beta, nightly]
  steps:
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
    - run: cargo test --workspace
```

### Conditional Workflows

Run workflows only when contract files change:

```yaml
on:
  push:
    paths:
      - 'contracts/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
```

---

## Branch Protection

Branch protection must be enabled on `main` so CI failures block merges. Configure it under **Settings → Branches → Add branch protection rule** for the pattern `main`.

### Required Status Checks

These checks must pass before any PR can be merged:

| Check Name | Workflow | Job |
|---|---|---|
| `CI / ci-success` | `ci.yml` | Gate job — passes only when all of Format, Clippy, Test, and Build WASM succeed |
| `CI / WASM Size Check` | `wasm-size-check.yml` | Fails if any WASM binary exceeds the size budget |

The `CI / ci-success` job is the primary gate. It is defined in `ci.yml` as the `ci-success` job and depends on `[format, clippy, test, build-wasm]`.

### Minimum Rule Settings

```
✅ Require a pull request before merging
    ✅ Require at least 1 approval
✅ Require status checks to pass before merging
    ✅ Require branches to be up to date before merging
    Required checks:
      - CI / ci-success
      - CI / WASM Size Check
✅ Do not allow bypassing the above settings
```

Direct pushes to `main` must be disabled. All changes must go through a PR that passes the required checks above.

---

## Best Practices

1. **Test locally first** — run all CI checks locally before pushing. See `.github/workflows/README.md` for the exact commands.
2. **Keep secrets secure** — never commit secrets, rotate them regularly, and use separate accounts for testnet and mainnet.
3. **Use the Secrets Reference table** — when adding a new workflow that needs credentials, update the table at the top of this document at the same time.
4. **Monitor CI performance** — review workflow run times, optimize caching, and parallelize where possible.
5. **Stay updated** — keep GitHub Actions versions, Rust, and `stellar-cli` current and review security advisories weekly.
6. **Document changes** — update this file and the workflow README whenever secrets or CLI commands change.

---

## Getting Help

- **GitHub Actions Issues:** [GitHub Community Forum](https://github.community/)
- **Stellar CLI Issues:** [Stellar Discord](https://discord.gg/stellar)
- **Security Issues:** Create a private security advisory in the repository

---

## Setup Checklist

Use this checklist to verify your setup is complete:

- [ ] `STELLAR_SECRET_KEY` secret configured (used by `deploy-testnet.yml`)
- [ ] `TESTNET_DEPLOYER_IDENTITY` secret configured (used by `extend-ttls.yml`)
- [ ] `MAINNET_DEPLOYER_IDENTITY` secret configured (used by `extend-ttls.yml`)
- [ ] GitHub Actions enabled with read and write permissions
- [ ] Branch protection rules configured for `main`
- [ ] Required status checks added to branch protection (`CI Success`, `Format Check`, `Clippy Lint`, `Test Suite`, `Build WASM`)
- [ ] CI workflow tested with a pull request
- [ ] Deployment workflow tested (manual trigger with dry run)
- [ ] TTL extension workflow tested (manual trigger, testnet, dry run)
- [ ] Security audit workflow tested
- [ ] Notifications configured (email/Slack)
- [ ] Dependabot configured (optional)
- [ ] Team members have appropriate access levels
- [ ] Monitoring and maintenance schedule established

---

## Next Steps

After completing this setup:

1. **Create a test PR** to verify all CI checks pass
2. **Trigger a manual testnet deployment** to verify `STELLAR_SECRET_KEY` is working
3. **Run a dry-run TTL extension** to verify `TESTNET_DEPLOYER_IDENTITY` is working
4. **Review the security audit** results
5. **Train team members** on the CI/CD process and point them to the Secrets Reference table first
