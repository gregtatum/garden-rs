use chrono::Utc;
use ring::digest::{Context, SHA256};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

#[derive(Debug)]
pub struct BlockChain {
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

impl BlockChain {
    fn new() -> Self {
        Self { blocks: vec![] }
    }

    fn add_payload(&mut self, payload: BlockPayload) -> &Block {
        self.blocks.push(Block {
            hash: payload.hash(),
            payload: payload,
        });
        self.blocks.last().unwrap()
    }

    pub fn add(&mut self, data: Vec<u8>) -> &Block {
        let prev_block = match self.blocks.last() {
            Some(block) => block,
            None => self.add_payload(BlockPayload {
                previous_hash: Hash([0; 32]),
                id: 0,
                timestamp: Utc::now().timestamp(),
                data: Vec::new(),
            }),
        };
        let payload = BlockPayload {
            previous_hash: prev_block.hash.clone(),
            id: prev_block.payload.id + 1,
            timestamp: Utc::now().timestamp(),
            data,
        };
        self.add_payload(payload)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct BlockPayload {
    pub id: u32,
    pub previous_hash: Hash,
    pub timestamp: i64,
    pub data: Vec<u8>,
}

impl BlockPayload {
    fn hash(&self) -> Hash {
        let mut context = Context::new(&SHA256);
        context.update(&self.id.to_le_bytes());
        context.update(&self.previous_hash.0);
        context.update(&self.timestamp.to_le_bytes());
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub hash: Hash,
    pub payload: BlockPayload,
}

fn main() {
    let mut block_chain = BlockChain::new();
    block_chain.add("First block".into());
    block_chain.add("Second block".into());
    println!("{:#?}", block_chain);
}
