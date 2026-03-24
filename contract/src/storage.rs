use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::types::{TierConfig, Trade, TradeTemplate, UserTierInfo, Subscription, Proposal, TradePrivacy, DisclosureGrant};

const INITIALIZED: &str = "INIT";
const ADMIN: &str = "ADMIN";
const USDC_TOKEN: &str = "USDC";
const FEE_BPS: &str = "FEE_BPS";
const TRADE_COUNTER: &str = "COUNTER";
const ACCUMULATED_FEES: &str = "ACC_FEES";
const TRADE_PREFIX: &str = "TRADE";
const ARBITRATOR_PREFIX: &str = "ARB";
const PAUSED: &str = "PAUSED";
const TIER_CONFIG: &str = "TIER_CFG";
const USER_TIER_PREFIX: &str = "UTIER";
const TEMPLATE_COUNTER: &str = "TMPL_CTR";
const TEMPLATE_PREFIX: &str = "TMPL";
const SUBSCRIPTION_PREFIX: &str = "SUB";
const GOV_TOKEN: &str = "GOV_TKN";
const PROPOSAL_COUNTER: &str = "PROP_CTR";
const PROPOSAL_PREFIX: &str = "PROP";
const VOTE_PREFIX: &str = "VOTE";
const DELEGATE_PREFIX: &str = "DELEG";
const TRADE_PRIVACY_PREFIX: &str = "TPRIV";
const DISCLOSURE_PREFIX: &str = "DISC";

// Initialization
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

pub fn has_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&INITIALIZED, &true);
}

// Admin
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&ADMIN)
        .ok_or(ContractError::NotInitialized)
}

// USDC Token
pub fn set_usdc_token(env: &Env, token: &Address) {
    env.storage().instance().set(&USDC_TOKEN, token);
}

pub fn get_usdc_token(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&USDC_TOKEN)
        .ok_or(ContractError::NotInitialized)
}

// Fee BPS
pub fn set_fee_bps(env: &Env, fee_bps: u32) {
    env.storage().instance().set(&FEE_BPS, &fee_bps);
}

pub fn get_fee_bps(env: &Env) -> Result<u32, ContractError> {
    env.storage()
        .instance()
        .get(&FEE_BPS)
        .ok_or(ContractError::NotInitialized)
}

// Trade Counter
pub fn set_trade_counter(env: &Env, counter: u64) {
    env.storage().instance().set(&TRADE_COUNTER, &counter);
}

pub fn get_trade_counter(env: &Env) -> Result<u64, ContractError> {
    env.storage()
        .instance()
        .get(&TRADE_COUNTER)
        .ok_or(ContractError::NotInitialized)
}

pub fn increment_trade_counter(env: &Env) -> Result<u64, ContractError> {
    let current = get_trade_counter(env)?;
    let next = current.checked_add(1).ok_or(ContractError::Overflow)?;
    set_trade_counter(env, next);
    Ok(next)
}

// Accumulated Fees
pub fn set_accumulated_fees(env: &Env, fees: u64) {
    env.storage().instance().set(&ACCUMULATED_FEES, &fees);
}

pub fn get_accumulated_fees(env: &Env) -> Result<u64, ContractError> {
    env.storage()
        .instance()
        .get(&ACCUMULATED_FEES)
        .ok_or(ContractError::NotInitialized)
}

// Trades
pub fn save_trade(env: &Env, trade_id: u64, trade: &Trade) {
    let key = (TRADE_PREFIX, trade_id);
    env.storage().persistent().set(&key, trade);
}

pub fn get_trade(env: &Env, trade_id: u64) -> Result<Trade, ContractError> {
    let key = (TRADE_PREFIX, trade_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::TradeNotFound)
}

// Arbitrators
pub fn save_arbitrator(env: &Env, arbitrator: &Address) {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().set(&key, &true);
}

pub fn remove_arbitrator(env: &Env, arbitrator: &Address) {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().remove(&key);
}

pub fn has_arbitrator(env: &Env, arbitrator: &Address) -> bool {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().has(&key)
}

// ---------------------------------------------------------------------------
// Pause state
// ---------------------------------------------------------------------------

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&PAUSED, &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&PAUSED).unwrap_or(false)
}

// Tier config
pub fn save_tier_config(env: &Env, config: &TierConfig) {
    env.storage().instance().set(&TIER_CONFIG, config);
}

pub fn get_tier_config(env: &Env) -> Option<TierConfig> {
    env.storage().instance().get(&TIER_CONFIG)
}

// Per-user tier
pub fn save_user_tier(env: &Env, user: &Address, info: &UserTierInfo) {
    let key = (USER_TIER_PREFIX, user);
    env.storage().persistent().set(&key, info);
}

pub fn get_user_tier(env: &Env, user: &Address) -> Option<UserTierInfo> {
    let key = (USER_TIER_PREFIX, user);
    env.storage().persistent().get(&key)
}

// Template storage
pub fn get_template_counter(env: &Env) -> u64 {
    env.storage().instance().get(&TEMPLATE_COUNTER).unwrap_or(0)
}

pub fn increment_template_counter(env: &Env) -> Result<u64, crate::errors::ContractError> {
    let next = get_template_counter(env)
        .checked_add(1)
        .ok_or(crate::errors::ContractError::Overflow)?;
    env.storage().instance().set(&TEMPLATE_COUNTER, &next);
    Ok(next)
}

pub fn save_template(env: &Env, template_id: u64, template: &TradeTemplate) {
    let key = (TEMPLATE_PREFIX, template_id);
    env.storage().persistent().set(&key, template);
}

pub fn get_template(env: &Env, template_id: u64) -> Result<TradeTemplate, crate::errors::ContractError> {
    let key = (TEMPLATE_PREFIX, template_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(crate::errors::ContractError::TemplateNotFound)
}

// Subscriptions
pub fn save_subscription(env: &Env, subscriber: &Address, sub: &Subscription) {
    let key = (SUBSCRIPTION_PREFIX, subscriber);
    env.storage().persistent().set(&key, sub);
}

pub fn get_subscription(env: &Env, subscriber: &Address) -> Option<Subscription> {
    let key = (SUBSCRIPTION_PREFIX, subscriber);
    env.storage().persistent().get(&key)
}

pub fn remove_subscription(env: &Env, subscriber: &Address) {
    let key = (SUBSCRIPTION_PREFIX, subscriber);
    env.storage().persistent().remove(&key);
}


// Governance
pub fn set_gov_token(env: &Env, token: &Address) {
    env.storage().instance().set(&GOV_TOKEN, token);
}
pub fn get_gov_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&GOV_TOKEN)
}
pub fn get_proposal_counter(env: &Env) -> u64 {
    env.storage().instance().get(&PROPOSAL_COUNTER).unwrap_or(0)
}
pub fn increment_proposal_counter(env: &Env) -> Result<u64, crate::errors::ContractError> {
    let next = get_proposal_counter(env).checked_add(1).ok_or(crate::errors::ContractError::Overflow)?;
    env.storage().instance().set(&PROPOSAL_COUNTER, &next);
    Ok(next)
}
pub fn save_proposal(env: &Env, id: u64, proposal: &Proposal) {
    let key = (PROPOSAL_PREFIX, id);
    env.storage().persistent().set(&key, proposal);
}
pub fn get_proposal(env: &Env, id: u64) -> Result<Proposal, crate::errors::ContractError> {
    let key = (PROPOSAL_PREFIX, id);
    env.storage().persistent().get(&key).ok_or(crate::errors::ContractError::ProposalNotFound)
}
pub fn has_voted(env: &Env, proposal_id: u64, voter: &Address) -> bool {
    let key = (VOTE_PREFIX, proposal_id, voter);
    env.storage().persistent().has(&key)
}
pub fn mark_voted(env: &Env, proposal_id: u64, voter: &Address) {
    let key = (VOTE_PREFIX, proposal_id, voter);
    env.storage().persistent().set(&key, &true);
}
pub fn set_delegate(env: &Env, delegator: &Address, delegatee: &Address) {
    let key = (DELEGATE_PREFIX, delegator);
    env.storage().persistent().set(&key, delegatee);
}
pub fn get_delegate(env: &Env, delegator: &Address) -> Option<Address> {
    let key = (DELEGATE_PREFIX, delegator);
    env.storage().persistent().get(&key)
}
pub fn remove_delegate(env: &Env, delegator: &Address) {
    let key = (DELEGATE_PREFIX, delegator);
    env.storage().persistent().remove(&key);
}

// Privacy
pub fn save_trade_privacy(env: &Env, trade_id: u64, privacy: &TradePrivacy) {
    let key = (TRADE_PRIVACY_PREFIX, trade_id);
    env.storage().persistent().set(&key, privacy);
}
pub fn get_trade_privacy(env: &Env, trade_id: u64) -> Option<TradePrivacy> {
    let key = (TRADE_PRIVACY_PREFIX, trade_id);
    env.storage().persistent().get(&key)
}
pub fn save_disclosure_grant(env: &Env, trade_id: u64, grantee: &Address, grant: &DisclosureGrant) {
    let key = (DISCLOSURE_PREFIX, trade_id, grantee);
    env.storage().persistent().set(&key, grant);
}
pub fn get_disclosure_grant(env: &Env, trade_id: u64, grantee: &Address) -> Option<DisclosureGrant> {
    let key = (DISCLOSURE_PREFIX, trade_id, grantee);
    env.storage().persistent().get(&key)
}
pub fn remove_disclosure_grant(env: &Env, trade_id: u64, grantee: &Address) {
    let key = (DISCLOSURE_PREFIX, trade_id, grantee);
    env.storage().persistent().remove(&key);
}

