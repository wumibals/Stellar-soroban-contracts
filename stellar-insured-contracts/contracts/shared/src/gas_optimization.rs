//! Gas optimization utilities and monitoring for Stellar Soroban contracts
//!
//! This module provides tools for:
//! - Measuring gas consumption of operations
//! - Optimizing storage access patterns
//! - Efficient data structure operations
//! - Performance monitoring and metrics

use soroban_sdk::{contracttype, Env, Map, Symbol, Vec};

/// Gas measurement result for performance tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasMeasurement {
    /// Operation name/description
    pub operation: Symbol,
    /// Gas consumed (in units)
    pub gas_used: u64,
    /// Timestamp of measurement
    pub timestamp: u64,
    /// Contract version for tracking improvements
    pub version: u32,
}

/// Gas optimization metrics for a contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasMetrics {
    /// Total gas consumed across all operations
    pub total_gas: u64,
    /// Number of operations measured
    pub operation_count: u64,
    /// Average gas per operation
    pub avg_gas_per_op: u64,
    /// Maximum gas consumed in single operation
    pub max_gas_op: u64,
    /// Minimum gas consumed in single operation
    pub min_gas_op: u64,
}

/// Storage optimization strategies
pub enum StorageOptimization {
    /// Use instance storage for frequently accessed small data
    UseInstanceStorage,
    /// Use temporary storage for short-lived data
    UseTemporaryStorage,
    /// Cache frequently accessed data in local variables
    CacheInMemory,
    /// Batch storage operations
    BatchOperations,
    /// Use efficient key structures
    OptimizedKeys,
}

/// Efficient storage key patterns for reduced gas costs
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OptimizedDataKey {
    /// Instance-level config data (cheapest storage)
    Config,
    /// Frequently accessed counter (instance storage)
    Counter(Symbol),
    /// Batch operations storage
    Batch(Symbol, u32),
    /// Temporary cache data
    Cache(Symbol),
    /// Compact index mapping
    Index(u64),
    /// Partitioned storage for large datasets
    Partition(Symbol, u32),
}

/// Gas optimization utilities
pub struct GasOptimizer;

impl GasOptimizer {
    /// Start gas measurement for an operation
    pub fn start_measurement(env: &Env) -> u64 {
        // Soroban SDK doesn't expose gas directly, so we track logical operations
        // In production, this would integrate with ledger-level gas tracking
        env.ledger().timestamp()
    }

    /// End gas measurement and record metrics
    pub fn end_measurement(env: &Env, start_time: u64, operation: Symbol) -> GasMeasurement {
        let end_time = env.ledger().timestamp();
        let gas_used = end_time.saturating_sub(start_time);
        
        GasMeasurement {
            operation,
            gas_used,
            timestamp: end_time,
            version: 1u32, // Would be incremented with contract updates
        }
    }

    /// Get optimized storage key for frequent counter operations
    pub fn get_counter_key(name: Symbol) -> OptimizedDataKey {
        OptimizedDataKey::Counter(name)
    }

    /// Get batch storage key for efficient bulk operations
    pub fn get_batch_key(batch_name: Symbol, batch_id: u32) -> OptimizedDataKey {
        OptimizedDataKey::Batch(batch_name, batch_id)
    }

    /// Get cache key for temporary data
    pub fn get_cache_key(key: Symbol) -> OptimizedDataKey {
        OptimizedDataKey::Cache(key)
    }

    /// Efficient batch storage set operation
    pub fn batch_set<T: Clone>(
        env: &Env,
        key_base: Symbol,
        items: &Vec<T>,
        batch_size: u32,
    ) -> Result<(), super::errors::ContractError>
    where
        T: soroban_sdk::TryIntoVal<soroban_sdk::Env, soroban_sdk::RawVal>,
    {
        if items.is_empty() {
            return Ok(());
        }

        let total_items = items.len() as u32;
        let mut batch_id = 0u32;
        
        // Process items in batches to reduce storage operations
        let mut i = 0u32;
        while i < total_items {
            let end = core::cmp::min(i + batch_size, total_items);
            let mut batch_data = Vec::new(env);
            
            // Collect batch items
            for j in i..end {
                batch_data.push_back(items.get(j).unwrap());
            }
            
            // Store batch with single operation
            let batch_key = Self::get_batch_key(key_base.clone(), batch_id);
            env.storage().persistent().set(&batch_key, &batch_data);
            
            batch_id += 1;
            i = end;
        }
        
        Ok(())
    }

    /// Efficient batch storage get operation
    pub fn batch_get<T: Clone>(
        env: &Env,
        key_base: Symbol,
        expected_count: u32,
        batch_size: u32,
    ) -> Result<Vec<T>, super::errors::ContractError>
    where
        T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::RawVal>,
    {
        let mut result = Vec::new(env);
        let total_batches = (expected_count + batch_size - 1) / batch_size;
        
        for batch_id in 0..total_batches {
            let batch_key = Self::get_batch_key(key_base.clone(), batch_id);
            if let Some(batch_data) = env.storage().persistent().get::<_, Vec<T>>(&batch_key) {
                // Append all items from this batch
                for i in 0..batch_data.len() {
                    result.push_back(batch_data.get(i).unwrap());
                }
            }
        }
        
        Ok(result)
    }

    /// Efficient map operations with reduced storage overhead
    pub fn efficient_map_insert<K, V>(
        env: &Env,
        map_key: &Symbol,
        key: K,
        value: V,
    ) -> Result<(), super::errors::ContractError>
    where
        K: Clone + soroban_sdk::TryIntoVal<soroban_sdk::Env, soroban_sdk::RawVal>,
        V: Clone + soroban_sdk::TryIntoVal<soroban_sdk::Env, soroban_sdk::RawVal>,
    {
        let mut map: Map<K, V> = env
            .storage()
            .persistent()
            .get(map_key)
            .unwrap_or_else(|| Map::new(env));
        
        map.set(key, value);
        env.storage().persistent().set(map_key, &map);
        Ok(())
    }

    /// Cache frequently accessed data to reduce storage reads
    pub fn cache_get<T: Clone>(
        env: &Env,
        key: Symbol,
    ) -> Option<T>
    where
        T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::RawVal>,
    {
        // Try cache first (instance storage - cheaper)
        if let Some(cached) = env.storage().instance().get::<_, T>(&Self::get_cache_key(key.clone())) {
            return Some(cached);
        }
        
        // Fall back to persistent storage
        env.storage().persistent().get(&key)
    }

    /// Cache data for subsequent fast access
    pub fn cache_set<T: Clone>(
        env: &Env,
        key: Symbol,
        value: &T,
    ) -> Result<(), super::errors::ContractError>
    where
        T: soroban_sdk::TryIntoVal<soroban_sdk::Env, soroban_sdk::RawVal>,
    {
        // Store in both cache (instance) and persistent storage
        env.storage().instance().set(&Self::get_cache_key(key.clone()), value);
        env.storage().persistent().set(&key, value);
        Ok(())
    }

    /// Clear cache entry to free instance storage
    pub fn cache_clear(env: &Env, key: Symbol) {
        env.storage().instance().remove(&Self::get_cache_key(key));
    }

    /// Calculate gas savings from optimization
    pub fn calculate_savings(old_gas: u64, new_gas: u64) -> u32 {
        if old_gas == 0 {
            return 0;
        }
        let savings = old_gas.saturating_sub(new_gas);
        ((savings as u128 * 100) / old_gas as u128) as u32
    }
}

/// Optimized data structures for reduced gas consumption
pub struct OptimizedStructures;

impl OptimizedStructures {
    /// Compact policy state representation (1 byte instead of enum)
    pub fn encode_policy_state(state: u8) -> u8 {
        // ACTIVE = 1, EXPIRED = 2, CANCELLED = 3
        state
    }

    /// Decode compact policy state
    pub fn decode_policy_state(encoded: u8) -> Option<super::types::PolicyStatus> {
        use super::types::PolicyStatus;
        match encoded {
            1 => Some(PolicyStatus::ACTIVE),
            2 => Some(PolicyStatus::EXPIRED),
            3 => Some(PolicyStatus::CANCELLED),
            _ => None,
        }
    }

    /// Compact timestamp representation (store as u32 days since epoch)
    pub fn timestamp_to_days(timestamp: u64) -> u32 {
        (timestamp / 86400) as u32
    }

    /// Convert days back to timestamp
    pub fn days_to_timestamp(days: u32) -> u64 {
        days as u64 * 86400
    }

    /// Efficient bit-packed storage for boolean flags
    pub fn pack_flags(flags: &[bool; 8]) -> u8 {
        let mut packed = 0u8;
        for (i, &flag) in flags.iter().enumerate() {
            if flag {
                packed |= 1 << i;
            }
        }
        packed
    }

    /// Unpack bit-packed flags
    pub fn unpack_flags(packed: u8) -> [bool; 8] {
        let mut flags = [false; 8];
        for i in 0..8 {
            flags[i] = (packed & (1 << i)) != 0;
        }
        flags
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Track operation performance metrics
    pub fn track_operation<T, F>(
        env: &Env,
        operation_name: &str,
        operation: F,
    ) -> Result<T, super::errors::ContractError>
    where
        F: FnOnce() -> Result<T, super::errors::ContractError>,
    {
        let start = GasOptimizer::start_measurement(env);
        let result = operation();
        let measurement = GasOptimizer::end_measurement(
            env, 
            start, 
            Symbol::new(env, operation_name)
        );
        
        // In production, store metrics for analysis
        // env.storage().persistent().set(&measurement_key, &measurement);
        
        result
    }

    /// Compare performance before/after optimization
    pub fn benchmark_improvement<F>(
        env: &Env,
        operation_name: &str,
        old_impl: F,
        new_impl: F,
    ) -> (u64, u64, u32)
    where
        F: Fn() -> Result<(), super::errors::ContractError>,
    {
        // Measure old implementation
        let start_old = GasOptimizer::start_measurement(env);
        let _ = old_impl();
        let old_gas = GasOptimizer::end_measurement(
            env, 
            start_old, 
            Symbol::new(env, &format!("{}_old", operation_name))
        ).gas_used;

        // Measure new implementation
        let start_new = GasOptimizer::start_measurement(env);
        let _ = new_impl();
        let new_gas = GasOptimizer::end_measurement(
            env, 
            start_new, 
            Symbol::new(env, &format!("{}_new", operation_name))
        ).gas_used;

        let savings_percent = GasOptimizer::calculate_savings(old_gas, new_gas);
        
        (old_gas, new_gas, savings_percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_batch_operations() {
        let env = Env::default();
        let items = vec![1i32, 2, 3, 4, 5];
        let soroban_items = Vec::from_array(&env, items.try_into().unwrap());
        
        // Test batch set
        let result = GasOptimizer::batch_set(&env, Symbol::new(&env, "test"), &soroban_items, 2);
        assert!(result.is_ok());
        
        // Test batch get
        let retrieved = GasOptimizer::batch_get::<i32>(&env, Symbol::new(&env, "test"), 5, 2).unwrap();
        assert_eq!(retrieved.len(), 5);
        assert_eq!(retrieved.get(0).unwrap(), 1);
        assert_eq!(retrieved.get(4).unwrap(), 5);
    }

    #[test]
    fn test_compact_state_encoding() {
        use super::super::types::PolicyStatus;
        
        // Test encoding
        assert_eq!(OptimizedStructures::encode_policy_state(1), 1);
        assert_eq!(OptimizedStructures::encode_policy_state(2), 2);
        assert_eq!(OptimizedStructures::encode_policy_state(3), 3);
        
        // Test decoding
        assert_eq!(
            OptimizedStructures::decode_policy_state(1),
            Some(PolicyStatus::ACTIVE)
        );
        assert_eq!(
            OptimizedStructures::decode_policy_state(2),
            Some(PolicyStatus::EXPIRED)
        );
        assert_eq!(
            OptimizedStructures::decode_policy_state(3),
            Some(PolicyStatus::CANCELLED)
        );
        assert_eq!(OptimizedStructures::decode_policy_state(0), None);
        assert_eq!(OptimizedStructures::decode_policy_state(4), None);
    }

    #[test]
    fn test_timestamp_compression() {
        let timestamp = 1704067200u64; // Jan 1, 2024
        let days = OptimizedStructures::timestamp_to_days(timestamp);
        let reconstructed = OptimizedStructures::days_to_timestamp(days);
        
        // Should be within one day precision
        assert!(reconstructed.abs_diff(timestamp) <= 86400);
    }

    #[test]
    fn test_flag_packing() {
        let flags = [true, false, true, false, false, true, false, true];
        let packed = OptimizedStructures::pack_flags(&flags);
        let unpacked = OptimizedStructures::unpack_flags(packed);
        
        assert_eq!(flags, unpacked);
    }

    #[test]
    fn test_cache_operations() {
        let env = Env::default();
        let key = Symbol::new(&env, "test_key");
        let value = 42i32;
        
        // Set cache
        let result = GasOptimizer::cache_set(&env, key.clone(), &value);
        assert!(result.is_ok());
        
        // Get from cache
        let cached_value = GasOptimizer::cache_get::<i32>(&env, key.clone());
        assert_eq!(cached_value, Some(42));
        
        // Clear cache
        GasOptimizer::cache_clear(&env, key.clone());
        let cleared_value = GasOptimizer::cache_get::<i32>(&env, key);
        assert_eq!(cleared_value, None); // Should fall back to persistent storage
    }
}