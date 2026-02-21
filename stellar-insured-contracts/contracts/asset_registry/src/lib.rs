#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

use shared::types::{Asset, AssetMetadata, AssetConversionRate};

// ============================================================================
// Constants
// ============================================================================

const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const ASSET_COUNT: Symbol = symbol_short!("ASSET_CNT");
const SUPPORTED_ASSETS: Symbol = symbol_short!("ASSETS");

// ============================================================================
// Error Handling
// ============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssetRegistryError {
    Unauthorized = 1,
    Paused = 2,
    InvalidInput = 3,
    AssetNotFound = 4,
    AssetAlreadyExists = 5,
    NotInitialized = 6,
    AlreadyInitialized = 7,
    AssetNotActive = 8,
    InvalidAssetConfig = 9,
    ConversionRateNotFound = 10,
    InvalidConversionRate = 11,
}

// ============================================================================
// Type Definitions
// ============================================================================

/// Configuration for the asset registry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryConfig {
    /// Admin address
    pub admin: Address,
    /// Oracle contract address for price feeds
    pub oracle_contract: Option<Address>,
    /// Whether new assets require oracle price feed
    pub require_oracle: bool,
}

/// Asset registration request
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegistration {
    /// The asset to register
    pub asset: Asset,
    /// Asset symbol
    pub symbol: Symbol,
    /// Asset name
    pub name: Symbol,
    /// Number of decimal places
    pub decimals: u32,
    /// Minimum transaction amount
    pub min_amount: i128,
    /// Maximum transaction amount
    pub max_amount: i128,
    /// Accept for premium payments
    pub accept_for_premium: bool,
    /// Accept for claim payouts
    pub accept_for_claims: bool,
}

/// Summary of asset registry state
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegistrySummary {
    /// Total number of registered assets
    pub total_assets: u32,
    /// Number of active assets
    pub active_assets: u32,
    /// Number of assets accepting premiums
    pub premium_assets: u32,
    /// Number of assets accepting claims
    pub claim_assets: u32,
}

// ============================================================================
// Asset Registry Contract
// ============================================================================

#[contract]
pub struct AssetRegistryContract;

// ============================================================================
// Helper Functions
// ============================================================================

fn require_admin(env: &Env) -> Result<Address, AssetRegistryError> {
    let admin: Address = env
        .storage()
        .persistent()
        .get(&ADMIN)
        .ok_or(AssetRegistryError::NotInitialized)?;
    Ok(admin)
}

fn is_paused(env: &Env) -> bool {
    env.storage().persistent().get(&PAUSED).unwrap_or(false)
}

fn get_asset_key(asset: &Asset) -> Symbol {
    match asset {
        Asset::Native => symbol_short!("XLM"),
        Asset::Stellar((code, _)) => code.clone(),
        Asset::Contract(_) => symbol_short!("CONTR"),
    }
}

fn validate_asset_registration(
    registration: &AssetRegistration,
) -> Result<(), AssetRegistryError> {
    if registration.decimals > 18 {
        return Err(AssetRegistryError::InvalidAssetConfig);
    }
    if registration.min_amount < 0 || registration.max_amount <= 0 {
        return Err(AssetRegistryError::InvalidAssetConfig);
    }
    if registration.min_amount >= registration.max_amount {
        return Err(AssetRegistryError::InvalidAssetConfig);
    }
    Ok(())
}

// ============================================================================
// Contract Implementation
// ============================================================================

#[contractimpl]
impl AssetRegistryContract {
    /// Initialize the asset registry contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), AssetRegistryError> {
        if env.storage().persistent().has(&ADMIN) {
            return Err(AssetRegistryError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&ASSET_COUNT, &0u32);

        // Initialize supported assets list
        let supported: Vec<Asset> = Vec::new(&env);
        env.storage().persistent().set(&SUPPORTED_ASSETS, &supported);

        // Register native XLM by default
        let xlm_metadata = AssetMetadata {
            asset: Asset::Native,
            symbol: symbol_short!("XLM"),
            name: Symbol::new(&env, "Stellar Lumens"),
            decimals: 7,
            is_active: true,
            accept_for_premium: true,
            accept_for_claims: true,
            min_amount: 1_000_000, // 0.1 XLM
            max_amount: 1_000_000_000_000_000, // 100M XLM
            registered_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&(symbol_short!("ASSET"), get_asset_key(&Asset::Native)), &xlm_metadata);

        // Update asset count
        env.storage().persistent().set(&ASSET_COUNT, &1u32);

        // Add to supported list
        let mut supported_list: Vec<Asset> = env
            .storage()
            .persistent()
            .get(&SUPPORTED_ASSETS)
            .unwrap_or_else(|| Vec::new(&env));
        supported_list.push_back(Asset::Native);
        env.storage().persistent().set(&SUPPORTED_ASSETS, &supported_list);

        Ok(())
    }

    /// Pause or unpause the contract
    pub fn set_paused(env: Env, paused: bool) -> Result<(), AssetRegistryError> {
        let admin = require_admin(&env)?;
        admin.require_auth();

        env.storage().persistent().set(&PAUSED, &paused);
        Ok(())
    }

    /// Register a new asset in the registry
    pub fn register_asset(
        env: Env,
        registration: AssetRegistration,
    ) -> Result<(), AssetRegistryError> {
        if is_paused(&env) {
            return Err(AssetRegistryError::Paused);
        }

        let admin = require_admin(&env)?;
        admin.require_auth();

        validate_asset_registration(&registration)?;

        let asset_key = get_asset_key(&registration.asset);

        // Check if asset already exists
        if env.storage().persistent().has(&(symbol_short!("ASSET"), asset_key.clone())) {
            return Err(AssetRegistryError::AssetAlreadyExists);
        }

        let metadata = AssetMetadata {
            asset: registration.asset.clone(),
            symbol: registration.symbol,
            name: registration.name,
            decimals: registration.decimals,
            is_active: true,
            accept_for_premium: registration.accept_for_premium,
            accept_for_claims: registration.accept_for_claims,
            min_amount: registration.min_amount,
            max_amount: registration.max_amount,
            registered_at: env.ledger().timestamp(),
        };

        // Store asset metadata
        env.storage()
            .persistent()
            .set(&(symbol_short!("ASSET"), asset_key), &metadata);

        // Update asset count
        let current_count: u32 = env
            .storage()
            .persistent()
            .get(&ASSET_COUNT)
            .unwrap_or(0u32);
        env.storage().persistent().set(&ASSET_COUNT, &(current_count + 1));

        // Add to supported list
        let mut supported_list: Vec<Asset> = env
            .storage()
            .persistent()
            .get(&SUPPORTED_ASSETS)
            .unwrap_or_else(|| Vec::new(&env));
        supported_list.push_back(registration.asset);
        env.storage().persistent().set(&SUPPORTED_ASSETS, &supported_list);

        Ok(())
    }

    /// Update asset status (active/inactive)
    pub fn set_asset_status(
        env: Env,
        asset: Asset,
        is_active: bool,
    ) -> Result<(), AssetRegistryError> {
        let admin = require_admin(&env)?;
        admin.require_auth();

        let asset_key = get_asset_key(&asset);
        let mut metadata: AssetMetadata = env
            .storage()
            .persistent()
            .get(&(symbol_short!("ASSET"), asset_key.clone()))
            .ok_or(AssetRegistryError::AssetNotFound)?;

        metadata.is_active = is_active;

        env.storage()
            .persistent()
            .set(&(symbol_short!("ASSET"), asset_key), &metadata);

        Ok(())
    }

    /// Update asset configuration
    pub fn update_asset_config(
        env: Env,
        asset: Asset,
        accept_for_premium: Option<bool>,
        accept_for_claims: Option<bool>,
        min_amount: Option<i128>,
        max_amount: Option<i128>,
    ) -> Result<(), AssetRegistryError> {
        let admin = require_admin(&env)?;
        admin.require_auth();

        let asset_key = get_asset_key(&asset);
        let mut metadata: AssetMetadata = env
            .storage()
            .persistent()
            .get(&(symbol_short!("ASSET"), asset_key.clone()))
            .ok_or(AssetRegistryError::AssetNotFound)?;

        if let Some(premium) = accept_for_premium {
            metadata.accept_for_premium = premium;
        }
        if let Some(claims) = accept_for_claims {
            metadata.accept_for_claims = claims;
        }
        if let Some(min) = min_amount {
            metadata.min_amount = min;
        }
        if let Some(max) = max_amount {
            metadata.max_amount = max;
        }

        env.storage()
            .persistent()
            .set(&(symbol_short!("ASSET"), asset_key), &metadata);

        Ok(())
    }

    /// Get asset metadata
    pub fn get_asset_metadata(env: Env, asset: Asset) -> Result<AssetMetadata, AssetRegistryError> {
        let asset_key = get_asset_key(&asset);
        env.storage()
            .persistent()
            .get(&(symbol_short!("ASSET"), asset_key))
            .ok_or(AssetRegistryError::AssetNotFound)
    }

    /// Check if asset is supported and active
    pub fn is_asset_active(env: Env, asset: Asset) -> bool {
        let asset_key = get_asset_key(&asset);
        if let Some(metadata) = env
            .storage()
            .persistent()
            .get::<_, AssetMetadata>(&(symbol_short!("ASSET"), asset_key))
        {
            metadata.is_active
        } else {
            false
        }
    }

    /// Check if asset accepts premiums
    pub fn accepts_premium(env: Env, asset: Asset) -> bool {
        let asset_key = get_asset_key(&asset);
        if let Some(metadata) = env
            .storage()
            .persistent()
            .get::<_, AssetMetadata>(&(symbol_short!("ASSET"), asset_key))
        {
            metadata.is_active && metadata.accept_for_premium
        } else {
            false
        }
    }

    /// Check if asset accepts claims
    pub fn accepts_claims(env: Env, asset: Asset) -> bool {
        let asset_key = get_asset_key(&asset);
        if let Some(metadata) = env
            .storage()
            .persistent()
            .get::<_, AssetMetadata>(&(symbol_short!("ASSET"), asset_key))
        {
            metadata.is_active && metadata.accept_for_claims
        } else {
            false
        }
    }

    /// Get list of all supported assets
    pub fn get_supported_assets(env: Env) -> Vec<Asset> {
        env.storage()
            .persistent()
            .get(&SUPPORTED_ASSETS)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get list of assets that accept premiums
    pub fn get_premium_assets(env: Env) -> Vec<Asset> {
        let all_assets: Vec<Asset> = env
            .storage()
            .persistent()
            .get(&SUPPORTED_ASSETS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut premium_assets: Vec<Asset> = Vec::new(&env);
        for i in 0..all_assets.len() {
            let asset = all_assets.get(i).unwrap();
            if Self::accepts_premium(env.clone(), asset.clone()) {
                premium_assets.push_back(asset);
            }
        }
        premium_assets
    }

    /// Get list of assets that accept claims
    pub fn get_claim_assets(env: Env) -> Vec<Asset> {
        let all_assets: Vec<Asset> = env
            .storage()
            .persistent()
            .get(&SUPPORTED_ASSETS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut claim_assets: Vec<Asset> = Vec::new(&env);
        for i in 0..all_assets.len() {
            let asset = all_assets.get(i).unwrap();
            if Self::accepts_claims(env.clone(), asset.clone()) {
                claim_assets.push_back(asset);
            }
        }
        claim_assets
    }

    /// Get asset registry summary
    pub fn get_registry_summary(env: Env) -> AssetRegistrySummary {
        let all_assets: Vec<Asset> = env
            .storage()
            .persistent()
            .get(&SUPPORTED_ASSETS)
            .unwrap_or_else(|| Vec::new(&env));

        let mut active_count = 0u32;
        let mut premium_count = 0u32;
        let mut claim_count = 0u32;

        for i in 0..all_assets.len() {
            let asset = all_assets.get(i).unwrap();
            if Self::is_asset_active(env.clone(), asset.clone()) {
                active_count += 1;
            }
            if Self::accepts_premium(env.clone(), asset.clone()) {
                premium_count += 1;
            }
            if Self::accepts_claims(env.clone(), asset.clone()) {
                claim_count += 1;
            }
        }

        AssetRegistrySummary {
            total_assets: all_assets.len(),
            active_assets: active_count,
            premium_assets: premium_count,
            claim_assets: claim_count,
        }
    }

    /// Set conversion rate between assets (admin only)
    pub fn set_conversion_rate(
        env: Env,
        from_asset: Asset,
        to_asset: Asset,
        rate_bps: u32,
    ) -> Result<(), AssetRegistryError> {
        let admin = require_admin(&env)?;
        admin.require_auth();

        if rate_bps == 0 {
            return Err(AssetRegistryError::InvalidConversionRate);
        }

        // Verify both assets exist
        let from_key = get_asset_key(&from_asset);
        let to_key = get_asset_key(&to_asset);

        if !env.storage().persistent().has(&(symbol_short!("ASSET"), from_key)) {
            return Err(AssetRegistryError::AssetNotFound);
        }
        if !env.storage().persistent().has(&(symbol_short!("ASSET"), to_key)) {
            return Err(AssetRegistryError::AssetNotFound);
        }

        let conversion_rate = AssetConversionRate {
            from_asset: from_asset.clone(),
            to_asset: to_asset.clone(),
            rate_bps,
            updated_at: env.ledger().timestamp(),
            oracle_source: admin.clone(),
        };

        env.storage().persistent().set(
            &(symbol_short!("RATE"), get_asset_key(&from_asset), get_asset_key(&to_asset)),
            &conversion_rate,
        );

        Ok(())
    }

    /// Get conversion rate between two assets
    pub fn get_conversion_rate(
        env: Env,
        from_asset: Asset,
        to_asset: Asset,
    ) -> Result<AssetConversionRate, AssetRegistryError> {
        env.storage()
            .persistent()
            .get(&(
                symbol_short!("RATE"),
                get_asset_key(&from_asset),
                get_asset_key(&to_asset),
            ))
            .ok_or(AssetRegistryError::ConversionRateNotFound)
    }

    /// Convert amount from one asset to another
    pub fn convert_amount(
        env: Env,
        from_asset: Asset,
        to_asset: Asset,
        amount: i128,
    ) -> Result<i128, AssetRegistryError> {
        if amount <= 0 {
            return Err(AssetRegistryError::InvalidInput);
        }

        // Same asset, no conversion needed
        if get_asset_key(&from_asset) == get_asset_key(&to_asset) {
            return Ok(amount);
        }

        let rate = Self::get_conversion_rate(env, from_asset, to_asset)?;

        // Convert: (amount * rate_bps) / 10000
        let converted = amount
            .checked_mul(rate.rate_bps as i128)
            .and_then(|v| v.checked_div(10000))
            .ok_or(AssetRegistryError::InvalidConversionRate)?;

        Ok(converted)
    }

    /// Validate amount is within asset bounds
    pub fn validate_amount(
        env: Env,
        asset: Asset,
        amount: i128,
    ) -> Result<(), AssetRegistryError> {
        let metadata = Self::get_asset_metadata(env, asset)?;

        if amount < metadata.min_amount {
            return Err(AssetRegistryError::InvalidInput);
        }
        if amount > metadata.max_amount {
            return Err(AssetRegistryError::InvalidInput);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup_env() -> (Env, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        (env, admin)
    }

    #[test]
    fn test_initialize() {
        let (env, admin) = setup_env();

        let result = AssetRegistryContract::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());

        // Check XLM is registered by default
        let xlm_metadata = AssetRegistryContract::get_asset_metadata(env.clone(), Asset::Native).unwrap();
        assert_eq!(xlm_metadata.symbol, symbol_short!("XLM"));
        assert!(xlm_metadata.is_active);
        assert!(xlm_metadata.accept_for_premium);
        assert!(xlm_metadata.accept_for_claims);
    }

    #[test]
    fn test_register_asset() {
        let (env, admin) = setup_env();
        AssetRegistryContract::initialize(env.clone(), admin.clone()).unwrap();

        let usdc_asset = Asset::Stellar((symbol_short!("USDC"), Address::generate(&env)));
        let registration = AssetRegistration {
            asset: usdc_asset.clone(),
            symbol: symbol_short!("USDC"),
            name: Symbol::new(&env, "USD Coin"),
            decimals: 7,
            min_amount: 1_000_000,
            max_amount: 1_000_000_000_000_000,
            accept_for_premium: true,
            accept_for_claims: true,
        };

        let result = AssetRegistryContract::register_asset(env.clone(), registration);
        assert!(result.is_ok());

        let metadata = AssetRegistryContract::get_asset_metadata(env.clone(), usdc_asset).unwrap();
        assert_eq!(metadata.symbol, symbol_short!("USDC"));
        assert!(metadata.is_active);
    }

    #[test]
    fn test_duplicate_asset_registration_fails() {
        let (env, admin) = setup_env();
        AssetRegistryContract::initialize(env.clone(), admin.clone()).unwrap();

        let usdc_asset = Asset::Stellar((symbol_short!("USDC"), Address::generate(&env)));
        let registration = AssetRegistration {
            asset: usdc_asset.clone(),
            symbol: symbol_short!("USDC"),
            name: Symbol::new(&env, "USD Coin"),
            decimals: 7,
            min_amount: 1_000_000,
            max_amount: 1_000_000_000_000_000,
            accept_for_premium: true,
            accept_for_claims: true,
        };

        AssetRegistryContract::register_asset(env.clone(), registration.clone()).unwrap();
        let result = AssetRegistryContract::register_asset(env.clone(), registration);
        assert_eq!(result, Err(AssetRegistryError::AssetAlreadyExists));
    }

    #[test]
    fn test_asset_status_management() {
        let (env, admin) = setup_env();
        AssetRegistryContract::initialize(env.clone(), admin.clone()).unwrap();

        let usdc_asset = Asset::Stellar((symbol_short!("USDC"), Address::generate(&env)));
        let registration = AssetRegistration {
            asset: usdc_asset.clone(),
            symbol: symbol_short!("USDC"),
            name: Symbol::new(&env, "USD Coin"),
            decimals: 7,
            min_amount: 1_000_000,
            max_amount: 1_000_000_000_000_000,
            accept_for_premium: true,
            accept_for_claims: true,
        };

        AssetRegistryContract::register_asset(env.clone(), registration).unwrap();
        assert!(AssetRegistryContract::is_asset_active(env.clone(), usdc_asset.clone()));

        // Deactivate asset
        AssetRegistryContract::set_asset_status(env.clone(), usdc_asset.clone(), false).unwrap();
        assert!(!AssetRegistryContract::is_asset_active(env.clone(), usdc_asset.clone()));
    }

    #[test]
    fn test_conversion_rate() {
        let (env, admin) = setup_env();
        AssetRegistryContract::initialize(env.clone(), admin.clone()).unwrap();

        // Register USDC
        let usdc_asset = Asset::Stellar((symbol_short!("USDC"), Address::generate(&env)));
        let registration = AssetRegistration {
            asset: usdc_asset.clone(),
            symbol: symbol_short!("USDC"),
            name: Symbol::new(&env, "USD Coin"),
            decimals: 7,
            min_amount: 1_000_000,
            max_amount: 1_000_000_000_000_000,
            accept_for_premium: true,
            accept_for_claims: true,
        };
        AssetRegistryContract::register_asset(env.clone(), registration).unwrap();

        // Set conversion rate: 1 XLM = 0.1 USDC (rate_bps = 1000)
        AssetRegistryContract::set_conversion_rate(
            env.clone(),
            Asset::Native,
            usdc_asset.clone(),
            1000,
        ).unwrap();

        // Convert 10 XLM to USDC
        let converted = AssetRegistryContract::convert_amount(
            env.clone(),
            Asset::Native,
            usdc_asset,
            10_000_000_000, // 1000 XLM (with 7 decimals)
        ).unwrap();

        // Expected: 1000 * 0.1 = 100 USDC
        assert_eq!(converted, 1_000_000_000);
    }

    #[test]
    fn test_get_premium_and_claim_assets() {
        let (env, admin) = setup_env();
        AssetRegistryContract::initialize(env.clone(), admin.clone()).unwrap();

        // Initially only XLM
        let premium_assets = AssetRegistryContract::get_premium_assets(env.clone());
        assert_eq!(premium_assets.len(), 1);

        let claim_assets = AssetRegistryContract::get_claim_assets(env.clone());
        assert_eq!(claim_assets.len(), 1);
    }

    #[test]
    fn test_registry_summary() {
        let (env, admin) = setup_env();
        AssetRegistryContract::initialize(env.clone(), admin.clone()).unwrap();

        let summary = AssetRegistryContract::get_registry_summary(env.clone());
        assert_eq!(summary.total_assets, 1); // XLM only
        assert_eq!(summary.active_assets, 1);
        assert_eq!(summary.premium_assets, 1);
        assert_eq!(summary.claim_assets, 1);
    }
}
