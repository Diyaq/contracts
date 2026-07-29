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
#[test]#[should_panic]fn disburse_to_ineligible_recipient_panics(){let(env,c,admin)=setup();let donor=Address::generate(&env);let s=Address::generate(&env);c.deposit(&donor,&1_000_000);c.disburse(&admin,&s,&1,&String::from_str(&env,"x"));}
#[test]fn disburse_tracks_recipient_awards(){let(env,c,admin)=setup();let donor=Address::generate(&env);let student=Address::generate(&env);c.deposit(&donor,&2_000_000);c.set_recipient_eligibility(&admin,&student,&true);c.disburse(&admin,&student,&500_000,&String::from_str(&env,"award-1"));c.disburse(&admin,&student,&300_000,&String::from_str(&env,"award-2"));assert_eq!(c.get_recipient_awards(&student),800_000);}
#[test]#[should_panic]fn disburse_exceeding_cap_panics(){let(env,c,admin)=setup();let donor=Address::generate(&env);let student=Address::generate(&env);c.deposit(&donor,&2_000_000);c.set_recipient_eligibility(&admin,&student,&true);c.set_recipient_cap(&admin,&student,&500_000);c.disburse(&admin,&student,&400_000,&String::from_str(&env,"award-1"));c.disburse(&admin,&student,&200_000,&String::from_str(&env,"award-2"));}
#[test]fn disburse_up_to_cap_succeeds(){let(env,c,admin)=setup();let donor=Address::generate(&env);let student=Address::generate(&env);c.deposit(&donor,&2_000_000);c.set_recipient_eligibility(&admin,&student,&true);c.set_recipient_cap(&admin,&student,&500_000);c.disburse(&admin,&student,&500_000,&String::from_str(&env,"award"));assert_eq!(c.get_recipient_awards(&student),500_000);}
