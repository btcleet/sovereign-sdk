#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod capabilities;
#[cfg(feature = "native")]
mod query;
#[cfg(feature = "native")]
pub use query::{BlobStorageRpcImpl, BlobStorageRpcServer, Response};
use sov_modules_api::{Module, ModuleInfo};
use sov_rollup_interface::da::BlobReaderTrait;
use sov_state::{StateMap, StateValue, WorkingSet};

/// For how many slots deferred blobs are stored before being executed
const DEFERRED_SLOTS_COUNT: u64 = 1;

/// Blob storage contains only address and vector of blobs
#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
#[derive(Clone, ModuleInfo)]
pub struct BlobStorage<C: sov_modules_api::Context, B: BlobReaderTrait>
where
    B::Address: borsh::BorshSerialize + borsh::BorshDeserialize,
{
    /// The address of blob storage module
    /// Note: this is address is generated by the module framework and the corresponding private key is unknown.
    #[address]
    pub(crate) address: C::Address,

    /// Actual storage of blobs
    /// DA block number => vector of blobs
    /// Caller controls the order of blobs in the vector
    #[state]
    pub(crate) blobs: StateMap<u64, Vec<Vec<u8>>>,

    #[module]
    pub(crate) sequencer_registry: sov_sequencer_registry::SequencerRegistry<C, B::Address>,

    #[state]
    pub(crate) slot_number: StateValue<u64>,
}

/// Non standard methods for blob storage
impl<C: sov_modules_api::Context, B: BlobReaderTrait> BlobStorage<C, B>
where
    B::Address: borsh::BorshSerialize + borsh::BorshDeserialize,
{
    /// Store blobs for given block number, overwrite if already exists
    pub fn store_blobs(
        &self,
        block_number: u64,
        blobs: &[&B],
        working_set: &mut WorkingSet<C::Storage>,
    ) -> anyhow::Result<()> {
        let mut raw_blobs: Vec<Vec<u8>> = Vec::with_capacity(blobs.len());
        for blob in blobs {
            raw_blobs.push(bincode::serialize(blob)?);
        }
        self.blobs.set(&block_number, &raw_blobs, working_set);
        Ok(())
    }

    /// Take all blobs for given block number, return empty vector if not exists
    /// Returned blobs are removed from the storage
    pub fn take_blobs_for_block_number(
        &self,
        block_number: u64,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Vec<B> {
        self.blobs
            .remove(&block_number, working_set)
            .unwrap_or_default()
            .iter()
            .map(|b| bincode::deserialize(b).expect("malformed blob was stored previously"))
            .collect()
    }

    pub(crate) fn get_preferred_sequencer(
        &self,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Option<B::Address> {
        self.sequencer_registry.get_preferred_sequencer(working_set)
    }

    pub(crate) fn get_current_slot_number(&self, working_set: &mut WorkingSet<C::Storage>) -> u64 {
        self.slot_number
            .get(working_set)
            .expect("slot number is not set in genesis")
    }

    pub(crate) fn get_deferred_slots_count(
        &self,
        _working_set: &mut WorkingSet<C::Storage>,
    ) -> u64 {
        DEFERRED_SLOTS_COUNT
    }
}

/// Empty module implementation
impl<C: sov_modules_api::Context, B: BlobReaderTrait> Module for BlobStorage<C, B>
where
    B::Address: borsh::BorshSerialize + borsh::BorshDeserialize,
{
    type Context = C;
    type Config = ();
    type CallMessage = sov_modules_api::NonInstantiable;

    /// TODO: Remove this when chain-state is available https://github.com/Sovereign-Labs/sovereign-sdk/pull/598
    fn genesis(
        &self,
        _config: &Self::Config,
        working_set: &mut WorkingSet<<Self::Context as sov_modules_api::Spec>::Storage>,
    ) -> Result<(), sov_modules_api::Error> {
        self.slot_number.set(&0, working_set);
        Ok(())
    }
}
