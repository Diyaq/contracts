Issue1:#684 security: prior-authorization review_authorization has no insurer binding — any self-registered reviewer can decide any request

Summary
AuthorizationRequest in contracts/prior-authorization/src/types.rs (lines 58-82) never stores the insurer_wallet passed into submit_prior_authorization — it's used once for the coverage-plan check and discarded. review_authorization (lines 301-449) only checks that the reviewer exists, is active, and optionally has the right role for the SLA config — it never verifies the reviewer's insurer_id matches the insurer the request was actually submitted against. register_reviewer (~lines 776-807) lets any address require_auth() itself as insurer_id and register itself as a medical_director reviewer with no verification.

Risk
Any unaffiliated actor can register as a reviewer for an arbitrary self-declared insurer, then approve or deny any patient's authorization request in the system regardless of which real insurer owns the policy — a full authorization-decision bypass.

Acceptance Criteria
 Store insurer_wallet/insurer_id on AuthorizationRequest at submission
 review_authorization verifies the reviewer's insurer_id matches the request's insurer
 register_reviewer verifies the caller is actually authorized to register reviewers for the claimed insurer_id (e.g. cross-check against insurer-registry)
 Add a test: reviewer registered under insurer B attempts to review a request submitted against insurer A, assert rejection



Issue2:#682 bug: hospital-registry has no function that ever sets revoked_at — credentials can never actually be revoked

Summary
CredentialAnchor.revoked_at: Option (line 64) and Error::CredentialRevoked (line 48) exist, and assert_active_credential (~lines 193-201) actively checks revoked_at.is_some() to reject revoked hospitals. But revoked_at is only ever written once, at registration, hard-coded to None (line 316) — no public function anywhere in the contract can ever set it to Some(_).

Risk
The CredentialRevoked error path is dead/unreachable — an admin has no way to disable a compromised or fraudulent hospital's credential before its natural expiry. The sibling provider-registry::revoke_provider implements exactly this capability, reinforcing that this is an omission.

Acceptance Criteria
 Add an admin-only revoke_hospital_credential function that sets revoked_at
 Add a test: revoke a hospital's credential, assert subsequent state-mutating calls fail with CredentialRevoked


Issue3:#681 bug: mental-health therapy session, symptom, and outcome records silently overwrite on same-day collision

Summary
record_therapy_session stores under key Session(treatment_plan_id, session_date), track_symptom_severity under Symptom(patient_id, symptom_type, measurement_date), and track_treatment_outcomes under Outcomes(treatment_plan_id, measurement_date) in contracts/mental-health/src/lib.rs (~lines 600, 636-639, 755-758). None check for an existing entry before set().

Risk
Two therapy sessions, symptom measurements, or outcome recordings on the same patient/day (e.g. individual + group therapy same date, or two symptom check-ins same day) silently overwrite and permanently destroy the earlier clinical record, with no error or event indicating data loss.

Acceptance Criteria
 Key these records with a unique id (e.g. counter or timestamp with sub-day granularity) instead of date alone, or append to a list per day
 Add a test: record two sessions/symptoms/outcomes on the same day, assert both are retained



Issue4:#683 security: multisig-governance initialize has no require_auth, allowing admin-set hijack via front-running

Summary
initialize() in contracts/multisig-governance/src/lib.rs (~lines 130-152) never calls .require_auth() on any address — it's only gated by has(&DataKey::Signers).

Risk
Any account can call initialize first, front-running the legitimate deployer's initialization transaction, and install itself as the sole signer with threshold=1, permanently locking out the intended admin/signer set — a classic unprotected-initializer vulnerability on a governance contract that gates contract upgrades.

Acceptance Criteria
 Require the deploying/admin address to sign initialize (e.g. via a constructor pattern or a designated deployer check)
 Add a test simulating a front-run initialize call and assert it cannot hijack signer control (or document the deployment-ordering guarantee that prevents this)