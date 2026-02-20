#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger, Events};
use soroban_sdk::{vec, IntoVal};

#[test]
fn test_risk_monitoring_lifecycle() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register_contract(None, RiskMonitoringContract);
    let client = RiskMonitoringContractClient::new(&env, &contract_id);

    // 1. Initialize
    client.initialize(&admin);

    // 2. Add Sensor (Price Sensor)
    let description = String::from_str(&env, "BTC/USD Price Sensor");
    let sensor_id = client.add_sensor(
        &RiskFactorType::Price,
        &description,
        &90_000,  // threshold_low
        &110_000, // threshold_high
        &None,    // source_contract
        &1        // data_id
    );

    assert_eq!(sensor_id, 1);

    // 3. Add Mitigation Trigger
    let target_contract = Address::generate(&env);
    client.add_mitigation_trigger(&sensor_id, &MitigationAction::Pause, &target_contract);

    // 4. Check Risk - Normal Value
    let result_normal = client.check_risk(&sensor_id, &100_000);
    assert_eq!(result_normal, false);

    // 5. Check Risk - High Violation (Alert Trigger)
    let result_violation = client.check_risk(&sensor_id, &120_000);
    assert_eq!(result_violation, true);

    // 6. Verify Alert History
    let history = client.get_alert_history();
    assert_eq!(history.len(), 1);
    let alert = history.get(0).unwrap();
    assert_eq!(alert.sensor_id, 1);
    assert_eq!(alert.value, 120_000);

    // 7. Verify Events (Notification System)
    let last_event = env.events().all().last().unwrap();
    // (risk, alert) event
    assert_eq!(last_event.0, contract_id);
}

#[test]
fn test_threshold_update() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register_contract(None, RiskMonitoringContract);
    let client = RiskMonitoringContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let description = String::from_str(&env, "Liquidity Sensor");
    let sensor_id = client.add_sensor(
        &RiskFactorType::Liquidity,
        &description,
        &1000,
        &5000,
        &None,
        &2
    );

    // Initial check - 800 is violation
    assert_eq!(client.check_risk(&sensor_id, &800), true);

    // Update threshold
    client.update_threshold(&sensor_id, &500, &5000);

    // Check again - 800 is now normal
    assert_eq!(client.check_risk(&sensor_id, &800), false);
}
