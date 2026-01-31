use soroban_sdk::Env;

pub fn current_time(env: &Env) -> u64 {
    env.ledger().timestamp()
}
