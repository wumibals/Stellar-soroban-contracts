#![no_std]

use soroban_sdk::{contract, contractimpl};

mod contract;
mod storage;
mod types;
mod utils;

use contract::DaoContract;

#[contract]
pub struct Dao;

#[contractimpl]
impl Dao {
    pub fn create_proposal(
        env: soroban_sdk::Env,
        creator: soroban_sdk::Address,
        title: soroban_sdk::String,
        description: soroban_sdk::String,
        voting_duration: u64,
    ) -> u64 {
        DaoContract::create_proposal(env, creator, title, description, voting_duration)
    }

    pub fn vote(
        env: soroban_sdk::Env,
        proposal_id: u64,
        voter: soroban_sdk::Address,
        choice: types::VoteChoice,
    ) {
        DaoContract::vote(env, proposal_id, voter, choice)
    }

    pub fn get_proposal(
        env: soroban_sdk::Env,
        proposal_id: u64,
    ) -> types::Proposal {
        DaoContract::get_proposal(env, proposal_id)
    }

    pub fn proposal_count(env: soroban_sdk::Env) -> u64 {
        DaoContract::proposal_count(env)
    }
}
