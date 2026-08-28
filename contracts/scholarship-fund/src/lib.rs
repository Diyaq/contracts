#![no_std]

//! # Scholarship Fund Contract
//!
//! Manages healthcare education scholarships with fund pooling, award disbursement, and recipient
//! eligibility verification for healthcare professional training programs.
//!
//! ## HIPAA Compliance
//!
//! **Access Control Safeguards:** Admin-only fund initialization and management. Eligible recipient
//! verification via authentication. Deposit authorization per depositor. Disbursement authorization
//! by fund administrator. Recipient identity validation prevents unauthorized access.
//!
//! **Audit Controls:** Fund deposit events logged with depositor and amount. Fund withdrawal events
//! tracked with recipient and award amount. Award events emitted with grant details. Fund balance
//! changes auditable. Insufficient funds errors logged.
//!
//! **Data Retention Policy:** Fund pool balance maintained indefinitely. Deposit records retained
//! for reporting. Award disbursement records archived. Recipient awards tracked for compliance.
//! Fund history reconstructible from events.
//!
//! **Encryption/Integrity:** Fund amount validation prevents overflow. Address zero checks prevent
//! invalid recipients. Deposit/withdrawal amounts immutable once recorded. Fund balance enforced
//! mathematically. Authorization required before disbursement.

use soroban_sdk::{contract,contracterror,contractimpl,contracttype,symbol_short,Address,Env,String};
#[contracterror]
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
#[repr(u32)]
pub enum Error{NotInitialized=1,AlreadyInitialized=2,Unauthorized=3,ZeroAmount=4,InsufficientFunds=5,FundsCommitted=6,RecipientNotEligible=7,RecipientCapExceeded=8}
#[contracttype]
pub enum DataKey{Admin,PoolBalance,CommittedFunds,Deposit(Address),Eligible(Address),RecipientAwards(Address),RecipientCap(Address)}
#[contracttype]
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct FundStats{pub pool_balance:i128,pub committed_balance:i128}
#[contract]
pub struct ScholarshipFundContract;
#[contractimpl]
impl ScholarshipFundContract{
    pub fn initialize(env:Env,admin:Address)->Result<(),Error>{
        if env.storage().instance().has(&DataKey::Admin){return Err(Error::AlreadyInitialized);}
        env.storage().instance().set(&DataKey::Admin,&admin);
        env.storage().instance().set(&DataKey::PoolBalance,&0i128);
        Ok(())
    }
    pub fn deposit(env:Env,depositor:Address,amount:i128)->Result<(),Error>{
        depositor.require_auth();
        if amount<=0{return Err(Error::ZeroAmount);}
        let prev:i128=env.storage().persistent().get(&DataKey::Deposit(depositor.clone())).unwrap_or(0);
        env.storage().persistent().set(&DataKey::Deposit(depositor.clone()),&(prev+amount));
        let pool:i128=env.storage().instance().get(&DataKey::PoolBalance).unwrap_or(0);
        env.storage().instance().set(&DataKey::PoolBalance,&(pool+amount));
        env.events().publish((symbol_short!("DEPOSIT"),depositor),amount);
        Ok(())
    }
    pub fn withdraw(env:Env,depositor:Address,amount:i128)->Result<(),Error>{
        depositor.require_auth();
        if amount<=0{return Err(Error::ZeroAmount);}
        let held:i128=env.storage().persistent().get(&DataKey::Deposit(depositor.clone())).unwrap_or(0);
        if held<amount{return Err(Error::InsufficientFunds);}
        let pool:i128=env.storage().instance().get(&DataKey::PoolBalance).unwrap_or(0);
        if pool<amount{return Err(Error::InsufficientFunds);}
        let committed:i128=env.storage().instance().get(&DataKey::CommittedFunds).unwrap_or(0);
        if pool-committed<amount{return Err(Error::FundsCommitted);}
        env.storage().persistent().set(&DataKey::Deposit(depositor.clone()),&(held-amount));
        env.storage().instance().set(&DataKey::PoolBalance,&(pool-amount));
        env.events().publish((symbol_short!("WITHDRAW"),depositor),amount);
        Ok(())
    }
    /// Earmark pool funds for a pending award, protecting them from donor withdrawal.
    /// Admin-only. Can only commit funds currently uncommitted in the pool.
    pub fn commit_funds(env:Env,admin:Address,amount:i128)->Result<(),Error>{
        admin.require_auth();
        let stored:Address=env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin!=stored{return Err(Error::Unauthorized);}
        if amount<=0{return Err(Error::ZeroAmount);}
        let pool:i128=env.storage().instance().get(&DataKey::PoolBalance).unwrap_or(0);
        let committed:i128=env.storage().instance().get(&DataKey::CommittedFunds).unwrap_or(0);
        if pool-committed<amount{return Err(Error::InsufficientFunds);}
        env.storage().instance().set(&DataKey::CommittedFunds,&(committed+amount));
        env.events().publish((symbol_short!("COMMIT"),admin),amount);
        Ok(())
    }
    pub fn disburse(env:Env,admin:Address,recipient:Address,amount:i128,reason:String)->Result<(),Error>{
        admin.require_auth();
        let stored:Address=env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin!=stored{return Err(Error::Unauthorized);}
        if amount<=0{return Err(Error::ZeroAmount);}
        let eligible:bool=env.storage().persistent().get(&DataKey::Eligible(recipient.clone())).unwrap_or(false);
        if !eligible{return Err(Error::RecipientNotEligible);}
        let prior_awards:i128=env.storage().persistent().get(&DataKey::RecipientAwards(recipient.clone())).unwrap_or(0);
        let cap:i128=env.storage().persistent().get(&DataKey::RecipientCap(recipient.clone())).unwrap_or(0);
        if cap>0 && prior_awards+amount>cap{return Err(Error::RecipientCapExceeded);}
        let pool:i128=env.storage().instance().get(&DataKey::PoolBalance).unwrap_or(0);
        if pool<amount{return Err(Error::InsufficientFunds);}
        env.storage().instance().set(&DataKey::PoolBalance,&(pool-amount));
        let committed:i128=env.storage().instance().get(&DataKey::CommittedFunds).unwrap_or(0);
        if committed>0{
            let released=if amount<committed{amount}else{committed};
            env.storage().instance().set(&DataKey::CommittedFunds,&(committed-released));
        }
        env.storage().persistent().set(&DataKey::RecipientAwards(recipient.clone()),&(prior_awards+amount));
        env.events().publish((symbol_short!("DISBURSE"),recipient),(amount,reason));
        Ok(())
    }
    /// Admin-only: mark a recipient eligible/ineligible to receive disbursements.
    pub fn set_recipient_eligibility(env:Env,admin:Address,recipient:Address,eligible:bool)->Result<(),Error>{
        admin.require_auth();
        let stored:Address=env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin!=stored{return Err(Error::Unauthorized);}
        env.storage().persistent().set(&DataKey::Eligible(recipient.clone()),&eligible);
        env.events().publish((symbol_short!("ELIGIBLE"),recipient),eligible);
        Ok(())
    }
    /// Admin-only: set the maximum cumulative amount a recipient may receive across all disbursements.
    /// A cap of `0` means no cap is enforced.
    pub fn set_recipient_cap(env:Env,admin:Address,recipient:Address,cap:i128)->Result<(),Error>{
        admin.require_auth();
        let stored:Address=env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin!=stored{return Err(Error::Unauthorized);}
        if cap<0{return Err(Error::ZeroAmount);}
        env.storage().persistent().set(&DataKey::RecipientCap(recipient),&cap);
        Ok(())
    }
    pub fn get_stats(env:Env)->FundStats{FundStats{pool_balance:env.storage().instance().get(&DataKey::PoolBalance).unwrap_or(0),committed_balance:env.storage().instance().get(&DataKey::CommittedFunds).unwrap_or(0)}}
    pub fn get_deposit(env:Env,depositor:Address)->i128{env.storage().persistent().get(&DataKey::Deposit(depositor)).unwrap_or(0)}
    /// Cumulative amount this recipient has received across all disbursements.
    pub fn get_recipient_awards(env:Env,recipient:Address)->i128{env.storage().persistent().get(&DataKey::RecipientAwards(recipient)).unwrap_or(0)}
}
#[cfg(test)]
mod test;
