use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum VoteChoice {
    Yes,
    No,
}

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u64,
    pub creator: Address,
    pub title: soroban_sdk::String,
    pub description: soroban_sdk::String,
    pub start_time: u64,
    pub end_time: u64,
    pub yes_votes: u32,
    pub no_votes: u32,
    pub executed: bool,
}
