use crate::hash::Hash;
use std::{collections::HashSet, fs, path::PathBuf};

trait ChainStore {
    fn refresh_chains(&mut self) -> Result<(), ChainStoreError>;
}

/// Persists blockchains on the file system.
#[derive(Debug, PartialEq)]
pub struct FsChainStore {
    /// The path to where the chains are stored.
    ///   Example path: .garden
    pub root_path: PathBuf,

    /// Stores all of the pieces of chains based on their tip hash. It can be partial
    /// block chains, and can store multiple roots.
    ///   Example path: .garden/chains
    pub chains_path: PathBuf,

    /// Named references to block chains.
    pub refs_path: PathBuf,

    // The list of all known chains in the store
    pub chains: HashSet<Hash>,
}

#[derive(Debug, PartialEq)]
pub enum ChainStoreError {
    RootPathNotValid,
    FailedToCreateRootDirectory,
    FailedToCreateChainsDirectory,
    FailedToCreateRefsDirectory,
    CouldNotReadDirectory,
}

impl FsChainStore {
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

        let mut refs_path = root_path.clone();
        refs_path.push("refs");
        if !refs_path.exists() {
            if fs::create_dir(refs_path.clone()).is_err() {
                return Err(ChainStoreError::FailedToCreateRefsDirectory);
            }
        }

        let mut chain_store = Self {
            root_path,
            chains_path,
            refs_path,
            chains: HashSet::new(),
        };
        if let Err(err) = chain_store.refresh_chains() {
            return Err(err);
        }

        Ok(chain_store)
    }
}

impl ChainStore for FsChainStore {
    fn refresh_chains(&mut self) -> Result<(), ChainStoreError> {
        let paths = fs::read_dir(self.chains_path.clone());
        if paths.is_err() {
            return Err(ChainStoreError::CouldNotReadDirectory);
        }

        self.chains.clear();
        for path in paths.unwrap() {
            if let Ok(path) = path {
                let file_name = path.file_name();
                let path_str: &str = &file_name.to_string_lossy();
                if let Ok(hash) = Hash::try_from(path_str) {
                    self.chains.insert(hash);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
        FsChainStore::try_new(path.clone()).expect("Failed to create ChainStore");

        assert!(subpath_exists(&path, "chains"));
        assert!(subpath_exists(&path, "refs"));
    }

    #[test]
    fn test_chainstore_try_new_subdir() {
        let tmp_dir = TempDir::new("example").expect("Failed to create a temp directory");
        let mut path: PathBuf = tmp_dir.into_path();
        path.push(".garden");
        FsChainStore::try_new(path.clone()).expect("Failed to create ChainStore");

        assert!(subpath_exists(&path, "chains"));
        assert!(subpath_exists(&path, "refs"));
    }

    #[test]
    fn test_chainstore_try_new_invalid() {
        let tmp_dir = TempDir::new("example").expect("Failed to create a temp directory");
        let mut path: PathBuf = tmp_dir.into_path();
        path.push("not-here");
        path.push(".garden");
        assert_eq!(
            FsChainStore::try_new(path.clone()).unwrap_err(),
            ChainStoreError::RootPathNotValid
        );
    }

    #[test]
    fn test_chainstore_try_new_chains() {
        let tmp_dir = TempDir::new("example").expect("Failed to create a temp directory");
        let path: PathBuf = tmp_dir.into_path();
        let chains = subpath(&path, "chains");
        let refs = subpath(&path, "refs");

        fs::create_dir(chains.clone()).expect("Failed to create chains dir");
        fs::create_dir(refs.clone()).expect("Failed to create refs dir");

        assert!(chains.exists());
        assert!(refs.exists());

        touch(&subpath(
            &chains,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ));
        touch(&subpath(
            &chains,
            "1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ));
        touch(&subpath(
            &chains,
            "2123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ));

        touch(&subpath(&refs, "garden-1"));
        touch(&subpath(&refs, "garden-2"));

        let store = FsChainStore::try_new(path.clone()).expect("Failed to create ChainStore");

        assert_eq!(store.chains.len(), 3, "Three chains were found.");

        store
            .chains
            .get(&hash(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ))
            .expect("Failed to find expected chain.");
        store
            .chains
            .get(&hash(
                "1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ))
            .expect("Failed to find expected chain.");

        store
            .chains
            .get(&hash(
                "2123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ))
            .expect("Failed to find expected chain.");
    }
}
