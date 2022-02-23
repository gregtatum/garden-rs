use std::rc::Rc;

use anyhow::{bail, Result};

use crate::{garden::GardenPlot, reducers, Action, ChainStore, Hash, State};

#[derive(Debug)]
pub struct Store {
    pub chains: Box<dyn ChainStore<Action>>,
    pub state: Rc<State>,
}

impl Store {
    pub fn try_new(chain_store: Box<dyn ChainStore<Action>>) -> Result<Self> {
        let mut store = Self {
            chains: chain_store,
            state: Rc::new(State::new()),
        };

        store.load_untrusted_chain_store()?;

        Ok(store)
    }

    pub fn dispatch(&mut self, action: Action) {
        self.state = Rc::from(self.state.reduce(&action));
        self.chains.add(action);
    }

    pub fn load_untrusted_chain_store(&mut self) -> Result<()> {
        let mut prev_hash = Hash::empty();

        for block in self.chains.iter_all()? {
            if prev_hash != block.payload.parent {
                bail!(
                    "A block did not match the previous hash.\nPrevious: {:#?},\n{:#?}",
                    prev_hash,
                    block
                )
            }
            if block.payload.hash() != block.hash {
                bail!(
                    "A block's hash did not match.\nComputed: {:#?}\nBlock:{:#?}",
                    block.payload.hash(),
                    block
                )
            }
            prev_hash = block.hash.clone();

            self.state = Rc::from(self.state.reduce(&block.payload.data));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        actions,
        chain_store::{FsChainStore, HeadRef},
    };
    use std::path::PathBuf;
    use tempdir::TempDir;

    struct StateStoreTest {
        #[allow(dead_code)] // RAII
        pub tmp_dir: TempDir,
        pub path: PathBuf,
        pub store: Store,
    }

    impl StateStoreTest {
        pub fn new() -> Self {
            let tmp_dir =
                TempDir::new("example").expect("Failed to create a temp directory");
            let path: PathBuf = tmp_dir.path().into();
            let head_ref =
                HeadRef::try_from("my-garden").expect("Failed to create HeadRef");
            let chain_store = Box::new(
                FsChainStore::<Action>::try_new(path.clone(), head_ref)
                    .expect("Failed to create ChainStore"),
            );
            let store = Store::try_new(chain_store).expect("Failed to create StateStore");

            Self {
                tmp_dir,
                path,
                store,
            }
        }
    }

    #[test]
    fn test_garden() {
        let mut test = StateStoreTest::new();
        let StateStoreTest {
            ref mut store,
            ref path,
            ..
        } = test;
        store.dispatch(actions::create_garden_plot("The Secret Garden".into()));
        store
            .chains
            .persist()
            .expect("Failed to persist chain store");

        let chains = Box::new(
            FsChainStore::<Action>::try_new(
                path.clone(),
                store.chains.head_ref().clone(),
            )
            .expect("Failed to create ChainStore"),
        );

        let store2 = Store::try_new(chains).expect("Failed to create StateStore.");

        assert_eq!(store.state, store2.state);
    }
}
