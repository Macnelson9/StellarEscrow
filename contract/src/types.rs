use soroban_sdk::{contracttype, Address, String, Vec};

/// Maximum byte length for a single metadata value string
pub const METADATA_MAX_VALUE_LEN: u32 = 256;
/// Maximum number of key-value pairs in metadata
pub const METADATA_MAX_ENTRIES: u32 = 10;

// ---------------------------------------------------------------------------
// Fee Tier System
// ---------------------------------------------------------------------------

pub const TIER_SILVER_THRESHOLD: u64 = 10_000_000_000;
pub const TIER_GOLD_THRESHOLD: u64 = 100_000_000_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserTier {
    Bronze,
    Silver,
    Gold,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserTierInfo {
    pub tier: UserTier,
    pub total_volume: u64,
    pub custom_fee_bps: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    pub bronze_fee_bps: u32,
    pub silver_fee_bps: u32,
    pub gold_fee_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
}

/// A single metadata key-value entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

/// Structured metadata attached to a trade (e.g. product description, shipping info)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeMetadata {
    pub entries: Vec<MetadataEntry>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trade {
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub arbitrator: Option<Address>,
    pub status: TradeStatus,
    /// Optional structured metadata (product info, shipping details, etc.)
    pub metadata: Option<TradeMetadata>,
}

// ---------------------------------------------------------------------------
// Subscription Model
// ---------------------------------------------------------------------------

/// Duration of a subscription in ledgers (~1 ledger ≈ 5 s; 30 days ≈ 518_400 ledgers)
pub const SUBSCRIPTION_DURATION_LEDGERS: u32 = 518_400;

/// Monthly price in stroops (USDC micro-units) per tier
pub const SUB_PRICE_BASIC: u64 = 5_000_000;   // 5 USDC
pub const SUB_PRICE_PRO: u64 = 15_000_000;    // 15 USDC
pub const SUB_PRICE_ENTERPRISE: u64 = 50_000_000; // 50 USDC

/// Fee discounts in bps applied on top of the tier/base fee
pub const SUB_DISCOUNT_BASIC_BPS: u32 = 20;       // −0.20 %
pub const SUB_DISCOUNT_PRO_BPS: u32 = 50;          // −0.50 %
pub const SUB_DISCOUNT_ENTERPRISE_BPS: u32 = 100;  // −1.00 %

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionTier {
    Basic,
    Pro,
    Enterprise,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub subscriber: Address,
    pub tier: SubscriptionTier,
    /// Ledger sequence at which the subscription expires
    pub expires_at: u32,
    /// Ledger sequence of the last renewal / purchase
    pub renewed_at: u32,
}

// ---------------------------------------------------------------------------
// Governance
// ---------------------------------------------------------------------------

/// Total supply of governance tokens minted at initialization
pub const GOV_TOTAL_SUPPLY: i128 = 1_000_000_000_000_000; // 1 billion (7 decimals)

/// Voting period in ledgers (~7 days)
pub const GOV_VOTING_PERIOD: u32 = 1_209_600;

/// Minimum tokens required to create a proposal
pub const GOV_PROPOSAL_THRESHOLD: i128 = 10_000_000_000; // 10,000 tokens

/// Minimum quorum (% of total supply * 100, i.e. 400 = 4%)
pub const GOV_QUORUM_BPS: u32 = 400;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalAction {
    UpdateFeeBps(u32),
    UpdateTierConfig(TierConfig),
    DistributeFees(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub action: ProposalAction,
    pub votes_for: i128,
    pub votes_against: i128,
    pub status: ProposalStatus,
    pub created_at: u32,
    pub ends_at: u32,
}

// ---------------------------------------------------------------------------
// Privacy Features
// ---------------------------------------------------------------------------

/// Max length for an encrypted data pointer (e.g. IPFS CID or URL)
pub const PRIVACY_DATA_PTR_MAX_LEN: u32 = 256;

/// Commitment to sensitive trade data stored off-chain (e.g. SHA-256 hex)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradePrivacy {
    /// Hash of the plaintext trade details (commitment scheme)
    pub data_hash: String,
    /// Encrypted data pointer (e.g. IPFS CID) — only parties can decrypt
    pub encrypted_ptr: Option<String>,
    /// Whether arbitration is private (arbitrator identity hidden from public)
    pub private_arbitration: bool,
}

/// A selective disclosure grant — allows `grantee` to access private trade data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisclosureGrant {
    pub trade_id: u64,
    pub grantee: Address,
    /// Encrypted decryption key for the grantee (encrypted with grantee's public key off-chain)
    pub encrypted_key: String,
}

// ---------------------------------------------------------------------------
// Trade Templates
// ---------------------------------------------------------------------------

pub const TEMPLATE_NAME_MAX_LEN: u32 = 64;
pub const TEMPLATE_MAX_VERSIONS: u32 = 10;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateTerms {
    pub description: String,
    pub default_arbitrator: Option<Address>,
    pub fixed_amount: Option<u64>,
    pub default_metadata: Option<TradeMetadata>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateVersion {
    pub version: u32,
    pub terms: TemplateTerms,
    pub created_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeTemplate {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub current_version: u32,
    pub versions: Vec<TemplateVersion>,
    pub active: bool,
    pub created_at: u32,
    pub updated_at: u32,
}
