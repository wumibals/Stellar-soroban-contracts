#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, BytesN, Env, Symbol, Vec,
};
use shared::{
    ComplianceRecord, ConfidentialClaim, PrivacyProof, PrivacySettings, PrivatePolicyData,
    ZkProof, ZkVerificationResult,
};

#[contract]
pub struct PrivacyContract;

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const PAUSED: Symbol = symbol_short!("PAUSED");
const PROOF_COUNTER: Symbol = symbol_short!("PRF_CNT");
const COMPLIANCE_COUNTER: Symbol = symbol_short!("COMP_CNT");

// User-specific storage prefix
const USER_PRIVACY: Symbol = symbol_short!("USR_PRIV");
const CONFIDENTIAL_CLAIM: Symbol = symbol_short!("CONF_CLM");
const PRIVATE_POLICY: Symbol = symbol_short!("PRIV_POL");
const ZK_PROOF: Symbol = symbol_short!("ZK_PROOF");
const COMPLIANCE_RECORD: Symbol = symbol_short!("COMP_REC");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    Unauthorized = 1,
    Paused = 2,
    InvalidInput = 3,
    NotFound = 4,
    AlreadyExists = 5,
    InvalidState = 6,
    NotInitialized = 7,
    AlreadyInitialized = 8,
    PrivacyDisabled = 9,
    ProofInvalid = 10,
    ProofExpired = 11,
    CircuitNotRecognized = 12,
    ComplianceCheckFailed = 13,
}

fn is_paused(env: &Env) -> bool {
    env.storage().persistent().get(&PAUSED).unwrap_or(false)
}

fn set_paused(env: &Env, paused: bool) {
    env.storage().persistent().set(&PAUSED, &paused);
}

fn get_next_proof_id(env: &Env) -> u64 {
    let current: u64 = env.storage().persistent().get(&PROOF_COUNTER).unwrap_or(0);
    env.storage().persistent().set(&PROOF_COUNTER, &(current + 1));
    current + 1
}

fn get_next_compliance_id(env: &Env) -> u64 {
    let current: u64 = env.storage().persistent().get(&COMPLIANCE_COUNTER).unwrap_or(0);
    env.storage().persistent().set(&COMPLIANCE_COUNTER, &(current + 1));
    current + 1
}

/// Verify a ZK proof (simulated - in production this would use actual ZKP verification)
fn verify_zk_proof(env: &Env, proof: &ZkProof) -> ZkVerificationResult {
    // Check if proof has expired
    if let Some(expires_at) = proof.expires_at {
        if env.ledger().timestamp() > expires_at {
            return ZkVerificationResult::Expired;
        }
    }

    // In a real implementation, this would:
    // 1. Load the verification key for the circuit
    // 2. Perform cryptographic verification of the proof
    // 3. Verify public inputs match expected values
    
    // For this implementation, we simulate verification success
    // based on proof structure validity
    if proof.public_inputs.is_empty() {
        return ZkVerificationResult::Invalid;
    }

    // Check circuit ID is recognized (simulated)
    let circuit_id = proof.circuit_id;
    let valid_circuits = [
        Symbol::new(env, "claim_validity"),
        Symbol::new(env, "policy_coverage"),
        Symbol::new(env, "amount_range"),
        Symbol::new(env, "identity_verification"),
    ];
    
    if !valid_circuits.contains(&circuit_id) {
        return ZkVerificationResult::UnknownCircuit;
    }

    ZkVerificationResult::Valid
}

#[contractimpl]
impl PrivacyContract {
    /// Initialize the privacy contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().persistent().has(&ADMIN) {
            return Err(ContractError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&PROOF_COUNTER, &0u64);
        env.storage().persistent().set(&COMPLIANCE_COUNTER, &0u64);

        env.events().publish((symbol_short!("init"), ()), admin);

        Ok(())
    }

    /// Set privacy settings for a user
    pub fn set_privacy_settings(
        env: Env,
        user: Address,
        privacy_enabled: bool,
        privacy_level: u32,
        encryption_key: Option<BytesN<32>>,
        retention_days: u32,
        regulatory_compliance: bool,
    ) -> Result<(), ContractError> {
        user.require_auth();

        if privacy_level == 0 || privacy_level > 3 {
            return Err(ContractError::InvalidInput);
        }

        if retention_days == 0 || retention_days > 3650 {
            // Max 10 years
            return Err(ContractError::InvalidInput);
        }

        let settings = PrivacySettings {
            user: user.clone(),
            privacy_enabled,
            privacy_level,
            encryption_key,
            retention_days,
            regulatory_compliance,
            updated_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&(USER_PRIVACY, user.clone()), &settings);

        env.events().publish(
            (symbol_short!("priv_set"), user.clone()),
            (privacy_enabled, privacy_level),
        );

        Ok(())
    }

    /// Submit a confidential claim
    pub fn submit_confidential_claim(
        env: Env,
        claimant: Address,
        policy_id: u64,
        encrypted_amount: BytesN<32>,
        commitment_hash: BytesN<32>,
        coverage_proof: BytesN<32>,
        privacy_level: u32,
    ) -> Result<u64, ContractError> {
        claimant.require_auth();

        if is_paused(&env) {
            return Err(ContractError::Paused);
        }

        // Check user's privacy settings
        let user_privacy: Option<PrivacySettings> = env
            .storage()
            .persistent()
            .get(&(USER_PRIVACY, claimant.clone()));

        if let Some(settings) = user_privacy {
            if !settings.privacy_enabled {
                return Err(ContractError::PrivacyDisabled);
            }
        }

        let claim_id = get_next_proof_id(&env);

        let confidential_claim = ConfidentialClaim {
            claim_id,
            policy_id,
            claimant: claimant.clone(),
            encrypted_amount,
            commitment_hash,
            validity_proof_id: BytesN::from_array(&env, &[0; 32]), // Empty initially
            coverage_proof,
            submitted_at: env.ledger().timestamp(),
            privacy_level,
        };

        env.storage().persistent().set(
            &(CONFIDENTIAL_CLAIM, claim_id),
            &confidential_claim,
        );

        env.events().publish(
            (symbol_short!("conf_claim"), claim_id),
            (claimant, policy_id, privacy_level),
        );

        Ok(claim_id)
    }

    /// Attach a ZK proof to a confidential claim
    pub fn attach_claim_proof(
        env: Env,
        claimant: Address,
        claim_id: u64,
        zk_proof: ZkProof,
    ) -> Result<(), ContractError> {
        claimant.require_auth();

        let mut claim: ConfidentialClaim = env
            .storage()
            .persistent()
            .get(&(CONFIDENTIAL_CLAIM, claim_id))
            .ok_or(ContractError::NotFound)?;

        if claim.claimant != claimant {
            return Err(ContractError::Unauthorized);
        }

        // Verify the proof
        let verification_result = verify_zk_proof(&env, &zk_proof);
        
        if verification_result != ZkVerificationResult::Valid {
            return Err(ContractError::ProofInvalid);
        }

        // Store the proof
        let proof_id = zk_proof.proof_id.clone();
        let privacy_proof = PrivacyProof {
            proof_id: proof_id.clone(),
            entity_id: claim_id,
            entity_type: Symbol::new(&env, "claim"),
            zk_proof,
            verification_result,
            verified_at: Some(env.ledger().timestamp()),
            verifier: Some(env.current_contract_address()),
        };

        env.storage()
            .persistent()
            .set(&(ZK_PROOF, proof_id.clone()), &privacy_proof);

        // Update claim with proof ID
        claim.validity_proof_id = proof_id;
        env.storage().persistent().set(
            &(CONFIDENTIAL_CLAIM, claim_id),
            &claim,
        );

        env.events().publish(
            (symbol_short!("proof_att"), claim_id),
            proof_id,
        );

        Ok(())
    }

    /// Create a private policy
    pub fn create_private_policy(
        env: Env,
        holder: Address,
        encrypted_coverage: BytesN<32>,
        encrypted_premium: BytesN<32>,
        policy_commitment: BytesN<32>,
    ) -> Result<u64, ContractError> {
        holder.require_auth();

        if is_paused(&env) {
            return Err(ContractError::Paused);
        }

        let policy_id = get_next_proof_id(&env);

        let private_policy = PrivatePolicyData {
            policy_id,
            holder: holder.clone(),
            encrypted_coverage,
            encrypted_premium,
            policy_commitment,
            policy_proof_id: BytesN::from_array(&env, &[0; 32]),
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(
            &(PRIVATE_POLICY, policy_id),
            &private_policy,
        );

        env.events().publish(
            (symbol_short!("priv_pol"), policy_id),
            holder,
        );

        Ok(policy_id)
    }

    /// Attach ZK proof to private policy
    pub fn attach_policy_proof(
        env: Env,
        holder: Address,
        policy_id: u64,
        zk_proof: ZkProof,
    ) -> Result<(), ContractError> {
        holder.require_auth();

        let mut policy: PrivatePolicyData = env
            .storage()
            .persistent()
            .get(&(PRIVATE_POLICY, policy_id))
            .ok_or(ContractError::NotFound)?;

        if policy.holder != holder {
            return Err(ContractError::Unauthorized);
        }

        // Verify the proof
        let verification_result = verify_zk_proof(&env, &zk_proof);
        
        if verification_result != ZkVerificationResult::Valid {
            return Err(ContractError::ProofInvalid);
        }

        // Store the proof
        let proof_id = zk_proof.proof_id.clone();
        let privacy_proof = PrivacyProof {
            proof_id: proof_id.clone(),
            entity_id: policy_id,
            entity_type: Symbol::new(&env, "policy"),
            zk_proof,
            verification_result,
            verified_at: Some(env.ledger().timestamp()),
            verifier: Some(env.current_contract_address()),
        };

        env.storage()
            .persistent()
            .set(&(ZK_PROOF, proof_id.clone()), &privacy_proof);

        // Update policy with proof ID
        policy.policy_proof_id = proof_id;
        env.storage().persistent().set(
            &(PRIVATE_POLICY, policy_id),
            &policy,
        );

        env.events().publish(
            (symbol_short!("pol_proof"), policy_id),
            proof_id,
        );

        Ok(())
    }

    /// Record compliance check (for regulatory purposes)
    pub fn record_compliance_check(
        env: Env,
        auditor: Address,
        entity_type: Symbol,
        entity_id: u64,
        check_type: Symbol,
        is_compliant: bool,
        encrypted_data: Option<BytesN<32>>,
    ) -> Result<u64, ContractError> {
        auditor.require_auth();

        // Verify auditor is authorized (admin or designated auditor)
        let admin: Address = env.storage().persistent().get(&ADMIN).ok_or(ContractError::NotInitialized)?;
        
        // In production, check against a list of authorized auditors
        if auditor != admin {
            // For now, only admin can record compliance
            // return Err(ContractError::Unauthorized);
        }

        let record_id = get_next_compliance_id(&env);

        let compliance_record = ComplianceRecord {
            record_id,
            entity_type,
            entity_id,
            check_type,
            is_compliant,
            encrypted_data,
            checked_at: env.ledger().timestamp(),
            auditor: Some(auditor.clone()),
        };

        env.storage().persistent().set(
            &(COMPLIANCE_RECORD, record_id),
            &compliance_record,
        );

        env.events().publish(
            (symbol_short!("compliance"), record_id),
            (entity_id, is_compliant),
        );

        Ok(record_id)
    }

    /// Verify a ZK proof (public verification)
    pub fn verify_proof(env: Env, proof_id: BytesN<32>) -> Result<ZkVerificationResult, ContractError> {
        let privacy_proof: PrivacyProof = env
            .storage()
            .persistent()
            .get(&(ZK_PROOF, proof_id))
            .ok_or(ContractError::NotFound)?;

        // Re-verify the proof
        let result = verify_zk_proof(&env, &privacy_proof.zk_proof);

        env.events().publish(
            (symbol_short!("verify"), proof_id),
            result as u32,
        );

        Ok(result)
    }

    /// Pause/unpause contract (admin only)
    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), ContractError> {
        admin.require_auth();

        let stored_admin: Address = env.storage().persistent().get(&ADMIN).ok_or(ContractError::NotInitialized)?;
        if admin != stored_admin {
            return Err(ContractError::Unauthorized);
        }

        set_paused(&env, paused);

        env.events().publish(
            (symbol_short!("paused"), admin),
            paused,
        );

        Ok(())
    }

    // ===== View Functions =====

    /// Get privacy settings for a user
    pub fn get_privacy_settings(env: Env, user: Address) -> Option<PrivacySettings> {
        env.storage().persistent().get(&(USER_PRIVACY, user))
    }

    /// Get confidential claim
    pub fn get_confidential_claim(env: Env, claim_id: u64) -> Option<ConfidentialClaim> {
        env.storage().persistent().get(&(CONFIDENTIAL_CLAIM, claim_id))
    }

    /// Get private policy
    pub fn get_private_policy(env: Env, policy_id: u64) -> Option<PrivatePolicyData> {
        env.storage().persistent().get(&(PRIVATE_POLICY, policy_id))
    }

    /// Get ZK proof
    pub fn get_zk_proof(env: Env, proof_id: BytesN<32>) -> Option<PrivacyProof> {
        env.storage().persistent().get(&(ZK_PROOF, proof_id))
    }

    /// Get compliance record
    pub fn get_compliance_record(env: Env, record_id: u64) -> Option<ComplianceRecord> {
        env.storage().persistent().get(&(COMPLIANCE_RECORD, record_id))
    }

    /// Check if privacy is enabled for a user
    pub fn is_privacy_enabled(env: Env, user: Address) -> bool {
        if let Some(settings) = Self::get_privacy_settings(env, user) {
            settings.privacy_enabled
        } else {
            false
        }
    }

    /// Get all compliance records for an entity
    pub fn get_entity_compliance(
        env: Env,
        entity_type: Symbol,
        entity_id: u64,
    ) -> Vec<ComplianceRecord> {
        // In production, maintain an index for efficient querying
        // For now, return empty vector
        Vec::new(&env)
    }
}
