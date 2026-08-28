#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _,Address,Env,String};
fn setup()->(Env,ScholarshipFundContractClient<'static>,Address){
    let env=Env::default();env.mock_all_auths();
    let id=env.register(ScholarshipFundContract,());
    let c=ScholarshipFundContractClient::new(&env,&id);
    let admin=Address::generate(&env);c.initialize(&admin);(env,c,admin)
}
#[test]fn deposit_increases_pool(){let(_,c,_)=setup();let d=Address::generate(&c.env);c.deposit(&d,&500_000);assert_eq!(c.get_stats().pool_balance,500_000);}
#[test]fn withdraw_reduces_pool(){let(_,c,_)=setup();let d=Address::generate(&c.env);c.deposit(&d,&1_000_000);c.withdraw(&d,&400_000);assert_eq!(c.get_stats().pool_balance,600_000);}
#[test]fn disburse_reduces_pool(){let(env,c,admin)=setup();let donor=Address::generate(&env);let student=Address::generate(&env);c.deposit(&donor,&2_000_000);c.set_recipient_eligibility(&admin,&student,&true);c.disburse(&admin,&student,&1_000_000,&String::from_str(&env,"award"));assert_eq!(c.get_stats().pool_balance,1_000_000);}
#[test]#[should_panic]fn disburse_empty_pool_panics(){let(env,c,admin)=setup();let s=Address::generate(&env);c.set_recipient_eligibility(&admin,&s,&true);c.disburse(&admin,&s,&1,&String::from_str(&env,"x"));}
#[test]#[should_panic]fn non_admin_disburse_panics(){let(env,c,admin)=setup();let a=Address::generate(&env);let s=Address::generate(&env);let d=Address::generate(&env);c.deposit(&d,&1_000_000);c.set_recipient_eligibility(&admin,&s,&true);c.disburse(&a,&s,&1,&String::from_str(&env,"x"));}
#[test]#[should_panic]fn over_withdraw_panics(){let(_,c,_)=setup();let d=Address::generate(&c.env);c.deposit(&d,&100_000);c.withdraw(&d,&200_000);}
#[test]
fn committed_funds_survive_a_pending_award() {
    let (env, c, admin) = setup();
    let donor = Address::generate(&env);
    let student = Address::generate(&env);

    // Donor deposits, admin plans an award and earmarks the funds for it.
    c.deposit(&donor, &1_000_000);
    c.commit_funds(&admin, &1_000_000);

    // Donor can no longer withdraw funds that are earmarked for the pending award.
    let result = c.try_withdraw(&donor, &1_000_000);
    assert_eq!(result, Err(Ok(Error::FundsCommitted)));

    // The planned disbursement still succeeds because the funds were protected.
    c.set_recipient_eligibility(&admin, &student, &true);
    c.disburse(&admin, &student, &1_000_000, &String::from_str(&env, "award"));
    assert_eq!(c.get_stats().pool_balance, 0);
    assert_eq!(c.get_stats().committed_balance, 0);
}
#[test]
fn uncommitted_funds_remain_withdrawable() {
    let (env, c, admin) = setup();
    let donor = Address::generate(&env);

    c.deposit(&donor, &1_000_000);
    c.commit_funds(&admin, &400_000);

    // Only the committed portion is protected; the rest can still be withdrawn.
    c.withdraw(&donor, &600_000);
    assert_eq!(c.get_stats().pool_balance, 400_000);
}
#[test]
fn disburse_to_ineligible_recipient_is_rejected() {
    let (env, c, admin) = setup();
    let donor = Address::generate(&env);
    let student = Address::generate(&env);
    c.deposit(&donor, &1_000_000);
    let result = c.try_disburse(&admin, &student, &500_000, &String::from_str(&env, "award"));
    assert_eq!(result, Err(Ok(Error::RecipientNotEligible)));
    assert_eq!(c.get_stats().pool_balance, 1_000_000);
}
#[test]
fn recipient_cap_limits_cumulative_awards() {
    let (env, c, admin) = setup();
    let donor = Address::generate(&env);
    let student = Address::generate(&env);
    c.deposit(&donor, &3_000_000);
    c.set_recipient_eligibility(&admin, &student, &true);
    c.set_recipient_cap(&admin, &student, &1_500_000);

    c.disburse(&admin, &student, &1_000_000, &String::from_str(&env, "award-1"));
    assert_eq!(c.get_recipient_awards(&student), 1_000_000);

    // Second award would push cumulative total past the cap.
    let result = c.try_disburse(&admin, &student, &1_000_000, &String::from_str(&env, "award-2"));
    assert_eq!(result, Err(Ok(Error::RecipientCapExceeded)));

    // A smaller award that stays within the cap still succeeds.
    c.disburse(&admin, &student, &500_000, &String::from_str(&env, "award-2"));
    assert_eq!(c.get_recipient_awards(&student), 1_500_000);
}
#[test]
#[should_panic]
fn non_admin_set_eligibility_panics() {
    let (env, c, _admin) = setup();
    let attacker = Address::generate(&env);
    let student = Address::generate(&env);
    c.set_recipient_eligibility(&attacker, &student, &true);
}
