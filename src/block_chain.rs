use std::borrow::Cow;

use crate::hash::Hash;

use chrono::Utc;
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};

pub trait BlockData:
    SerializedBytes + Clone + std::cmp::PartialEq + std::marker::Send
{
}

impl<T> BlockData for T where
    T: SerializedBytes + Clone + std::cmp::PartialEq + std::marker::Send
{
}

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

/// A block chain is a series of blocks that reference back to the previous block with
/// a cryptographically secure hash. It is a form of a distributed ledger.
///
/// This implementation does not feature a proof of work, as it's not a cryptocurrency.
/// See commit dfc6dd0 for the last time the proof of work was used for this.
///
/// If you are reading this comment, I recommend this article:
/// https://pfrazee.github.io/blog/secure-ledgers-dont-require-proof-of-work
#[derive(PartialEq, Debug, Clone)]
pub struct BlockChain<T>
where
    T: BlockData,
{
    pub blocks: Vec<Block<T>>,
}

impl<T> BlockChain<T>
where
    T: BlockData,
{
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    fn add_payload(&mut self, payload: BlockPayload<T>) -> &Block<T> {
        let hash = payload.hash();
        self.blocks.push(Block { hash, payload });
        self.blocks.last().unwrap()
    }

    // The public interface to add data. It calls out to the proper internal methods
    /// to create aa payload.
    pub fn add_data(&mut self, data: T) -> &Block<T> {
        self.add_payload(BlockPayload {
            parent: match self.tip() {
                Some(block) => block.hash.clone(),
                None => Hash::empty(),
            },
            timestamp: if cfg!(test) {
                // For tests only monotonically increase the timestamp.
                match self.tip() {
                    Some(block) => block.payload.timestamp + 1,
                    None => 0,
                }
            } else {
                Utc::now().timestamp()
            },
            data,
        })
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

    pub fn reconcile(
        &mut self,
        mut foreign_blocks: &[Block<T>],
    ) -> Result<(), ReconcileError> {
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
            let result =
                self.hash_to_block_index(&foreign_blocks.first().unwrap().payload.parent);
            if result.is_none() {
                return Err(ReconcileError::NoMatchingParent);
            }
            result.unwrap()
        };

        // Fast forward through blocks that share the common roots.
        let mut last_trusted_index = parent_block_index;
        for index in (parent_block_index + 1)..self.blocks.len() {
            let trusted_block = self.blocks.get(index).unwrap();
            let foreign_block =
                foreign_blocks.get(index - parent_block_index - 1).unwrap();

            if trusted_block == foreign_block {
                last_trusted_index = index;
            } else {
                break;
            }
        }

        let new_foreign_block_base_index = last_trusted_index - parent_block_index;
        let trusted_len = self.blocks.len() - last_trusted_index - 1;
        let foreign_len =
            foreign_blocks.len() - (last_trusted_index - parent_block_index);

        if trusted_len > foreign_len {
            return Err(ReconcileError::ShorterForeignBlocks);
        }

        let last_trusted_block = self.blocks.get(last_trusted_index).unwrap();
        let new_foreign_blocks = &foreign_blocks[new_foreign_block_base_index..];

        if !verify_blocks(new_foreign_blocks, last_trusted_block.hash.clone()) {
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
fn verify_blocks<T: BlockData>(blocks: &[Block<T>], mut parent: Hash) -> bool {
    for block in blocks {
        if block.payload.parent != parent {
            return false;
        }
        let hash = block.payload.hash();
        if block.hash != hash {
            return false;
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
        let digest = context.finish();
        let data: &[u8] = digest.as_ref();
        assert_eq!(data.len(), 32, "Expected the hash to be 32 bytes");
        let mut hash = Hash::empty();
        for (index, byte) in data.iter().enumerate() {
            hash.0[index] = *byte;
        }
        hash
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Block<T: BlockData> {
    pub hash: Hash,
    pub payload: BlockPayload<T>,
}

#[cfg(test)]
mod test {
    use serde_json::json;

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
        let mut block_chain = BlockChain::<String>::new();

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

        let mut trusted = BlockChain::<String>::new();

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
        let mut trusted = BlockChain::<String>::new();

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
        let mut trusted = BlockChain::<String>::new();

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
        let mut trusted = BlockChain::<String>::new();

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
        let mut trusted = BlockChain::<String>::new();

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
    fn test_serialize_block() {
        let mut chain = BlockChain::<String>::new();
        chain.add_data("data 1".into());
        let value = serde_json::to_value(chain.tip().unwrap())
            .expect("failed to convert to JSON value");

        // println!("{}", serde_json::to_string_pretty(&value).unwrap());

        assert_eq!(
            value,
            json!({
                "hash": "0aa8416c618aa6f5243c8a273a4398991ed5f8e097d6807b30164d37c8d84b33",
                "payload": {
                    "data": "data 1",
                    "parent": "0000000000000000000000000000000000000000000000000000000000000000",
                    "timestamp": 0
                }
            })
        );
    }

    #[test]
    fn test_serialize_blocks() {
        let mut chain = BlockChain::<String>::new();
        chain.add_data("data 1".into());
        chain.add_data("data 2".into());
        let value =
            serde_json::to_value(chain.blocks).expect("failed to convert to JSON value");

        // println!("{}", serde_json::to_string_pretty(&value).unwrap());

        assert_eq!(
            value,
            json!([
                {
                    "hash": "0aa8416c618aa6f5243c8a273a4398991ed5f8e097d6807b30164d37c8d84b33",
                    "payload": {
                        "data": "data 1",
                        "parent": "0000000000000000000000000000000000000000000000000000000000000000",
                        "timestamp": 0
                    }
                },
                {
                    "hash": "dc8243497f48f2fbb2677646456d4d3f123250a95c838082bfc97716b775b5ff",
                    "payload": {
                        "data": "data 2",
                        "parent": "0aa8416c618aa6f5243c8a273a4398991ed5f8e097d6807b30164d37c8d84b33",
                        "timestamp": 1
                    }
                }
            ])
        );
    }

    #[test]
    fn test_serialize_blocks_slice() {
        let mut chain = BlockChain::<String>::new();
        chain.add_data("data 1".into());
        chain.add_data("data 2".into());
        chain.add_data("data 3".into());
        let value = serde_json::to_value(&chain.blocks[1..])
            .expect("failed to convert to JSON value");

        println!("{}", serde_json::to_string_pretty(&value).unwrap());

        assert_eq!(
            value,
            json!([
              {
                "hash": "dc8243497f48f2fbb2677646456d4d3f123250a95c838082bfc97716b775b5ff",
                "payload": {
                  "data": "data 2",
                  "parent": "0aa8416c618aa6f5243c8a273a4398991ed5f8e097d6807b30164d37c8d84b33",
                  "timestamp": 1
                }
              },
              {
                "hash": "fba2f217aa0411b48bc370769b9018dbbd1996f7d6ef0221e9db829975931330",
                "payload": {
                  "data": "data 3",
                  "parent": "dc8243497f48f2fbb2677646456d4d3f123250a95c838082bfc97716b775b5ff",
                  "timestamp": 2
                }
              }
            ])
        );
    }
}
