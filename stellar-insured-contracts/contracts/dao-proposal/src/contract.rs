use soroban_sdk::{Env, Address, String, Vec};

use crate::storage::DataKey;
use crate::types::{Proposal, VoteChoice};
use crate::utils::current_time;

pub struct DaoContract;

impl DaoContract {
    // -------------------------------
    // Proposal Creation
    // -------------------------------
    pub fn create_proposal(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        voting_duration: u64,
    ) -> u64 {
        creator.require_auth();

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0);

        let now = current_time(&env);

        let proposal = Proposal {
            id,
            creator,
            title,
            description,
            start_time: now,
            end_time: now + voting_duration,
            yes_votes: 0,
            no_votes: 0,
            executed: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::Proposal(id), &proposal);

        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &(id + 1));

        id
    }

    // -------------------------------
    // Voting
    // -------------------------------
    pub fn vote(
        env: Env,
        proposal_id: u64,
        voter: Address,
        choice: VoteChoice,
    ) {
        voter.require_auth();

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("Proposal not found");

        let now = current_time(&env);

        if now < proposal.start_time || now > proposal.end_time {
            panic!("Voting period closed");
        }

        let vote_key = DataKey::Vote(proposal_id, voter.clone());

        if env.storage().instance().has(&vote_key) {
            panic!("Already voted");
        }

        match choice {
            VoteChoice::Yes => proposal.yes_votes += 1,
            VoteChoice::No => proposal.no_votes += 1,
        }

        env.storage().instance().set(&vote_key, &choice);
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    // -------------------------------
    // Read-only Queries
    // -------------------------------
    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .expect("Proposal not found")
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }
}
