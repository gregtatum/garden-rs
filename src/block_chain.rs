use std::borrow::Cow;

use crate::hash::Hash;

use chrono::Utc;
use rayon::prelude::*;
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};

pub trait BlockData: SerializedBytes + Clone + std::cmp::PartialEq + std::marker::Send {}

impl<T> BlockData for T where T: SerializedBytes + Clone + std::cmp::PartialEq + std::marker::Send {}

/// When serializing a struct, we need to consistently hash a byte slice of consistent
/// endianness.
pub trait SerializedBytes {
    fn serialized_bytes(&self) -> Cow<[u8]>;
}

impl SerializedBytes for String {
    fn serialized_bytes(&self) -> Cow<[u8]> {
        Cow::from(self.as_bytes())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ReconcileError {
    NoMatchingParent,
    ShorterForeignBlocks,
    MalformedBlocks,
}

/// A block chain is a series of blocks that reference back to the previous block.
/// This implementation has an optional proof of work if you want to burn down the
/// world.
#[derive(PartialEq, Debug, Clone)]
pub struct BlockChain<T>
where
    T: BlockData,
{
    pub proof_of_work_size: usize,
    pub blocks: Vec<Block<T>>,
}

impl<T> BlockChain<T>
where
    T: BlockData,
{
    pub fn new(proof_of_work_size: usize) -> Self {
        Self {
            proof_of_work_size,
            blocks: vec![],
        }
    }

    /// Add a payload. It decides either the fast path of no work, or the slower highly
    /// optimized proof of work path.
    fn add_payload(&mut self, payload: BlockPayload<T>) {
        if self.proof_of_work_size == 0 {
            self.add_payload_no_work(payload);
        } else {
            // Ensure there is always a root block.
            self.add_payload_pow(payload);
        }
    }

    /// This is a fast path for adding a proof of work.
    fn add_payload_no_work(&mut self, payload: BlockPayload<T>) {
        let start = std::time::Instant::now();
        let hash = payload.hash();
        self.blocks.push(Block {
            hash,
            payload,
            computation_time: start.elapsed(),
        });
    }

    /// This is a highly optimized method for adding a payload and computing the proof
    /// of work.
    fn add_payload_pow(&mut self, mut payload: BlockPayload<T>) {
        // If this code gets used for real, it would probably be worth hoisting this
        // higher in the app. As it is I don't care because I don't really plan on using
        // this beyond demos.
        std::env::set_var("RAYON_NUM_THREADS", num_cpus::get().to_string());

        // Time this function, it can take awhile.
        let start = std::time::Instant::now();

        let proof_of_work_size = self.proof_of_work_size;
        // Compute the partial hash to do as much work as possible before going into
        // full parallelism mode.
        let partial_hash = payload.partial_hash();

        // Spin up as many threads as possible to compute the hash.
        payload.proof_of_work = (0..u64::MAX)
            .into_par_iter()
            .find_map_any(move |proof_of_work| {
                // Make a copy of this partial hash.
                let mut partial_hash = partial_hash.clone();
                partial_hash.update(&proof_of_work.to_le_bytes());
                let digest = partial_hash.finish();
                let data: &[u8] = digest.as_ref();

                for index in 0..proof_of_work_size {
                    if data[index] != 0 {
                        // Not enough zeros.
                        return None;
                    }
                }
                if data[proof_of_work_size] == 0 {
                    // Too many zeros.
                    return None;
                }
                Some(proof_of_work)
            })
            .expect("Expected to find a proof of work.");

        // After finding the proof of work, compute the final hash.
        let hash = payload.hash();
        assert!(
            hash.meets_proof_of_work(proof_of_work_size),
            "The hash must meet the proof of work."
        );

        // We're done!
        self.blocks.push(Block {
            hash,
            payload,
            computation_time: start.elapsed(),
        });
    }

    // The public interface to add data. It calls out to the proper internal methods
    /// to create aa payload.
    pub fn add_data(&mut self, data: T) {
        self.add_payload(BlockPayload {
            parent: match self.tip() {
                Some(block) => block.hash.clone(),
                None => Hash::empty(),
            },
            timestamp: Utc::now().timestamp(),
            data,
            proof_of_work: 0,
        });
    }

    /// Get the current tip of the block chain.
    pub fn tip(&self) -> Option<&Block<T>> {
        self.blocks.last()
    }

    pub fn hash_to_block_index(&self, hash: &Hash) -> Option<usize> {
        for (block_index, block) in self.blocks.iter().rev().enumerate() {
            if block.hash == *hash {
                return Some(self.blocks.len() - block_index - 1);
            }
        }
        None
    }

    pub fn reconcile(&mut self, mut foreign_blocks: &[Block<T>]) -> Result<(), ReconcileError> {
        if foreign_blocks.is_empty() {
            // No blocks to add. This is weird, but fine.
            return Ok(());
        }

        if foreign_blocks.first().unwrap().payload.parent.is_root() {
            // The first block appears to be a root block, ignore it.
            foreign_blocks = &foreign_blocks[1..];
        }

        // Try to find the parent block.
        let parent_block_index = {
            let result = self.hash_to_block_index(&foreign_blocks.first().unwrap().payload.parent);
            if result.is_none() {
                return Err(ReconcileError::NoMatchingParent);
            }
            result.unwrap()
        };

        // Fast forward through blocks that share the common roots.
        let mut last_trusted_index = parent_block_index;
        for index in (parent_block_index + 1)..self.blocks.len() {
            let trusted_block = self.blocks.get(index).unwrap();
            let foreign_block = foreign_blocks.get(index - parent_block_index - 1).unwrap();

            if trusted_block == foreign_block {
                last_trusted_index = index;
            } else {
                break;
            }
        }

        let new_foreign_block_base_index = last_trusted_index - parent_block_index;
        let trusted_len = self.blocks.len() - last_trusted_index - 1;
        let foreign_len = foreign_blocks.len() - (last_trusted_index - parent_block_index);

        if trusted_len > foreign_len {
            return Err(ReconcileError::ShorterForeignBlocks);
        }

        let last_trusted_block = self.blocks.get(last_trusted_index).unwrap();
        let new_foreign_blocks = &foreign_blocks[new_foreign_block_base_index..];

        if !verify_blocks(
            self.proof_of_work_size,
            new_foreign_blocks,
            last_trusted_block.hash.clone(),
        ) {
            return Err(ReconcileError::MalformedBlocks);
        }

        self.blocks.truncate(last_trusted_index + 1);
        self.blocks.extend_from_slice(new_foreign_blocks);

        Ok(())
    }
}

/// Debug print the string content of blocks.
#[allow(dead_code)] // Useful for debugging, and used in tests.
fn debug_blocks(blocks: &[Block<String>]) -> Vec<&str> {
    blocks
        .iter()
        .map(|s| s.payload.data.as_str())
        .collect::<Vec<&str>>()
}

/// Ensure the blocks are valid in their structure.
fn verify_blocks<T: BlockData>(
    proof_of_work_size: usize,
    blocks: &[Block<T>],
    mut parent: Hash,
) -> bool {
    for block in blocks {
        if block.payload.parent != parent {
            return false;
        }
        let hash = block.payload.hash();
        if block.hash != hash {
            return false;
        }
        for i in 0..proof_of_work_size {
            if block.hash.0[i] != 0 {
                return false;
            }
        }
        parent = hash;
    }
    true
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct BlockPayload<T>
where
    T: BlockData,
{
    pub parent: Hash,
    pub timestamp: i64,
    pub data: T,
    pub proof_of_work: u64,
}

impl<T> BlockPayload<T>
where
    T: BlockData,
{
    fn hash(&self) -> Hash {
        let mut context = Context::new(&SHA256);
        context.update(&self.parent.0);
        context.update(&self.timestamp.to_le_bytes());
        context.update(&self.data.serialized_bytes());
        context.update(&self.proof_of_work.to_le_bytes());
        let digest = context.finish();
        let data: &[u8] = digest.as_ref();
        assert_eq!(data.len(), 32, "Expected the hash to be 32 bytes");
        let mut hash = Hash::empty();
        for (index, byte) in data.iter().enumerate() {
            hash.0[index] = *byte;
        }
        hash
    }

    // In order to speed up the proof of work hashing, only partially do the hashing work.
    fn partial_hash(&self) -> Context {
        let mut context = Context::new(&SHA256);
        context.update(&self.parent.0);
        context.update(&self.timestamp.to_le_bytes());
        context.update(&self.data.serialized_bytes());
        context
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Block<T: BlockData> {
    pub hash: Hash,
    pub computation_time: std::time::Duration,
    pub payload: BlockPayload<T>,
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_block_text(block_chain: &BlockChain<String>, index: usize) -> &str {
        &block_chain
            .blocks
            .get(index)
            .expect("Failed to get the block at the index.")
            .payload
            .data
    }

    #[test]
    fn test_add_data() {
        let mut block_chain = BlockChain::<String>::new(1);

        block_chain.add_data("First block".into());
        block_chain.add_data("Second block".into());
        block_chain.add_data("Third block".into());

        assert_eq!(get_block_text(&block_chain, 0), "First block");
        assert_eq!(get_block_text(&block_chain, 1), "Second block");
        assert_eq!(get_block_text(&block_chain, 2), "Third block");
    }

    #[test]
    fn test_rooted_reconcile() {
        // This will reconcile a longer blockchain with our shorter one.

        let mut trusted = BlockChain::<String>::new(1);

        trusted.add_data("a".into());
        trusted.add_data("b".into());
        trusted.add_data("c".into());

        let mut foreign = trusted.clone();

        foreign.add_data("d".into());
        foreign.add_data("e".into());

        assert_ne!(trusted, foreign, "The two are different");

        trusted
            .reconcile(&foreign.blocks)
            .expect("Failed to reconcile blockchains.");

        assert_eq!(trusted, foreign, "The two are equal");
    }

    #[test]
    fn test_rootless_reconcile() {
        let mut trusted = BlockChain::<String>::new(1);

        trusted.add_data("a".into());
        trusted.add_data("b".into());
        trusted.add_data("c".into());

        let mut foreign = trusted.clone();

        foreign.add_data("d".into());
        foreign.add_data("e".into());

        assert_ne!(trusted, foreign, "The two are different");

        trusted
            .reconcile(&foreign.blocks[1..])
            .expect("Failed to reconcile blockchains.");

        assert_eq!(trusted, foreign, "The two are equal");
        assert_eq!(debug_blocks(&trusted.blocks), vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn test_foreign_wins() {
        let mut trusted = BlockChain::<String>::new(1);

        trusted.add_data("a".into());
        trusted.add_data("b".into());
        trusted.add_data("c".into());

        let mut foreign = trusted.clone();

        trusted.add_data("losing".into());
        foreign.add_data("d".into());
        foreign.add_data("e".into());

        assert_ne!(trusted, foreign, "The two are different");

        trusted
            .reconcile(&foreign.blocks[3..])
            .expect("Failed to reconcile blockchains.");

        assert_eq!(trusted, foreign, "The two are equal");

        assert_eq!(debug_blocks(&trusted.blocks), vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn test_trusting_wins() {
        let mut trusted = BlockChain::<String>::new(1);

        trusted.add_data("a".into());
        trusted.add_data("b".into());
        trusted.add_data("c".into());

        let mut foreign = trusted.clone();

        trusted.add_data("d".into());
        trusted.add_data("e".into());
        foreign.add_data("losing".into());

        assert_ne!(trusted, foreign, "The two are different");

        assert_eq!(
            trusted
                .reconcile(&foreign.blocks[3..])
                .expect_err("Expected an error"),
            ReconcileError::ShorterForeignBlocks
        );

        assert_eq!(debug_blocks(&trusted.blocks), vec!["a", "b", "c", "d", "e"]);
        assert_eq!(debug_blocks(&foreign.blocks), vec!["a", "b", "c", "losing"]);
    }

    #[test]
    fn test_failed_reconcile() {
        let mut trusted = BlockChain::<String>::new(1);

        trusted.add_data("a".into());
        trusted.add_data("b".into());
        trusted.add_data("c".into());

        let mut foreign = trusted.clone();

        trusted.add_data("d".into());
        trusted.add_data("e".into());
        foreign.add_data("D".into());
        foreign.add_data("E".into());
        foreign.add_data("F".into());

        assert_ne!(trusted, foreign, "The two are different");

        // Carve off the blocks at the end that don't match anymore.
        let e_index = foreign
            .blocks
            .iter()
            .position(|b| b.payload.data == "E")
            .unwrap();

        let blocks = &foreign.blocks[e_index..];
        assert_eq!(debug_blocks(&blocks), vec!["E", "F"]);

        assert_eq!(
            trusted
                .reconcile(&blocks)
                .expect_err("It should have failed."),
            ReconcileError::NoMatchingParent
        );
    }

    #[test]
    fn test_invalid_proof_of_work() {
        let mut trusted = BlockChain::<String>::new(1);

        trusted.add_data("a".into());
        trusted.add_data("b".into());
        trusted.add_data("c".into());

        let mut foreign = trusted.clone();

        foreign.add_data("d".into());
        let mut block_d = foreign.blocks.get_mut(2).unwrap();

        // Do not provide the proof of work for the last block.
        block_d.payload.proof_of_work = 0;
        block_d.hash = block_d.payload.hash();

        assert_ne!(trusted, foreign, "The two are different");

        // Carve off the blocks at the end that don't match anymore.
        assert_eq!(debug_blocks(&foreign.blocks), vec!["a", "b", "c", "d"]);

        assert_eq!(
            trusted
                .reconcile(&foreign.blocks)
                .expect_err("It should have failed."),
            ReconcileError::MalformedBlocks
        );
    }

    #[test]
    fn test_no_proof_of_work() {
        let mut quick_chain = BlockChain::<String>::new(0);

        quick_chain.add_data("a".into());
        quick_chain.add_data("b".into());
        quick_chain.add_data("c".into());

        assert_eq!(debug_blocks(&quick_chain.blocks), vec!["a", "b", "c"]);
    }
}
