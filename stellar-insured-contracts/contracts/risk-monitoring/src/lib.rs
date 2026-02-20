#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
    String, Map,
};

// ============================================================================
// Error Handling
// ============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RiskError {
    Unauthorized = 1,
    Paused = 2,
    InvalidInput = 3,
    NotFound = 4,
    AlreadyInitialized = 5,
    NotInitialized = 6,
    ThresholdExceeded = 7,
    SensorInactive = 8,
}

// ============================================================================
// Type Definitions
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RiskFactorType {
    Price,
    Liquidity,
    Volatility,
    Invariant,
    Custom(Symbol),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskSensor {
    pub id: u64,
    pub factor_type: RiskFactorType,
    pub description: String,
    pub threshold_low: i128,
    pub threshold_high: i128,
    pub is_active: bool,
    pub source_contract: Option<Address>,
    pub data_id: u64, // Used for oracle data_id or internal metric ID
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MitigationAction {
    Pause,
    EmergencyWithdraw,
    LimitCoverage,
    FlashAlert,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MitigationTrigger {
    pub sensor_id: u64,
    pub action: MitigationAction,
    pub target_contract: Address,
    pub is_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskAlert {
    pub sensor_id: u64,
    pub factor_type: RiskFactorType,
    pub value: i128,
    pub threshold_violated: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    Paused,
    Sensors,
    Triggers,
    NextSensorId,
    AlertHistory,
}

// ============================================================================
// Risk Monitoring Contract
// ============================================================================

#[contract]
pub struct RiskMonitoringContract;

#[contractimpl]
impl RiskMonitoringContract {
    /// Initialize the risk monitoring contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), RiskError> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(RiskError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage().persistent().set(&DataKey::NextSensorId, &1u64);

        let sensors: Map<u64, RiskSensor> = Map::new(&env);
        env.storage().persistent().set(&DataKey::Sensors, &sensors);

        let triggers: Map<u64, Vec<MitigationTrigger>> = Map::new(&env);
        env.storage().persistent().set(&DataKey::Triggers, &triggers);

        let alerts: Vec<RiskAlert> = Vec::new(&env);
        env.storage().persistent().set(&DataKey::AlertHistory, &alerts);

        Ok(())
    }

    /// Add a new risk monitoring sensor
    pub fn add_sensor(
        env: Env,
        factor_type: RiskFactorType,
        description: String,
        threshold_low: i128,
        threshold_high: i128,
        source_contract: Option<Address>,
        data_id: u64,
    ) -> Result<u64, RiskError> {
        let admin = Self::require_admin(&env)?;
        admin.require_auth();

        let mut next_id: u64 = env.storage().persistent().get(&DataKey::NextSensorId).unwrap();
        let sensor_id = next_id;
        
        let sensor = RiskSensor {
            id: sensor_id,
            factor_type,
            description,
            threshold_low,
            threshold_high,
            is_active: true,
            source_contract,
            data_id,
        };

        let mut sensors: Map<u64, RiskSensor> = env.storage().persistent().get(&DataKey::Sensors).unwrap();
        sensors.set(sensor_id, sensor);
        env.storage().persistent().set(&DataKey::Sensors, &sensors);

        next_id += 1;
        env.storage().persistent().set(&DataKey::NextSensorId, &next_id);

        env.events().publish(
            (symbol_short!("sensor"), symbol_short!("added")),
            sensor_id
        );

        Ok(sensor_id)
    }

    /// Update alert threshold for a sensor
    pub fn update_threshold(
        env: Env,
        sensor_id: u64,
        threshold_low: i128,
        threshold_high: i128,
    ) -> Result<(), RiskError> {
        let admin = Self::require_admin(&env)?;
        admin.require_auth();

        let mut sensors: Map<u64, RiskSensor> = env.storage().persistent().get(&DataKey::Sensors).unwrap();
        let mut sensor = sensors.get(sensor_id).ok_or(RiskError::NotFound)?;
        
        sensor.threshold_low = threshold_low;
        sensor.threshold_high = threshold_high;
        
        sensors.set(sensor_id, sensor);
        env.storage().persistent().set(&DataKey::Sensors, &sensors);

        Ok(())
    }

    /// Configure an automated mitigation trigger
    pub fn add_mitigation_trigger(
        env: Env,
        sensor_id: u64,
        action: MitigationAction,
        target_contract: Address,
    ) -> Result<(), RiskError> {
        let admin = Self::require_admin(&env)?;
        admin.require_auth();

        let sensors: Map<u64, RiskSensor> = env.storage().persistent().get(&DataKey::Sensors).unwrap();
        if !sensors.contains_key(sensor_id) {
            return Err(RiskError::NotFound);
        }

        let mut triggers: Map<u64, Vec<MitigationTrigger>> = env.storage().persistent().get(&DataKey::Triggers).unwrap();
        let mut sensor_triggers = triggers.get(sensor_id).unwrap_or_else(|| Vec::new(&env));
        
        sensor_triggers.push_back(MitigationTrigger {
            sensor_id,
            action,
            target_contract,
            is_enabled: true,
        });

        triggers.set(sensor_id, sensor_triggers);
        env.storage().persistent().set(&DataKey::Triggers, &triggers);

        Ok(())
    }

    /// Perform a real-time risk check for a specific sensor
    /// Integration point for external risk data
    pub fn check_risk(env: Env, sensor_id: u64, current_value: i128) -> Result<bool, RiskError> {
        if Self::is_paused(&env) {
            return Err(RiskError::Paused);
        }

        let sensors: Map<u64, RiskSensor> = env.storage().persistent().get(&DataKey::Sensors).unwrap();
        let sensor = sensors.get(sensor_id).ok_or(RiskError::NotFound)?;

        if !sensor.is_active {
            return Err(RiskError::SensorInactive);
        }

        let mut violated = false;
        let mut violation_threshold = 0i128;

        if current_value < sensor.threshold_low {
            violated = true;
            violation_threshold = sensor.threshold_low;
        } else if current_value > sensor.threshold_high {
            violated = true;
            violation_threshold = sensor.threshold_high;
        }

        if violated {
            // Log Alert
            let alert = RiskAlert {
                sensor_id,
                factor_type: sensor.factor_type.clone(),
                value: current_value,
                threshold_violated: violation_threshold,
                timestamp: env.ledger().timestamp(),
            };

            let mut history: Vec<RiskAlert> = env.storage().persistent().get(&DataKey::AlertHistory).unwrap();
            history.push_back(alert.clone());
            env.storage().persistent().set(&DataKey::AlertHistory, &history);

            // Emit Alert Event (Notification System)
            env.events().publish(
                (symbol_short!("risk"), symbol_short!("alert")),
                alert
            );

            // Trigger Automated Mitigation
            Self::trigger_mitigation(&env, sensor_id)?;
        }

        Ok(violated)
    }

    /// Internal function to execute automated mitigation actions
    fn trigger_mitigation(env: &Env, sensor_id: u64) -> Result<(), RiskError> {
        let triggers_map: Map<u64, Vec<MitigationTrigger>> = env.storage().persistent().get(&DataKey::Triggers).unwrap();
        let triggers = match triggers_map.get(sensor_id) {
            Some(t) => t,
            None => return Ok(()), // No triggers for this sensor
        };

        for i in 0..triggers.len() {
            let trigger = triggers.get(i).unwrap();
            if !trigger.is_enabled {
                continue;
            }

            // Execute action based on type
            // Note: In real implementation, this would use cross-contract calls
            match trigger.action {
                MitigationAction::Pause => {
                    // Example: Call set_paused on target contract
                    // env.invoke_contract::<()>(
                    //     &trigger.target_contract,
                    //     &Symbol::new(env, "set_paused"),
                    //     (true,).into_val(env),
                    // );
                    env.events().publish(
                        (symbol_short!("mitigate"), symbol_short!("pause")),
                        trigger.target_contract.clone()
                    );
                },
                MitigationAction::EmergencyWithdraw => {
                    env.events().publish(
                        (symbol_short!("mitigate"), symbol_short!("withdraw")),
                        trigger.target_contract.clone()
                    );
                },
                MitigationAction::LimitCoverage => {
                    env.events().publish(
                        (symbol_short!("mitigate"), symbol_short!("limit")),
                        trigger.target_contract.clone()
                    );
                },
                MitigationAction::FlashAlert => {
                    env.events().publish(
                        (symbol_short!("mitigate"), symbol_short!("flash")),
                        trigger.target_contract.clone()
                    );
                }
            }
        }

        Ok(())
    }

    /// Get alert history
    pub fn get_alert_history(env: Env) -> Vec<RiskAlert> {
        env.storage().persistent().get(&DataKey::AlertHistory).unwrap_or_else(|| Vec::new(&env))
    }

    /// Get all sensors
    pub fn get_sensors(env: Env) -> Vec<RiskSensor> {
        let sensors_map: Map<u64, RiskSensor> = env.storage().persistent().get(&DataKey::Sensors).unwrap_or_else(|| Map::new(&env));
        let mut sensors_vec = Vec::new(&env);
        for (_id, sensor) in sensors_map.iter() {
            sensors_vec.push_back(sensor);
        }
        sensors_vec
    }

    // ============================================================================
    // Internal Helpers
    // ============================================================================

    fn require_admin(env: &Env) -> Result<Address, RiskError> {
        env.storage().persistent().get(&DataKey::Admin).ok_or(RiskError::NotInitialized)
    }

    fn is_paused(env: &Env) -> bool {
        env.storage().persistent().get(&DataKey::Paused).unwrap_or(false)
    }
}

mod test;

