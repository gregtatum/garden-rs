use crate::{
    block_chain::{Block, BlockChain, BlockData},
    hash::{Hash, StackStringHash},
};
use anyhow::{bail, Context, Result};
use std::{borrow::Cow, collections::HashSet, fs, io::BufReader, path::PathBuf};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct HeadRef(Cow<'static, str>);

impl HeadRef {
    /// For now only allow [a-zA-Z0-9-_]. This could be made more permissive, but
    /// security concerns should be considered, as this is used to serialize to disk.
    fn validate_name(name: &str) -> bool {
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                return true;
            }
        }
        false
    }

    pub fn str(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<String> for HeadRef {
    type Error = anyhow::Error;
    fn try_from(other: String) -> Result<Self, Self::Error> {
        if !HeadRef::validate_name(&other) {
            bail!("The head ref did not have a valid name: {:?}", other);
        }
        Ok(Self(Cow::Owned(other)))
    }
}

impl TryFrom<&'static str> for HeadRef {
    type Error = anyhow::Error;
    fn try_from(other: &'static str) -> Result<Self, Self::Error> {
        if !HeadRef::validate_name(&other) {
            bail!("The head ref did not have a valid name: {:?}", other);
        }
        Ok(Self(Cow::Borrowed(other)))
    }
}

pub trait ChainStore<T: BlockData> {
    /// The list of known hashes without doing additional loading. Blocks can be
    /// grouped together into chain chunks.
    fn get_known_hashes(&mut self) -> Result<HashSet<Hash>>;

    /// Persist the blocks to the backing storage.
    fn persist(&mut self) -> Result<()>;

    /// Iterate from root to tip all of the loaded blocks. The first block is not
    /// guaranteed to be rooted.
    fn iter_loaded(&self) -> Box<dyn DoubleEndedIterator<Item = &Block<T>> + '_>;

    /// Iterate through all the chains from the root. This is fallible since the chain
    /// store may need to deserialize blocks from storage.
    fn iter_all(&mut self)
        -> Result<Box<dyn DoubleEndedIterator<Item = &Block<T>> + '_>>;

    fn add(&mut self, data: T) -> &Block<T>;
    fn head_ref(&self) -> &HeadRef;
}

impl<T: BlockData> std::fmt::Debug for dyn ChainStore<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("dyn ChainStore<T>")
            .field("head_ref", self.head_ref())
            .finish()
    }
}

/// Persists blockchains on the file system.
/// .
/// └── .garden
///     ├── chains
///     │   ├── 01
///     │   │   ├── 23456789abcdef0123456789abcdef0123456789abcdef0123456789000001
///     │   │   └── 23456789abcdef0123456789abcdef0123456789abcdef0123456789000002
///     │   └── ab
///     │   │   ├── cdef0123456789abcdef0123456789abcdef0123456789abcdef0000000001
///     │   │   └── cdef0123456789abcdef0123456789abcdef0123456789abcdef0000000002
///     └── heads
///     │   ├── garden-1
///     │   └── garden-2
///     └── HEAD
#[derive(Debug, PartialEq)]
pub struct FsChainStore<T: BlockData> {
    /// The path to where the chains are stored.
    ///   Example path: .garden
    pub root_path: PathBuf,

    /// Stores all of the pieces of chains based on their tip hash. It can be partial
    /// block chains, and can store multiple roots.
    ///   Example path: .garden/chains
    pub chains_path: PathBuf,

    /// Named references to the heads of block chains.
    pub heads_path: PathBuf,

    /// The ref to the head. This is a string like "garden-1". This points to
    /// a file in .garden/heads/garden-1. That file contains the hash of a block.
    /// This block must be serialized in the .garden/chains folder.
    pub head_ref: HeadRef,

    pub chain: BlockChain<T>,

    /// The number of blocks that need to be persisted.
    unpersisted_block_count: usize,
}

impl<T: BlockData> FsChainStore<T> {
    pub fn try_new(root_path: PathBuf, head_ref: HeadRef) -> Result<Self> {
        if !root_path.as_path().exists() {
            let parent = root_path.as_path().parent();
            if parent.is_none() {
                bail!("the root path is not valid: {}", root_path.display())
            }

            let parent = parent.unwrap();
            if !parent.is_dir() {
                bail!("the parent path is a file: {}", parent.display());
            }

            // Make the directory.
            if fs::create_dir(root_path.clone()).is_err() {
                bail!("failed to make the root path for {}", root_path.display());
            }
        } else if !root_path.as_path().is_dir() {
            bail!("A file exists at that root path {}", root_path.display());
        }

        // Double check the logic above was correct.
        assert!(root_path.as_path().is_dir());

        let mut chains_path = root_path.clone();
        chains_path.push("chains");
        if !chains_path.exists() {
            if fs::create_dir(chains_path.clone()).is_err() {
                bail!(
                    "failed to create chains directory {}",
                    chains_path.display()
                )
            }
        }

        let mut heads_path = root_path.clone();
        heads_path.push("heads");
        if !heads_path.exists() {
            if fs::create_dir(heads_path.clone()).is_err() {
                bail!("failed to create heads directory {}", heads_path.display())
            }
        }

        Ok(Self {
            root_path,
            chains_path,
            heads_path,
            head_ref,
            chain: BlockChain::new(),
            unpersisted_block_count: 0,
        })
    }

    pub fn head_path(&self, head_ref: &HeadRef) -> PathBuf {
        let mut head_path = self.heads_path.clone();
        head_path.push(head_ref.str());
        head_path
    }

    pub fn load_next_parent_chain<'a>(&'a mut self) -> Result<Option<&'a Block<T>>> {
        let hash = {
            if let Some(root_most_block) = self.chain.blocks.front() {
                root_most_block.payload.parent.clone()
            } else {
                let mut head_path = self.heads_path.clone();
                head_path.push(self.head_ref.str());
                if !head_path.exists() {
                    return Ok(None);
                }
                resolve_fs_ref(&head_path)?
            }
        };

        if hash.is_root() {
            return Ok(None);
        }

        let hash_str = StackStringHash::from(&hash);
        let mut path = self.chains_path.clone();

        path.push(&hash_str.str()[0..2]);
        path.push(&hash_str.str()[2..64]);

        let file = fs::File::open(path.clone()).with_context(|| {
            format!("attempted to load the next chain {}", path.display())
        })?;

        let reader = BufReader::new(file);

        //                     [block(5), block(6), block(7)]
        // [block(3), block(4)]                             └── Existing block chain
        //                    └── .json file to load

        let mut blocks =
            serde_json::from_reader::<BufReader<fs::File>, Vec<Block<T>>>(reader)
                .context("failed to deserialize block")?;

        for block in blocks.drain(..).rev() {
            self.chain.blocks.push_front(block);
        }

        Ok(self.chain.blocks.front())
    }

    pub fn load_all_chains(&mut self) -> Result<()> {
        loop {
            let chain = self.load_next_parent_chain()?;
            if chain.is_none() {
                break;
            }
        }
        return Ok(());
    }
}

impl<T: BlockData> ChainStore<T> for FsChainStore<T> {
    fn get_known_hashes(&mut self) -> Result<HashSet<Hash>> {
        let mut chain_hashes = HashSet::<Hash>::new();
        let dir_entries = fs::read_dir(self.chains_path.clone());
        if dir_entries.is_err() {
            bail!("could not read directory {}", self.chains_path.display());
        }

        let mut path_str = String::new();
        for dir_entry in dir_entries.unwrap() {
            if dir_entry.is_err() {
                continue;
            }
            let dir_entry = dir_entry.unwrap();

            let postfix_dir_entries = fs::read_dir(dir_entry.path());
            if postfix_dir_entries.is_err() {
                bail!("could not read directory {}", dir_entry.path().display());
            }
            let prefix_file_name = dir_entry.file_name();
            let prefix_path_str: &str = &prefix_file_name.to_string_lossy();
            for postfix_dir_entry in postfix_dir_entries.unwrap() {
                if postfix_dir_entry.is_err() {
                    continue;
                }
                let postfix_dir_entry = postfix_dir_entry.unwrap();
                path_str.clear();
                path_str.push_str(prefix_path_str);
                path_str.push_str(&postfix_dir_entry.file_name().to_string_lossy());

                if let Ok(hash) = Hash::try_from(path_str.as_str()) {
                    chain_hashes.insert(hash);
                } else {
                    eprintln!("Could not read chain {:?}", dir_entry);
                }
            }
        }

        Ok(chain_hashes)
    }

    fn persist(&mut self) -> Result<()> {
        let tip = self.chain.tip();
        if tip.is_none() {
            // There is nothing to persist.
            return Ok(());
        }
        let tip = tip.unwrap();
        let tip_string = StackStringHash::from(&tip.hash);

        let mut target_path = self.chains_path.clone();

        // Use the same optimization as git and store the chains in multiple
        // sub-folders.
        //
        // 0123456789abcdefffffffffffffffffffffffffffffffffffffffffffffffff
        //   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //   └── chain file
        // ^^
        // └── prefix

        // Add the prefix, e.g "01" in th example above.
        target_path.push(&tip_string.str()[0..2]);

        // Ensure the prefix folder exists.
        if !target_path.as_path().is_dir() {
            // Make the directory.
            fs::create_dir(target_path.clone()).with_context(|| {
                format!(
                    "failed to create the hash prefix directory {}",
                    target_path.display()
                )
            })?;
        }

        // Add the chain file name, e.g "23456789abcdefff...f" in th example above.
        target_path.push(&tip_string.str()[2..64]);

        if target_path.as_path().exists() {
            // This block has already been serialized, we are done.
            self.unpersisted_block_count = 0;
            return Ok(());
        }

        // Look for a root that has been serialized.
        let mut root_index = 0;
        {
            let mut root_path = self.chains_path.clone();
            let mut root_hash_string = StackStringHash::from(&tip.hash);

            for (i, block) in self.chain.blocks.iter().enumerate().rev() {
                let parent = &block.payload.parent;
                root_index = i;
                if parent.is_root() {
                    // At a root hash, serialize the entire chain.
                    break;
                }
                let hash_str = root_hash_string.set(&parent);
                root_path.push(&hash_str[0..2]);
                root_path.push(&hash_str[2..64]);
                if root_path.exists() {
                    break;
                }
                root_path.pop();
                root_path.pop();
            }
        }

        // Create the target file.
        let target_file = fs::File::create(target_path.clone()).with_context(|| {
            format!("attempted to load the next chain {}", target_path.display())
        })?;

        // Build a list of the blocks from the VecDeque.
        let mut blocks: Vec<&Block<T>> = Vec::new();
        for i in root_index..self.chain.blocks.len() {
            blocks.push(
                self.chain
                    .blocks
                    .get(i)
                    .expect("Logic error, couldn't get block"),
            );
        }

        // Write out the chain as JSON.
        serde_json::to_writer_pretty(target_file, &blocks).with_context(|| {
            format!("failed to write blocks to file {}", target_path.display())
        })?;

        fs::write(self.head_path(&self.head_ref), String::from(&(tip.hash)))
            .with_context(|| {
                format!(
                    "failed to write head reference {}",
                    self.head_path(&self.head_ref).display()
                )
            })?;

        self.unpersisted_block_count = 0;
        Ok(())
    }

    fn iter_loaded(&self) -> Box<dyn DoubleEndedIterator<Item = &Block<T>> + '_> {
        Box::new(self.chain.blocks.iter())
    }

    fn iter_all(
        &mut self,
    ) -> Result<Box<dyn DoubleEndedIterator<Item = &Block<T>> + '_>> {
        if self.chain.is_partial() {
            self.load_all_chains()?;
        }
        if self.chain.is_partial() && !self.chain.blocks.is_empty() {
            bail!(
                "The chain is incomplete, the first block is {:?}",
                self.chain
                    .blocks
                    .front()
                    .expect("Logic error, unable to get front block.")
            );
        }
        Ok(Box::new(self.chain.blocks.iter()))
    }

    fn add(&mut self, data: T) -> &Block<T> {
        self.chain.add_data(data);
        self.unpersisted_block_count += 1;
        self.chain
            .blocks
            .back()
            .expect("Logic error, failed to get the back block.")
    }

    fn head_ref(&self) -> &HeadRef {
        &self.head_ref
    }
}

// #[derive(Debug)]
// struct FsBlockIterator<'a, T: BlockData + 'a> {
//     next_chain_index: usize,
//     next_block_index: usize,
//     back_chain_index: usize,
//     back_block_index: usize,
//     fs_chain_store: &'a FsChainStore<T>,
// }

// impl<'a, T: BlockData> FsBlockIterator<'a, T> {
//     fn new(fs_chain_store: &'a FsChainStore<T>) -> FsBlockIterator<'a, T> {
//         Self {
//             next_chain_index: 0,
//             next_block_index: 0,
//             back_chain_index: 0,
//             back_block_index: 0,
//             fs_chain_store,
//         }
//     }
// }

// impl<'a, T: BlockData> Iterator for FsBlockIterator<'a, T> {
//     type Item = &'a Block<T>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if (self.next_chain_index > 1 && self.back_chain_index > 1)
//             || (self.next_block_index > 1 && self.back_block_index > 1)
//         {
//             panic!("TODO - This iterator doesn't support both forward and reverse iteration at the same time. {:#?}", self);
//         }
//         // chain[block(4), block(5), block(6)], chain[block(1), block(2), block(3)]
//         // └─ Start here -->
//         if let Some(BlockChain { ref blocks, .. }) =
//             self.fs_chain_store.chain.get(self.next_chain_index)
//         {
//             if blocks.is_empty() {
//                 return None;
//             }
//             // chain[block(4), block(5), block(6)]
//             //                      <--- └─ Start here
//             let last_block_index = blocks.len() - 1;
//             if let Some(block) = blocks.get(last_block_index - self.next_block_index) {
//                 self.next_block_index += 1;

//                 if self.next_block_index == blocks.len() {
//                     // We got to the end of the blocks in the chain, go to the next chain.
//                     self.next_chain_index += 1;
//                     self.next_block_index = 0;
//                 }
//                 return Some(block);
//             }
//         };

//         None
//     }
// }

// impl<'a, T: BlockData> DoubleEndedIterator for FsBlockIterator<'a, T> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         if (self.next_chain_index > 0 && self.back_chain_index > 0)
//             || (self.next_block_index > 0 && self.back_block_index > 0)
//         {
//             panic!("TODO - This iterator doesn't support both forward and reverse iteration at the same time. {:#?}", self);
//         }
//         // chain[block(4), block(5), block(6)], chain[block(1), block(2), block(3)]
//         //                                 <--  └─ Start here
//         let last_chain_index = self.fs_chain_store.chain.len() - 1;
//         if self.back_chain_index > last_chain_index {
//             return None;
//         }
//         if let Some(BlockChain { ref blocks, .. }) = self
//             .fs_chain_store
//             .chain
//             .get(last_chain_index - self.back_chain_index)
//         {
//             if blocks.is_empty() {
//                 return None;
//             }
//             // chain[block(4), block(5), block(6)]
//             //       └─ Start here ->
//             if let Some(block) = blocks.get(self.back_block_index) {
//                 self.back_block_index += 1;

//                 if self.back_block_index == blocks.len() {
//                     // We got to the end of the blocks in the chain, go to the next chain.
//                     self.back_chain_index += 1;
//                     self.back_block_index = 0;
//                 }
//                 return Some(block);
//             }
//         };

//         None
//     }
// }

/// Expensively consolidates the blocks in the chain store into a single blockchain.
/// This involves a full copy
impl<T: BlockData> From<&FsChainStore<T>> for BlockChain<T> {
    fn from(store: &FsChainStore<T>) -> Self {
        let mut blocks: Vec<Block<T>> = Vec::new();
        for block in store.iter_loaded() {
            blocks.push(block.clone())
        }
        BlockChain::from(blocks)
    }
}

impl<T: BlockData> From<&mut FsChainStore<T>> for BlockChain<T> {
    fn from(store: &mut FsChainStore<T>) -> Self {
        let mut blocks: Vec<Block<T>> = Vec::new();
        for block in store.iter_loaded() {
            blocks.push(block.clone())
        }
        BlockChain::from(blocks)
    }
}

#[derive(Error, Debug)]
pub enum ResolveRefError {
    #[error("the hash ref is invalid")]
    InvalidRefHash,
    #[error("failed to read ref")]
    FailedToReadRef,
}

fn resolve_fs_ref(path: &PathBuf) -> Result<Hash, ResolveRefError> {
    match fs::read_to_string(path) {
        Ok(contents) => match Hash::try_from(contents.as_str()) {
            Ok(hash) => Ok(hash),
            Err(_) => Err(ResolveRefError::InvalidRefHash),
        },
        Err(_) => Err(ResolveRefError::FailedToReadRef),
    }
}

#[cfg(test)]
mod test {
    use crate::{
        utils::{tree_lines, TimeStampScope},
        ChainAction,
    };

    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tempdir::TempDir;

    fn subpath(path: &PathBuf, folder: &str) -> PathBuf {
        let mut path = path.clone();
        path.push(folder);
        path
    }

    fn subpath_exists(path: &PathBuf, folder: &str) -> bool {
        subpath(path, folder).exists()
    }

    #[test]
    fn test_chainstore_try_new() {
        let tmp_dir = TempDir::new("example").expect("Failed to create a temp directory");
        let path: PathBuf = tmp_dir.into_path();
        let head_ref = HeadRef::try_from("my-garden").unwrap();
        FsChainStore::<ChainAction>::try_new(path.clone(), head_ref)
            .expect("Failed to create ChainStore");

        assert!(subpath_exists(&path, "chains"));
        assert!(subpath_exists(&path, "heads"));
    }

    #[test]
    fn test_chainstore_try_new_subdir() {
        let tmp_dir = TempDir::new("example").expect("Failed to create a temp directory");
        let mut path: PathBuf = tmp_dir.into_path();
        path.push(".garden");
        let head_ref = HeadRef::try_from("my-garden").unwrap();
        FsChainStore::<ChainAction>::try_new(path.clone(), head_ref)
            .expect("Failed to create ChainStore");

        assert!(subpath_exists(&path, "chains"));
        assert!(subpath_exists(&path, "heads"));
    }

    #[test]
    fn test_chainstore_try_new_invalid() {
        let tmp_dir = TempDir::new("example").expect("Failed to create a temp directory");
        let mut path: PathBuf = tmp_dir.into_path();
        path.push("not-here");
        path.push(".garden");
        let head_ref = HeadRef::try_from("my-garden").unwrap();
        assert!(FsChainStore::<ChainAction>::try_new(path.clone(), head_ref).is_err());
    }

    fn ls(path: &PathBuf) -> Vec<String> {
        let paths = fs::read_dir(path).expect("Failed to read dir.");
        let mut result: Vec<String> = Vec::new();
        for path in paths {
            if let Some(file_name) = path.expect("Failed to get path").path().file_name()
            {
                result.push(file_name.to_string_lossy().into());
            }
        }
        result
    }

    pub fn join_path(path: &PathBuf, parts: &[&str]) -> PathBuf {
        let mut path = path.clone();
        for part in parts {
            path.push(part);
        }
        path
    }

    const HASH_ROOT: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";
    const HASH_1: &str =
        "0aa8416c618aa6f5243c8a273a4398991ed5f8e097d6807b30164d37c8d84b33";
    const HASH_2: &str =
        "dc8243497f48f2fbb2677646456d4d3f123250a95c838082bfc97716b775b5ff";
    const HASH_3: &str =
        "fba2f217aa0411b48bc370769b9018dbbd1996f7d6ef0221e9db829975931330";
    const HASH_4: &str =
        "d722da39a7e34043683136eb3048b7ac1f3c68778875b17ffc01d8809632bb9c";
    const HASH_2_FILE_NAME: &str =
        "8243497f48f2fbb2677646456d4d3f123250a95c838082bfc97716b775b5ff";
    const HASH_4_FILE_NAME: &str =
        "22da39a7e34043683136eb3048b7ac1f3c68778875b17ffc01d8809632bb9c";

    #[allow(unused)]
    const HASH_1_SUBFOLDER: &str = "0a";
    const HASH_2_SUBFOLDER: &str = "dc";
    #[allow(unused)]
    const HASH_3_SUBFOLDER: &str = "fb";
    const HASH_4_SUBFOLDER: &str = "d7";

    struct ChainStoreTest {
        #[allow(dead_code)] // RAII
        pub tmp_dir: TempDir,
        pub path: PathBuf,
        pub chain_store: FsChainStore<String>,
        #[allow(unused)] // Side effect.
        pub timestamp: TimeStampScope,
    }

    impl ChainStoreTest {
        pub fn new() -> Self {
            let tmp_dir =
                TempDir::new("example").expect("Failed to create a temp directory");
            let path: PathBuf = tmp_dir.path().into();
            let head_ref =
                HeadRef::try_from("my-garden").expect("Failed to create HeadRef");
            let chain_store = FsChainStore::<String>::try_new(path.clone(), head_ref)
                .expect("Failed to create ChainStore");

            Self {
                tmp_dir,
                path,
                chain_store,
                timestamp: TimeStampScope::new(),
            }
        }
    }

    #[test]
    fn test_chainstore_hashes() {
        // Test that the hashes are all the same as expected.
        let mut chain = BlockChain::<String>::new();
        chain.add_data("data 1".into());
        assert_eq!(&String::from(&chain.tip().unwrap().hash), HASH_1);
        chain.add_data("data 2".into());
        assert_eq!(&String::from(&chain.tip().unwrap().hash), HASH_2);
        chain.add_data("data 3".into());
        assert_eq!(&String::from(&chain.tip().unwrap().hash), HASH_3);
        chain.add_data("data 4".into());
        assert_eq!(&String::from(&chain.tip().unwrap().hash), HASH_4);
    }

    #[test]
    fn test_chainstore_dirs_one_store() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ref path,
            ..
        } = test;

        assert_eq!(ls(&chain_store.chains_path), Vec::<String>::new());
        assert_eq!(ls(&join_path(path, &["heads"])), Vec::<String>::new(),);

        // Store each time a block is added.
        chain_store.add("data 1".into());
        chain_store.add("data 2".into());
        chain_store.add("data 3".into());
        chain_store.add("data 4".into());
        chain_store.persist().expect("Failed to store chains.");

        assert_eq!(
            tree_lines(&chain_store.root_path),
            vec![
                ".",
                "├── chains",
                "│   └── d7",
                "│       └── 22da39a7e34043683136eb3048b7ac1f3c68778875b17ffc01d8809632bb9c",
                "└── heads",
                "    └── my-garden",
            ]
        );
    }

    #[test]
    fn test_chainstore_dirs_two_store() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ..
        } = test;

        // Store each time a block is added.
        chain_store.add("data 1".into());
        chain_store.add("data 2".into());
        chain_store.persist().expect("Failed to store chains.");
        chain_store.add("data 3".into());
        chain_store.add("data 4".into());
        chain_store.persist().expect("Failed to store chains.");

        assert_eq!(
            tree_lines(&chain_store.root_path),
            vec![
                ".",
                "├── chains",
                "│   ├── d7",
                "│   │   └── 22da39a7e34043683136eb3048b7ac1f3c68778875b17ffc01d8809632bb9c",
                "│   └── dc",
                "│       └── 8243497f48f2fbb2677646456d4d3f123250a95c838082bfc97716b775b5ff",
                "└── heads",
                "    └── my-garden",
            ]
        );
    }

    #[test]
    fn test_chainstore_dirs_four_store() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ..
        } = test;

        // Store each time a block is added.
        chain_store.add("data 1".into());
        chain_store.persist().expect("Failed to store chains.");
        chain_store.add("data 2".into());
        chain_store.persist().expect("Failed to store chains.");
        chain_store.add("data 3".into());
        chain_store.persist().expect("Failed to store chains.");
        chain_store.add("data 4".into());
        chain_store.persist().expect("Failed to store chains.");

        assert_eq!(
            tree_lines(&chain_store.root_path),
            vec![
                ".",
                "├── chains",
                "│   ├── 0a",
                "│   │   └── a8416c618aa6f5243c8a273a4398991ed5f8e097d6807b30164d37c8d84b33",
                "│   ├── d7",
                "│   │   └── 22da39a7e34043683136eb3048b7ac1f3c68778875b17ffc01d8809632bb9c",
                "│   ├── dc",
                "│   │   └── 8243497f48f2fbb2677646456d4d3f123250a95c838082bfc97716b775b5ff",
                "│   └── fb",
                "│       └── a2f217aa0411b48bc370769b9018dbbd1996f7d6ef0221e9db829975931330",
                "└── heads",
                "    └── my-garden",
            ]
        );
    }

    #[test]
    fn test_chainstore_store_one() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ref path,
            ..
        } = test;

        chain_store.add("data 1".into());
        chain_store.add("data 2".into());
        chain_store.persist().expect("Failed to store chains.");

        let file_contents = fs::read_to_string(&join_path(
            path,
            &["chains", HASH_2_SUBFOLDER, HASH_2_FILE_NAME],
        ))
        .expect("Failed to read file.");

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&file_contents)
                .expect("JSON was not well-formatted"),
            json!([
                {
                    "hash": HASH_1,
                    "payload": {
                        "data": "data 1",
                        "parent": HASH_ROOT,
                        "timestamp": 0,
                    },
                },
                {
                    "hash": HASH_2,
                    "payload": {
                        "data": "data 2",
                        "parent": HASH_1,
                        "timestamp": 1,
                    },
                },
            ])
        );
    }

    #[test]
    fn test_chainstore_store_two() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ref path,
            ..
        } = test;

        chain_store.add("data 1".into());
        chain_store.add("data 2".into());
        chain_store.persist().expect("Failed to store chains.");
        chain_store.add("data 3".into());
        chain_store.add("data 4".into());
        chain_store.persist().expect("Failed to store chains.");

        let file_contents = fs::read_to_string(&join_path(
            path,
            &["chains", HASH_4_SUBFOLDER, HASH_4_FILE_NAME],
        ))
        .expect("Failed to read file.");

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&file_contents)
                .expect("JSON was not well-formatted"),
            json!([
                {
                    "hash": HASH_3,
                    "payload": {
                        "data": "data 3",
                        "parent": HASH_2,
                        "timestamp": 2,
                    },
                },
                {
                    "hash": HASH_4,
                    "payload": {
                        "data": "data 4",
                        "parent": HASH_3,
                        "timestamp": 3,
                    },
                },
            ])
        );
    }

    #[test]
    fn test_chainstore_store_load() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ref path,
            ..
        } = test;

        chain_store.add("data 1".into());
        chain_store.add("data 2".into());
        chain_store.persist().expect("Failed to store chains.");
        chain_store.add("data 3".into());
        chain_store.add("data 4".into());
        chain_store.persist().expect("Failed to store chains.");

        let mut new_chain_store =
            FsChainStore::<String>::try_new(path.clone(), chain_store.head_ref.clone())
                .expect("Failed to create ChainStore");

        assert_eq!(new_chain_store.chain.blocks.len(), 0);
        {
            let data: Vec<&str> = new_chain_store
                .iter_all()
                .expect("Failed to load all chains.")
                .map(|block| block.payload.data.as_str())
                .collect();

            assert_eq!(data, vec!["data 1", "data 2", "data 3", "data 4"]);
        }
        assert_eq!(new_chain_store.chain.blocks.len(), 4);

        assert_eq!(
            BlockChain::from(chain_store),
            BlockChain::from(&new_chain_store)
        );
    }

    fn get_store_for_iterator_tests() -> FsChainStore<String> {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ref path,
            ..
        } = test;

        chain_store.add("data 1".into());
        chain_store.add("data 2".into());
        chain_store.persist().expect("Failed to store chains.");
        chain_store.add("data 3".into());
        chain_store.add("data 4".into());
        chain_store.persist().expect("Failed to store chains.");

        let mut new_chain_store =
            FsChainStore::<String>::try_new(path.clone(), chain_store.head_ref.clone())
                .expect("Failed to create ChainStore");

        new_chain_store
            .load_all_chains()
            .expect("Failed to load all chains.");

        new_chain_store
    }

    #[test]
    fn test_iter() {
        let chain_store = get_store_for_iterator_tests();

        let data: Vec<&str> = chain_store
            .iter_loaded()
            .map(|block| block.payload.data.as_str())
            .collect();

        assert_eq!(data, vec!["data 1", "data 2", "data 3", "data 4"]);
    }
}
