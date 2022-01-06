use chrono::Utc;
use ring::digest::{Context, SHA256};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub struct BlockChain {
    pub proof_of_work_size: usize,
    pub blocks: Vec<Block>,
}

impl fmt::Debug for Hash {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash({})", String::from(self))?;
        Ok(())
    }
}

/// A representation of a Hash.
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn new() -> Self {
        Hash([0; 32])
    }

    /// Use the proof of work size to determine how many leading 0 values to use.
    pub fn meets_proof_of_work(&self, proof_of_work_size: usize) -> bool {
        for i in 0..proof_of_work_size {
            if self.0[i] != 0 {
                return false;
            }
        }
        // Ensure that there aren't any additional leading zeros.
        self.0[proof_of_work_size] != 0
    }

    pub fn is_root(&self) -> bool {
        for byte in self.0 {
            if byte != 0 {
                return false;
            }
        }
        true
    }
}

impl From<&Hash> for String {
    fn from(other: &Hash) -> Self {
        format!("{}", other)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&String::from(self))
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HashVisitor)
    }
}

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a 64 character sha256 hash")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.len() != 64 {
            return Err(E::custom(format!("Hash is not 64 characters: {}", value)));
        }
        let mut result = [0; 32];
        for i in 0..32 {
            result[i] = match u8::from_str_radix(&value[i * 2..i * 2 + 2], 16) {
                Ok(v) => v,
                Err(_) => return Err(E::custom(format!("Hash could not be parsed: {}", value))),
            };
        }
        Ok(Hash(result))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ReconcileError {
    NoMatchingParent,
    ShorterForeignBlocks,
    MalformedBlocks,
}

impl BlockChain {
    fn new(proof_of_work_size: usize) -> Self {
        let mut block_chain = Self {
            proof_of_work_size,
            blocks: vec![],
        };

        // Ensure there is always a root block.
        block_chain.add_payload_impl(BlockPayload {
            parent: Hash([0; 32]),
            timestamp: Utc::now().timestamp(),
            data: Vec::new(),
            proof_of_work: 0,
        });

        block_chain
    }

    fn add_payload_impl(&mut self, mut payload: BlockPayload) {
        loop {
            let hash = payload.hash();
            if hash.meets_proof_of_work(self.proof_of_work_size) {
                self.blocks.push(Block {
                    hash: payload.hash(),
                    payload: payload,
                });
                return;
            }
            payload.proof_of_work += 1;
        }
    }

    pub fn add_data(&mut self, data: Vec<u8>) {
        self.add_payload_impl(BlockPayload {
            parent: self.tip().hash.clone(),
            timestamp: Utc::now().timestamp(),
            data,
            proof_of_work: 0,
        });
    }

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

#[allow(dead_code)] // Useful for debugging, and used in tests.
fn debug_blocks(blocks: &[Block]) -> Vec<&str> {
    blocks
        .iter()
        .map(|b| b.parse_data_as_utf8().unwrap())
        .collect::<Vec<&str>>()
}

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
        context.update(&self.proof_of_work.to_le_bytes());
        context.update(&self.data);
        let digest = context.finish();
        let data: &[u8] = digest.as_ref();
        assert_eq!(data.len(), 32, "Expected the hash to be 32 bytes");
        let mut hash = Hash::new();
        for (index, byte) in data.iter().enumerate() {
            hash.0[index] = *byte;
        }
        hash
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub hash: Hash,
    pub payload: BlockPayload,
}

impl Block {
    fn parse_data_as_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.payload.data)
    }
}

fn main() {
    let mut block_chain = BlockChain::new(2);

    let start = std::time::Instant::now();
    block_chain.add_data("First block".into());
    println!("Timing: {:?}", start.elapsed());
    block_chain.add_data("Second block".into());
    println!("Timing: {:?}", start.elapsed());
    block_chain.add_data("Third block".into());
    println!("Timing: {:?}", start.elapsed());

    println!("{:#?}", block_chain);

    for (block_index, block) in block_chain.blocks.iter().enumerate() {
        println!(
            "Block {} {:?}",
            block_index,
            match std::str::from_utf8(&block.payload.data) {
                Ok(string) => string,
                Err(_) => "invalid utf8",
            }
        )
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
    fn test_blockchain_add_data() {
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
    fn test_blockchain_rooted_reconcile() {
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
    fn test_blockchain_rootless_reconcile() {
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
    fn test_blockchain_foreign_wins() {
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
    fn test_blockchain_trusting_wins() {
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
    fn test_blockchain_failed_reconcile() {
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
    fn test_blockchain_no_proof_of_work() {
        let mut trusted = BlockChain::new(1);

        trusted.add_data("a".into());
        trusted.add_data("b".into());
        trusted.add_data("c".into());

        let mut foreign = trusted.clone();

        foreign.add_data("d".into());
        let mut block_d = foreign.blocks.get_mut(3).unwrap();

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
}
