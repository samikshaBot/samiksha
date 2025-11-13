#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Symbol, String, symbol_short};

#[contracttype]
#[derive(Clone)]
pub struct TradeCredit {
    pub credit_id: u64,
    pub issuer: Address,
    pub recipient: Address,
    pub amount: u64,
    pub issue_time: u64,
    pub is_redeemed: bool,
}

#[contracttype]
pub enum CreditBook {
    Credit(u64)
}

const CREDIT_COUNT: Symbol = symbol_short!("CR_COUNT");
const TOTAL_STATS: Symbol = symbol_short!("TOT_STAT");

#[contracttype]
#[derive(Clone)]
pub struct MarketStats {
    pub total_issued: u64,
    pub total_redeemed: u64,
    pub active_credits: u64,
}

#[contract]
pub struct TradeCreditContract;

#[contractimpl]
impl TradeCreditContract {

    pub fn issue_credit(env: Env, issuer: Address, recipient: Address, amount: u64) -> u64 {
        issuer.require_auth();

        let mut credit_count: u64 = env.storage().instance().get(&CREDIT_COUNT).unwrap_or(0);
        credit_count += 1;

        let time = env.ledger().timestamp();
        let mut stats = Self::get_market_stats(env.clone());

        let new_credit = TradeCredit {
            credit_id: credit_count,
            issuer: issuer.clone(),
            recipient: recipient.clone(),
            amount,
            issue_time: time,
            is_redeemed: false,
        };

        stats.total_issued += 1;
        stats.active_credits += 1;

        env.storage().instance().set(&CreditBook::Credit(credit_count), &new_credit);
        env.storage().instance().set(&CREDIT_COUNT, &credit_count);
        env.storage().instance().set(&TOTAL_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Trade Credit Issued - ID: {}, Amount: {}", credit_count, amount);

        credit_count
    }

    pub fn redeem_credit(env: Env, credit_id: u64, recipient: Address) {
        recipient.require_auth();

        let mut credit = Self::view_credit(env.clone(), credit_id);

        if credit.credit_id == 0 {
            log!(&env, "Credit not found!");
            panic!("Credit not found!");
        }

        if credit.recipient != recipient {
            log!(&env, "Unauthorized! Only recipient can redeem.");
            panic!("Unauthorized redemption attempt!");
        }

        if credit.is_redeemed {
            log!(&env, "Credit already redeemed!");
            panic!("Credit already redeemed!");
        }

        credit.is_redeemed = true;

        let mut stats = Self::get_market_stats(env.clone());
        stats.total_redeemed += 1;
        stats.active_credits -= 1;

        env.storage().instance().set(&CreditBook::Credit(credit_id), &credit);
        env.storage().instance().set(&TOTAL_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Trade Credit Redeemed - ID: {}", credit_id);
    }

    pub fn view_credit(env: Env, credit_id: u64) -> TradeCredit {
        let key = CreditBook::Credit(credit_id);

        env.storage().instance().get(&key).unwrap_or(TradeCredit {
            credit_id: 0,
            issuer: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            recipient: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            amount: 0,
            issue_time: 0,
            is_redeemed: false,
        })
    }

    pub fn get_market_stats(env: Env) -> MarketStats {
        env.storage().instance().get(&TOTAL_STATS).unwrap_or(MarketStats {
            total_issued: 0,
            total_redeemed: 0,
            active_credits: 0,
        })
    }
}