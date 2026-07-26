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
#[test]fn disburse_reduces_pool(){let(env,c,admin)=setup();let donor=Address::generate(&env);let student=Address::generate(&env);c.deposit(&donor,&2_000_000);c.disburse(&admin,&student,&1_000_000,&String::from_str(&env,"award"));assert_eq!(c.get_stats().pool_balance,1_000_000);}
#[test]#[should_panic]fn disburse_empty_pool_panics(){let(env,c,admin)=setup();let s=Address::generate(&env);c.disburse(&admin,&s,&1,&String::from_str(&env,"x"));}
#[test]#[should_panic]fn non_admin_disburse_panics(){let(env,c,_)=setup();let a=Address::generate(&env);let s=Address::generate(&env);let d=Address::generate(&env);c.deposit(&d,&1_000_000);c.disburse(&a,&s,&1,&String::from_str(&env,"x"));}
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
