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

impl TryFrom<&str> for Hash {
    type Error = ();
    fn try_from(other: &str) -> Result<Self, Self::Error> {
        if other.len() != 64 {
            return Err(());
        }
        let mut result = [0; 32];
        for i in 0..32 {
            result[i] = match u8::from_str_radix(&other[i * 2..i * 2 + 2], 16) {
                Ok(v) => v,
                Err(_) => return Err(()),
            };
        }
        Ok(Hash(result))
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
        match Hash::try_from(value) {
            Ok(hash) => Ok(hash),
            Err(_) => Err(E::custom(format!("Hash could not be parsed: {}", value))),
        }
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

/// A stack-based string for a hash.
pub struct StackStringHash {
    // Hashes require 64 letters [0-9a-f]
    data: [u8; 64],
}

impl StackStringHash {
    pub fn new() -> Self {
        Self { data: [0; 64] }
    }

    fn to_hex_ascii(nibble: u8) -> u8 {
        let offset = if nibble < 10 {
            '0' as u8
        } else {
            'a' as u8 - 10
        };
        nibble + offset
    }

    /// I'm not sure if this is really worth it, but make getting hashes as strings
    /// as fast as possible.
    pub fn set<'a>(&'a mut self, hash: &Hash) -> &'a str {
        for i in 0..32 {
            let byte = hash.0[i];
            self.data[i * 2] = StackStringHash::to_hex_ascii(byte >> 4);
            self.data[i * 2 + 1] = StackStringHash::to_hex_ascii(byte & 0b0000_1111);
        }
        self.str()
    }

    pub fn str<'a>(&'a mut self) -> &'a str {
        // See the test test_stack_string_fuzzed below where this is fuzzed.
        // The data is assumed to be only ascii-ranged code points.
        unsafe { std::str::from_utf8_unchecked(&self.data) }
    }
}

impl From<&Hash> for StackStringHash {
    fn from(other: &Hash) -> Self {
        let mut s = Self::new();
        s.set(other);
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EMPTY_HASH_STR: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";
    const REPEATING_HASH_STR: &str =
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const INVALID_HASH_STR: &str =
        "g000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn test_hash_serialization() {
        let make_hash = |hash_str: &str| {
            let hash = Hash::try_from(hash_str).expect("Failed to make hash");
            String::from(&hash)
        };
        assert_eq!(&String::from(&Hash::empty()), EMPTY_HASH_STR);
        assert_eq!(&make_hash(EMPTY_HASH_STR), EMPTY_HASH_STR);

        assert_eq!(&make_hash(REPEATING_HASH_STR), REPEATING_HASH_STR);
    }

    #[test]
    fn test_invalid_hash() {
        Hash::try_from("000000").expect_err("Too short.");
        Hash::try_from(INVALID_HASH_STR).expect_err("Not [0-9a-f]");
    }

    #[test]
    fn test_stack_string() {
        let mut stack_string_hash = StackStringHash::new();
        let hash = Hash::try_from(REPEATING_HASH_STR).expect("Failed to make hash");
        let str = stack_string_hash.set(&hash);
        assert_eq!(str, REPEATING_HASH_STR);
    }

    #[test]
    fn test_stack_string_fuzzed() {
        // Fuzz this since it has unsafe in the implementation.
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut stack_string_hash = StackStringHash::new();
        for _ in 0..100 {
            let mut hash = Hash::empty();
            for i in 0..hash.0.len() {
                hash.0[i] = rng.gen();
            }
            assert_eq!(&String::from(&hash), stack_string_hash.set(&hash))
        }
    }
}
