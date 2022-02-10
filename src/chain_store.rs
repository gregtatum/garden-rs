use crate::{
    block_chain::{BlockChain, BlockData},
    hash::{Hash, StackStringHash},
};
use std::{borrow::Cow, collections::HashSet, fs, marker::PhantomData, path::PathBuf};

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

    fn str(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<String> for HeadRef {
    type Error = ();
    fn try_from(other: String) -> Result<Self, Self::Error> {
        if HeadRef::validate_name(&other) {
            Ok(Self(Cow::Owned(other)))
        } else {
            Err(())
        }
    }
}

impl TryFrom<&'static str> for HeadRef {
    type Error = ();
    fn try_from(other: &'static str) -> Result<Self, Self::Error> {
        if HeadRef::validate_name(&other) {
            Ok(Self(Cow::Borrowed(other)))
        } else {
            Err(())
        }
    }
}

pub trait ChainStore<T: BlockData> {
    fn refresh_chains(&mut self) -> Result<(), ChainStoreError>;
    fn store(
        &mut self,
        block_chain: &BlockChain<T>,
        head_ref: &HeadRef,
    ) -> Result<(), ChainStoreError>;
}

/// Persists blockchains on the file system.
/// .
/// └── .garden
///     ├── chains
///     │   ├── 0123456789abcdef0123456789abcdef
///     │   └── 0123456789abcdef0123456789abcdef
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

    /// The list of all known chains in the store
    pub chains: HashSet<Hash>,

    pub head: Hash,
    pub block_data_: PhantomData<T>,
}

#[derive(Debug, PartialEq)]
pub enum ChainStoreError {
    RootPathNotValid,
    FailedToCreateRootDirectory,
    FailedToCreateChainsDirectory,
    FailedToCreateHeadsDirectory,
    FailedToCreateDirectory,
    FailedToCreateFile,
    FailedToSerializeToFile,
    FailedToWriteFile,
    CouldNotReadDirectory,
    InvalidRefHash,
    FailedToReadRef,
}

impl<T: BlockData> FsChainStore<T> {
    pub fn try_new(root_path: PathBuf) -> Result<Self, ChainStoreError> {
        if !root_path.as_path().exists() {
            let parent = root_path.as_path().parent();
            if parent.is_none() {
                // This file path does not exist.
                return Err(ChainStoreError::RootPathNotValid);
            }

            let parent = parent.unwrap();
            if !parent.is_dir() {
                // The parent directory is a file.
                return Err(ChainStoreError::RootPathNotValid);
            }

            // Make the directory.
            if fs::create_dir(root_path.clone()).is_err() {
                return Err(ChainStoreError::FailedToCreateRootDirectory);
            }
        } else if !root_path.as_path().is_dir() {
            // A file exists at that path.
            return Err(ChainStoreError::RootPathNotValid);
        }

        // Double check the logic above was correct.
        assert!(root_path.as_path().is_dir());

        let mut chains_path = root_path.clone();
        chains_path.push("chains");
        if !chains_path.exists() {
            if fs::create_dir(chains_path.clone()).is_err() {
                return Err(ChainStoreError::FailedToCreateChainsDirectory);
            }
        }

        let mut heads_path = root_path.clone();
        heads_path.push("heads");
        if !heads_path.exists() {
            if fs::create_dir(heads_path.clone()).is_err() {
                return Err(ChainStoreError::FailedToCreateHeadsDirectory);
            }
        }

        let mut head_path = root_path.clone();
        head_path.push("HEAD");
        let head = if head_path.exists() {
            resolve_ref(&head_path)?
        } else {
            Hash::empty()
        };

        let mut chain_store = Self {
            root_path,
            chains_path,
            heads_path,
            chains: HashSet::new(),
            head,
            block_data_: PhantomData,
        };
        if let Err(err) = chain_store.refresh_chains() {
            return Err(err);
        }

        Ok(chain_store)
    }

    fn head_path(&self, head_ref: &HeadRef) -> PathBuf {
        let mut head_path = self.heads_path.clone();
        head_path.push(head_ref.str());
        head_path
    }
}

impl<T: BlockData> ChainStore<T> for FsChainStore<T> {
    fn refresh_chains(&mut self) -> Result<(), ChainStoreError> {
        let dir_entries = fs::read_dir(self.chains_path.clone());
        if dir_entries.is_err() {
            return Err(ChainStoreError::CouldNotReadDirectory);
        }

        self.chains.clear();
        let mut path_str = String::new();
        for dir_entry in dir_entries.unwrap() {
            if dir_entry.is_err() {
                continue;
            }
            let dir_entry = dir_entry.unwrap();

            let postfix_dir_entries = fs::read_dir(dir_entry.path());
            if postfix_dir_entries.is_err() {
                return Err(ChainStoreError::CouldNotReadDirectory);
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
                    self.chains.insert(hash);
                } else {
                    eprintln!("Could not read chain {:?}", dir_entry);
                }
            }
        }

        Ok(())
    }

    fn store(
        &mut self,
        block_chain: &BlockChain<T>,
        head_ref: &HeadRef,
    ) -> Result<(), ChainStoreError> {
        let tip = block_chain.tip();
        if tip.is_none() {
            // This nothing to serialize.
            return Ok(());
        }
        let tip = tip.unwrap();

        let mut tip_string = StackStringHash::from(&tip.hash);

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
            if fs::create_dir(target_path.clone()).is_err() {
                return Err(ChainStoreError::FailedToCreateDirectory);
            }
        }

        // Add the chain file name, e.g "23456789abcdefff...f" in th example above.
        target_path.push(&tip_string.str()[2..64]);

        if target_path.as_path().exists() {
            // This block has already been serialized.
            return Ok(());
        }

        // Look for a root that has been serialized.
        let mut root_index = 0;
        {
            let mut root_path = self.chains_path.clone();
            let mut root_hash_string = StackStringHash::from(&tip.hash);

            for (i, block) in block_chain.blocks.iter().enumerate().rev() {
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
        let target_file = match fs::File::create(target_path) {
            Ok(f) => f,
            Err(_) => return Err(ChainStoreError::FailedToCreateFile),
        };

        // Write out the chain as JSON.
        if let Err(_) =
            serde_json::to_writer_pretty(target_file, &block_chain.blocks[root_index..])
        {
            return Err(ChainStoreError::FailedToSerializeToFile);
        };

        if let Err(_) = fs::write(self.head_path(head_ref), String::from(&(tip.hash))) {
            return Err(ChainStoreError::FailedToWriteFile);
        }

        Ok(())
    }
}

fn resolve_ref(path: &PathBuf) -> Result<Hash, ChainStoreError> {
    match fs::read_to_string(path) {
        Ok(contents) => match Hash::try_from(contents.as_str()) {
            Ok(hash) => Ok(hash),
            Err(_) => Err(ChainStoreError::InvalidRefHash),
        },
        Err(_) => Err(ChainStoreError::FailedToReadRef),
    }
}

#[cfg(test)]
mod test {
    use crate::garden::Event;

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

    fn touch(path: &PathBuf) {
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path.clone())
            .expect("Failed to touch path.");
    }

    fn hash(str: &str) -> Hash {
        str.try_into().expect("Failed to create hash for test.")
    }

    #[test]
    fn test_chainstore_try_new() {
        let tmp_dir = TempDir::new("example").expect("Failed to create a temp directory");
        let path: PathBuf = tmp_dir.into_path();
        FsChainStore::<Event>::try_new(path.clone())
            .expect("Failed to create ChainStore");

        assert!(subpath_exists(&path, "chains"));
        assert!(subpath_exists(&path, "heads"));
    }

    #[test]
    fn test_chainstore_try_new_subdir() {
        let tmp_dir = TempDir::new("example").expect("Failed to create a temp directory");
        let mut path: PathBuf = tmp_dir.into_path();
        path.push(".garden");
        FsChainStore::<Event>::try_new(path.clone())
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
        assert_eq!(
            FsChainStore::<Event>::try_new(path.clone()).unwrap_err(),
            ChainStoreError::RootPathNotValid
        );
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
    const HASH_1_SUBFOLDER: &str = "0a";
    const HASH_2_SUBFOLDER: &str = "dc";
    const HASH_3_SUBFOLDER: &str = "fb";
    const HASH_4_SUBFOLDER: &str = "d7";

    struct ChainStoreTest {
        #[allow(dead_code)] // RAII
        pub tmp_dir: TempDir,
        pub path: PathBuf,
        pub chain_store: FsChainStore<String>,
        pub chain: BlockChain<String>,
        pub head_ref: HeadRef,
    }

    impl ChainStoreTest {
        pub fn new() -> Self {
            let tmp_dir =
                TempDir::new("example").expect("Failed to create a temp directory");
            let path: PathBuf = tmp_dir.path().into();
            let chain_store = FsChainStore::<String>::try_new(path.clone())
                .expect("Failed to create ChainStore");
            let chain = BlockChain::<String>::new();
            let head_ref =
                HeadRef::try_from("my-garden").expect("Failed to create HeadRef");

            Self {
                tmp_dir,
                path,
                chain_store,
                chain,
                head_ref,
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
            ref mut chain,
            ref path,
            ref head_ref,
            ..
        } = test;

        assert_eq!(ls(&chain_store.chains_path), Vec::<String>::new());
        assert_eq!(ls(&join_path(path, &["heads"])), Vec::<String>::new(),);

        // Store each time a block is added.
        chain.add_data("data 1".into());
        chain.add_data("data 2".into());
        chain.add_data("data 3".into());
        chain.add_data("data 4".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");

        assert_eq!(
            ls(&join_path(path, &["chains"])),
            vec![String::from(HASH_4_SUBFOLDER)],
        );

        assert_eq!(
            ls(&join_path(path, &["heads"])),
            vec![String::from("my-garden")],
        );
    }

    #[test]
    fn test_chainstore_dirs_two_store() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ref mut chain,
            ref path,
            ref head_ref,
            ..
        } = test;

        // Store each time a block is added.
        chain.add_data("data 1".into());
        chain.add_data("data 2".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");
        chain.add_data("data 3".into());
        chain.add_data("data 4".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");

        assert_eq!(
            ls(&join_path(path, &["chains"])),
            vec![
                String::from(HASH_4_SUBFOLDER),
                String::from(HASH_2_SUBFOLDER)
            ],
        );
    }

    #[test]
    fn test_chainstore_dirs_four_store() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ref mut chain,
            ref path,
            ref head_ref,
            ..
        } = test;

        // Store each time a block is added.
        chain.add_data("data 1".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");
        chain.add_data("data 2".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");
        chain.add_data("data 3".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");
        chain.add_data("data 4".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");

        assert_eq!(
            ls(&join_path(path, &["chains"])),
            vec![
                String::from(HASH_4_SUBFOLDER),
                String::from(HASH_3_SUBFOLDER),
                String::from(HASH_1_SUBFOLDER),
                String::from(HASH_2_SUBFOLDER),
            ],
        );
    }

    #[test]
    fn test_chainstore_store_one() {
        let mut test = ChainStoreTest::new();
        let ChainStoreTest {
            ref mut chain_store,
            ref mut chain,
            ref path,
            ref head_ref,
            ..
        } = test;

        chain.add_data("data 1".into());
        chain.add_data("data 2".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");

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
            ref mut chain,
            ref path,
            ref head_ref,
            ..
        } = test;

        chain.add_data("data 1".into());
        chain.add_data("data 2".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");
        chain.add_data("data 3".into());
        chain.add_data("data 4".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");

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
            ref mut chain,
            ref path,
            ref head_ref,
            ..
        } = test;

        chain.add_data("data 1".into());
        chain.add_data("data 2".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");
        chain.add_data("data 3".into());
        chain.add_data("data 4".into());
        chain_store
            .store(&chain, head_ref)
            .expect("Failed to store chains.");

        let mut chain_store = FsChainStore::<String>::try_new(path.clone())
            .expect("Failed to create ChainStore");
        println!("{:#?}", chain_store);
        chain_store
            .refresh_chains()
            .expect("Failed to refresh chains");
        let mut chain_store = FsChainStore::<String>::try_new(path.clone())
            .expect("Failed to create ChainStore");
        println!("{:#?}", chain_store);

        // TODO - Create a block iterator that works regardless of chain memory layout.

        // [chain: [block(4), block(5), block(6)]][chain: [block(1), block(2), block(3)]]
        // iter -> 6, 5, 4, 3, 2, 1

        // chain_store.iter_head();
        // chain_store.iter(hash);

        // let chain = BlockChain::<String>::new();
    }
}
