use crate::hash::Hash;

use chrono::Utc;
use rayon::prelude::*;
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};

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
pub struct BlockChain {
    pub proof_of_work_size: usize,
    pub blocks: Vec<Block>,
}

impl BlockChain {
    pub fn new(proof_of_work_size: usize) -> Self {
        let mut block_chain = Self {
            proof_of_work_size,
            blocks: vec![],
        };

        block_chain.add_payload(BlockPayload {
            parent: Hash([0; 32]),
            timestamp: Utc::now().timestamp(),
            data: Vec::new(),
            proof_of_work: 0,
        });

        block_chain
    }

    /// Add a payload. It decides either the fast path of no work, or the slower highly
    /// optimized proof of work path.
    fn add_payload(&mut self, payload: BlockPayload) {
        if self.proof_of_work_size == 0 {
            self.add_payload_no_work(payload);
        } else {
            // Ensure there is always a root block.
            self.add_payload_pow(payload);
        }
    }

    /// This is a fast path for adding a proof of work.
    fn add_payload_no_work(&mut self, payload: BlockPayload) {
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
    fn add_payload_pow(&mut self, payload: BlockPayload) {
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
        let payload = (0..u64::MAX)
            .into_par_iter()
            .find_map_any(move |proof_of_work| {
                // Make a copy of this partial hash.
                let mut partial_hash = partial_hash.clone();
                partial_hash.update(&proof_of_work.to_le_bytes());
                let digest = partial_hash.finish();
                let data: &[u8] = digest.as_ref();

                for index in 0..proof_of_work_size {
                    if data[index] != 0 {
                        return None;
                    }
                }
                if data[proof_of_work_size] == 0 {
                    return None;
                }

                let mut payload = payload.clone();
                payload.proof_of_work = proof_of_work;
                if payload.hash().meets_proof_of_work(proof_of_work_size) {
                    Some(payload)
                } else {
                    None
                }
            })
            .unwrap();

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
    pub fn add_data(&mut self, data: Vec<u8>) {
        self.add_payload(BlockPayload {
            parent: self.tip().hash.clone(),
            timestamp: Utc::now().timestamp(),
            data,
            proof_of_work: 0,
        });
    }

    /// Get the current tip of the block chain.
    pub fn tip(&self) -> &Block {
        self.blocks.last().expect("Unable to find a root block.")
    }

    pub fn hash_to_block_index(&self, hash: &Hash) -> Option<usize> {
        for (block_index, block) in self.blocks.iter().rev().enumerate() {
            if block.hash == *hash {
                return Some(self.blocks.len() - block_index - 1);
            }
        }
        None
    }

    pub fn reconcile(&mut self, mut foreign_blocks: &[Block]) -> Result<(), ReconcileError> {
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
fn debug_blocks(blocks: &[Block]) -> Vec<&str> {
    blocks
        .iter()
        .map(|b| b.parse_data_as_utf8().unwrap())
        .collect::<Vec<&str>>()
}

/// Ensure the blocks are valid in their structure.
fn verify_blocks(proof_of_work_size: usize, blocks: &[Block], mut parent: Hash) -> bool {
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
pub struct BlockPayload {
    pub parent: Hash,
    pub timestamp: i64,
    pub data: Vec<u8>,
    pub proof_of_work: u64,
}

impl BlockPayload {
    fn hash(&self) -> Hash {
        let mut context = Context::new(&SHA256);
        context.update(&self.parent.0);
        context.update(&self.timestamp.to_le_bytes());
        context.update(&self.data);
        context.update(&self.proof_of_work.to_le_bytes());
        let digest = context.finish();
        let data: &[u8] = digest.as_ref();
        assert_eq!(data.len(), 32, "Expected the hash to be 32 bytes");
        let mut hash = Hash::new();
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
        context.update(&self.data);
        context
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub hash: Hash,
    pub computation_time: std::time::Duration,
    pub payload: BlockPayload,
}

impl Block {
    fn parse_data_as_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.payload.data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_block_text(block_chain: &BlockChain, index: usize) -> &str {
        block_chain
            .blocks
            .get(index)
            .expect("Failed to get the block at the index.")
            .parse_data_as_utf8()
            .expect("Failed to parse the block data as text.")
    }

    #[test]
    fn test_add_data() {
        let mut block_chain = BlockChain::new(1);

        block_chain.add_data("First block".into());
        block_chain.add_data("Second block".into());
        block_chain.add_data("Third block".into());

        assert_eq!(get_block_text(&block_chain, 0), "");
        assert_eq!(get_block_text(&block_chain, 1), "First block");
        assert_eq!(get_block_text(&block_chain, 2), "Second block");
        assert_eq!(get_block_text(&block_chain, 3), "Third block");
    }

    #[test]
    fn test_rooted_reconcile() {
        // This will reconcile a longer blockchain with our shorter one.

        let mut trusted = BlockChain::new(1);

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
        let mut trusted = BlockChain::new(1);

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
        assert_eq!(
            debug_blocks(&trusted.blocks),
            vec!["", "a", "b", "c", "d", "e"]
        );
    }

    #[test]
    fn test_foreign_wins() {
        let mut trusted = BlockChain::new(1);

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

        assert_eq!(
            debug_blocks(&trusted.blocks),
            vec!["", "a", "b", "c", "d", "e"]
        );
    }

    #[test]
    fn test_trusting_wins() {
        let mut trusted = BlockChain::new(1);

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

        assert_eq!(
            debug_blocks(&trusted.blocks),
            vec!["", "a", "b", "c", "d", "e"]
        );
        assert_eq!(
            debug_blocks(&foreign.blocks),
            vec!["", "a", "b", "c", "losing"]
        );
    }

    #[test]
    fn test_failed_reconcile() {
        let mut trusted = BlockChain::new(1);

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
        let blocks = &foreign.blocks[5..];
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
        let mut trusted = BlockChain::new(1);

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
        assert_eq!(debug_blocks(&foreign.blocks), vec!["", "a", "b", "c", "d"]);

        assert_eq!(
            trusted
                .reconcile(&foreign.blocks)
                .expect_err("It should have failed."),
            ReconcileError::MalformedBlocks
        );
    }

    #[test]
    fn test_no_proof_of_work() {
        let mut quick_chain = BlockChain::new(0);

        quick_chain.add_data("a".into());
        quick_chain.add_data("b".into());
        quick_chain.add_data("c".into());

        assert_eq!(debug_blocks(&quick_chain.blocks), vec!["", "a", "b", "c"]);
    }
}
