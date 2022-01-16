use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

/// A representation of a Hash for use on a blockchain.
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// Create a new empty Hash.
    pub fn empty() -> Self {
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

    /// A root hash would be all 0 values. This is a somewhat hacky way to create a root
    /// block without adding another property to the Block struct.
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

/// Turn a Hash into a human readable string.
impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&String::from(self))
    }
}

impl fmt::Debug for Hash {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash({})", String::from(self))?;
        Ok(())
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
