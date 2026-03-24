#![no_std]

mod errors;
mod events;
mod governance;
mod privacy;
mod storage;
mod subscription;
mod templates;
mod tiers;
mod types;

use soroban_sdk::{contract, contractimpl, token, Address, Env};

use types::{METADATA_MAX_ENTRIES, METADATA_MAX_VALUE_LEN};

pub use errors::ContractError;
pub use types::{
    DisclosureGrant, DisputeResolution, MetadataEntry, Proposal, ProposalAction, ProposalStatus,
    Subscription, SubscriptionTier, TierConfig, TemplateTerms, TemplateVersion,
    Trade, TradeMetadata, TradePrivacy, TradeStatus, TradeTemplate, UserTier, UserTierInfo,
};

use storage::{
    get_accumulated_fees, get_admin, get_fee_bps, get_trade, get_usdc_token,
    has_arbitrator, has_initialized, increment_trade_counter, is_initialized, is_paused,
    remove_arbitrator, save_arbitrator, save_trade, set_accumulated_fees, set_admin, set_fee_bps,
    set_initialized, set_paused, set_trade_counter, set_usdc_token,
};

fn token_client<'a>(env: &'a Env, token: &Address) -> token::Client<'a> {
    token::Client::new(env, token)
}

fn validate_metadata(meta: &TradeMetadata) -> Result<(), ContractError> {
    if meta.entries.len() > METADATA_MAX_ENTRIES {
        return Err(ContractError::MetadataTooManyEntries);
    }
    for i in 0..meta.entries.len() {
        let entry = meta.entries.get(i).unwrap();
        if entry.value.len() > METADATA_MAX_VALUE_LEN {
            return Err(ContractError::MetadataValueTooLong);
        }
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}

#[contract]
pub struct StellarEscrowContract;

#[contractimpl]
impl StellarEscrowContract {
    /// Initialize the contract with admin, USDC token address, and platform fee
    pub fn initialize(env: Env, admin: Address, usdc_token: Address, fee_bps: u32) -> Result<(), ContractError> {
        if is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        if fee_bps > 10000 {
            return Err(ContractError::InvalidFeeBps);
        }
        admin.require_auth();
        set_admin(&env, &admin);
        set_usdc_token(&env, &usdc_token);
        set_fee_bps(&env, fee_bps);
        set_trade_counter(&env, 0);
        set_accumulated_fees(&env, 0);
        set_initialized(&env);
        Ok(())
    }

    /// Register an arbitrator (admin only)
    pub fn register_arbitrator(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let admin = get_admin(&env)?;
        admin.require_auth();
        save_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_registered(&env, arbitrator);
        Ok(())
    }

    /// Remove an arbitrator (admin only)
    pub fn remove_arbitrator_fn(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let admin = get_admin(&env)?;
        admin.require_auth();
        remove_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_removed(&env, arbitrator);
        Ok(())
    }

    /// Update platform fee (admin only)
    pub fn update_fee(env: Env, fee_bps: u32) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        if fee_bps > 10000 {
            return Err(ContractError::InvalidFeeBps);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_fee_bps(&env, fee_bps);
        events::emit_fee_updated(&env, fee_bps);
        Ok(())
    }

    /// Withdraw accumulated fees (admin only)
    pub fn withdraw_fees(env: Env, to: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        let fees = get_accumulated_fees(&env)?;
        if fees == 0 {
            return Err(ContractError::NoFeesToWithdraw);
        }
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &to, &(fees as i128));
        set_accumulated_fees(&env, 0);
        events::emit_fees_withdrawn(&env, fees, to);
        Ok(())
    }

    /// Create a new trade with optional metadata
    pub fn create_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
        metadata: Option<TradeMetadata>,
    ) -> Result<u64, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        seller.require_auth();
        if let Some(ref arb) = arbitrator {
            if !has_arbitrator(&env, arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }
        if let Some(ref meta) = metadata {
            validate_metadata(meta)?;
        }
        let trade_id = increment_trade_counter(&env)?;
        let fee_bps = get_fee_bps(&env)?;
        let effective_bps = tiers::effective_fee_bps(&env, &seller, fee_bps);
        let discount = subscription::subscription_discount_bps(&env, &seller);
        let final_bps = effective_bps.saturating_sub(discount);
        let fee = amount
            .checked_mul(final_bps as u64)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::Overflow)?;

        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator,
            status: TradeStatus::Created,
            metadata,
        };
        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount);
        Ok(trade_id)
    }

    /// Buyer funds the trade
    pub fn fund_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(
            &trade.buyer,
            &env.current_contract_address(),
            &(trade.amount as i128),
        );
        trade.status = TradeStatus::Funded;
        save_trade(&env, trade_id, &trade);
        events::emit_trade_funded(&env, trade_id);
        Ok(())
    }

    /// Seller marks trade as completed
    pub fn complete_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Completed;
        save_trade(&env, trade_id, &trade);
        events::emit_trade_completed(&env, trade_id);
        Ok(())
    }

    /// Buyer confirms receipt and releases funds
    pub fn confirm_receipt(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        token_client.transfer(
            &env.current_contract_address(),
            &trade.seller,
            &(payout as i128),
        );
        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);
        tiers::record_volume(&env, &trade.seller, trade.amount)?;
        tiers::record_volume(&env, &trade.buyer, trade.amount)?;
        events::emit_trade_confirmed(&env, trade_id, payout, trade.fee);
        Ok(())
    }

    /// Raise a dispute
    pub fn raise_dispute(env: Env, trade_id: u64, caller: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        if trade.arbitrator.is_none() {
            return Err(ContractError::ArbitratorNotRegistered);
        }
        if caller != trade.buyer && caller != trade.seller {
            return Err(ContractError::Unauthorized);
        }
        caller.require_auth();
        trade.status = TradeStatus::Disputed;
        save_trade(&env, trade_id, &trade);
        events::emit_dispute_raised(&env, trade_id, caller);
        Ok(())
    }

    /// Resolve a dispute (arbitrator only)
    pub fn resolve_dispute(
        env: Env,
        trade_id: u64,
        resolution: DisputeResolution,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Disputed {
            return Err(ContractError::InvalidStatus);
        }
        let arbitrator = trade.arbitrator.ok_or(ContractError::ArbitratorNotRegistered)?;
        arbitrator.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);
        let recipient = match resolution {
            DisputeResolution::ReleaseToBuyer => trade.buyer.clone(),
            DisputeResolution::ReleaseToSeller => trade.seller.clone(),
        };
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        token_client.transfer(
            &env.current_contract_address(),
            &recipient,
            &(payout as i128),
        );
        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);
        events::emit_dispute_resolved(&env, trade_id, resolution, recipient);
        Ok(())
    }

    /// Cancel an unfunded trade
    pub fn cancel_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        save_trade(&env, trade_id, &trade);
        events::emit_trade_cancelled(&env, trade_id);
        Ok(())
    }

    /// Get trade details
    pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, ContractError> {
        get_trade(&env, trade_id)
    }

    /// Get accumulated fees
    pub fn get_accumulated_fees(env: Env) -> Result<u64, ContractError> {
        get_accumulated_fees(&env)
    }

    /// Check if arbitrator is registered
    pub fn is_arbitrator_registered(env: Env, arbitrator: Address) -> bool {
        has_arbitrator(&env, &arbitrator)
    }

    /// Get platform fee in basis points
    pub fn get_platform_fee_bps(env: Env) -> Result<u32, ContractError> {
        get_fee_bps(&env)
    }

    // -------------------------------------------------------------------------
    // Emergency Pause
    // -------------------------------------------------------------------------

    /// Pause all contract operations (admin only).
    pub fn pause(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, true);
        events::emit_paused(&env, admin);
        Ok(())
    }

    /// Unpause the contract (admin only).
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, false);
        events::emit_unpaused(&env, admin);
        Ok(())
    }

    /// Emergency withdrawal of all contract token balance (admin only).
    pub fn emergency_withdraw(env: Env, to: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = token::Client::new(&env, &token);
        let balance = token_client.balance(&env.current_contract_address());
        if balance > 0 {
            token_client.transfer(&env.current_contract_address(), &to, &balance);
        }
        set_accumulated_fees(&env, 0);
        events::emit_emergency_withdraw(&env, to, balance as u64);
        Ok(())
    }

    /// Returns true if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    /// Update or replace metadata on an existing trade (seller only)
    pub fn update_trade_metadata(
        env: Env,
        trade_id: u64,
        metadata: Option<TradeMetadata>,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let mut trade = get_trade(&env, trade_id)?;
        trade.seller.require_auth();
        if let Some(ref meta) = metadata {
            validate_metadata(meta)?;
        }
        trade.metadata = metadata;
        save_trade(&env, trade_id, &trade);
        events::emit_metadata_updated(&env, trade_id);
        Ok(())
    }

    /// Get metadata for a trade
    pub fn get_trade_metadata(
        env: Env,
        trade_id: u64,
    ) -> Result<Option<TradeMetadata>, ContractError> {
        let trade = get_trade(&env, trade_id)?;
        Ok(trade.metadata)
    }

    // -------------------------------------------------------------------------
    // Fee Tier System
    // -------------------------------------------------------------------------

    /// Admin: configure fee rates per tier.
    pub fn set_tier_config(env: Env, config: TierConfig) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::set_tier_config(&env, &config)
    }

    /// Admin: assign a custom fee rate to a specific user.
    pub fn set_user_custom_fee(env: Env, user: Address, fee_bps: u32) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::set_custom_fee(&env, &user, fee_bps)
    }

    /// Admin: remove a user's custom fee, reverting to volume-based tier.
    pub fn remove_user_custom_fee(env: Env, user: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        tiers::remove_custom_fee(&env, &user);
        Ok(())
    }

    /// Query a user's current tier info.
    pub fn get_user_tier(env: Env, user: Address) -> Option<UserTierInfo> {
        storage::get_user_tier(&env, &user)
    }

    /// Query the current tier fee configuration.
    pub fn get_tier_config(env: Env) -> Option<TierConfig> {
        storage::get_tier_config(&env)
    }

    /// Query the effective fee bps for a user's next trade.
    pub fn get_effective_fee_bps(env: Env, user: Address) -> Result<u32, ContractError> {
        let base = get_fee_bps(&env)?;
        Ok(tiers::effective_fee_bps(&env, &user, base))
    }

    // -------------------------------------------------------------------------
    // Trade Templates
    // -------------------------------------------------------------------------

    /// Create a reusable trade template (owner = seller).
    pub fn create_template(
        env: Env,
        owner: Address,
        name: soroban_sdk::String,
        terms: TemplateTerms,
    ) -> Result<u64, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        owner.require_auth();
        templates::create_template(&env, &owner, name, terms)
    }

    /// Update a template with new terms, bumping its version.
    pub fn update_template(
        env: Env,
        caller: Address,
        template_id: u64,
        name: soroban_sdk::String,
        terms: TemplateTerms,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        caller.require_auth();
        templates::update_template(&env, &caller, template_id, name, terms)
    }

    /// Deactivate a template so it can no longer be used to create trades.
    pub fn deactivate_template(
        env: Env,
        caller: Address,
        template_id: u64,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        caller.require_auth();
        templates::deactivate_template(&env, &caller, template_id)
    }

    /// Create a trade from a template. Applies the template's current terms;
    /// `amount` must match `terms.fixed_amount` when one is set.
    pub fn create_trade_from_template(
        env: Env,
        seller: Address,
        buyer: Address,
        template_id: u64,
        amount: u64,
    ) -> Result<u64, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        seller.require_auth();

        let (terms, version) = templates::resolve_terms(&env, template_id)?;

        if let Some(fixed) = terms.fixed_amount {
            if amount != fixed {
                return Err(ContractError::TemplateAmountMismatch);
            }
        }
        if let Some(ref arb) = terms.default_arbitrator {
            if !has_arbitrator(&env, arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }

        let trade_id = increment_trade_counter(&env)?;
        let base_fee_bps = get_fee_bps(&env)?;
        let effective_bps = tiers::effective_fee_bps(&env, &seller, base_fee_bps);
        let discount = subscription::subscription_discount_bps(&env, &seller);
        let final_bps = effective_bps.saturating_sub(discount);
        let fee = amount
            .checked_mul(final_bps as u64)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::Overflow)?;

        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator: terms.default_arbitrator,
            status: TradeStatus::Created,
            metadata: terms.default_metadata,
        };

        save_trade(&env, trade_id, &trade);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount);
        events::emit_template_trade(&env, trade_id, template_id, version);
        Ok(trade_id)
    }

    /// Get a template by ID.
    pub fn get_template(env: Env, template_id: u64) -> Result<TradeTemplate, ContractError> {
        storage::get_template(&env, template_id)
    }

    // -------------------------------------------------------------------------
    // Subscription Model
    // -------------------------------------------------------------------------

    /// Purchase a new subscription. Payment (USDC) is transferred to the admin.
    pub fn subscribe(
        env: Env,
        subscriber: Address,
        tier: SubscriptionTier,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        subscriber.require_auth();
        let admin = get_admin(&env)?;
        subscription::subscribe(&env, &subscriber, tier, &admin)
    }

    /// Renew an existing subscription for another period.
    pub fn renew_subscription(env: Env, subscriber: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        subscriber.require_auth();
        let admin = get_admin(&env)?;
        subscription::renew(&env, &subscriber, &admin)
    }

    /// Cancel a subscription immediately (no refund).
    pub fn cancel_subscription(env: Env, subscriber: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        subscriber.require_auth();
        subscription::cancel(&env, &subscriber)
    }

    /// Get subscription details for a user.
    pub fn get_subscription(env: Env, subscriber: Address) -> Option<Subscription> {
        subscription::get(&env, &subscriber)
    }

    // -------------------------------------------------------------------------
    // Governance
    // -------------------------------------------------------------------------

    /// Admin: set the governance token address (one-time setup).
    pub fn set_gov_token(env: Env, token: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        storage::set_gov_token(&env, &token);
        Ok(())
    }

    /// Create a governance proposal.
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        action: ProposalAction,
    ) -> Result<u64, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        proposer.require_auth();
        governance::create_proposal(&env, &proposer, action)
    }

    /// Vote on a proposal.
    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        support: bool,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        voter.require_auth();
        governance::cast_vote(&env, &voter, proposal_id, support)
    }

    /// Execute a passed proposal after voting ends.
    pub fn execute_proposal(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        governance::execute_proposal(&env, proposal_id)
    }

    /// Delegate voting power to another address.
    pub fn delegate(env: Env, delegator: Address, delegatee: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        delegator.require_auth();
        governance::delegate(&env, &delegator, &delegatee);
        Ok(())
    }

    /// Remove delegation, reclaiming own voting power.
    pub fn undelegate(env: Env, delegator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        delegator.require_auth();
        governance::undelegate(&env, &delegator);
        Ok(())
    }

    /// Get a proposal by ID.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        governance::get(&env, proposal_id)
    }

    /// Get total number of proposals created.
    pub fn get_proposal_count(env: Env) -> u64 {
        governance::proposal_count(&env)
    }

    // -------------------------------------------------------------------------
    // Privacy Features
    // -------------------------------------------------------------------------

    /// Set privacy settings for a trade (seller or buyer only).
    pub fn set_trade_privacy(
        env: Env,
        caller: Address,
        trade_id: u64,
        data_hash: soroban_sdk::String,
        encrypted_ptr: Option<soroban_sdk::String>,
        private_arbitration: bool,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        caller.require_auth();
        privacy::set_trade_privacy(&env, &caller, trade_id, data_hash, encrypted_ptr, private_arbitration)
    }

    /// Grant selective disclosure to a third party.
    pub fn grant_disclosure(
        env: Env,
        caller: Address,
        trade_id: u64,
        grantee: Address,
        encrypted_key: soroban_sdk::String,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        caller.require_auth();
        privacy::grant_disclosure(&env, &caller, trade_id, grantee, encrypted_key)
    }

    /// Revoke a disclosure grant.
    pub fn revoke_disclosure(
        env: Env,
        caller: Address,
        trade_id: u64,
        grantee: Address,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        caller.require_auth();
        privacy::revoke_disclosure(&env, &caller, trade_id, grantee)
    }

    /// Get privacy settings for a trade.
    pub fn get_trade_privacy(env: Env, trade_id: u64) -> Option<TradePrivacy> {
        privacy::get_privacy(&env, trade_id)
    }

    /// Get a disclosure grant for a specific grantee.
    pub fn get_disclosure_grant(
        env: Env,
        trade_id: u64,
        grantee: Address,
    ) -> Result<DisclosureGrant, ContractError> {
        privacy::get_grant(&env, trade_id, &grantee)
    }
}
