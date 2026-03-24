use soroban_sdk::{symbol_short, Address, Env};

use crate::types::DisputeResolution;

pub fn emit_trade_created(env: &Env, trade_id: u64, seller: Address, buyer: Address, amount: u64) {
    env.events().publish(
        (symbol_short!("created"),),
        (trade_id, seller, buyer, amount),
    );
}

pub fn emit_trade_funded(env: &Env, trade_id: u64) {
    env.events().publish((symbol_short!("funded"),), trade_id);
}

pub fn emit_trade_completed(env: &Env, trade_id: u64) {
    env.events()
        .publish((symbol_short!("complete"),), trade_id);
}

pub fn emit_trade_confirmed(env: &Env, trade_id: u64, payout: u64, fee: u64) {
    env.events()
        .publish((symbol_short!("confirm"),), (trade_id, payout, fee));
}

pub fn emit_dispute_raised(env: &Env, trade_id: u64, raised_by: Address) {
    env.events()
        .publish((symbol_short!("dispute"),), (trade_id, raised_by));
}

pub fn emit_dispute_resolved(
    env: &Env,
    trade_id: u64,
    resolution: DisputeResolution,
    recipient: Address,
) {
    env.events()
        .publish((symbol_short!("resolved"),), (trade_id, resolution, recipient));
}

pub fn emit_trade_cancelled(env: &Env, trade_id: u64) {
    env.events()
        .publish((symbol_short!("cancel"),), trade_id);
}

pub fn emit_arbitrator_registered(env: &Env, arbitrator: Address) {
    env.events()
        .publish((symbol_short!("arb_reg"),), arbitrator);
}

pub fn emit_arbitrator_removed(env: &Env, arbitrator: Address) {
    env.events()
        .publish((symbol_short!("arb_rem"),), arbitrator);
}

pub fn emit_fee_updated(env: &Env, fee_bps: u32) {
    env.events().publish((symbol_short!("fee_upd"),), fee_bps);
}

pub fn emit_fees_withdrawn(env: &Env, amount: u64, to: Address) {
    env.events()
        .publish((symbol_short!("fees_out"),), (amount, to));
}

pub fn emit_paused(env: &Env, admin: Address) {
    env.events().publish((symbol_short!("paused"),), admin);
}

pub fn emit_unpaused(env: &Env, admin: Address) {
    env.events().publish((symbol_short!("unpaused"),), admin);
}

pub fn emit_emergency_withdraw(env: &Env, to: Address, amount: u64) {
    env.events()
        .publish((symbol_short!("emrg_wd"),), (to, amount));
}

pub fn emit_metadata_updated(env: &Env, trade_id: u64) {
    env.events()
        .publish((symbol_short!("meta_upd"),), trade_id);
}

pub fn emit_tier_upgraded(env: &Env, user: Address, new_tier: crate::types::UserTier) {
    env.events()
        .publish((symbol_short!("tier_up"),), (user, new_tier));
}

pub fn emit_tier_downgraded(env: &Env, user: Address, new_tier: crate::types::UserTier) {
    env.events()
        .publish((symbol_short!("tier_dn"),), (user, new_tier));
}

pub fn emit_tier_config_updated(env: &Env) {
    env.events()
        .publish((symbol_short!("tier_cfg"),), ());
}

pub fn emit_custom_fee_set(env: &Env, user: Address, fee_bps: u32) {
    env.events()
        .publish((symbol_short!("cust_fee"),), (user, fee_bps));
}

pub fn emit_template_created(env: &Env, template_id: u64, owner: Address) {
    env.events()
        .publish((symbol_short!("tmpl_cr"),), (template_id, owner));
}

pub fn emit_template_updated(env: &Env, template_id: u64, version: u32) {
    env.events()
        .publish((symbol_short!("tmpl_up"),), (template_id, version));
}

pub fn emit_template_deactivated(env: &Env, template_id: u64) {
    env.events()
        .publish((symbol_short!("tmpl_off"),), template_id);
}

pub fn emit_template_trade(env: &Env, trade_id: u64, template_id: u64, version: u32) {
    env.events()
        .publish((symbol_short!("tmpl_tr"),), (trade_id, template_id, version));
}

pub fn emit_subscribed(env: &Env, subscriber: Address, tier: crate::types::SubscriptionTier, expires_at: u32) {
    env.events()
        .publish((symbol_short!("sub_new"),), (subscriber, tier, expires_at));
}

pub fn emit_subscription_renewed(env: &Env, subscriber: Address, tier: crate::types::SubscriptionTier, expires_at: u32) {
    env.events()
        .publish((symbol_short!("sub_ren"),), (subscriber, tier, expires_at));
}

pub fn emit_subscription_cancelled(env: &Env, subscriber: Address) {
    env.events()
        .publish((symbol_short!("sub_can"),), subscriber);
}

pub fn emit_proposal_created(env: &Env, proposal_id: u64, proposer: Address) {
    env.events()
        .publish((symbol_short!("prop_cr"),), (proposal_id, proposer));
}

pub fn emit_vote_cast(env: &Env, proposal_id: u64, voter: Address, support: bool, weight: i128) {
    env.events()
        .publish((symbol_short!("voted"),), (proposal_id, voter, support, weight));
}

pub fn emit_proposal_executed(env: &Env, proposal_id: u64) {
    env.events()
        .publish((symbol_short!("prop_ex"),), proposal_id);
}

pub fn emit_delegated(env: &Env, delegator: Address, delegatee: Address) {
    env.events()
        .publish((symbol_short!("delegat"),), (delegator, delegatee));
}

pub fn emit_fees_distributed(env: &Env, to: Address, amount: u64) {
    env.events()
        .publish((symbol_short!("fee_dst"),), (to, amount));
}

pub fn emit_privacy_set(env: &Env, trade_id: u64) {
    env.events()
        .publish((symbol_short!("priv_set"),), trade_id);
}

pub fn emit_disclosure_granted(env: &Env, trade_id: u64, grantee: Address) {
    env.events()
        .publish((symbol_short!("disc_gr"),), (trade_id, grantee));
}

pub fn emit_disclosure_revoked(env: &Env, trade_id: u64, grantee: Address) {
    env.events()
        .publish((symbol_short!("disc_rv"),), (trade_id, grantee));
}