use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Proposal(u64),
    ProposalCount,
    Vote(u64, Address), // (proposal_id, voter)
}
