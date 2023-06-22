//! Utilities for building an optimistic state machine
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::zk::traits::StateTransition;

/// An attestation that a particular DA layer block transitioned the rollup state to some value
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Attestation<StateProof> {
    /// The alleged state root before applying the contents of the da block
    pub initial_state_root: [u8; 32],
    /// The hash of the block in which the transition occurred
    pub da_block_hash: [u8; 32],
    /// The alleged post-state root
    pub post_state_root: [u8; 32],
    /// A proof that the attester was bonded as of `initial_state_root`.
    /// For rollups using the `jmt`, this will be a `jmt::SparseMerkleProof`
    pub proof_of_bond: StateProof,
}

/// The contents of a challenge to an attestation, which are contained as a public output of the proof
/// Generic over an address type and a validity condition
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ChallengeContents<Address, VC> {
    /// The rollup address of the originator of this challenge
    pub challenger_address: Address,
    /// The state transition that was proven
    pub state_transition: StateTransition<VC>,
}

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, Serialize, Deserialize)]
pub struct Challenge<'a>(&'a [u8]);
